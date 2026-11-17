//! Sincronização do banco com o desenho: o desenho é a verdade, nos dois sentidos.
//!
//! É o **Compilar** do Eletron: o botão Salvar da janela de compilação chama
//! `cDadosAtualizaBanco.LimpaDWGApagadosDoProjeto()` (o que sumiu do desenho sai
//! do banco) e, em seguida, `AdicionaItemInterligacao` para **cada item do
//! desenho**, lendo o XData da entidade (`Bootstrap/EventosDB.cs`,
//! `Forms/frmCompilarInterligacao.cs`). O banco é consequência do desenho: apagar
//! um elemento o tira do banco, e o `UNDO` que o traz de volta o recadastra no
//! Sincronizar seguinte.
//!
//! 1. **Saiu do desenho** (ERASE, bloco apagado, outro desenho aberto): o
//!    registro com `Handle` que não existe mais sai de `Barra`, `Linha`,
//!    `Equipamento`, `Componente` e `Legenda`. O que nunca foi desenhado (handle
//!    vazio, importado do `.lst`/`.ANA`) fica: é a topologia que o ANAFAS mostra
//!    em **Elementos Não Desenhados**.
//! 2. **Entrou no desenho** (o `UNDO` de um apagar, um desenho aberto contra
//!    outro `.db`): a entidade com XData do domínio e sem registro no banco é
//!    recadastrada com os campos do próprio XData (`codigo`, `de`/`para`,
//!    `tipo`…), e o `Id` novo volta para o XData. O nome da barra não fica no
//!    XData do traço: é o texto de identificação desenhado com ela
//!    (`ANACAD_BARRA_TEXTO`, campo `identificador`), e é de lá que o recadastro
//!    o tira — junto com `dados`/`vbase`. Sem XData suficiente para
//!    recadastrar, a entidade é relatada em `sem_banco`.
//!
//! Apagar a barra leva junto as linhas presas a ela (`Linha.DeBarra`/
//! `ParaBarra`), do banco **e** do desenho: o vínculo De/Para é a derivação do
//! Eletron, que sai junto da conexão que a sustenta
//! (`clsConexao.ApagarDerivacoesLinhas`); linha sem extremidade não serve para
//! nada e ficaria no desenho sem registro no banco.

use cadcraft_doc::Drawing;

use super::esquematico::{APP_BARRA, APP_CONEXAO, APP_LINHA, APP_TEXTO_BARRA, APP_TEXTO_LINHA, app_xdata, campo_xdata, textos_ligados};

/// App name do XData do equipamento (a tabela `Equipamento`).
const APP_EQ: &str = "ANACAD_EQ";
use super::*;

/// Tabelas sincronizadas e o app name do XData que as representa no desenho.
const TABELAS: [(&str, &str); 5] = [
    ("Barra", "ANACAD_BARRA"),
    ("Linha", "ANACAD_LINHA"),
    ("Equipamento", "ANACAD_EQ"),
    ("Componente", "ANACAD_COMP"),
    ("Legenda", "ANACAD_LEGENDA"),
];

