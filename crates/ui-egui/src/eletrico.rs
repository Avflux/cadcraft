//! Interface do modulo **eletrico**: o painel dockado e o comando
//! `ui.ele.painel`.
//!
//! A UI e fina: os botoes chamam os comandos do motor (`app.run`, `ele.*`)
//! e o estado da tela (banco, pagina, ultimo resultado) vive na memoria do egui,
//! como no painel do anacad.

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::CadApp;
use crate::domain::{DomainUi, UiCommand};

/// O menu que o modulo acrescenta a barra de menus.
pub const MENU: &str = "Elétrico";
/// A aba do Tool Sets do módulo.
pub const TAB: &str = "Elétrico";

const ID_ESTADO: &str = "eletrico.estado";
/// Tool Sets do módulo: (grupo, ferramentas grandes, ferramentas pequenas).
pub const TOOLSET: &[(&str, &[crate::palettes::Tool], &[crate::palettes::Tool])] = &[
    (
        "Páginas",
        &[
            (crate::icons::Icon::Eletrico, "ui.ele.painel", "Painel Elétrico"),
            (crate::icons::Icon::Text, "ui.ele.titulos", "Editar Títulos"),
            (crate::icons::Icon::Text, "ui.ele.titulos_textos", "Editar Títulos (Textos)"),
        ],
        &[],
    ),
    (
        "Projeto",
        &[
            (crate::icons::Icon::Open, "ui.ele.banco", "Banco do Projeto"),
            (crate::icons::Icon::AnacadLinha, "ui.ele.circuitos", "Editar Circuitos"),
            (crate::icons::Icon::Text, "ui.ele.revisao", "Revisão Simplificada"),
            (crate::icons::Icon::Text, "ui.ele.revisao_completa", "Revisão Completa"),
            (crate::icons::Icon::Text, "ui.ele.xdata", "Pesquisa XDatas"),
        ],
        &[],
    ),
];

/// Fileira do módulo na Tool Bar (painel, títulos, banco e circuitos).
pub const TOOLBAR: &[(crate::icons::Icon, &str, &str, Option<&str>)] = &[
    (crate::icons::Icon::Eletrico, "ui.ele.painel", "Painel Elétrico", None),
    (crate::icons::Icon::Text, "ui.ele.titulos", "Editar Títulos", None),
    (crate::icons::Icon::Text, "ui.ele.titulos_textos", "Editar Títulos (Textos)", None),
    (crate::icons::Icon::Open, "ui.ele.banco", "Banco do Projeto", None),
    (crate::icons::Icon::AnacadLinha, "ui.ele.circuitos", "Editar Circuitos", None),
    (crate::icons::Icon::Text, "ui.ele.revisao", "Revisão Simplificada", None),
    (crate::icons::Icon::Text, "ui.ele.revisao_completa", "Revisão Completa", None),
    (crate::icons::Icon::Text, "ui.ele.xdata", "Pesquisa XDatas", None),
];


/// Comandos so de interface: `(id, rotulo, caminho de menu, atalho)`.
pub const UI_COMMANDS: &[UiCommand] = &[(
    "ui.ele.painel",
    "Painel Elétrico",
    &["Elétrico", "Painel"],
    None,
),
    (
        "ui.ele.xdata",
        "Pesquisa XDatas",
        &["Elétrico", "Pesquisa XDatas"],
        None,
    ),
    (
        "ui.ele.titulos",
        "Editar Títulos",
        &["Elétrico", "Editar Títulos"],
        None,
    ),
    (
        "ui.ele.banco",
        "Banco do Projeto",
        &["Elétrico", "Banco do Projeto"],
        None,
    ),
    (
        "ui.ele.circuitos",
        "Editar Circuitos",
        &["Elétrico", "Editar Circuitos"],
        None,
    ),
];

#[derive(Clone, Default)]
struct Estado {
    arquivo: String,
    pagina: String,
    titulo1: String,
    unidade: String,
    lista: Vec<String>,
    mensagem: String,
    /// Desenho de que o painel ja leu o banco (troca de arquivo = troca de banco).
    carregado: String,
}

/// O modulo como a casca o ve.
pub struct UiEletrico;

impl DomainUi for UiEletrico {
    fn id(&self) -> &'static str {
        "eletrico"
    }
    fn ui_commands(&self) -> &'static [UiCommand] {
        UI_COMMANDS
    }
    fn tabs(&self) -> &'static [&'static str] {
        &[TAB]
    }
    fn toolset(&self, tab: &str) -> Option<&'static [crate::domain::ToolGroup]> {
        (tab == TAB).then_some(TOOLSET)
    }
    fn toolbar(&self) -> Option<crate::domain::ToolbarRow> {
        Some(TOOLBAR)
    }
    /// O banco do desenho, relido a cada troca de aba: e o XData do proprio
    /// arquivo que decide qual `.db` o painel usa, como o `LerXDataConfiguracao`
    /// do original.
    fn begin_frame(&self, app: &mut CadApp, ctx: &egui::Context) {
        sincronizar_desenho(app, ctx);
    }
    fn dialogs(&self, app: &mut CadApp, ctx: &egui::Context) {
        janela_titulos(app, ctx);
        janela_textos(app, ctx);
        janela_revisao(app, ctx);
        janela_revisao_completa(app, ctx);
        janela_xdata(app, ctx);
        janela_banco(app, ctx);
        janela_circuitos(app, ctx);
    }
    fn panel(&self, panel: crate::docking::Panel, app: &mut CadApp, ui: &mut egui::Ui) -> bool {
        if panel != crate::docking::Panel::Eletrico {
            return false;
        }
        egui::ScrollArea::vertical()
            .id_salt("dock-eletrico")
            .auto_shrink([false, false])
            .show(ui, |ui| painel(app, ui));
        true
    }
    fn route(&self, app: &mut CadApp, id: &str, _params: &Value) -> Option<Result<Value, String>> {
        match id {
            "ui.ele.painel" => Some(abrir(app)),
            "ui.ele.titulos" => Some(abrir_titulos(app)),
            "ui.ele.titulos_textos" => Some(abrir_titulos_textos(app)),
            "ui.ele.revisao" => Some(abrir_revisao(app)),
            "ui.ele.revisao_completa" => Some(abrir_revisao_completa(app)),
            "ui.ele.xdata" => Some(abrir_xdata(app)),
            "ui.ele.banco" => Some(abrir_banco(app)),
            "ui.ele.circuitos" => Some(abrir_circuitos(app)),
            _ => None,
        }
    }
}

/// O modulo registrado em [`crate::domain::builtin`].
pub static ELETRICO_UI: UiEletrico = UiEletrico;

/// Identidade do desenho ativo (caminho, ou aba e titulo quando ainda nao foi salvo).
fn chave_do_desenho(app: &CadApp) -> String {
    match app.session.docs.get(app.session.active) {
        Some(d) => d
            .path
            .clone()
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| format!("#{} {}", app.session.active, d.title)),
        None => String::new(),
    }
}