pub(super) fn run_sincronizar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.sincronizar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    // `remover: false` só relata o que sairia/voltaria, sem tocar no banco.
    let aplicar = p.get("remover").and_then(Value::as_bool).unwrap_or(true);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.sincronizar", format!("{db}: {e}")))?;
    let documento = banco.garantir_documento(documento).map_err(|e| bad("anacad.sincronizar", e.to_string()))?;
    set_banco(s, db, documento);

    // Entidades do domínio no desenho, por tabela, com os campos do XData.
    let desenho = varrer(s)?;
    let no_desenho: usize = desenho.values().map(Vec::len).sum();
    // Desenho sem elemento do domínio não sincroniza nada: apagar tudo do
    // documento porque o desenho aberto é outro seria pior que não fazer nada.
    if no_desenho == 0 {
        return Err(bad("anacad.sincronizar", "o desenho nao tem elementos AnaCAD; abra o desenho do documento antes de sincronizar o banco"));
    }

    let mut removidos: BTreeMap<String, Value> = BTreeMap::new();
    let mut total = 0usize;
    // Linhas que caem junto com as barras que saíram (o vínculo De/Para).
    let mut presas: Vec<String> = Vec::new();
    for (tabela, _) in TABELAS {
        // A linha que caiu junto com a barra já foi resolvida no passo da
        // barra: não é "elemento no desenho sem banco" nem entra duas vezes.
        let presentes: Vec<String> = desenho
            .get(tabela)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|entidade| entidade.handle)
            .filter(|handle| !presas.contains(handle))
            .collect();
        let do_banco = banco.handles_do_documento(tabela, documento).map_err(|e| bad("anacad.sincronizar", e.to_string()))?;
        let mut ausentes: Vec<String> = do_banco.iter().filter(|handle| !presentes.contains(handle)).cloned().collect();
        if tabela == "Barra" && !ausentes.is_empty() {
            // A barra apagada leva as linhas presas a ela, no banco e no
            // desenho: o vínculo De/Para não sobrevive à perda de uma ponta.
            let linhas = banco.linhas_das_barras(documento, &ausentes).map_err(|e| bad("anacad.sincronizar", e.to_string()))?;
            if aplicar {
                let ids: Vec<i64> = linhas.iter().map(|(id, _)| *id).collect();
                banco.apagar_linhas(&ids).map_err(|e| bad("anacad.sincronizar", e.to_string()))?;
            }
            for (_, handle) in &linhas {
                if handle.is_empty() || presas.contains(handle) {
                    continue;
                }
                if aplicar {
                    apagar_linha_do_desenho(s, handle);
                }
                presas.push(handle.clone());
            }
        }
        if tabela == "Linha" {
            for handle in presas.clone() {
                if !ausentes.contains(&handle) {
                    ausentes.push(handle);
                }
            }
        }
        total += ausentes.len();
        if tabela == "Equipamento" && !ausentes.is_empty() {
            let conexoes = conexoes_presas_aos_handles(s, &ausentes);
            for h in &conexoes {
                if aplicar {
                    apagar_conexao_do_desenho(s, h);
                }
            }
        }
        if aplicar && !ausentes.is_empty() {
            banco.apagar_handles_ausentes(tabela, documento, &presentes).map_err(|e| bad("anacad.sincronizar", e.to_string()))?;
        }
        removidos.insert(tabela.to_ascii_lowercase(), json!(ausentes));
    }

    // O caminho de volta (passo 2 do topo): o que está no desenho e não está no
    // banco volta a ser registrado, com os campos do XData da entidade.
    let mut recriados: Vec<Value> = Vec::new();
    let mut sem_banco: Vec<Value> = Vec::new();
    for (tabela, app) in TABELAS {
        let no_banco = banco.handles_do_documento(tabela, documento).map_err(|e| bad("anacad.sincronizar", e.to_string()))?;
        for entidade in desenho.get(tabela).cloned().unwrap_or_default() {
            if no_banco.contains(&entidade.handle) || presas.contains(&entidade.handle) {
                continue;
            }
            // A entidade pode ter saído do desenho na cascata desta rodada.
            let Some(handle) = Handle::parse_hex(&entidade.handle) else { continue };
            if !s.doc().is_ok_and(|doc| doc.entity(handle).is_some()) {
                continue;
            }
            if !aplicar {
                recriados.push(json!({ "tabela": tabela.to_ascii_lowercase(), "handle": entidade.handle, "aplicado": false }));
                continue;
            }
            match recriar(&banco, documento, tabela, &entidade) {
                Ok(Some(id)) => {
                    // O `Id` no XData é o do registro no banco: o recadastro
                    // costuma gerar outro, e os comandos leem o `id` de lá.
                    if entidade.inteiro("id") != Some(id) {
                        atualizar_id_xdata(s, app, handle, &entidade, id)?;
                    }
                    recriados.push(json!({ "tabela": tabela.to_ascii_lowercase(), "handle": entidade.handle, "id": id, "aplicado": true }));
                }
                Ok(None) => sem_banco.push(json!({ "tabela": tabela, "handle": entidade.handle })),
                Err(e) => sem_banco.push(json!({ "tabela": tabela, "handle": entidade.handle, "erro": e.to_string() })),
            }
        }
    }

    Ok(json!({
        "documento": documento,
        "removidos": removidos,
        "total": total,
        "recriados": recriados,
        "desenho": {
            "barras": desenho.get("Barra").map(Vec::len).unwrap_or(0),
            "linhas": desenho.get("Linha").map(Vec::len).unwrap_or(0),
            "equipamentos": desenho.get("Equipamento").map(Vec::len).unwrap_or(0),
            "componentes": desenho.get("Componente").map(Vec::len).unwrap_or(0),
            "legendas": desenho.get("Legenda").map(Vec::len).unwrap_or(0),
        },
        "sem_banco": sem_banco,
    }))
}

/// Entidade do domínio no desenho: handle, os pares do XData e o ponto que
/// representa a entidade (o insert do bloco, o ponto do texto, o meio do traço).
#[derive(Clone)]
struct DoDesenho {
    handle: String,
    campos: Vec<(String, String)>,
    ponto: Option<(f64, f64)>,
    /// Nome que o desenho mostra para a entidade (o texto de identificação da
    /// barra): é ele que o recadastro devolve ao banco.
    nome: Option<String>,
}

impl DoDesenho {
    fn campo(&self, nome: &str) -> Option<&str> {
        self.campos.iter().find(|(chave, _)| chave == nome).map(|(_, valor)| valor.as_str())
    }
    fn inteiro(&self, nome: &str) -> Option<i64> {
        self.campo(nome).and_then(|v| v.trim().parse::<i64>().ok())
    }
    fn real(&self, nome: &str) -> Option<f64> {
        self.campo(nome).and_then(|v| v.trim().parse::<f64>().ok())
    }
    /// JSON `dados` do XData (o que a janela de dados gravou).
    fn dados(&self) -> Option<Value> {
        serde_json::from_str(self.campo("dados")?).ok()
    }
}

/// Varre o espaço modelo e agrupa as entidades do domínio pela tabela que o
/// XData delas representa.
fn varrer(s: &Session) -> Result<BTreeMap<&'static str, Vec<DoDesenho>>> {
    let doc = s.doc()?;
    let mut mapa: BTreeMap<&'static str, Vec<DoDesenho>> = TABELAS.iter().map(|(tabela, _)| (*tabela, Vec::new())).collect();
    for e in doc.model.iter() {
        let Some(app) = app_xdata(e) else { continue };
        let Some(&(tabela, _)) = TABELAS.iter().find(|(_, nome)| *nome == app) else { continue };
        let tags: Vec<&cadcraft_doc::RawTag> = e.common.xdata.iter().filter(|tag| tag.code == 1000).collect();
        let campos: Vec<(String, String)> =
            tags.chunks(2).filter(|par| par.len() == 2).map(|par| (par[0].value.clone(), par[1].value.clone())).collect();
        let nome = if app == "ANACAD_BARRA" {
            // O nome da barra não fica no XData do traço: ele é o texto de
            // identificação desenhado junto dela (`ANACAD_BARRA_TEXTO`). Se a
            // janela de dados foi gravada, o mesmo nome também está no `dados`.
            textos_ligados(doc, APP_TEXTO_BARRA, e.handle)
                .into_iter()
                .find(|(campo, _)| campo == "identificador")
                .and_then(|(_, texto)| doc.model.get(texto))
                .and_then(|t| match &t.kind {
                    EntityKind::Text(t) => Some(t.value.trim().to_string()),
                    _ => None,
                })
                .filter(|nome| !nome.is_empty())
                .or_else(|| {
                    let dados: Value =
                        serde_json::from_str(campos.iter().find(|(chave, _)| chave == "dados").map(|(_, valor)| valor.as_str())?).ok()?;
                    dados.get("nome").and_then(Value::as_str).map(str::to_string)
                })
        } else {
            None
        };
        mapa.entry(tabela).or_default().push(DoDesenho { handle: e.handle.hex(), campos, ponto: ponto_da_entidade(e), nome });
    }
    Ok(mapa)
}

/// Ponto que representa a entidade no desenho (posição do bloco, do texto ou o
/// meio do traço).
fn ponto_da_entidade(e: &cadcraft_doc::Entity) -> Option<(f64, f64)> {
    match &e.kind {
        EntityKind::Insert(ins) => Some((ins.insert.x, ins.insert.y)),
        EntityKind::Text(texto) => Some((texto.insert.x, texto.insert.y)),
        // Donut (barra fictícia/auxiliar): o centro é a posição da barra.
        EntityKind::Circle(c) => Some((c.center.x, c.center.y)),
        EntityKind::LwPolyline(poly) => {
            let a = poly.vertices.first()?;
            let b = poly.vertices.last().unwrap_or(a);
            Some(((a.p.x + b.p.x) / 2.0, (a.p.y + b.p.y) / 2.0))
        }
        _ => None,
    }
}

/// Recadastra no banco a entidade que está no desenho, com os campos do XData
/// dela (o `AdicionaItemInterligacao` do Compilar do Eletron). Devolve o `Id` do
/// registro ou `None` quando o XData não traz o que a tabela precisa.
fn recriar(banco: &anacad_data::Banco, documento: i64, tabela: &str, entidade: &DoDesenho) -> std::result::Result<Option<i64>, anacad_data::Erro> {
    match tabela {
        "Barra" => {
            let Some(codigo) = entidade.inteiro("codigo").filter(|codigo| *codigo > 0) else { return Ok(None) };
            let (x, y) = match (entidade.real("x1"), entidade.real("y1"), entidade.real("x2"), entidade.real("y2")) {
                (Some(x1), Some(y1), Some(x2), Some(y2)) => ((x1 + x2) / 2.0, (y1 + y2) / 2.0),
                _ => entidade.ponto.unwrap_or((0.0, 0.0)),
            };
            // Nome do desenho, base de tensão do XData (ou do `dados` da janela):
            // o registro recadastrado é o que o desenho mostra, não um padrão.
            let id = banco.criar_barra(documento, codigo, entidade.nome.as_deref(), x, y)?;
            let vbase = entidade.real("vbase").or_else(|| entidade.dados().and_then(|dados| dados.get("vbase").and_then(Value::as_f64)));
            if let Some(vbase) = vbase {
                banco.atualizar_barra_vbase(Some(documento), codigo, vbase)?;
            }
            banco.definir_handle("Barra", id, &entidade.handle)?;
            if let Some(dados) = entidade.campo("dados").filter(|dados| !dados.trim().is_empty() && dados.trim() != "{}") {
                banco.atualizar_dados_barra(&entidade.handle, dados)?;
            }
            Ok(Some(id))
        }
        "Linha" => {
            let barra = |codigo: Option<i64>| -> Option<i64> {
                codigo.and_then(|codigo| banco.barra_por_codigo(documento, codigo).ok().flatten()).map(|barra| barra.id)
            };
            let id = banco.criar_linha(documento, barra(entidade.inteiro("de")), barra(entidade.inteiro("para")))?;
            banco.definir_handle("Linha", id, &entidade.handle)?;
            if let Some(dados) = entidade.campo("dados").filter(|dados| !dados.trim().is_empty() && dados.trim() != "{}") {
                banco.atualizar_dados_linha(&entidade.handle, dados)?;
            }
            Ok(Some(id))
        }
        "Equipamento" => {
            let Some(tipo) = entidade.campo("tipo").filter(|tipo| !tipo.is_empty()) else { return Ok(None) };
            let codigo = entidade.inteiro("codigo").unwrap_or(0);
            let (x, y) = entidade.ponto.unwrap_or((0.0, 0.0));
            // O nome do equipamento no banco é o nome do tipo (`run_equipamento_inserir`).
            let id = banco.criar_equipamento(documento, tipo, codigo, tipo, x, y)?;
            banco.definir_handle("Equipamento", id, &entidade.handle)?;
            if let Some(dados) = entidade.campo("dados")
                && let Ok(valor) = serde_json::from_str::<Value>(dados)
            {
                banco.atualizar_dados_equipamento(&entidade.handle, dados)?;
                banco.sincronizar_equipamento(documento, tipo, &valor)?;
            }
            Ok(Some(id))
        }
        "Componente" => {
            let Some(tipo) = entidade.inteiro("tipo") else { return Ok(None) };
            let nome = entidade.campo("nome").unwrap_or("");
            let codigo = entidade.inteiro("codigo").filter(|codigo| *codigo != 0);
            let (x, y) = entidade.ponto.unwrap_or((0.0, 0.0));
            let id = banco.criar_componente(documento, tipo, codigo, nome, x, y)?;
            banco.definir_handle("Componente", id, &entidade.handle)?;
            Ok(Some(id))
        }
        "Legenda" => {
            let Some(tipo) = entidade.campo("tipo").filter(|tipo| !tipo.is_empty()) else { return Ok(None) };
            let numero = entidade.inteiro("numero").unwrap_or(0) as i32;
            let (x, y) = entidade.ponto.unwrap_or((0.0, 0.0));
            let id = banco.criar_legenda(documento, tipo, numero, None, numero, 0, x, y)?;
            banco.definir_handle("Legenda", id, &entidade.handle)?;
            Ok(Some(id))
        }
        _ => Ok(None),
    }
}