/// Troca de arquivo, troca de banco: relê o vinculo do desenho uma vez por aba.
fn sincronizar_desenho(app: &mut CadApp, ctx: &egui::Context) {
    let desenho = chave_do_desenho(app);
    if desenho.is_empty() {
        return;
    }
    let mut estado: Estado =
        ctx.data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    if estado.carregado == desenho {
        return;
    }
    estado.carregado = desenho;
    estado.arquivo = cadcraft_engine::cmd::eletrico::banco_do_desenho(&app.session);
    estado.lista.clear();
    estado.mensagem = if estado.arquivo.is_empty() {
        "desenho sem banco: vincule um .db".to_string()
    } else {
        String::new()
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

/// Abre (ou ativa) o painel dockado, como o QuickCalc faz.
pub fn abrir(app: &mut CadApp) -> Result<Value, String> {
    let panel = crate::docking::Panel::Eletrico;
    crate::docking::command(app, false, &json!({ "operation": "open", "panel": panel.id() }))
}

fn painel(app: &mut CadApp, ui: &mut Ui) {
    let mut estado: Estado = ui
        .ctx()
        .data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    ui.add_space(6.0);
    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);

    ui.label("Banco do projeto (.db)");
    ui.add(
        egui::TextEdit::singleline(&mut estado.arquivo)
            .desired_width(f32::INFINITY)
            .hint_text(r"C:\...\projeto.db"),
    );
    ui.horizontal(|ui| {
        if ui.button("Abrir / criar").clicked() {
            estado.mensagem = executar(
                app,
                "ele.banco.abrir",
                json!({ "arquivo": estado.arquivo.trim() }),
                |v| match v.get("paginas").and_then(Value::as_u64) {
                    Some(total) => format!("banco aberto: {total} página(s)"),
                    None => "banco aberto".to_string(),
                },
            );
        }
        if ui.button("Listar páginas").clicked() {
            let arquivo = estado.arquivo.trim().to_string();
            match app.run("ele.pagina.listar", json!({ "arquivo": arquivo })) {
                Ok(v) => {
                    estado.lista = v
                        .get("paginas")
                        .and_then(Value::as_array)
                        .map(|lista| {
                            lista
                                .iter()
                                .map(|p| {
                                    let chave = p.get("pagina").and_then(Value::as_str).unwrap_or("?");
                                    let titulo = p.get("titulo1").and_then(Value::as_str).unwrap_or("");
                                    if titulo.is_empty() {
                                        chave.to_string()
                                    } else {
                                        format!("{chave}  {titulo}")
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                    estado.mensagem = format!("{total} página(s) no banco");
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Vincular ao desenho").clicked() {
            let arquivo = estado.arquivo.trim().to_string();
            estado.mensagem = executar(
                app,
                "ele.projeto.gravar",
                json!({ "banco": arquivo }),
                |v| {
                    let handle = v.get("handle").and_then(Value::as_str).unwrap_or("?");
                    format!("banco vinculado ao desenho ({handle})")
                },
            );
        }
        if ui.button("Ler do desenho").clicked() {
            match app.run("ele.pagina.ler", json!({})) {
                Ok(v) => {
                    estado.lista = lista_do_json(&v);
                    let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                    estado.mensagem = format!("{total} página(s) no desenho");
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
        if ui.button("Editar Títulos").clicked() {
            let _ = abrir_titulos(app);
        }
        if ui.button("Banco do Projeto").clicked() {
            let _ = abrir_banco(app);
        }
        if ui.button("Editar Circuitos").clicked() {
            let _ = abrir_circuitos(app);
        }
    });

    ui.separator();
    ui.label("Página");
    ui.add(egui::TextEdit::singleline(&mut estado.pagina).desired_width(f32::INFINITY).hint_text("12A"));
    ui.label("Título 1");
    ui.add(egui::TextEdit::singleline(&mut estado.titulo1).desired_width(f32::INFINITY));
    ui.label("Unidade");
    ui.add(egui::TextEdit::singleline(&mut estado.unidade).desired_width(f32::INFINITY));
    ui.horizontal(|ui| {
        if ui.button("Gravar página").clicked() {
            estado.mensagem = gravar_pagina(app, &estado, "ele.pagina.gravar");
        }
        if ui.button("Gravar no desenho").clicked() {
            estado.mensagem = gravar_pagina(app, &estado, "ele.pagina.escrever");
        }
    });
    if ui.button("Sincronizar banco com o desenho").clicked() {
        estado.mensagem = executar(
            app,
            "ele.pagina.sincronizar",
            json!({ "banco": estado.arquivo.trim() }),
            |v| {
                let gravadas = v.get("gravadas").and_then(Value::as_u64).unwrap_or(0);
                format!("banco sincronizado: {gravadas} página(s)")
            },
        );
    }

    if !estado.lista.is_empty() {
        ui.separator();
        for item in &estado.lista {
            ui.monospace(item);
        }
    }
    if !estado.mensagem.is_empty() {
        ui.separator();
        ui.label(&estado.mensagem);
    }

    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

/// Roda um comando do motor e resume o resultado (ou o erro) numa mensagem.
fn executar(
    app: &mut CadApp,
    cmd: &str,
    params: Value,
    resumo: impl FnOnce(&Value) -> String,
) -> String {
    match app.run(cmd, params) {
        Ok(v) => resumo(&v),
        Err(e) => format!("erro: {e}"),
    }
}

/// Roda `pagina.gravar` ou `pagina.escrever` com os campos do painel.
fn gravar_pagina(app: &mut CadApp, estado: &Estado, cmd: &str) -> String {
    executar(
        app,
        cmd,
        json!({
            "arquivo": estado.arquivo.trim(),
            "banco": estado.arquivo.trim(),
            "pagina": estado.pagina.trim(),
            "titulo1": estado.titulo1.trim(),
            "unidade": estado.unidade.trim(),
        }),
        |v| {
            let chave = v.get("pagina").and_then(Value::as_str).unwrap_or("?");
            match v.get("gravado_no_banco").and_then(Value::as_bool) {
                Some(true) => format!("página {chave} gravada no desenho e no banco"),
                Some(false) => format!("página {chave} gravada no desenho (sem banco)"),
                None => {
                    let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                    format!("página {chave} gravada ({total} no banco)")
                }
            }
        },
    )
}

/// Lista de paginas de um resultado de comando, como o painel mostra.
fn lista_do_json(v: &Value) -> Vec<String> {
    v.get("paginas")
        .and_then(Value::as_array)
        .map(|lista| {
            lista
                .iter()
                .map(|p| {
                    let chave = p.get("pagina").and_then(Value::as_str).unwrap_or("?");
                    let titulo = p.get("titulo1").and_then(Value::as_str).unwrap_or("");
                    if titulo.is_empty() {
                        chave.to_string()
                    } else {
                        format!("{chave}  {titulo}")
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

const ID_JANELA: &str = "eletrico.janela_titulos";
const ID_JANELA_BANCO: &str = "eletrico.janela_banco";

/// Uma linha da grade de títulos: uma página e seus campos editáveis.
#[derive(Clone, Default)]
struct LinhaTitulos {
    pagina: String,
    titulo1: String,
    titulo2: String,
    titulo3: String,
    titulo4: String,
    unidade: String,
    alternativo: String,
}

/// Estado da janela **Editar Títulos**, na memória do egui (como o painel).
#[derive(Clone, Default)]
struct JanelaTitulos {
    lista: Vec<LinhaTitulos>,
    carregada: bool,
    pagina: String,
    titulo1: String,
    titulo2: String,
    titulo3: String,
    titulo4: String,
    unidade: String,
    alternativo: String,
    de: String,
    para: String,
    mensagem: String,
    /// Célula selecionada na grade: (página, coluna 0–6).
    celula: Option<(String, usize)>,
    /// Copiado pela janela (`Copiar`): o valor de uma célula.
    clipboard: Option<CopiaCelula>,
    /// Texto do `Exportar`/`Importar` (TSV, com cabeçalho).
    texto_tsv: String,
}

/// O que `Copiar` guardou: o valor de uma célula da grade (como `method_2`).
#[derive(Clone, Default)]
struct CopiaCelula {
    /// O texto copiado.
    valor: String,
}

/// Abre a janela **Editar Títulos** (`frmEditaTitulos` no original).
pub fn abrir_titulos(app: &mut CadApp) -> Result<Value, String> {
    app.ui.eletrico_janela_titulos = true;
    Ok(json!({ "janela": "titulos" }))
}

/// Texto de um campo do resultado de um comando.
fn campo_json(v: &Value, chave: &str) -> String {
    v.get(chave).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Lê as páginas do desenho para a grade (e pede a janela para redesenhar).
fn carregar_lista(app: &mut CadApp, janela: &mut JanelaTitulos) {
    janela.carregada = true;
    match app.run("ele.pagina.ler", json!({})) {
        Ok(v) => {
            janela.lista = v
                .get("paginas")
                .and_then(Value::as_array)
                .map(|paginas| {
                    paginas
                        .iter()
                        .map(|p| LinhaTitulos {
                            pagina: campo_json(p, "pagina"),
                            titulo1: campo_json(p, "titulo1"),
                            titulo2: campo_json(p, "titulo2"),
                            titulo3: campo_json(p, "titulo3"),
                            titulo4: campo_json(p, "titulo4"),
                            unidade: campo_json(p, "unidade"),
                            alternativo: campo_json(p, "alternativo"),
                        })
                        .collect()
                })
                .unwrap_or_default();
            janela.mensagem.clear();
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Parâmetros de um `ele.pagina.escrever` com os campos da linha.
fn linha_para_json(janela: &JanelaTitulos) -> Value {
    json!({
        "pagina": janela.pagina.trim(),
        "titulo1": janela.titulo1.trim(),
        "titulo2": janela.titulo2.trim(),
        "titulo3": janela.titulo3.trim(),
        "titulo4": janela.titulo4.trim(),
        "unidade": janela.unidade.trim(),
        "alternativo": janela.alternativo.trim(),
    })
}

/// Grava a página da janela no desenho (e espelha no `.db`) e relê a grade.
fn gravar_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    if janela.pagina.trim().is_empty() {
        janela.mensagem = "informe a página (nome da camada)".to_string();
        return;
    }
    match app.run("ele.pagina.escrever", linha_para_json(janela)) {
        Ok(v) => {
            let criados = v.get("criados").and_then(Value::as_u64).unwrap_or(0);
            let atualizados = v.get("atualizados").and_then(Value::as_u64).unwrap_or(0);
            let nome = janela.pagina.trim().to_string();
            carregar_lista(app, janela);
            janela.mensagem = format!("página {nome}: {criados} criado(s), {atualizados} atualizado(s)");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Limpa os títulos da página (unidade e alternativo ficam como estão).
fn limpar_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    if janela.pagina.trim().is_empty() {
        janela.mensagem = "informe a página (nome da camada)".to_string();
        return;
    }
    let nome = janela.pagina.trim().to_string();
    janela.titulo1.clear();
    janela.titulo2.clear();
    janela.titulo3.clear();
    janela.titulo4.clear();
    match app.run("ele.pagina.escrever", linha_para_json(janela)) {
        Ok(_) => {
            carregar_lista(app, janela);
            janela.mensagem = format!("títulos de {nome} limpos");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Substituir De→Para em todos os títulos de todas as páginas (como o
/// `DeToolStripTextBox`/`ParaToolStripTextBox` do original).
fn substituir_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    let de = janela.de.trim().to_string();
    let para = janela.para.trim().to_string();
    if de.is_empty() {
        janela.mensagem = "informe o texto a substituir em `de`".to_string();
        return;
    }
    let mut contadas = 0u32;
    for linha in janela.lista.clone() {
        let titulos = [
            linha.titulo1.replace(&de, &para),
            linha.titulo2.replace(&de, &para),
            linha.titulo3.replace(&de, &para),
            linha.titulo4.replace(&de, &para),
        ];
        if titulos[0] != linha.titulo1
            || titulos[1] != linha.titulo2
            || titulos[2] != linha.titulo3
            || titulos[3] != linha.titulo4
        {
            let parametros = json!({
                "pagina": linha.pagina,
                "titulo1": titulos[0],
                "titulo2": titulos[1],
                "titulo3": titulos[2],
                "titulo4": titulos[3],
                "unidade": linha.unidade,
                "alternativo": linha.alternativo,
            });
            if app.run("ele.pagina.escrever", parametros).is_ok() {
                contadas += 1;
            }
        }
    }
    carregar_lista(app, janela);
    janela.mensagem = format!("{contadas} página(s) substituída(s)");
}

/// Valor de uma célula (`coluna` 0–6) como texto (alternativo truncado
/// a 20 como no original: `Strings.Mid(text, 1, 20)`).
fn celula_valor(linha: &LinhaTitulos, coluna: usize) -> String {
    match coluna {
        1 => linha.titulo1.clone(),
        2 => linha.titulo2.clone(),
        3 => linha.titulo3.clone(),
        4 => linha.titulo4.clone(),
        5 => linha.unidade.clone(),
        6 => linha.alternativo.chars().take(20).collect(),
        _ => linha.pagina.clone(),
    }
}

/// Grava um valor numa célula (o alternativo é truncado a 20, como `method_3`).
fn definir_celula(linha: &mut LinhaTitulos, coluna: usize, valor: &str) {
    match coluna {
        1 => linha.titulo1 = valor.to_string(),
        2 => linha.titulo2 = valor.to_string(),
        3 => linha.titulo3 = valor.to_string(),
        4 => linha.titulo4 = valor.to_string(),
        5 => linha.unidade = valor.to_string(),
        6 => linha.alternativo = valor.chars().take(20).collect(),
        _ => {}
    }
}

/// `Copiar` (como `method_2`): o valor da célula selecionada da grade.
fn copiar_janela(ctx: &egui::Context, janela: &mut JanelaTitulos) {
    let Some((pagina, coluna)) = janela.celula.clone() else {
        janela.mensagem = "clique numa célula da grade para copiar".to_string();
        return;
    };
    match janela.lista.iter().find(|l| l.pagina == pagina) {
        Some(linha) => {
            let valor = celula_valor(linha, coluna);
            ctx.copy_text(valor.clone());
            janela.clipboard = Some(CopiaCelula {
                valor,
            });
            janela.mensagem = format!("célula copiada de {pagina}");
        }
        None => janela.mensagem = format!("página {pagina} não encontrada na grade"),
    }
}

/// `Colar` (como `method_3`): cola o valor copiado na célula selecionada
/// (colunas 1–6; a coluna Página nunca é alterada). Escreve a página inteira
/// com o valor atualizado.
fn colar_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    let Some(copia) = janela.clipboard.clone() else {
        janela.mensagem = "nada copiado ainda (use Copiar primeiro)".to_string();
        return;
    };
    let Some((pagina, coluna)) = janela.celula.clone() else {
        janela.mensagem = "clique numa célula de destino para colar".to_string();
        return;
    };
    if coluna == 0 {
        janela.mensagem = "a coluna Página não aceita colar".to_string();
        return;
    }
    let Some(mut destino) = janela.lista.iter().find(|l| l.pagina == pagina).cloned() else {
        janela.mensagem = format!("página {pagina} não encontrada na grade");
        return;
    };
    definir_celula(&mut destino, coluna, &copia.valor);
    let parametros = json!({
        "pagina": destino.pagina,
        "titulo1": destino.titulo1,
        "titulo2": destino.titulo2,
        "titulo3": destino.titulo3,
        "titulo4": destino.titulo4,
        "unidade": destino.unidade,
        "alternativo": destino.alternativo,
    });
    match app.run("ele.pagina.escrever", parametros) {
        Ok(_) => {
            janela.pagina = destino.pagina.clone();
            janela.titulo1 = destino.titulo1.clone();
            janela.titulo2 = destino.titulo2.clone();
            janela.titulo3 = destino.titulo3.clone();
            janela.titulo4 = destino.titulo4.clone();
            janela.unidade = destino.unidade.clone();
            janela.alternativo = destino.alternativo.clone();
            carregar_lista(app, janela);
            janela.mensagem = format!("colado em {pagina}");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Cabeçalho do TSV (como `ExportaPaginas` escreve A1:G1).
const TSV_CABECALHO: &str = "Página\tTítulo 1\tTítulo 2\tTítulo 3\tTítulo 4\tUnidade\tAlternativo";

/// Uma linha da grade como TSV (7 colunas, como A–G da planilha).
fn linha_tsv(linha: &LinhaTitulos) -> String {
    [
        linha.pagina.clone(),
        linha.titulo1.clone(),
        linha.titulo2.clone(),
        linha.titulo3.clone(),
        linha.titulo4.clone(),
        linha.unidade.clone(),
        linha.alternativo.chars().take(20).collect(),
    ]
    .join("\t")
}

/// `Exportar` (como `method_7`/`ExportaPaginas`): a grade inteira como TSV
/// na área de texto da janela — sem Excel, sem Interop.
fn exportar_janela(janela: &mut JanelaTitulos) {
    if janela.lista.is_empty() {
        janela.mensagem = "nada para exportar (grade vazia)".to_string();
        return;
    }
    let mut texto = String::from(TSV_CABECALHO);
    texto.push('\n');
    for linha in &janela.lista {
        texto.push_str(&linha_tsv(linha));
        texto.push('\n');
    }
    janela.texto_tsv = texto;
    janela.mensagem = format!("{} página(s) exportada(s) para texto", janela.lista.len());
}

/// `Importar` (como `method_8`/`ImportaTitulos`): lê o TSV da área de texto
/// e aplica cada linha na página de mesmo nome (títulos 1–4, unidade e,
/// quando houver 7ª coluna, o alternativo — truncado a 20).
fn importar_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    let texto = janela.texto_tsv.clone();
    let mut aplicadas = 0u32;
    for linha_txt in texto.lines() {
        let linha_txt = linha_txt.trim();
        if linha_txt.is_empty() || linha_txt == TSV_CABECALHO {
            continue;
        }
        let partes: Vec<&str> = linha_txt.split('\t').collect();
        let pagina = partes.first().map_or("", |s| s.trim());
        if pagina.is_empty() {
            continue;
        }
        let get = |i: usize| partes.get(i).map_or("", |s| s.trim()).to_string();
        let parametros = json!({
            "pagina": pagina,
            "titulo1": get(1),
            "titulo2": get(2),
            "titulo3": get(3),
            "titulo4": get(4),
            "unidade": get(5),
            "alternativo": get(6).chars().take(20).collect::<String>(),
        });
        if app.run("ele.pagina.escrever", parametros).is_ok() {
            aplicadas += 1;
        }
    }
    carregar_lista(app, janela);
    janela.mensagem = format!("{aplicadas} página(s) importada(s)");
}

/// Estado da janela **Banco do Projeto**, na memória do egui (como o painel).
#[derive(Clone, Default)]
struct JanelaBanco {
    arquivo: String,
    carregada: bool,
    mensagem: String,
}

/// Abre a janela **Banco do Projeto** (`frmConfiguracaoBDProjeto` no original).
pub fn abrir_banco(app: &mut CadApp) -> Result<Value, String> {
    app.ui.eletrico_janela_banco = true;
    Ok(json!({ "janela": "banco" }))
}

/// Preenche o campo com o banco vinculado no desenho (XData do próprio arquivo).
fn carregar_banco(app: &mut CadApp, janela: &mut JanelaBanco) {
    janela.carregada = true;
    janela.arquivo = cadcraft_engine::cmd::eletrico::banco_do_desenho(&app.session);
    janela.mensagem = if janela.arquivo.is_empty() {
        "desenho sem banco: escolha um .db".to_string()
    } else {
        String::new()
    };
}

/// A janela **Banco do Projeto** (`frmConfiguracaoBDProjeto` do original):
/// mostra o `.db` vinculado no XData do desenho, escolhe um arquivo com o
/// service `pick_db_open` da casca (o diálogo do original) e grava/abre o
/// banco sobre os comandos do M4b. Não usa file picker quando ele não
/// existe (host de teste): o caminho pode ser digitado.
fn janela_banco(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.eletrico_janela_banco {
        return;
    }
    let mut janela: JanelaBanco = ctx
        .data_mut(|d| d.get_temp_mut_or_default::<JanelaBanco>(egui::Id::new(ID_JANELA_BANCO)).clone());
    if !janela.carregada {
        carregar_banco(app, &mut janela);
    }
    let mut aberta = true;
    egui::Window::new("Banco do Projeto")
        .id(egui::Id::new(ID_JANELA_BANCO))
        .open(&mut aberta)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Arquivo do projeto (.db)");
            ui.add(
                egui::TextEdit::singleline(&mut janela.arquivo)
                    .desired_width(380.0)
                    .hint_text(r"C:\\...\\projeto.db"),
            );
            ui.horizontal(|ui| {
                if ui.button("Escolher...").clicked() {
                    let escolhido = if app.services.pick_db_open.is_some() {
                        app.services.pick_db_open.as_ref().and_then(|f| f())
                    } else if janela.arquivo.trim().is_empty() {
                        None
                    } else {
                        Some(janela.arquivo.clone())
                    };
                    if let Some(arquivo) = escolhido {
                        janela.arquivo = arquivo;
                    }
                }
                if ui.button("Ler do desenho").clicked() {
                    carregar_banco(app, &mut janela);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Vincular ao desenho").clicked() {
                    let arquivo = janela.arquivo.trim().to_string();
                    if arquivo.is_empty() {
                        janela.mensagem = "informe o arquivo .db".to_string();
                    } else {
                        janela.mensagem = match app.run("ele.projeto.gravar", json!({ "banco": arquivo })) {
                            Ok(v) => {
                                let handle = v.get("handle").and_then(Value::as_str).unwrap_or("?");
                                format!("banco vinculado ao desenho ({handle})")
                            }
                            Err(e) => format!("erro: {e}"),
                        };
                    }
                }
                if ui.button("Abrir / criar").clicked() {
                    let arquivo = janela.arquivo.trim().to_string();
                    if arquivo.is_empty() {
                        janela.mensagem = "informe o arquivo .db".to_string();
                    } else {
                        janela.mensagem = match app.run("ele.banco.abrir", json!({ "arquivo": arquivo })) {
                            Ok(v) => {
                                let total = v.get("paginas").and_then(Value::as_u64).unwrap_or(0);
                                format!("banco aberto: {total} página(s)")
                            }
                            Err(e) => format!("erro: {e}"),
                        };
                    }
                }
            });
            if !janela.mensagem.is_empty() {
                ui.label(&janela.mensagem);
            }
        });
    if !aberta {
        app.ui.eletrico_janela_banco = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA_BANCO), janela));
}

/// A janela **Editar Títulos** (`frmEditaTitulos` do original): grade de
/// páginas com título 1–4, unidade e alternativo, botões de cima espelhando o
/// menu de contexto (`Atualizar lista`, `Copiar`/`Colar` de uma célula
/// pela área de transferência,
/// transferência em TSV, `Exportar`/`Importar` numa área de texto com o
/// mesmo TSV), `Substituir` com `De`/`Para` e `Limpar`, e a edição da
/// página selecionada com `Gravar` — tudo sobre os comandos do M4b. O
/// formato TSV (página, títulos 1–4, unidade, alternativo, com cabeçalho)
/// é o mesmo que `ExportaPaginas`/`ImportaTitulos` do original escrevem
/// na planilha (colunas A–G), mas em texto puro: sem Excel, sem Interop.
const ID_JANELA_CIRCUITOS: &str = "eletrico.janela_circuitos";

/// Uma pagina do circuito na janela de circuitos.
#[derive(Clone, Default)]
struct PaginaCircuito {
    pagina: String,
    handle: String,
}

/// Uma linha da grade de circuitos (um potencial num painel).
#[derive(Clone, Default)]
struct LinhaCircuito {
    painel: i64,
    nome_painel: String,
    potencial: i64,
    nome: String,
    nome_original: String,
    paginas: Vec<PaginaCircuito>,
    paginas_txt: String,
}

/// Um painel com seus circuitos.
#[derive(Clone, Default)]
struct PainelCircuito {
    numero: i64,
    nome: String,
    circuitos: Vec<LinhaCircuito>,
}

/// Estado da janela **Editar Circuitos**, na memória do egui.
#[derive(Clone, Default)]
struct JanelaCircuitos {
    paineis: Vec<PainelCircuito>,
    painel: Option<i64>,
    circuito: Option<i64>,
    carregada: bool,
    mensagem: String,
    /// Texto do `Exportar`/`Importar` (TSV, com cabeçalho).
    texto_tsv: String,
}

/// Abre a janela **Editar Circuitos** (`frmEditaNomeCircuitos` no original).
pub fn abrir_circuitos(app: &mut CadApp) -> Result<Value, String> {
    app.ui.eletrico_janela_circuitos = true;
    Ok(json!({ "janela": "circuitos" }))
}

/// Numero de um campo do resultado de ferramenta.
fn numero_json(v: &Value, chave: &str) -> i64 {
    v.get(chave).and_then(Value::as_i64).unwrap_or(0)
}

/// Le os paineis/circuitos do comando `ele.circuito.listar` para a janela.
fn carregar_circuitos(app: &mut CadApp, janela: &mut JanelaCircuitos) {
    janela.carregada = true;
    match app.run("ele.circuito.listar", json!({})) {
        Ok(v) => {
            janela.paineis = v
                .get("paineis")
                .and_then(Value::as_array)
                .map(|paineis| {
                    paineis
                        .iter()
                        .map(|painel| PainelCircuito {
                            numero: numero_json(painel, "numero"),
                            nome: campo_json(painel, "nome"),
                            circuitos: painel
                                .get("circuitos")
                                .and_then(Value::as_array)
                                .map(|circuitos| {
                                    circuitos
                                        .iter()
                                        .map(|c| LinhaCircuito {
                                            painel: numero_json(c, "painel"),
                                            nome_painel: campo_json(c, "nome_painel"),
                                            potencial: numero_json(c, "potencial"),
                                            nome: campo_json(c, "nome"),
                                            nome_original: campo_json(c, "nome"),
                                            paginas: c
                                                .get("paginas")
                                                .and_then(Value::as_array)
                                                .map(|paginas| {
                                                    paginas
                                                        .iter()
                                                        .map(|p| PaginaCircuito {
                                                            pagina: campo_json(p, "pagina"),
                                                            handle: campo_json(p, "handle"),
                                                        })
                                                        .collect()
                                                })
                                                .unwrap_or_default(),
                                            paginas_txt: campo_json(c, "paginas_txt"),
                                        })
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            janela.painel = janela.paineis.first().map(|p| p.numero);
            janela.circuito = janela.paineis.first().and_then(|p| p.circuitos.first()).map(|c| c.potencial);
            janela.mensagem.clear();
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Um `Editar` (como `btEditar`): grava todos os nomes editados na memoria
/// sobre o comando `ele.circuito.renomear`, que espelha o
/// `EditaValorNomeCircuito` do original (textos AUXCONEXAO com o mesmo
/// potencial + XData CONEXAO da linha).
fn editar_circuitos(app: &mut CadApp, janela: &mut JanelaCircuitos) {
    let mut feitos = 0u32;
    let mut unique: Vec<(i64, String)> = Vec::new();
    for painel in &janela.paineis {
        for circuito in &painel.circuitos {
            if circuito.nome.trim() != circuito.nome_original.trim()
                && !unique.iter().any(|(p, nome)| *p == circuito.potencial && nome.trim() == circuito.nome.trim())
            {
                unique.push((circuito.potencial, circuito.nome.trim().to_string()));
            }
        }
    }
    for (potencial, nome) in unique {
        if app
            .run(
                "ele.circuito.renomear",
                json!({"potencial": potencial, "nome": nome}),
            )
            .is_ok()
        {
            feitos += 1;
        }
    }
    carregar_circuitos(app, janela);
    janela.mensagem = format!("{feitos} circuito(s) renomeado(s)");
}

/// Cabecalho do TSV (Painel, Nome, Potencial, Nome, Paginas), sem Excel.
const TSV_CIRCUITOS_CABECALHO: &str = "Painel\tNome do painel\tPotencial\tNome\tPáginas";

/// Uma linha de circuito como TSV (5 colunas).
fn linha_circuito_tsv(linha: &LinhaCircuito) -> String {
    [
        linha.painel.to_string(),
        linha.nome_painel.clone(),
        linha.potencial.to_string(),
        linha.nome.clone(),
        linha.paginas_txt.clone(),
    ]
    .join("\t")
}

/// `Exportar` (como o `btExportar` do original): a grade inteira como TSV
/// na area de texto da janela.
fn exportar_circuitos(janela: &mut JanelaCircuitos) {
    let mut texto = String::from(TSV_CIRCUITOS_CABECALHO);
    texto.push('\n');
    let mut total = 0u32;
    for painel in &janela.paineis {
        for circuito in &painel.circuitos {
            texto.push_str(&linha_circuito_tsv(circuito));
            texto.push('\n');
            total += 1;
        }
    }
    if total == 0 {
        janela.mensagem = "nada para exportar (grade vazia)".to_string();
        return;
    }
    janela.texto_tsv = texto;
    janela.mensagem = format!("{total} circuito(s) exportado(s) para texto");
}

/// `Importar` (como o `btImportar`): le o TSV e aplica o nome de cada
/// linha no circuito de mesmo painel/potencial, na memoria da janela — o
/// `Editar` grava de fato.
fn importar_circuitos(janela: &mut JanelaCircuitos) {
    let texto = janela.texto_tsv.clone();
    let mut aplicadas = 0u32;
    for linha_txt in texto.lines() {
        let linha_txt = linha_txt.trim();
        if linha_txt.is_empty() || linha_txt == TSV_CIRCUITOS_CABECALHO {
            continue;
        }
        let partes: Vec<&str> = linha_txt.split('\t').collect();
        let (painel_txt, potencial_txt, nome_txt) = if partes.len() >= 5 {
            (partes[0].trim(), partes[2].trim(), partes[3].trim())
        } else if partes.len() == 3 {
            (partes[0].trim(), partes[1].trim(), partes[2].trim())
        } else {
            continue;
        };
        let Ok(potencial) = potencial_txt.parse::<i64>() else { continue };
        for painel in &mut janela.paineis {
            if !painel.numero.to_string().eq_ignore_ascii_case(painel_txt)
                && !painel.nome.eq_ignore_ascii_case(painel_txt)
            {
                continue;
            }
            for circuito in &mut painel.circuitos {
                if circuito.potencial == potencial {
                    circuito.nome = nome_txt.to_string();
                    aplicadas += 1;
                }
            }
        }
    }
    janela.mensagem = format!("{aplicadas} circuito(s) importado(s) — use Editar para gravar");
}

/// A janela **Editar Circuitos** (`frmEditaNomeCircuitos` do original):
/// papeis a esquerda, circuitos no meio (nome editavel, como o
/// `DataGridView` so de nomes), paginas do circuito a direita (o clique
/// espelha o `MostraBloco`), Exportar/Importar TSV e Editar.
fn janela_circuitos(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.eletrico_janela_circuitos {
        return;
    }
    let mut janela: JanelaCircuitos =
        ctx.data_mut(|d| d.get_temp_mut_or_default::<JanelaCircuitos>(egui::Id::new(ID_JANELA_CIRCUITOS)).clone());
    if !janela.carregada {
        carregar_circuitos(app, &mut janela);
    }
    let mut aberta = true;
    let mut painel_selecionado = janela.painel;
    let mut circuito_selecionado = janela.circuito;
    let mut paineis_copiados = janela.paineis.clone();
    let mut mostrar_handle: Option<String> = None;
    egui::Window::new("Editar Circuitos")
        .id(egui::Id::new(ID_JANELA_CIRCUITOS))
        .open(&mut aberta)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Atualizar lista").clicked() {
                    carregar_circuitos(app, &mut janela);
                    painel_selecionado = janela.painel;
                    circuito_selecionado = janela.circuito;
                    paineis_copiados = janela.paineis.clone();
                }
                if ui.button("Exportar").clicked() {
                    exportar_circuitos(&mut janela);
                    paineis_copiados = janela.paineis.clone();
                }
                if ui.button("Importar").clicked() {
                    importar_circuitos(&mut janela);
                    paineis_copiados = janela.paineis.clone();
                }
                let ha_editados = janela.paineis.iter().flat_map(|p| &p.circuitos).any(|c| c.nome.trim() != c.nome_original.trim());
                if ui.add_enabled(ha_editados, egui::Button::new("Editar")).clicked() {
                    editar_circuitos(app, &mut janela);
                    painel_selecionado = janela.painel;
                    circuito_selecionado = janela.circuito;
                    paineis_copiados = janela.paineis.clone();
                }
            });
            ui.add(
                egui::TextEdit::multiline(&mut janela.texto_tsv)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text("Exportar/Importar: TSV com cabeçalho — Painel, Nome do painel, Potencial, Nome, Páginas"),
            );
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.label("Painéis");
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for painel in &paineis_copiados {
                            if ui
                                .selectable_label(painel_selecionado == Some(painel.numero), &painel.nome)
                                .clicked()
                            {
                                painel_selecionado = Some(painel.numero);
                                circuito_selecionado = painel.circuitos.first().map(|c| c.potencial);
                            }
                        }
                        if paineis_copiados.is_empty() {
                            ui.label("(sem painéis)");
                        }
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Circuitos");
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        egui::Grid::new("grade_circuitos").striped(true).show(ui, |ui| {
                            for cabecalho in ["Potencial", "Nome", "Páginas", "Editado"] {
                                ui.label(cabecalho);
                            }
                            ui.end_row();
                            for painel in &mut paineis_copiados {
                                if painel_selecionado != Some(painel.numero) {
                                    continue;
                                }
                                for circuito in &mut painel.circuitos {
                                    let selecionado = circuito_selecionado == Some(circuito.potencial);
                                    if ui.selectable_label(selecionado, circuito.potencial.to_string()).clicked() {
                                        circuito_selecionado = Some(circuito.potencial);
                                        janela.painel = Some(painel.numero);
                                        janela.circuito = Some(circuito.potencial);
                                    }
                                    ui.add(egui::TextEdit::singleline(&mut circuito.nome).desired_width(180.0));
                                    ui.label(&circuito.paginas_txt);
                                    ui.label(if circuito.nome.trim() != circuito.nome_original.trim() { "E" } else { "" });
                                    ui.end_row();
                                }
                            }
                        });
                    });
                    ui.separator();
                    ui.label("Páginas do circuito");
                    if let Some(painel) = paineis_copiados.iter().find(|p| p.numero == painel_selecionado.unwrap_or(-1)) {
                        if let Some(circuito) = painel.circuitos.iter().find(|c| c.potencial == circuito_selecionado.unwrap_or(-1)) {
                            if circuito.paginas.is_empty() {
                                ui.label("(sem páginas)");
                            }
                            for pagina in &circuito.paginas {
                                if ui.button(&pagina.pagina).clicked() {
                                    mostrar_handle = Some(pagina.handle.clone());
                                }
                            }
                        } else {
                            ui.label("(selecione um circuito)");
                        }
                    } else {
                        ui.label("(selecione um painel)");
                    }
                });
            });
            if !janela.mensagem.is_empty() {
                let mensagem = janela.mensagem.clone();
                ui.label(mensagem);
            }
        });
    janela.paineis = paineis_copiados;
    janela.painel = painel_selecionado;
    janela.circuito = circuito_selecionado;
    if let Some(handle) = mostrar_handle {
        match app.run("ele.circuito.mostrar", json!({ "handle": handle })) {
            Ok(_) => janela.mensagem = format!("handle {handle} selecionado"),
            Err(e) => janela.mensagem = format!("erro: {e}"),
        }
    }
    if !aberta {
        app.ui.eletrico_janela_circuitos = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA_CIRCUITOS), janela));
}

fn janela_titulos(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.eletrico_janela_titulos {
        return;
    }
    let mut janela: JanelaTitulos = ctx
        .data_mut(|d| d.get_temp_mut_or_default::<JanelaTitulos>(egui::Id::new(ID_JANELA)).clone());
    if !janela.carregada {
        carregar_lista(app, &mut janela);
    }
    let mut aberta = true;
    egui::Window::new("Editar Títulos")
        .id(egui::Id::new(ID_JANELA))
        .open(&mut aberta)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Atualizar lista").clicked() {
                    carregar_lista(app, &mut janela);
                }
                if ui.button("Copiar").clicked() {
                    copiar_janela(ctx, &mut janela);
                }
                if ui.button("Colar").clicked() {
                    colar_janela(app, &mut janela);
                }
                if ui.button("Limpar página").clicked() {
                    limpar_janela(app, &mut janela);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Exportar").clicked() {
                    exportar_janela(&mut janela);
                }
                if ui.button("Importar").clicked() {
                    importar_janela(app, &mut janela);
                }
            });
            ui.add(
                egui::TextEdit::multiline(&mut janela.texto_tsv)
                    .desired_width(f32::INFINITY)
                    .desired_rows(4)
                    .hint_text("Exportar/Importar: TSV com cabeçalho — Página, Títulos 1–4, Unidade, Alternativo"),
            );
            ui.horizontal(|ui| {
                ui.label("Substituir");
                ui.add(egui::TextEdit::singleline(&mut janela.de).hint_text("de").desired_width(120.0));
                ui.label("para");
                ui.add(egui::TextEdit::singleline(&mut janela.para).hint_text("para").desired_width(120.0));
                if ui.button("Substituir").clicked() {
                    substituir_janela(app, &mut janela);
                }
            });

            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                egui::Grid::new("grade_titulos").striped(true).show(ui, |ui| {
                    for cabecalho in
                        ["Página", "Título 1", "Título 2", "Título 3", "Título 4", "Unidade", "Alternativo"]
                    {
                        ui.label(cabecalho);
                    }
                    ui.end_row();
                    for linha in janela.lista.clone() {
                        for coluna in 0..=6 {
                            let texto = celula_valor(&linha, coluna);
                            let selecionada = janela.celula == Some((linha.pagina.clone(), coluna));
                            if ui.selectable_label(selecionada, &texto).clicked() {
                                janela.celula = Some((linha.pagina.clone(), coluna));
                                janela.pagina = linha.pagina.clone();
                                janela.titulo1 = linha.titulo1.clone();
                                janela.titulo2 = linha.titulo2.clone();
                                janela.titulo3 = linha.titulo3.clone();
                                janela.titulo4 = linha.titulo4.clone();
                                janela.unidade = linha.unidade.clone();
                                janela.alternativo = linha.alternativo.clone();
                            }
                        }
                        ui.end_row();
                    }
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Página");
                ui.add(egui::TextEdit::singleline(&mut janela.pagina).desired_width(140.0).hint_text("12A"));
                if ui.button("Gravar").clicked() {
                    gravar_janela(app, &mut janela);
                }
            });
            ui.horizontal(|ui| {
                for (rotulo, valor) in [
                    ("Título 1", &mut janela.titulo1),
                    ("Título 2", &mut janela.titulo2),
                    ("Título 3", &mut janela.titulo3),
                    ("Título 4", &mut janela.titulo4),
                ] {
                    ui.label(rotulo);
                    ui.add(egui::TextEdit::singleline(valor).desired_width(160.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Unidade");
                ui.add(egui::TextEdit::singleline(&mut janela.unidade).desired_width(160.0));
                ui.label("Alternativo");
                ui.add(egui::TextEdit::singleline(&mut janela.alternativo).desired_width(160.0));
            });
            if !janela.mensagem.is_empty() {
                ui.label(&janela.mensagem);
            }
        });
    if !aberta {
        app.ui.eletrico_janela_titulos = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA), janela));
}


const ID_JANELA_TEXTOS: &str = "eletrico.janela_textos";

/// Estado da janela **Editar Títulos (Textos)**, na memória do egui.
#[derive(Clone, Default)]
struct JanelaTextos {
    paginas: Vec<String>,
    selecionadas: Vec<String>,
    carregada: bool,
    titulo1: bool,
    titulo2: bool,
    titulo3: bool,
    titulo4: bool,
    texto: bool,
    mtexto: bool,
    de: String,
    para: String,
    mensagem: String,
}

pub fn abrir_titulos_textos(app: &mut CadApp) -> Result<Value, String> {
    app.ui.eletrico_janela_textos = true;
    Ok(json!({ "janela": "titulos_textos" }))
}

fn carregar_textos(app: &mut CadApp, janela: &mut JanelaTextos) {
    janela.carregada = true;
    match app.run("ele.pagina.ler", json!({})) {
        Ok(v) => {
            janela.paginas = v
                .get("paginas")
                .and_then(Value::as_array)
                .map(|paginas| {
                    paginas.iter().map(|p| campo_json(p, "pagina")).filter(|s| !s.is_empty()).collect()
                })
                .unwrap_or_default();
            janela.selecionadas.retain(|p| janela.paginas.contains(p));
            janela.mensagem.clear();
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn editar_textos(app: &mut CadApp, janela: &mut JanelaTextos) {
    if janela.selecionadas.is_empty() {
        janela.mensagem = "selecione ao menos uma página".to_string();
        return;
    }
    let de = janela.de.trim().to_string();
    if de.is_empty() {
        janela.mensagem = "informe o texto a substituir em `de`".to_string();
        return;
    }
    let para = janela.para.trim().to_string();
    let parametros = json!({
        "paginas": janela.selecionadas.clone(),
        "titulo1": janela.titulo1,
        "titulo2": janela.titulo2,
        "titulo3": janela.titulo3,
        "titulo4": janela.titulo4,
        "texto": janela.texto,
        "mtexto": janela.mtexto,
        "de": de,
        "para": para,
    });
    match app.run("ele.titulos_textos.substituir", parametros) {
        Ok(v) => {
            let alterados = v.get("alterados").and_then(Value::as_u64).unwrap_or(0);
            let examinados = v.get("examinados").and_then(Value::as_u64).unwrap_or(0);
            janela.mensagem = format!("{alterados} texto(s) alterado(s) em {examinados} examinado(s)");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn janela_textos(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.eletrico_janela_textos {
        return;
    }
    let mut janela: JanelaTextos = ctx
        .data_mut(|d| d.get_temp_mut_or_default::<JanelaTextos>(egui::Id::new(ID_JANELA_TEXTOS)).clone());
    if !janela.carregada {
        carregar_textos(app, &mut janela);
    }
    let mut aberta = true;
    egui::Window::new("Editar Títulos (Textos)")
        .id(egui::Id::new(ID_JANELA_TEXTOS))
        .open(&mut aberta)
        .show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.label("Páginas");
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        for pagina in janela.paginas.clone() {
                            let selecionada = janela.selecionadas.contains(&pagina);
                            if ui.selectable_label(selecionada, &pagina).clicked() {
                                if selecionada {
                                    janela.selecionadas.retain(|p| p != &pagina);
                                } else {
                                    janela.selecionadas.push(pagina);
                                }
                            }
                        }
                        if janela.paginas.is_empty() {
                            ui.label("(sem páginas)");
                        }
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut janela.titulo1, "Título 1");
                        ui.checkbox(&mut janela.titulo2, "Título 2");
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut janela.titulo3, "Título 3");
                        ui.checkbox(&mut janela.titulo4, "Título 4");
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut janela.texto, "Texto");
                        ui.checkbox(&mut janela.mtexto, "MText");
                    });
                    ui.horizontal(|ui| {
                        ui.label("De");
                        ui.add(egui::TextEdit::singleline(&mut janela.de).hint_text("de").desired_width(140.0));
                        ui.label("Para");
                        ui.add(egui::TextEdit::singleline(&mut janela.para).hint_text("para").desired_width(140.0));
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Atualizar lista").clicked() {
                            carregar_textos(app, &mut janela);
                        }
                        if ui.button("Editar").clicked() {
                            editar_textos(app, &mut janela);
                        }
                    });
                });
            });
            if !janela.mensagem.is_empty() {
                ui.label(&janela.mensagem);
            }
        });
    if !aberta {
        app.ui.eletrico_janela_textos = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA_TEXTOS), janela));
}


const ID_JANELA_REVISAO: &str = "ele.janela_revisao";

/// Uma linha da grade de revisao: a pagina e o texto da revisao simplificada.
#[derive(Clone, Default)]
struct LinhaRevisao {
    pagina: String,
    revisao: String,
}

/// Estado da janela **Revisão Simplificada**, na memória do egui.
#[derive(Clone, Default)]
struct JanelaRevisao {
    lista: Vec<LinhaRevisao>,
    carregada: bool,
    mensagem: String,
    /// Celula selecionada na grade: (pagina, coluna 0-1).
    celula: Option<(String, usize)>,
    /// Copiado pela janela (`Copiar`): o valor de uma celula.
    clipboard: Option<String>,
}

pub fn abrir_revisao(app: &mut CadApp) -> Result<Value, String> {
    app.ui.eletrico_janela_revisao = true;
    Ok(json!({ "janela": "revisao" }))
}

fn carregar_revisao(app: &mut CadApp, janela: &mut JanelaRevisao) {
    janela.carregada = true;
    match app.run("ele.revisao.ler", json!({})) {
        Ok(v) => {
            janela.lista = v
                .get("revisoes")
                .and_then(Value::as_array)
                .map(|revisoes| {
                    revisoes
                        .iter()
                        .map(|r| LinhaRevisao { pagina: campo_json(r, "pagina"), revisao: campo_json(r, "revisao") })
                        .collect()
                })
                .unwrap_or_default();
            janela.mensagem.clear();
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn gravar_revisao(app: &mut CadApp, janela: &mut JanelaRevisao) {
    let revisoes: Vec<Value> = janela
        .lista
        .iter()
        .map(|l| json!({ "pagina": l.pagina, "revisao": l.revisao.trim() }))
        .collect();
    match app.run("ele.revisao.gravar", json!({ "revisoes": revisoes })) {
        Ok(v) => {
            let atualizados = v.get("atualizados").and_then(Value::as_u64).unwrap_or(0);
            let ignorados = v.get("ignorados").and_then(Value::as_u64).unwrap_or(0);
            carregar_revisao(app, janela);
            janela.mensagem = format!("{atualizados} revisao(oes) atualizada(s), {ignorados} ignorada(s) (sem texto de revisao)");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn janela_revisao(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.eletrico_janela_revisao {
        return;
    }
    let mut janela: JanelaRevisao = ctx
        .data_mut(|d| d.get_temp_mut_or_default::<JanelaRevisao>(egui::Id::new(ID_JANELA_REVISAO)).clone());
    if !janela.carregada {
        carregar_revisao(app, &mut janela);
    }
    let mut aberta = true;
    egui::Window::new("Revisão Simplificada")
        .id(egui::Id::new(ID_JANELA_REVISAO))
        .open(&mut aberta)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Copiar").clicked()
                    && let Some((pagina, coluna)) = janela.celula.clone()
                    && let Some(linha) = janela.lista.iter().find(|l| l.pagina == pagina)
                {
                    let valor = if coluna == 0 { linha.pagina.clone() } else { linha.revisao.clone() };
                    ctx.copy_text(valor.clone());
                    janela.clipboard = Some(valor);
                }
                if ui.button("Colar").clicked()
                    && let (Some((pagina, coluna)), Some(valor)) =
                        (janela.celula.clone(), janela.clipboard.clone())
                    && coluna == 1
                    && let Some(linha) = janela.lista.iter_mut().find(|l| l.pagina == pagina)
                {
                    linha.revisao = valor;
                }
                if ui.button("Limpar").clicked()
                    && let Some((pagina, coluna)) = janela.celula.clone()
                    && coluna == 1
                    && let Some(linha) = janela.lista.iter_mut().find(|l| l.pagina == pagina)
                {
                    linha.revisao.clear();
                }
                if ui.button("Atualizar").clicked() {
                    carregar_revisao(app, &mut janela);
                }
                if ui.button("Gravar").clicked() {
                    gravar_revisao(app, &mut janela);
                }
            });
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                egui::Grid::new("grade_revisao").striped(true).show(ui, |ui| {
                    ui.label("Página");
                    ui.label("Revisão");
                    ui.end_row();
                    for linha in janela.lista.clone() {
                        if ui.selectable_label(janela.celula == Some((linha.pagina.clone(), 0)), &linha.pagina).clicked() {
                            janela.celula = Some((linha.pagina.clone(), 0));
                        }
                        ui.end_row();
                        let mut revisao = linha.revisao.clone();
                        if ui.add(egui::TextEdit::singleline(&mut revisao)).clicked() {
                            janela.celula = Some((linha.pagina.clone(), 1));
                        }
                        if revisao != linha.revisao
                            && let Some(l) = janela.lista.iter_mut().find(|l| l.pagina == linha.pagina)
                        {
                            l.revisao = revisao;
                        }
                        ui.end_row();
                    }
                });
                if janela.lista.is_empty() {
                    ui.label("(sem páginas)");
                }
            });
            if !janela.mensagem.is_empty() {
                ui.label(&janela.mensagem);
            }
        });
    if !aberta {
        app.ui.eletrico_janela_revisao = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA_REVISAO), janela));
}


const ID_JANELA_REVISAO_COMPLETA: &str = "ele.janela_revisao_completa";

/// Uma linha da grade de revisao completa: pagina, posicao e os 6 campos
/// editaveis do carimbo (`frmEditaRevisaoCompleta`: Pagina, Revisao,
/// Descricao, Data, Elaborador, Verificador, Aprovador).
#[derive(Clone, Default)]
struct LinhaRevisaoCompleta {
    pagina: String,
    posicao: i32,
    revisao: String,
    descricao: String,
    data: String,
    elaborador: String,
    verificador: String,
    aprovador: String,
}

/// Estado da janela **Revisão Completa**, na memória do egui.
#[derive(Clone, Default)]
struct JanelaRevisaoCompleta {
    lista: Vec<LinhaRevisaoCompleta>,
    carregada: bool,
    de: String,
    para: String,
    mensagem: String,
    /// Celula selecionada na grade: (pagina, coluna 0-7).
    celula: Option<(String, usize)>,
    /// Copiado pela janela (`Copiar`): o valor de uma celula.
    clipboard: Option<String>,
}

pub fn abrir_revisao_completa(app: &mut CadApp) -> Result<Value, String> {
    app.ui.eletrico_janela_revisao_completa = true;
    Ok(json!({ "janela": "revisao_completa" }))
}

fn carregar_revisao_completa(app: &mut CadApp, janela: &mut JanelaRevisaoCompleta) {
    janela.carregada = true;
    match app.run("ele.revisao_completa.ler", json!({})) {
        Ok(v) => {
            janela.lista = v
                .get("revisoes")
                .and_then(Value::as_array)
                .map(|revisoes| {
                    revisoes
                        .iter()
                        .map(|r| LinhaRevisaoCompleta {
                            pagina: campo_json(r, "pagina"),
                            posicao: r.get("posicao").and_then(Value::as_i64).unwrap_or(0) as i32,
                            revisao: campo_json(r, "revisao"),
                            descricao: campo_json(r, "descricao"),
                            data: campo_json(r, "data"),
                            elaborador: campo_json(r, "elaborador"),
                            verificador: campo_json(r, "verificador"),
                            aprovador: campo_json(r, "aprovador"),
                        })
                        .collect()
                })
                .unwrap_or_default();
            janela.mensagem.clear();
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn celula_valor_completa(linha: &LinhaRevisaoCompleta, coluna: usize) -> String {
    match coluna {
        0 => linha.pagina.clone(),
        1 => linha.posicao.to_string(),
        2 => linha.revisao.clone(),
        3 => linha.descricao.clone(),
        4 => linha.data.clone(),
        5 => linha.elaborador.clone(),
        6 => linha.verificador.clone(),
        _ => linha.aprovador.clone(),
    }
}

fn definir_celula_completa(linha: &mut LinhaRevisaoCompleta, coluna: usize, valor: &str) {
    match coluna {
        2 => linha.revisao = valor.to_string(),
        3 => linha.descricao = valor.to_string(),
        4 => linha.data = valor.to_string(),
        5 => linha.elaborador = valor.to_string(),
        6 => linha.verificador = valor.to_string(),
        7 => linha.aprovador = valor.to_string(),
        _ => {}
    }
}

fn gravar_revisao_completa(app: &mut CadApp, janela: &mut JanelaRevisaoCompleta) {
    let revisoes: Vec<Value> = janela
        .lista
        .iter()
        .map(|l| {
            json!({
                "pagina": l.pagina,
                "posicao": l.posicao,
                "revisao": l.revisao.trim(),
                "descricao": l.descricao.trim(),
                "data": l.data.trim(),
                "elaborador": l.elaborador.trim(),
                "verificador": l.verificador.trim(),
                "aprovador": l.aprovador.trim(),
            })
        })
        .collect();
    match app.run("ele.revisao_completa.gravar", json!({ "revisoes": revisoes })) {
        Ok(v) => {
            let atualizados = v.get("atualizados").and_then(Value::as_u64).unwrap_or(0);
            let ignorados = v.get("ignorados").and_then(Value::as_u64).unwrap_or(0);
            carregar_revisao_completa(app, janela);
            janela.mensagem = format!("{atualizados} revisao(oes) atualizada(s), {ignorados} ignorada(s) (sem bloco de revisao)");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn janela_revisao_completa(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.eletrico_janela_revisao_completa {
        return;
    }
    let mut janela: JanelaRevisaoCompleta = ctx
        .data_mut(|d| d.get_temp_mut_or_default::<JanelaRevisaoCompleta>(egui::Id::new(ID_JANELA_REVISAO_COMPLETA)).clone());
    if !janela.carregada {
        carregar_revisao_completa(app, &mut janela);
    }
    let mut aberta = true;
    egui::Window::new("Revisão Completa")
        .id(egui::Id::new(ID_JANELA_REVISAO_COMPLETA))
        .open(&mut aberta)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Copiar").clicked()
                    && let Some((pagina, coluna)) = janela.celula.clone()
                    && let Some(linha) = janela.lista.iter().find(|l| l.pagina == pagina)
                {
                    let valor = celula_valor_completa(linha, coluna);
                    ctx.copy_text(valor.clone());
                    janela.clipboard = Some(valor);
                }
                if ui.button("Colar").clicked()
                    && let (Some((pagina, coluna)), Some(valor)) =
                        (janela.celula.clone(), janela.clipboard.clone())
                    && coluna >= 2
                    && let Some(linha) = janela.lista.iter_mut().find(|l| l.pagina == pagina)
                {
                    definir_celula_completa(linha, coluna, &valor);
                }
                if ui.button("Limpar").clicked()
                    && let Some((pagina, coluna)) = janela.celula.clone()
                    && coluna >= 2
                    && let Some(linha) = janela.lista.iter_mut().find(|l| l.pagina == pagina)
                {
                    definir_celula_completa(linha, coluna, "");
                }
                ui.label("De");
                ui.add(egui::TextEdit::singleline(&mut janela.de).hint_text("de").desired_width(90.0));
                ui.label("Para");
                ui.add(egui::TextEdit::singleline(&mut janela.para).hint_text("para").desired_width(90.0));
                if ui.button("Substituir").clicked() {
                    let de = janela.de.trim().to_string();
                    if de.is_empty() {
                        janela.mensagem = "informe o texto a substituir em `de`".to_string();
                    } else {
                        let para = janela.para.trim().to_string();
                        for linha in &mut janela.lista {
                            for coluna in 2..=7usize {
                                let valor = celula_valor_completa(linha, coluna);
                                if valor.contains(&de) {
                                    let novo = valor.replace(&de, &para);
                                    definir_celula_completa(linha, coluna, novo.trim());
                                }
                            }
                        }
                    }
                }
                if ui.button("Atualizar").clicked() {
                    carregar_revisao_completa(app, &mut janela);
                }
                if ui.button("Gravar").clicked() {
                    gravar_revisao_completa(app, &mut janela);
                }
            });
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                egui::Grid::new("grade_revisao_completa").striped(true).show(ui, |ui| {
                    for rotulo in ["Página", "Pos", "Revisão", "Descrição", "Data", "Elaborador", "Verificador", "Aprovador"] {
                        ui.label(rotulo);
                    }
                    ui.end_row();
                    for linha in janela.lista.clone() {
                        for coluna in 0..=7usize {
                            if coluna < 2 {
                                if ui.selectable_label(
                                    janela.celula == Some((linha.pagina.clone(), coluna)),
                                    celula_valor_completa(&linha, coluna),
                                )
                                .clicked()
                                {
                                    janela.celula = Some((linha.pagina.clone(), coluna));
                                }
                            } else {
                                let mut campo = celula_valor_completa(&linha, coluna);
                                if ui.add(egui::TextEdit::singleline(&mut campo).desired_width(90.0)).clicked() {
                                    janela.celula = Some((linha.pagina.clone(), coluna));
                                }
                                if campo != celula_valor_completa(&linha, coluna)
                                    && let Some(l) = janela.lista.iter_mut().find(|l| l.pagina == linha.pagina)
                                {
                                    definir_celula_completa(l, coluna, &campo);
                                }
                            }
                        }
                        ui.end_row();
                    }
                });
                if janela.lista.is_empty() {
                    ui.label("(sem páginas)");
                }
            });
            if !janela.mensagem.is_empty() {
                ui.label(&janela.mensagem);
            }
        });
    if !aberta {
        app.ui.eletrico_janela_revisao_completa = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA_REVISAO_COMPLETA), janela));
}


const ID_JANELA_XDATA: &str = "ele.janela_xdata";

/// Uma linha da lista de XDatas: o app e quantas entidades o tem.
#[derive(Clone, Default)]
struct LinhaXdata {
    app: String,
    quantidade: usize,
    selecionado: bool,
}

/// Estado da janela **Pesquisa XDatas**, na memória do egui.
#[derive(Clone, Default)]
struct JanelaXdata {
    lista: Vec<LinhaXdata>,
    carregada: bool,
    incluir_conhecidos: bool,
    mensagem: String,
}

pub fn abrir_xdata(app: &mut CadApp) -> Result<Value, String> {
    app.ui.eletrico_janela_xdata = true;
    Ok(json!({ "janela": "xdata" }))
}

fn carregar_xdata(app: &mut CadApp, janela: &mut JanelaXdata) {
    janela.carregada = true;
    let parametros = json!({ "incluir_conhecidos": janela.incluir_conhecidos });
    match app.run("ele.xdata.listar", parametros) {
        Ok(v) => {
            janela.lista = v
                .get("xdatas")
                .and_then(Value::as_array)
                .map(|xdatas| {
                    xdatas
                        .iter()
                        .map(|x| LinhaXdata {
                            app: campo_json(x, "app"),
                            quantidade: x.get("quantidade").and_then(Value::as_u64).unwrap_or(0) as usize,
                            selecionado: false,
                        })
                        .collect()
                })
                .unwrap_or_default();
            janela.mensagem.clear();
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn apagar_xdata(app: &mut CadApp, janela: &mut JanelaXdata) {
    let apps: Vec<Value> = janela
        .lista
        .iter()
        .filter(|l| l.selecionado)
        .map(|l| json!(l.app))
        .collect();
    if apps.is_empty() {
        janela.mensagem = "selecione ao menos um XData".to_string();
        return;
    }
    match app.run("ele.xdata.apagar", json!({ "apps": apps })) {
        Ok(v) => {
            let apagados = v.get("apagados").and_then(Value::as_u64).unwrap_or(0);
            carregar_xdata(app, janela);
            janela.mensagem = format!("{apagados} xData apagados");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

fn janela_xdata(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.eletrico_janela_xdata {
        return;
    }
    let mut janela: JanelaXdata = ctx
        .data_mut(|d| d.get_temp_mut_or_default::<JanelaXdata>(egui::Id::new(ID_JANELA_XDATA)).clone());
    if !janela.carregada {
        carregar_xdata(app, &mut janela);
    }
    let mut aberta = true;
    egui::Window::new("Pesquisa XDatas")
        .id(egui::Id::new(ID_JANELA_XDATA))
        .open(&mut aberta)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut janela.incluir_conhecidos, "Incluir xData conhecidos").changed() {
                    carregar_xdata(app, &mut janela);
                }
                if ui.button("Atualizar").clicked() {
                    carregar_xdata(app, &mut janela);
                }
                if ui.button("Apagar selecionados").clicked() {
                    apagar_xdata(app, &mut janela);
                }
            });
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                egui::Grid::new("grade_xdata").striped(true).show(ui, |ui| {
                    ui.label("xData");
                    ui.label("Quantidade");
                    ui.end_row();
                    for linha in &mut janela.lista {
                        ui.checkbox(&mut linha.selecionado, &linha.app);
                        ui.label(linha.quantidade.to_string());
                        ui.end_row();
                    }
                });
                if janela.lista.is_empty() {
                    ui.label("(nenhum XData fora dos conhecidos)");
                }
            });
            if !janela.mensagem.is_empty() {
                ui.label(&janela.mensagem);
            }
        });
    if !aberta {
        app.ui.eletrico_janela_xdata = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA_XDATA), janela));
}