/// Regrava o XData da entidade com o `Id` do registro recriado, preservando os
/// outros campos.
fn atualizar_id_xdata(s: &mut Session, app: &str, handle: Handle, entidade: &DoDesenho, id: i64) -> Result<()> {
    let mut campos = entidade.campos.clone();
    match campos.iter_mut().find(|(chave, _)| chave == "id") {
        Some(par) => par.1 = id.to_string(),
        None => campos.insert(0, ("id".to_string(), id.to_string())),
    }
    let refs: Vec<(&str, &str)> = campos.iter().map(|(chave, valor)| (chave.as_str(), valor.as_str())).collect();
    marcar(s, handle, app, &refs)
}

/// O desenho de antes é o do documento corrente? O `uid` de antes é o do
/// documento que a entidade pertencia: outro documento (ABRIR, ativar,
/// desfazer) não pode ser lido como "o desenho todo saiu". Edição dentro de
/// bloco não é o desenho do documento. O comando interativo chega sem `uid`
/// (`Session::finish`) e roda no documento corrente.
fn mesmo_documento(s: &Session, uid: Option<u64>) -> bool {
    match s.state() {
        Ok(estado) => uid.is_none_or(|uid| uid == estado.uid) && estado.block_edit.is_none(),
        Err(_) => false,
    }
}

/// Cascata do apagar **no momento do comando**, não só no Sincronizar: a barra
/// que sai do desenho leva as linhas presas a ela, com o texto do circuito e
/// os textos de identificação dela — no desenho e no banco.
///
/// É o `onObjectEraseded` do Eletron, que apaga as derivações da conexão
/// apagada (`clsConexao.ApagarDerivacoesLinhas`); sem isso a linha ficaria
/// apontando uma barra que não existe até alguém apertar Sincronizar.
///
/// O vínculo vem do **desenho**: a linha guarda os códigos das barras no XData
/// (`de`/`para`, a referência que o `.lst` do ANAFAS tem no registro `C`), e é
/// por eles que ela é encontrada — vale mesmo antes do primeiro Sincronizar,
/// quando o banco ainda não conhece a linha. O banco só completa com o que já
/// estava registrado (`Linha.DeBarra`/`ParaBarra`) e é apagado junto.
///
/// O equipamento ligado à barra por um circuito (conexão) sai junto com ela;
/// o componente e o equipamento **sem** conexão que apontavam a barra perdem
/// o vínculo no desenho também (o `barra` do `ANACAD_COMP`, o
/// `barra`/`de`/`para` do `dados` do `ANACAD_EQ`), como o banco fica com
/// `Componente.Barra` NULL: esses continuam desenhados, mas sem barra.
///
/// Roda no fim de todo comando bem-sucedido
/// ([`super::AnacadModule::after_command`]) e compara o desenho de antes com o
/// de agora: as barras que saíram são as que têm XData `ANACAD_BARRA` no
/// desenho anterior e não existem mais. O comando interativo chega sem `uid`
/// (`Session::finish`) e roda no documento corrente; só um `uid` diferente é
/// troca de desenho, que não é apagar tudo.
///
/// O `UNDO` que traz a barra e as linhas de volta as recadastra no banco no
/// `anacad.sincronizar` seguinte (o passo de recadastro): o desenho continua
/// sendo a verdade.
///
/// Devolve quantas entidades saíram do desenho junto com as barras.
pub(super) fn cascata_barras_apagadas(s: &mut Session, antes: Option<&Drawing>, uid: Option<u64>) -> usize {
    let Some(antes) = antes else { return 0 };
    if !mesmo_documento(s, uid) {
        return 0;
    }
    let apagadas: Vec<(Handle, Option<String>)> = match s.doc() {
        Ok(doc) => antes
            .model
            .iter()
            .filter(|e| app_xdata(e) == Some(APP_BARRA))
            .filter(|e| doc.entity(e.handle).is_none())
            .map(|e| (e.handle, campo_xdata(e, "codigo").map(str::to_string)))
            .collect(),
        Err(_) => return 0,
    };
    if apagadas.is_empty() {
        return 0;
    }
    // As linhas presas, achadas no desenho pelo código da barra de cada ponta.
    let codigos: Vec<String> = apagadas.iter().filter_map(|(_, codigo)| codigo.clone()).collect();
    let mut linhas = linhas_presas_aos_codigos(s, &codigos);
    let handles_das_barras: Vec<String> = apagadas.iter().map(|(handle, _)| handle.hex()).collect();
    let (db, documento) = db_e_documento(s);
    if !db.is_empty()
        && let Ok(banco) = anacad_data::Banco::abrir(&db)
    {
        // O registro pode estar no banco sem o `de`/`para` no XData (importado
        // do `.lst`, janela antiga): o vínculo De/Para cobre esses casos.
        for (_, handle) in banco.linhas_das_barras(documento, &handles_das_barras).unwrap_or_default() {
            if !handle.is_empty() && !linhas.contains(&handle) {
                linhas.push(handle);
            }
        }
        // As linhas saem antes da barra: a FK `Linha.DeBarra`/`ParaBarra`
        // impede o `DELETE FROM Barra` com elas no lugar.
        for handle in &linhas {
            if let Ok(Some(id)) = banco.linha_id_por_handle(handle) {
                let _ = banco.apagar_linhas(&[id]);
            }
        }
        // A barra sai do banco também (e o equipamento fica sem ela, como no
        // Sincronizar): o desenho é a verdade no instante do apagar.
        let presentes = barras_no_desenho(s);
        let _ = banco.apagar_handles_ausentes("Barra", documento, &presentes);
    }
    let mut saidas = 0;
    for handle in &linhas {
        saidas += apagar_linha_do_desenho(s, handle);
    }
    // Os textos de identificação pertencem à barra: sem ela ficariam soltos.
    for (barra, _) in &apagadas {
        saidas += apagar_textos_ligados(s, APP_TEXTO_BARRA, *barra);
    }
    // As conexões presas às barras que saíram (o circuito equipamento-barra) saem junto,
    // e o equipamento ligado a essas conexões sai junto também.
    let mut conexoes_barras = conexoes_presas_aos_handles(s, &handles_das_barras);
    let banco = if db.is_empty() { None } else { anacad_data::Banco::abrir(&db).ok() };
    if let Some(banco) = &banco {
        for h in &handles_das_barras {
            if let Ok(handles) = banco.conexoes_por_entidade(documento, h) {
                for ch in handles {
                    if !ch.is_empty() && !conexoes_barras.contains(&ch) {
                        conexoes_barras.push(ch);
                    }
                }
            }
        }
    }
    // Os equipamentos das conexões que vão sair, antes de apagar qualquer
    // coisa (o vínculo está no XData das conexões e nos registros do banco).
    let equipamentos = equipamentos_das_conexoes(s, Some(antes), &conexoes_barras, &handles_das_barras, banco.as_ref(), documento);
    if let Some(banco) = &banco {
        let _ = banco.apagar_conexoes_das_entidades(documento, &handles_das_barras);
    }
    for handle in &conexoes_barras {
        saidas += apagar_conexao_do_desenho(s, handle);
    }
    for handle in &equipamentos {
        saidas += apagar_entidade_do_desenho(s, handle);
    }
    // O componente e o equipamento sem conexão da barra perdem o vínculo no
    // desenho, como o banco fica com `Componente.Barra` NULL.
    desvincular_das_barras(s, &codigos);
    saidas
}

/// Cascata **inversa** do apagar da **conexão** (o circuito de ligação entre o
/// equipamento e a barra ou a linha): apagar o circuito apaga o equipamento
/// ligado a ele, no desenho e no banco — o espelho de
/// [`cascata_equipamentos_apagados`], que apaga o circuito junto com o
/// equipamento.
///
/// Roda **antes** das cascatas de barra, linha e equipamento no
/// `after_command`: só dispara quando a conexão saiu pelo próprio comando
/// (o `ERASE` dela). Quando a barra ou a linha é apagada, a conexão sai na
/// cascata da barra/linha, depois desta — e o equipamento ligado à conexão
/// sai junto com ela, na cascata da barra/linha (o equipamento sem conexão
/// continua desenhado, desvinculado, como já era). E quando o equipamento é
/// apagado, as conexões dele ainda estão no desenho nesta rodada, então nada
/// dispara aqui; é a cascata do equipamento que as leva depois.
///
/// Se o equipamento e a outra ponta saíram no mesmo comando (seleção
/// múltipla), ou se a outra ponta já não está no desenho, esta cascata não
/// apaga o equipamento — mas a cascata da barra/linha apaga, pela conexão
/// que o ligava a ela no desenho de antes.
///
/// Devolve quantos equipamentos saíram do desenho junto com as conexões.
/// O registro do banco e as outras conexões do equipamento saem na cascata
/// de equipamentos, que roda em seguida e encontra o equipamento faltando.
pub(super) fn cascata_conexoes_apagadas(s: &mut Session, antes: Option<&Drawing>, uid: Option<u64>) -> usize {
    let Some(antes) = antes else { return 0 };
    if !mesmo_documento(s, uid) {
        return 0;
    }
    // As conexões que saíram, com as pontas `de`/`para` do desenho de antes
    // (a entidade já não existe para ler o XData dela).
    let apagadas: Vec<(Handle, Option<String>, Option<String>)> = match s.doc() {
        Ok(doc) => antes
            .model
            .iter()
            .filter(|e| app_xdata(e) == Some(APP_CONEXAO))
            .filter(|e| doc.entity(e.handle).is_none())
            .map(|e| (e.handle, campo_xdata(e, "de").map(str::to_string), campo_xdata(e, "para").map(str::to_string)))
            .collect(),
        Err(_) => return 0,
    };
    if apagadas.is_empty() {
        return 0;
    }
    // As pontas que são equipamento no desenho de antes, cuja outra ponta
    // continua no desenho (só o circuito saiu).
    let mut equipamentos: Vec<Handle> = Vec::new();
    for (_, de, para) in &apagadas {
        for (ponta, outra) in [(de, para), (para, de)] {
            let Some(ponta) = ponta else { continue };
            let Some(handle) = Handle::parse_hex(ponta) else { continue };
            let eh_equipamento = antes.model.iter().find(|e| e.handle == handle).is_some_and(|e| app_xdata(e) == Some(APP_EQ));
            if !eh_equipamento || equipamentos.contains(&handle) {
                continue;
            }
            let outra_no_desenho = match (outra, s.doc()) {
                (None, _) => true,
                (Some(outra), Ok(doc)) => Handle::parse_hex(outra).is_none_or(|h| doc.entity(h).is_some()),
                (Some(_), Err(_)) => false,
            };
            let ainda_no_desenho = s.doc().is_ok_and(|doc| doc.entity(handle).is_some());
            if outra_no_desenho && ainda_no_desenho {
                equipamentos.push(handle);
            }
        }
    }
    if equipamentos.is_empty() {
        return 0;
    }
    let Ok(doc) = s.doc_mut() else { return 0 };
    equipamentos.iter().filter(|handle| doc.remove_entity(**handle).is_some()).count()
}

/// Cascata do apagar do **equipamento**: o vínculo de cálculo que ele criou sai
/// junto com ele, no desenho e no banco, assim como o traço da conexão (o circuito
/// que liga o equipamento à barra ou à linha).
///
/// O trafo, o compensador série e a mútua gravam o `Circuito` De/Para
/// ([`Banco::sincronizar_equipamento`]), e o reator grava o `Shunt`; sem esta
/// cascata o circuito ficaria apontando um equipamento que não existe mais até
/// alguém apertar Sincronizar (`apagar_handles_ausentes` já limpa os dois, e é
/// ela que roda aqui) — e, como no apagar da barra, o desenho é a verdade no
/// instante do comando.
///
/// Devolve quantas entidades saíram do desenho (o equipamento e suas conexões).
pub(super) fn cascata_equipamentos_apagados(s: &mut Session, antes: Option<&Drawing>, uid: Option<u64>) -> usize {
    let Some(antes) = antes else { return 0 };
    if !mesmo_documento(s, uid) {
        return 0;
    }
    let apagados: Vec<String> = match s.doc() {
        Ok(doc) => {
            antes.model.iter().filter(|e| app_xdata(e) == Some(APP_EQ)).filter(|e| doc.entity(e.handle).is_none()).map(|e| e.handle.hex()).collect()
        }
        Err(_) => return 0,
    };
    if apagados.is_empty() {
        return 0;
    }
    // As conexões presas aos equipamentos apagados (o traço que liga o equipamento à barra ou linha).
    let mut conexoes = conexoes_presas_aos_handles(s, &apagados);
    let (db, documento) = db_e_documento(s);
    let banco = if db.is_empty() { None } else { anacad_data::Banco::abrir(&db).ok() };
    if let Some(banco) = &banco {
        for h in &apagados {
            if let Ok(handles) = banco.conexoes_por_entidade(documento, h) {
                for ch in handles {
                    if !ch.is_empty() && !conexoes.contains(&ch) {
                        conexoes.push(ch);
                    }
                }
            }
        }
        let _ = banco.apagar_conexoes_das_entidades(documento, &apagados);
        let presentes = equipamentos_no_desenho(s);
        let _ = banco.apagar_handles_ausentes("Equipamento", documento, &presentes);
    }
    let mut saidas = apagados.len();
    for handle in &conexoes {
        saidas += apagar_conexao_do_desenho(s, handle);
    }
    saidas
}

/// Handles dos equipamentos que continuam no desenho.
fn equipamentos_no_desenho(s: &Session) -> Vec<String> {
    match s.doc() {
        Ok(doc) => doc.model.iter().filter(|e| app_xdata(e) == Some(APP_EQ)).map(|e| e.handle.hex()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Handles das linhas desenhadas presas a alguma das barras de `codigos`: o
/// vínculo De/Para está no XData da linha (`de`/`para`), como o `ref0..ref3`
/// do registro `C` no `.lst` do ANAFAS.
fn linhas_presas_aos_codigos(s: &Session, codigos: &[String]) -> Vec<String> {
    if codigos.is_empty() {
        return Vec::new();
    }
    let Ok(doc) = s.doc() else { return Vec::new() };
    doc.model
        .iter()
        .filter(|e| app_xdata(e) == Some(APP_LINHA))
        .filter(|e| ["de", "para"].iter().any(|campo| campo_xdata(e, campo).is_some_and(|codigo| codigos.iter().any(|apagado| apagado == codigo))))
        .map(|e| e.handle.hex())
        .collect()
}

/// Pares `chave`/`valor` do XData (o que vem depois do app name).
fn campos_xdata(e: &cadcraft_doc::Entity) -> Vec<(String, String)> {
    e.common
        .xdata
        .iter()
        .filter(|tag| tag.code == 1000)
        .map(|tag| tag.value.clone())
        .collect::<Vec<String>>()
        .chunks(2)
        .filter(|par| par.len() == 2)
        .map(|par| (par[0].clone(), par[1].clone()))
        .collect()
}

/// Tira do desenho o vínculo com as barras que saíram, como o banco faz com
/// `Componente.Barra = NULL`: o componente (o ponto do `.lst`) perde o `barra`
/// do próprio XData, e o equipamento perde `barra`/`de`/`para` do `dados` que a
/// janela de dados grava — zerados, para a janela continuar lendo o campo — e o
/// `T1`/`T2` do bloco quando mostrava o código. Devolve quantas entidades
/// mudaram.
fn desvincular_das_barras(s: &mut Session, codigos: &[String]) -> usize {
    if codigos.is_empty() {
        return 0;
    }
    let apagado = |valor: &str| codigos.iter().any(|codigo| codigo == valor);
    let mut mudancas: Vec<(Handle, &'static str, Vec<(String, String)>)> = Vec::new();
    {
        let Ok(doc) = s.doc() else { return 0 };
        for e in doc.model.iter() {
            match app_xdata(e) {
                Some("ANACAD_COMP") => {
                    let mut campos = campos_xdata(e);
                    let antes = campos.len();
                    campos.retain(|(chave, valor)| !(chave == "barra" && apagado(valor)));
                    if campos.len() != antes {
                        mudancas.push((e.handle, "ANACAD_COMP", campos));
                    }
                }
                Some("ANACAD_EQ") => {
                    let mut campos = campos_xdata(e);
                    let Some(pos) = campos.iter().position(|(chave, _)| chave == "dados") else { continue };
                    let Ok(mut dados) = serde_json::from_str::<Value>(&campos[pos].1) else { continue };
                    let Some(objeto) = dados.as_object_mut() else { continue };
                    let mut mudou = false;
                    for chave in ["barra", "de", "para"] {
                        let codigo = objeto.get(chave).and_then(Value::as_i64);
                        if codigo.is_some_and(|codigo| apagado(&codigo.to_string())) {
                            objeto.insert(chave.to_string(), json!(0));
                            mudou = true;
                        }
                    }
                    if !mudou {
                        continue;
                    }
                    campos[pos].1 = serde_json::to_string(&dados).unwrap_or_else(|_| campos[pos].1.clone());
                    mudancas.push((e.handle, "ANACAD_EQ", campos));
                }
                _ => {}
            }
        }
    }
    let mut mudadas = 0;
    for (handle, app, campos) in mudancas {
        let refs: Vec<(&str, &str)> = campos.iter().map(|(chave, valor)| (chave.as_str(), valor.as_str())).collect();
        if marcar(s, handle, app, &refs).is_err() {
            continue;
        }
        mudadas += 1;
        if app == "ANACAD_EQ" {
            let _ = s.doc_mut().map(|doc| {
                let _ = doc.modify_entity(handle, |e| {
                    if let EntityKind::Insert(ins) = &mut e.kind {
                        for atributo in &mut ins.attribs {
                            if matches!(atributo.tag.as_str(), "T1" | "T2") && apagado(&atributo.text.value) {
                                atributo.text.value.clear();
                            }
                        }
                    }
                });
            });
        }
    }
    mudadas
}

/// Tira do desenho os textos ligados a `alvo` (`app` = app name do texto) e
/// devolve quantos saíram.
fn apagar_textos_ligados(s: &mut Session, app: &str, alvo: Handle) -> usize {
    let alvos: Vec<Handle> = match s.doc() {
        Ok(doc) => textos_ligados(doc, app, alvo).into_iter().map(|(_, handle)| handle).collect(),
        Err(_) => Vec::new(),
    };
    let Ok(doc) = s.doc_mut() else { return 0 };
    alvos.into_iter().filter(|handle| doc.remove_entity(*handle).is_some()).count()
}

/// Handles das barras desenhadas (XData `ANACAD_BARRA`) no espaço modelo.
fn barras_no_desenho(s: &Session) -> Vec<String> {
    match s.doc() {
        Ok(doc) => doc
            .model
            .iter()
            .filter(|e| e.common.xdata.first().is_some_and(|tag| tag.code == 1001 && tag.value == "ANACAD_BARRA"))
            .map(|e| e.handle.hex())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Tira do desenho a linha que saiu do banco junto com a barra: o traço e o
/// texto do circuito, que aponta o handle dela no XData (`ANACAD_LINHA_TEXTO`).
/// Devolve quantas entidades saíram.
fn apagar_linha_do_desenho(s: &mut Session, linha: &str) -> usize {
    let Some(dono) = Handle::parse_hex(linha) else { return 0 };
    let alvos: Vec<Handle> = match s.doc() {
        Ok(doc) => {
            // O traço pode já ter saído (o `ERASE` do próprio comando): o texto
            // do circuito é achado pelo `handle` dele no XData, que não muda.
            let mut alvos: Vec<Handle> = doc.entity(dono).map(|_| dono).into_iter().collect();
            alvos.extend(textos_ligados(doc, APP_TEXTO_LINHA, dono).into_iter().map(|(_, h)| h));
            alvos
        }
        Err(_) => Vec::new(),
    };
    let Ok(doc) = s.doc_mut() else { return 0 };
    let mut n = 0;
    for handle in alvos {
        if doc.remove_entity(handle).is_some() {
            n += 1;
        }
    }
    n
}

/// Handles das conexões (polilinhas ANACAD_CONEXAO) presas a alguma das entidades em `handles`:
/// o vínculo De/Para está no XData da conexão (`de`/`para`).
fn conexoes_presas_aos_handles(s: &Session, handles: &[String]) -> Vec<String> {
    if handles.is_empty() {
        return Vec::new();
    }
    let Ok(doc) = s.doc() else { return Vec::new() };
    doc.model
        .iter()
        .filter(|e| app_xdata(e) == Some(APP_CONEXAO))
        .filter(|e| {
            let de = campo_xdata(e, "de");
            let para = campo_xdata(e, "para");
            handles.iter().any(|h| de.is_some_and(|d| d == h) || para.is_some_and(|p| p == h))
        })
        .map(|e| e.handle.hex())
        .collect()
}

/// Handles dos equipamentos ligados às conexões de `conexoes` (o circuito de
/// ligação): as pontas `de`/`para` das conexões no desenho, os registros do
/// banco e, cobrindo a seleção múltipla, as conexões do desenho de antes que
/// prendiam alguma das entidades de `alvos` (a barra ou a linha apagada).
fn equipamentos_das_conexoes(
    s: &Session,
    antes: Option<&Drawing>,
    conexoes: &[String],
    alvos: &[String],
    banco: Option<&anacad_data::Banco>,
    documento: i64,
) -> Vec<String> {
    let mut saidas: Vec<String> = Vec::new();
    let empurrar = |ponta: &str, saidas: &mut Vec<String>| {
        if ponta.is_empty() || saidas.contains(&ponta.to_string()) {
            return;
        }
        let eh_equipamento = antes
            .map(|antes| {
                antes.model.iter().find(|e| Handle::parse_hex(ponta).is_some_and(|h| h == e.handle)).is_some_and(|e| app_xdata(e) == Some(APP_EQ))
            })
            .unwrap_or(false)
            || s.doc().ok().and_then(|doc| Handle::parse_hex(ponta).and_then(|h| doc.entity(h))).is_some_and(|e| app_xdata(e) == Some(APP_EQ));
        if eh_equipamento {
            saidas.push(ponta.to_string());
        }
    };
    // As pontas das conexões que vão sair, no desenho atual.
    if let Ok(doc) = s.doc() {
        for handle in conexoes {
            let Some(conexao) = Handle::parse_hex(handle).and_then(|h| doc.entity(h)) else { continue };
            for campo in ["de", "para"] {
                if let Some(ponta) = campo_xdata(conexao, campo) {
                    empurrar(ponta, &mut saidas);
                }
            }
        }
    }
    // As pontas registradas no banco.
    if let Some(banco) = banco {
        for registro in banco.conexoes(documento).unwrap_or_default() {
            if registro.handle.as_deref().is_some_and(|h| conexoes.contains(&h.to_string())) {
                if let Some(ponta) = &registro.de_handle {
                    empurrar(ponta, &mut saidas);
                }
                if let Some(ponta) = &registro.para_handle {
                    empurrar(ponta, &mut saidas);
                }
            }
        }
    }
    // As conexões do desenho de antes presas às entidades apagadas (a
    // seleção múltipla já tirou a conexão do desenho atual).
    if let Some(antes) = antes {
        for e in antes.model.iter() {
            if app_xdata(e) != Some(APP_CONEXAO) {
                continue;
            }
            let de = campo_xdata(e, "de");
            let para = campo_xdata(e, "para");
            let presa =
                conexoes.contains(&e.handle.hex()) || [de, para].iter().any(|ponta| ponta.is_some_and(|ponta| alvos.contains(&ponta.to_string())));
            if !presa {
                continue;
            }
            for ponta in [de, para].into_iter().flatten() {
                empurrar(ponta, &mut saidas);
            }
        }
    }
    saidas
}

/// Tira do desenho a entidade de `handle` (o bloco do equipamento). Devolve 1
/// se saiu, 0 se não existia.
fn apagar_entidade_do_desenho(s: &mut Session, handle: &str) -> usize {
    let Some(handle) = Handle::parse_hex(handle) else { return 0 };
    let Ok(doc) = s.doc_mut() else { return 0 };
    if doc.remove_entity(handle).is_some() { 1 } else { 0 }
}

/// Tira do desenho o traço da conexão (polilinha ANACAD_CONEXAO) que ligava o equipamento
/// à barra ou à linha. Devolve 1 se saiu, 0 se não existia.
fn apagar_conexao_do_desenho(s: &mut Session, conexao: &str) -> usize {
    let Some(handle) = Handle::parse_hex(conexao) else { return 0 };
    let Ok(doc) = s.doc_mut() else { return 0 };
    if doc.remove_entity(handle).is_some() { 1 } else { 0 }
}

/// Cascata do apagar da **linha** no momento do comando: o texto do circuito
/// (`ANACAD_LINHA_TEXTO`, ligado ao dono pelo `handle`) e o registro da tabela
/// `Linha` saem junto com o traço, sem esperar o Sincronizar — é o mesmo
/// `onObjectEraseded` do Eletron, agora na derivação: apagou a linha, apagou o
/// circuito dela.
///
/// A identidade é o `Handle` da linha (o mesmo que o banco guarda em
/// `Linha.Handle`), e o vínculo do texto com ela vem do XData do texto, então
/// funciona com o traço já fora do desenho. Devolve quantas entidades do
/// desenho saíram junto (o texto; o traço já saiu por conta do comando).
pub(super) fn cascata_linhas_apagadas(s: &mut Session, antes: Option<&Drawing>, uid: Option<u64>) -> usize {
    let Some(antes) = antes else { return 0 };
    if !mesmo_documento(s, uid) {
        return 0;
    }
    let apagadas: Vec<String> = match s.doc() {
        Ok(doc) => antes
            .model
            .iter()
            .filter(|e| app_xdata(e) == Some(APP_LINHA))
            .filter(|e| doc.entity(e.handle).is_none())
            .map(|e| e.handle.hex())
            .collect(),
        Err(_) => return 0,
    };
    if apagadas.is_empty() {
        return 0;
    }
    let (db, documento) = db_e_documento(s);
    let banco = if db.is_empty() { None } else { anacad_data::Banco::abrir(&db).ok() };
    let _ = documento;
    let mut saidas = 0;
    for handle in &apagadas {
        saidas += apagar_linha_do_desenho(s, handle);
        // A linha apagada não volta: o registro sai do banco na hora, como no
        // Sincronizar. Sem banco aberto ou sem registro, nada a fazer.
        if let Some(banco) = &banco
            && let Ok(Some(id)) = banco.linha_id_por_handle(handle)
        {
            let _ = banco.apagar_linhas(&[id]);
        }
    }
    // As conexões presas às linhas apagadas (derivações para equipamentos) saem junto,
    // e o equipamento ligado a essas conexões sai junto também.
    let mut conexoes_linhas = conexoes_presas_aos_handles(s, &apagadas);
    if let Some(banco) = &banco {
        for h in &apagadas {
            if let Ok(handles) = banco.conexoes_por_entidade(documento, h) {
                for ch in handles {
                    if !ch.is_empty() && !conexoes_linhas.contains(&ch) {
                        conexoes_linhas.push(ch);
                    }
                }
            }
        }
    }
    let equipamentos = equipamentos_das_conexoes(s, Some(antes), &conexoes_linhas, &apagadas, banco.as_ref(), documento);
    if let Some(banco) = &banco {
        let _ = banco.apagar_conexoes_das_entidades(documento, &apagadas);
    }
    for handle in &conexoes_linhas {
        saidas += apagar_conexao_do_desenho(s, handle);
    }
    for handle in &equipamentos {
        saidas += apagar_entidade_do_desenho(s, handle);
    }
    saidas
}

#[cfg(test)]
#[path = "sincronizar_tests.rs"]
mod tests;
