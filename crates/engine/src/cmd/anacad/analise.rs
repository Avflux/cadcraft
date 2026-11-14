//! Calculos sobre a rede do banco: curto-circuito, fluxo, Ybus/Zbarra, niveis, superacao.

use super::*;

pub(super) fn run_consultar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.consultar", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.consultar", format!("{db}: {e}")))?;
    let documentos = serde_json::to_value(banco.documentos().map_err(|e| bad("anacad.consultar", e.to_string()))?)
        .map_err(|e| bad("anacad.consultar", e.to_string()))?;
    let mut resposta = json!({ "db": db, "documentos": documentos });
    if let Some(documento) = p.get("documento").and_then(Value::as_i64) {
        let componentes = serde_json::to_value(banco.componentes(documento, 500).map_err(|e| bad("anacad.consultar", e.to_string()))?)
            .map_err(|e| bad("anacad.consultar", e.to_string()))?;
        let linhas = serde_json::to_value(banco.linhas(documento, 500).map_err(|e| bad("anacad.consultar", e.to_string()))?)
            .map_err(|e| bad("anacad.consultar", e.to_string()))?;
        let ligacoes = serde_json::to_value(banco.ligacoes(documento, 500).map_err(|e| bad("anacad.consultar", e.to_string()))?)
            .map_err(|e| bad("anacad.consultar", e.to_string()))?;
        let barras = serde_json::to_value(banco.barras(documento, 1000).map_err(|e| bad("anacad.consultar", e.to_string()))?)
            .map_err(|e| bad("anacad.consultar", e.to_string()))?;
        let circuitos = serde_json::to_value(banco.circuitos(documento, 1000).map_err(|e| bad("anacad.consultar", e.to_string()))?)
            .map_err(|e| bad("anacad.consultar", e.to_string()))?;
        resposta["documento"] = json!(documento);
        resposta["componentes"] = componentes;
        resposta["linhas"] = linhas;
        resposta["ligacoes"] = ligacoes;
        resposta["barras"] = barras;
        resposta["circuitos"] = circuitos;
        resposta["contagens"] = json!({
            "barras": banco.contar("Barra").map_err(|e| bad("anacad.consultar", e.to_string()))?,
            "componentes": banco.contar("Componente").map_err(|e| bad("anacad.consultar", e.to_string()))?,
            "linhas": banco.contar("Linha").map_err(|e| bad("anacad.consultar", e.to_string()))?,
            "ligacoes": banco.contar("Ligacao").map_err(|e| bad("anacad.consultar", e.to_string()))?,
            "legendas": banco.contar("Legenda").map_err(|e| bad("anacad.consultar", e.to_string()))?,
        });
    }
    Ok(resposta)
}

pub(super) fn run_comprimento(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.comprimento", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.comprimento", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.comprimento", e.to_string()))?;
    let linhas = banco.linhas(documento, 100_000).map_err(|e| bad("anacad.comprimento", e.to_string()))?;
    let quebras = banco.ligacoes_quebras(documento, 100_000).map_err(|e| bad("anacad.comprimento", e.to_string()))?;

    let posicoes: BTreeMap<i64, (f64, f64)> = barras.iter().map(|b| (b.id, (b.x.unwrap_or(0.0), b.y.unwrap_or(0.0)))).collect();
    let mut comprimento_ligacoes = 0.0;
    for (_, pontos) in &quebras {
        comprimento_ligacoes += anacad_core::calc::comprimento_polilinha(pontos);
    }
    let mut comprimento_linhas = 0.0;
    for linha in &linhas {
        if let (Some(de), Some(para)) = (linha.de, linha.para)
            && let (Some(a), Some(b)) = (posicoes.get(&de), posicoes.get(&para))
        {
            comprimento_linhas += anacad_core::calc::comprimento_polilinha(&[*a, *b]);
        }
    }
    let total = comprimento_ligacoes + comprimento_linhas;
    let saida = json!({
        "ligacoes": quebras.len(),
        "linhas": linhas.len(),
        "comprimento_ligacoes": comprimento_ligacoes,
        "comprimento_linhas": comprimento_linhas,
        "comprimento_total": total,
    });
    let id =
        banco.inserir_resultado(Some(documento), "comprimento", "{}", &saida.to_string()).map_err(|e| bad("anacad.comprimento", e.to_string()))?;
    Ok(json!({
        "resultado": id,
        "ligacoes": quebras.len(),
        "comprimento_ligacoes": comprimento_ligacoes,
        "comprimento_linhas": comprimento_linhas,
        "comprimento_total": total,
    }))
}

pub(super) fn run_referencia(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.referencia", "`arquivo` (.OUT) is required"))?;
    let texto = String::from_utf8_lossy(&ler_arquivo(arquivo)?).into_owned();
    let referencias = anacad_core::referencia::ler_out(&texto);
    let mut comparacoes = Vec::new();
    if let Some(db) = str_param(p, "db") {
        let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
        let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.referencia", format!("{db}: {e}")))?;
        let resultados = banco.resultados(documento, 100).map_err(|e| bad("anacad.referencia", e.to_string()))?;
        let mut nossos: BTreeMap<i64, (f64, f64)> = BTreeMap::new();
        for r in &resultados {
            if r.tipo != "curto_circuito" {
                continue;
            }
            let Ok(saida) = serde_json::from_str::<Value>(r.saida.as_deref().unwrap_or("{}")) else { continue };
            let (Some(barra), Some(i_ka), Some(vbase)) =
                (saida.get("barra").and_then(Value::as_i64), saida.get("i_ka").and_then(Value::as_f64), saida.get("vbase").and_then(Value::as_f64))
            else {
                continue;
            };
            nossos.insert(barra, ((3.0f64).sqrt() * vbase * i_ka, i_ka));
        }
        for r in &referencias {
            if let (Some(i_mva), Some(&(nosso_mva, i_ka))) = (r.i_mva, nossos.get(&r.barra)) {
                let erro = if i_mva != 0.0 { (nosso_mva - i_mva) / i_mva } else { 0.0 };
                comparacoes.push(json!({
                    "barra": r.barra,
                    "referencia_mva": i_mva,
                    "nosso_mva": nosso_mva,
                    "i_ka": i_ka,
                    "erro_relativo": erro,
                }));
            }
        }
    }
    let refs: Vec<Value> = referencias.iter().map(|r| json!({ "barra": r.barra, "descricao": r.descricao, "i_mva": r.i_mva })).collect();
    Ok(json!({
        "arquivo": arquivo,
        "referencias": refs,
        "comparacoes": comparacoes,
    }))
}

pub(super) fn run_relatorio(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.relatorio", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.relatorio", format!("{db}: {e}")))?;
    let nome_doc = banco
        .documentos()
        .map_err(|e| bad("anacad.relatorio", e.to_string()))?
        .into_iter()
        .find(|d| d.id == documento)
        .map(|d| d.nome)
        .unwrap_or_else(|| format!("documento {documento}"));
    let resultados = banco.resultados(documento, 100).map_err(|e| bad("anacad.relatorio", e.to_string()))?;
    let mut texto = String::new();
    texto.push_str("# Relatorio AnaCAD\n\n");
    texto.push_str(&format!("Documento: **{}** ({})\n\n", nome_doc, documento));
    texto.push_str(&format!("Resultados: {}\n\n", resultados.len()));
    for r in &resultados {
        texto.push_str(&format!("## #{} — {}\n\n", r.id, r.tipo));
        if let Some(entrada) = &r.entrada {
            texto.push_str(&format!("- entrada: `{entrada}`\n"));
        }
        if let Some(saida) = &r.saida {
            texto.push_str(&format!("- saida: `{saida}`\n"));
        }
        texto.push('\n');
    }
    let arquivo = str_param(p, "arquivo").map(str::to_string).unwrap_or_else(|| format!("{db}.relatorio.md"));
    std::fs::write(&arquivo, texto.as_bytes()).map_err(|e| bad("anacad.relatorio", format!("{arquivo}: {e}")))?;
    Ok(json!({ "arquivo": arquivo, "bytes": texto.len(), "resultados": resultados.len() }))
}

pub(super) fn run_verificar(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.verificar", "`arquivo` (.lst) is required"))?;
    let bytes = ler_arquivo(arquivo)?;
    let tela = anacad_core::ler_tela_tipada(&bytes).map_err(|e| bad("anacad.verificar", format!("{arquivo}: {e}")))?;
    let db = str_param(p, "db").map(str::to_string).unwrap_or_else(|| format!("{arquivo}.verificacao.db"));
    for sufixo in ["", "-wal", "-shm"] {
        let caminho = format!("{db}{sufixo}");
        let _ = std::fs::remove_file(caminho);
    }
    let mut banco = anacad_data::Banco::abrir(&db).map_err(|e| bad("anacad.verificar", format!("{db}: {e}")))?;
    let resumo = banco.importar_screen(&tela, "verificacao", Some(arquivo)).map_err(|e| bad("anacad.verificar", e.to_string()))?;
    let exportada = banco.exportar_screen(resumo.documento).map_err(|e| bad("anacad.verificar", e.to_string()))?;
    let igual = exportada == tela;
    let mut diferencas: Vec<String> = Vec::new();
    if tela.extensao != exportada.extensao {
        diferencas.push("extensao".to_string());
    }
    if tela.modo != exportada.modo {
        diferencas.push("modo".to_string());
    }
    if tela.fonte != exportada.fonte {
        diferencas.push("fonte".to_string());
    }
    if tela.cores != exportada.cores {
        diferencas.push(format!("cores {} -> {}", tela.cores.len(), exportada.cores.len()));
    }
    if tela.grupos != exportada.grupos {
        diferencas.push(format!("grupos {} -> {}", tela.grupos.len(), exportada.grupos.len()));
    }
    if tela.componentes != exportada.componentes {
        let detalhe = tela
            .componentes
            .iter()
            .zip(exportada.componentes.iter())
            .position(|(a, b)| a != b)
            .map(|i| format!("componente[{i}]: {:?} -> {:?}", tela.componentes[i], exportada.componentes[i]))
            .unwrap_or_else(|| format!("componentes {} -> {}", tela.componentes.len(), exportada.componentes.len()));
        diferencas.push(detalhe);
    }
    if tela.linhas != exportada.linhas {
        diferencas.push(format!("linhas {} -> {}", tela.linhas.len(), exportada.linhas.len()));
    }
    if tela.ligacoes != exportada.ligacoes {
        let detalhe = tela
            .ligacoes
            .iter()
            .zip(exportada.ligacoes.iter())
            .position(|(a, b)| a != b)
            .map(|i| format!("ligacao[{i}]: {:?} -> {:?}", tela.ligacoes[i], exportada.ligacoes[i]))
            .unwrap_or_else(|| format!("ligacoes {} -> {}", tela.ligacoes.len(), exportada.ligacoes.len()));
        diferencas.push(detalhe);
    }
    if tela.legendas_componente != exportada.legendas_componente {
        diferencas.push(format!("legendas_componente {} -> {}", tela.legendas_componente.len(), exportada.legendas_componente.len()));
    }
    if tela.legendas_ligacao != exportada.legendas_ligacao {
        let detalhe = tela
            .legendas_ligacao
            .iter()
            .zip(exportada.legendas_ligacao.iter())
            .position(|(a, b)| a != b)
            .map(|i| format!("legenda_ligacao[{i}]: {:?} -> {:?}", tela.legendas_ligacao[i], exportada.legendas_ligacao[i]))
            .unwrap_or_else(|| format!("legendas_ligacao {} -> {}", tela.legendas_ligacao.len(), exportada.legendas_ligacao.len()));
        diferencas.push(detalhe);
    }
    let texto_original = tela.escrever_texto();
    let texto = exportada.escrever_texto();
    let texto_igual = texto_original == texto;
    let mut primeira_diferenca = None;
    let mut linha_original = None;
    let mut linha_exportada = None;
    if !texto_igual && let Some(i) = texto_original.lines().zip(texto.lines()).position(|(a, b)| a != b) {
        primeira_diferenca = Some(i + 1);
        linha_original = texto_original.lines().nth(i).map(str::to_string);
        linha_exportada = texto.lines().nth(i).map(str::to_string);
    }
    Ok(json!({
        "igual": igual,
        "texto_igual": texto_igual,
        "bytes_original": texto_original.len(),
        "primeira_diferenca": primeira_diferenca,
        "linha_original": linha_original,
        "linha_exportada": linha_exportada,
        "diferencas": diferencas,
        "arquivo": arquivo,
        "db": db,
        "barras": resumo.barras,
        "componentes": resumo.componentes,
        "linhas": resumo.linhas,
        "ligacoes": resumo.ligacoes,
        "bytes_exportados": texto.len(),
    }))
}

pub(super) fn run_ana_caso(s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.ana.caso", "`arquivo` is required"))?;
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ana.caso", "`db` is required"))?;
    let nome = str_param(p, "nome").unwrap_or("Caso ANA");
    let bytes = ler_arquivo(arquivo)?;
    let ana = anacad_core::ana::ArquivoAna::ler(&bytes).map_err(|e| bad("anacad.ana.caso", format!("{arquivo}: {e}")))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ana.caso", format!("{db}: {e}")))?;
    let projeto = banco.criar_projeto(nome).map_err(|e| bad("anacad.ana.caso", e.to_string()))?;
    let documento = banco.inserir_documento(projeto, nome, Some(arquivo), None, 0).map_err(|e| bad("anacad.ana.caso", e.to_string()))?;
    for barra in ana.ler_dbar() {
        banco.criar_barra(documento, barra.codigo, Some(&barra.nome), 0.0, 0.0).map_err(|e| bad("anacad.ana.caso", e.to_string()))?;
        if let Some(vbase) = barra.vbase {
            banco.atualizar_barra_vbase(Some(documento), barra.codigo, vbase).map_err(|e| bad("anacad.ana.caso", e.to_string()))?;
        }
    }
    let importacao = run_ana_importar(s, &json!({ "arquivo": arquivo, "db": db, "documento": documento }))?;
    Ok(json!({ "db": db, "documento": documento, "importacao": importacao }))
}

pub(super) fn run_ana(_s: &mut Session, p: &Value) -> Result<Value> {
    let caminho = str_param(p, "arquivo").ok_or_else(|| bad("anacad.ana", "`arquivo` is required"))?;
    let bytes = ler_arquivo(caminho)?;
    let arquivo = anacad_core::ana::ArquivoAna::ler(&bytes).map_err(|e| bad("anacad.ana", format!("{caminho}: {e}")))?;
    let blocos: Vec<Value> = arquivo
        .blocos
        .iter()
        .map(|b| match &b.conteudo {
            anacad_core::ana::Conteudo::Texto(linhas) => json!({
                "nome": b.nome,
                "tipo": "texto",
                "linhas": linhas.len(),
            }),
            anacad_core::ana::Conteudo::Tabular { cabecalho, linhas } => json!({
                "nome": b.nome,
                "tipo": "tabular",
                "linhas": linhas.len(),
                "cabecalho": cabecalho,
            }),
        })
        .collect();
    Ok(json!({ "arquivo": caminho, "tipo": arquivo.tipo, "blocos": blocos }))
}

pub(super) fn run_curto(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.curto";
    use anacad_core::calc::{Complexo, TipoFalta};
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("barra").and_then(Value::as_i64).ok_or_else(|| bad(CMD, "`barra` (codigo) is required"))?;
    let sbase = f64_or(p, "sbase", 100.0);
    let tipo_texto = str_param(p, "tipo").unwrap_or("trifasica");
    let tipo_falta = TipoFalta::parse(tipo_texto).ok_or_else(|| bad(CMD, "tipo deve ser trifasica, bifasica ou monofasica"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let rede = rede_do_banco(&banco, documento, p, CMD)?;
    let falta = rede.indice(codigo).ok_or_else(|| bad(CMD, format!("barra {codigo} fora da rede energizada (sem caminho ate a terra)")))?;
    if rede.indice_referencia() == Some(falta) {
        return Err(bad(CMD, "a barra da falta e a barra infinita de referencia"));
    }
    let fator = fatorar(&rede.y1, CMD)?;
    let z1 = fator.zth(falta).ok_or_else(|| bad(CMD, "Zth nulo"))?;
    let z0 = if tipo_falta == TipoFalta::Monofasica { Some(fatorar(&rede.y0, CMD)?.zth(falta).ok_or_else(|| bad(CMD, "Z0 nulo"))?) } else { None };
    let falta_calc = anacad_core::calc::curto_com_zth(z1, z0, tipo_falta).ok_or_else(|| bad(CMD, "Zth nulo"))?;
    let zf_pu = f64_or(p, "zf", 0.0);
    let equivalente = falta_calc.zth.soma(Complexo::novo(zf_pu, 0.0));

    // Fontes de corrente limitada (DEOL): I_src·Z_fg/Z_ff somada em fase.
    let mut contribuicao = Complexo::ZERO;
    let mut fontes = 0usize;
    for gerador in banco.geradores(documento, 100_000).map_err(|e| bad(CMD, e.to_string()))? {
        let (Some(nb), Some(imax)) = (gerador.nb, gerador.i_max) else { continue };
        let unidades = gerador.unidades.unwrap_or(1.0);
        let Some(k) = rede.indice(nb) else { continue };
        let ka = ka_por_pu(&banco, documento, nb, sbase, CMD)?;
        if imax <= 0.0 || unidades <= 0.0 || ka <= 0.0 {
            continue;
        }
        let mut b = vec![Complexo::ZERO; rede.y1.indices.len()];
        b[k] = Complexo::novo(1.0, 0.0);
        if let Some(zfg) = fator.resolver(&b).and_then(|x| x.get(falta).copied()) {
            contribuicao = contribuicao.soma(Complexo::novo(imax * unidades / 1e3 / ka, 0.0).multiplica(zfg.divide(z1)));
            fontes += 1;
        }
    }
    let total = Complexo::novo(1.0, 0.0).divide(equivalente).soma(contribuicao);

    let dados = banco.dados_barra(documento, codigo).map_err(|e| bad(CMD, e.to_string()))?.and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let prefalta = dados.as_ref().and_then(|d| d.get("preFaltaModulo")).and_then(Value::as_f64).filter(|v| *v > 0.0).unwrap_or(1.0);
    let prefalta_angulo = dados.as_ref().and_then(|d| d.get("preFaltaAngulo")).and_then(Value::as_f64).unwrap_or(0.0);
    let i_pu = total.modulo() * prefalta;
    let vbase = banco.barra_vbase(documento, codigo).map_err(|e| bad(CMD, e.to_string()))?.unwrap_or(0.0);
    let i_ka = i_pu * ka_por_pu(&banco, documento, codigo, sbase, CMD)?;
    let i_angulo = prefalta_angulo - equivalente.im.atan2(equivalente.re).to_degrees();

    let tensoes: Vec<Value> = if p.get("tensoes").and_then(Value::as_bool).unwrap_or(false) {
        anacad_core::calc::tensoes_pos_falta(&rede.y1, Some(&rede.y0), falta, 0, tipo_falta, 0.0)
            .unwrap_or_default()
            .iter()
            .enumerate()
            .map(|(k, fases)| {
                json!({
                    "barra": rede.codigo(k).unwrap_or(0),
                    "modulo": fases[0].modulo(),
                    "angulo": fases[0].im.atan2(fases[0].re).to_degrees(),
                    "fase_a": polar(fases[0]),
                    "fase_b": polar(fases[1]),
                    "fase_c": polar(fases[2]),
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    let contribuicoes: Vec<Value> = if p.get("contribuicoes").and_then(Value::as_bool).unwrap_or(false) {
        anacad_core::calc::correntes_de_ramo(&rede.y1, &rede.circuitos1, falta, 0, 0.0)
            .unwrap_or_default()
            .into_iter()
            .take(50)
            .map(|c| json!({"de": rede.codigo_do_id(c.de).unwrap_or(0), "para": rede.codigo_do_id(c.para).unwrap_or(0), "i_pu": c.i_pu}))
            .collect()
    } else {
        Vec::new()
    };
    let mut saida = json!({
        "barra": codigo,
        "referencia": rede.codigo_referencia(),
        "tipo": tipo_texto,
        "zth_re": equivalente.re,
        "zth_im": equivalente.im,
        "i_pu": i_pu,
        "i_fonte_pu": contribuicao.modulo(),
        "prefalta": prefalta,
        "prefalta_angulo": prefalta_angulo,
        "angulo_graus": i_angulo,
        "fontes": fontes,
        "zf_pu": zf_pu,
        "contribuicoes": contribuicoes,
        "tensoes": tensoes,
        "vbase": vbase,
        "i_ka": i_ka,
    });
    let entrada = json!({ "barra": codigo, "referencia": rede.codigo_referencia(), "sbase": sbase, "tipo": tipo_texto });
    let id =
        banco.inserir_resultado(Some(documento), "curto_circuito", &entrada.to_string(), &saida.to_string()).map_err(|e| bad(CMD, e.to_string()))?;
    saida["resultado"] = json!(id);
    Ok(saida)
}

pub(super) fn run_resultados(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.resultados", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let limite = p.get("limite").and_then(Value::as_i64).unwrap_or(10);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.resultados", format!("{db}: {e}")))?;
    let resultados = serde_json::to_value(banco.resultados(documento, limite).map_err(|e| bad("anacad.resultados", e.to_string()))?)
        .map_err(|e| bad("anacad.resultados", e.to_string()))?;
    Ok(json!({ "db": db, "documento": documento, "resultados": resultados }))
}

pub(super) fn run_ybus(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.ybus";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let rede = rede_do_banco(&banco, documento, p, CMD)?;
    let resumo = rede.y1.resumo();
    let mut saida = json!({
        "barras": resumo.barras,
        "circuitos_usados": resumo.circuitos_usados,
        "circuitos_sem_barra": resumo.circuitos_sem_barra,
        "circuitos_ignorados": rede.circuitos_ignorados,
        "desenergizadas": rede.desenergizadas,
        "referencia": rede.codigo_referencia(),
        "pares_nao_nulos": resumo.pares_nao_nulos,
        "simetrica": resumo.simetrica,
        "maior_diagonal": resumo.maior_diagonal,
    });
    let id = banco
        .inserir_resultado(Some(documento), "ybus", &json!({ "base_mva": 100 }).to_string(), &saida.to_string())
        .map_err(|e| bad(CMD, e.to_string()))?;
    saida["resultado"] = json!(id);
    Ok(saida)
}

pub(super) fn run_calcular(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.calcular", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.calcular", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.calcular", e.to_string()))?;
    let linhas = banco.linhas(documento, 100_000).map_err(|e| bad("anacad.calcular", e.to_string()))?;
    let entrada = anacad_core::calc::Entrada {
        barras: barras.iter().map(|b| b.id).collect(),
        linhas: linhas
            .iter()
            .filter_map(|l| match (l.de, l.para) {
                (Some(de), Some(para)) => Some((de, para)),
                _ => None,
            })
            .collect(),
    };
    let resumo = anacad_core::calc::topologia(&entrada);
    let entrada_json = json!({
        "barras": entrada.barras.len(),
        "linhas": entrada.linhas.len(),
    });
    let saida_json = json!({
        "barras": resumo.barras,
        "linhas": resumo.linhas,
        "ilhas": resumo.ilhas,
        "barras_isoladas": resumo.barras_isoladas,
        "grau_max": resumo.grau_max,
        "grau_medio": resumo.grau_medio,
        "linhas_sem_barra": resumo.linhas_sem_barra,
    });
    let id = banco
        .inserir_resultado(Some(documento), "topologia", &entrada_json.to_string(), &saida_json.to_string())
        .map_err(|e| bad("anacad.calcular", e.to_string()))?;
    Ok(json!({
        "resultado": id,
        "barras": resumo.barras,
        "linhas": resumo.linhas,
        "ilhas": resumo.ilhas,
        "barras_isoladas": resumo.barras_isoladas,
        "grau_max": resumo.grau_max,
        "grau_medio": resumo.grau_medio,
        "linhas_sem_barra": resumo.linhas_sem_barra,
    }))
}

pub(super) fn run_validar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.validar", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.validar", format!("{db}: {e}")))?;
    let documentos = banco.documentos().map_err(|e| bad("anacad.validar", e.to_string()))?;
    let mut componentes = 0usize;
    let mut componentes_ok = 0usize;
    let mut ligacoes = 0usize;
    let mut ligacoes_ok = 0usize;
    for documento in &documentos {
        for c in banco.componentes(documento.id, 10_000).map_err(|e| bad("anacad.validar", e.to_string()))? {
            let Some(texto) = c.handle else { continue };
            componentes += 1;
            if let Some(h) = Handle::parse_hex(&texto)
                && s.doc()?.entity(h).is_some()
            {
                componentes_ok += 1;
            }
        }
        for u in banco.ligacoes(documento.id, 10_000).map_err(|e| bad("anacad.validar", e.to_string()))? {
            let Some(texto) = u.handle else { continue };
            ligacoes += 1;
            if let Some(h) = Handle::parse_hex(&texto)
                && s.doc()?.entity(h).is_some()
            {
                ligacoes_ok += 1;
            }
        }
    }
    Ok(json!({
        "documentos": documentos.len(),
        "componentes": componentes,
        "componentes_ok": componentes_ok,
        "componentes_sem_entidade": componentes - componentes_ok,
        "ligacoes": ligacoes,
        "ligacoes_ok": ligacoes_ok,
        "ligacoes_sem_entidade": ligacoes - ligacoes_ok,
    }))
}

pub(super) fn run_anotar_curto(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.anotar.curto", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let limite = p.get("limite").and_then(Value::as_i64).unwrap_or(50);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.anotar.curto", format!("{db}: {e}")))?;
    let resultados = banco.resultados(documento, limite).map_err(|e| bad("anacad.anotar.curto", e.to_string()))?;
    let mut anotacoes = 0;
    for r in resultados {
        if r.tipo != "curto_circuito" {
            continue;
        }
        let Some(saida) = r.saida.as_deref().and_then(|t| serde_json::from_str::<Value>(t).ok()) else { continue };
        let Some(codigo) = saida.get("barra").and_then(Value::as_i64) else { continue };
        let Some(i_ka) = saida.get("i_ka").and_then(Value::as_f64) else { continue };
        let Ok(Some(barra)) = banco.barra_por_codigo(documento, codigo) else { continue };
        let (x, y) = (barra.x.unwrap_or(0.0), barra.y.unwrap_or(0.0) + 3.0);
        let texto = format!("Icc {i_ka:.2} kA");
        let entidade = EntityKind::Text(Text {
            insert: Vec3::new(x, y, 0.0),
            align_pt: None,
            height: 2.0,
            value: texto,
            rotation: 0.0,
            width_factor: 1.0,
            oblique: 0.0,
            style: "Standard".into(),
            halign: HAlign::Left,
            valign: VAlign::Baseline,
        });
        let handle = s.add_entity(entidade)?;
        marcar(s, handle, "ANACAD_ANOTACAO", &[("resultado", &r.id.to_string()), ("barra", &codigo.to_string())])?;
        anotacoes += 1;
    }
    Ok(json!({"anotacoes": anotacoes}))
}

pub(super) fn run_niveis_curto(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.niveis.curto";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let sbase = f64_or(p, "sbase", 100.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let (niveis, referencia, desenergizadas) = niveis_do_documento(&banco, documento, sbase, CMD, p)?;
    let mut saida = json!({"niveis": niveis, "referencia": referencia, "desenergizadas": desenergizadas});
    let id = banco
        .inserir_resultado(Some(documento), "niveis", &json!({"sbase": sbase}).to_string(), &saida.to_string())
        .map_err(|e| bad(CMD, e.to_string()))?;
    saida["resultado"] = json!(id);
    Ok(saida)
}

/// Estudo 4.6 do manual: evolução dos níveis de curto entre duas configurações
/// (documentos do mesmo banco), por barra, com delta em kA e percentual.
pub(super) fn run_evolucao_niveis(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.evolucao.niveis";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let base_doc = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let novo_doc = p.get("comparar").and_then(Value::as_i64).ok_or_else(|| bad(CMD, "`comparar` (documento) is required"))?;
    let sbase = f64_or(p, "sbase", 100.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let (base, _, _) = niveis_do_documento(&banco, base_doc, sbase, CMD, p)?;
    let (novo, _, _) = niveis_do_documento(&banco, novo_doc, sbase, CMD, p)?;
    let por_barra = |lista: &[Value]| {
        lista
            .iter()
            .filter_map(|n| n.get("barra").and_then(Value::as_i64).map(|c| (c, n.clone())))
            .collect::<std::collections::BTreeMap<i64, Value>>()
    };
    let antigos = por_barra(&base);
    let atuais = por_barra(&novo);
    let mut barras = Vec::new();
    for (codigo, antes) in &antigos {
        let Some(depois) = atuais.get(codigo) else {
            continue;
        };
        let i0 = antes.get("i3f_ka").and_then(Value::as_f64).unwrap_or(0.0);
        let i1 = depois.get("i3f_ka").and_then(Value::as_f64).unwrap_or(0.0);
        let delta = i1 - i0;
        barras.push(json!({
            "barra": codigo,
            "i3f_base_ka": i0,
            "i3f_novo_ka": i1,
            "delta_ka": delta,
            "delta_pct": if i0.abs() > 1e-12 { Some(delta / i0 * 100.0) } else { None },
        }));
    }
    let so_base = antigos.keys().filter(|c| !atuais.contains_key(c)).count();
    let so_novo = atuais.keys().filter(|c| !antigos.contains_key(c)).count();
    Ok(json!({
        "base": base_doc,
        "comparar": novo_doc,
        "barras": barras,
        "so_base": so_base,
        "so_novo": so_novo,
    }))
}

/// Estudo Macro (manual §4.2), fatia de barras × contingências: roda os níveis
/// para cada caso (conjunto de ramos abertos) e devolve a matriz por barra.
pub(super) fn run_macro(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.macro";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let sbase = f64_or(p, "sbase", 100.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let filtro: Option<Vec<i64>> = p.get("barras").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect());
    let casos: Vec<Value> = p.get("casos").and_then(Value::as_array).cloned().unwrap_or_else(|| vec![json!({})]);
    if casos.is_empty() {
        return Err(bad(CMD, "`casos` vazio"));
    }
    let mut saida = Vec::new();
    for (indice, caso) in casos.iter().enumerate() {
        let mut parametros = p.clone();
        if let Some(obj) = parametros.as_object_mut() {
            obj.insert("abrir".into(), caso.get("abrir").cloned().unwrap_or_else(|| json!([])));
        }
        let (niveis, _, _) = niveis_do_documento(&banco, documento, sbase, CMD, &parametros)?;
        let niveis: Vec<Value> = match &filtro {
            Some(lista) => niveis.into_iter().filter(|n| n.get("barra").and_then(Value::as_i64).is_some_and(|c| lista.contains(&c))).collect(),
            None => niveis,
        };
        saida.push(json!({
            "caso": indice + 1,
            "nome": caso.get("nome").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| format!("caso {}", indice + 1)),
            "abrir": caso.get("abrir").cloned().unwrap_or_else(|| json!([])),
            "niveis": niveis,
        }));
    }
    Ok(json!({"documento": documento, "casos": saida}))
}

/// Estudo 4.3 do manual (fatia inicial): superação dos disjuntores — compara o
/// I3F de cada barra com a capacidade de interrupção registrada nos dados da
/// barra (`capacidadeInterrupcao` ou a menor `capacidadesAmarre`).
pub(super) fn run_superacao(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.superacao";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let sbase = f64_or(p, "sbase", 100.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let (niveis, _, _) = niveis_do_documento(&banco, documento, sbase, CMD, p)?;
    // Lista de barras analisadas (vazia = todas).
    let filtro: Vec<i64> = p.get("barras").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    let mut barras = Vec::new();
    let mut superam = 0;
    let mut sem_capacidade = 0;
    for nivel in niveis {
        let Some(codigo) = nivel.get("barra").and_then(Value::as_i64) else {
            continue;
        };
        if !filtro.is_empty() && !filtro.contains(&codigo) {
            continue;
        }
        let i3f = nivel.get("i3f_ka").and_then(Value::as_f64).unwrap_or(0.0);
        let dados = banco.dados_barra(documento, codigo).ok().flatten().and_then(|texto| serde_json::from_str::<Value>(&texto).ok());
        let capacidade = dados.as_ref().and_then(|d| d.get("capacidadeInterrupcao")).and_then(Value::as_f64).filter(|c| *c > 0.0);
        let amarre: Vec<f64> = dados
            .as_ref()
            .and_then(|d| d.get("capacidadesAmarre"))
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_f64).collect())
            .unwrap_or_default();
        let menor_amarre = amarre.iter().cloned().fold(f64::INFINITY, f64::min);
        let capacidade = capacidade.or(if menor_amarre.is_finite() { Some(menor_amarre) } else { None });
        let Some(cap) = capacidade else {
            sem_capacidade += 1;
            continue;
        };
        let margem = i3f / cap;
        let supera = margem > 1.0;
        if supera {
            superam += 1;
        }
        barras.push(json!({
            "barra": codigo,
            "i3f_ka": i3f,
            "capacidade_ka": cap,
            "margem": margem,
            "supera": supera,
            "xr": nivel.get("xr").and_then(Value::as_f64).unwrap_or(0.0),
        }));
    }
    barras.sort_by(|a, b| {
        let ma = a.get("margem").and_then(Value::as_f64).unwrap_or(0.0);
        let mb = b.get("margem").and_then(Value::as_f64).unwrap_or(0.0);
        mb.total_cmp(&ma)
    });
    // Correntes por terminal de circuito (relatório por circuito do 4.3).
    // Aproximação conservadora: máxima corrente entre faltas nas duas barras
    // terminais, com a pré-falta da barra da falta.
    let mut circuitos = Vec::new();
    let mut superam_circuitos = 0usize;
    let mut sem_capacidade_circuitos = 0usize;
    if let Ok(rede) = rede_do_banco(&banco, documento, p, CMD) {
        if rede.circuitos1.is_empty() {
            circuitos.clear();
        }
        if let Some(zmat) = anacad_core::calc::zbus(&rede.y1, 0, 0.0) {
            for c in banco.circuitos(documento, i64::MAX).map_err(|e| bad(CMD, e.to_string()))? {
                let (Some(de), Some(para)) = (c.de, c.para) else {
                    continue;
                };
                if de == 0 || para == 0 {
                    continue;
                }
                let (Some(i), Some(j)) = (rede.indice(de), rede.indice(para)) else {
                    continue;
                };
                let (Some(r1), Some(x1)) = (c.r1, c.x1) else {
                    continue;
                };
                let z = anacad_core::calc::Complexo::novo(r1 / 100.0, x1 / 100.0);
                if z.modulo() < 1e-9 {
                    continue;
                }
                let tap = c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0);
                let mut corrente_de = 0.0f64;
                let mut corrente_para = 0.0f64;
                for (codigo, k) in [(de, i), (para, j)] {
                    let Some((ide, ipara)) = anacad_core::calc::corrente_de_circuito(&zmat, k, i, j, z, tap) else {
                        continue;
                    };
                    let prefalta = banco
                        .dados_barra(documento, codigo)
                        .ok()
                        .flatten()
                        .and_then(|texto| serde_json::from_str::<Value>(&texto).ok())
                        .and_then(|d| d.get("preFaltaModulo").and_then(Value::as_f64))
                        .filter(|v| *v > 0.0)
                        .unwrap_or(1.0);
                    let ka = ka_por_pu(&banco, documento, codigo, sbase, CMD)?;
                    corrente_de = corrente_de.max(ide.modulo() * prefalta * ka);
                    corrente_para = corrente_para.max(ipara.modulo() * prefalta * ka);
                }
                let percentual_bf = c.dj_bf.filter(|v| *v > 0.0).map(|cap| corrente_de / cap * 100.0);
                let percentual_bt = c.dj_bt.filter(|v| *v > 0.0).map(|cap| corrente_para / cap * 100.0);
                if percentual_bf.is_none() && percentual_bt.is_none() {
                    sem_capacidade_circuitos += 1;
                    continue;
                }
                let supera_circuito = percentual_bf.map(|p| p > 100.0).unwrap_or(false) || percentual_bt.map(|p| p > 100.0).unwrap_or(false);
                if supera_circuito {
                    superam_circuitos += 1;
                }
                circuitos.push(json!({
                    "de": de,
                    "para": para,
                    "i_de_ka": corrente_de,
                    "i_para_ka": corrente_para,
                    "capacidade_de_ka": c.dj_bf,
                    "capacidade_para_ka": c.dj_bt,
                    "percentual_de": percentual_bf,
                    "percentual_para": percentual_bt,
                    "supera": supera_circuito,
                }));
            }
        }
    }
    circuitos.sort_by(|a, b| {
        let ma = a.get("percentual_de").and_then(Value::as_f64).unwrap_or(0.0).max(a.get("percentual_para").and_then(Value::as_f64).unwrap_or(0.0));
        let mb = b.get("percentual_de").and_then(Value::as_f64).unwrap_or(0.0).max(b.get("percentual_para").and_then(Value::as_f64).unwrap_or(0.0));
        mb.total_cmp(&ma)
    });
    let mut saida = json!({
        "documento": documento,
        "barras": barras,
        "superam": superam,
        "sem_capacidade": sem_capacidade,
        "circuitos": circuitos,
        "superam_circuitos": superam_circuitos,
        "sem_capacidade_circuitos": sem_capacidade_circuitos,
    });
    let id = banco
        .inserir_resultado(Some(documento), "superacao", &json!({"sbase": sbase}).to_string(), &saida.to_string())
        .map_err(|e| bad(CMD, e.to_string()))?;
    saida["resultado"] = json!(id);
    Ok(saida)
}

pub(super) fn run_equivalente(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.equivalente";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let manter: Vec<i64> = p.get("barras").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    if manter.is_empty() {
        return Err(bad(CMD, "informe as barras a manter"));
    }
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let rede = rede_do_banco(&banco, documento, p, CMD)?;
    let manter_ids: Vec<i64> = manter.iter().filter_map(|c| rede.indice(*c)).map(|k| rede.y1.indices[k]).collect();
    let eq = anacad_core::calc::reduzir_ybus(&rede.y1, &manter_ids).ok_or_else(|| bad(CMD, "nao foi possivel reduzir (matriz singular)"))?;
    let resumo = eq.resumo();
    let barras_equivalentes: Vec<i64> = eq.indices.iter().map(|id| rede.codigo_do_id(*id).unwrap_or(0)).collect();
    // Validação: Zth de cada barra mantida na rede original × no equivalente.
    let validacao = if p.get("validar").and_then(Value::as_bool).unwrap_or(false) {
        let mut amostras = Vec::new();
        let mut maior = 0.0f64;
        for (k, id) in eq.indices.iter().enumerate() {
            let Some(pos_o) = rede.y1.indices.iter().position(|b| b == id) else { continue };
            let (Some(z_o), Some(z_e)) = (anacad_core::calc::zth(&rede.y1, pos_o, 0, 0.0), anacad_core::calc::zth(&eq, k, 0, 0.0)) else {
                continue;
            };
            let erro = if z_o.modulo() > 1e-12 { z_o.subtrai(z_e).modulo() / z_o.modulo() } else { 0.0 };
            maior = maior.max(erro);
            amostras.push(json!({
                "barra": barras_equivalentes[k],
                "z_original": {"re": z_o.re, "im": z_o.im},
                "z_equivalente": {"re": z_e.re, "im": z_e.im},
                "erro": erro,
            }));
        }
        Some(json!({"maior_erro": maior, "amostras": amostras}))
    } else {
        None
    };
    let mut saida = json!({
        "barras": barras_equivalentes,
        "pares_nao_nulos": resumo.pares_nao_nulos,
        "simetrica": resumo.simetrica,
        "maior_diagonal": resumo.maior_diagonal,
    });
    let entrada = json!({"barras": manter, "documento": documento}).to_string();
    let id = banco.inserir_resultado(Some(documento), "equivalente", &entrada, &saida.to_string()).map_err(|e| bad(CMD, e.to_string()))?;
    saida["resultado"] = json!(id);
    saida["validacao"] = json!(validacao);
    Ok(saida)
}

/// Estudo 4.7 do manual: impedância equivalente entre duas barras, pela Zbus
/// do caso (`Zii + Zjj − Zij − Zji`).
pub(super) fn run_impedancia_barras(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.impedancia.barras";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let de = p.get("de").and_then(Value::as_i64).ok_or_else(|| bad(CMD, "`de` is required"))?;
    let para = p.get("para").and_then(Value::as_i64).ok_or_else(|| bad(CMD, "`para` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let rede = rede_do_banco(&banco, documento, p, CMD)?;
    let unidade = str_param(p, "unidade").unwrap_or("polar").to_lowercase();
    if !matches!(unidade.as_str(), "polar" | "retangular" | "retangulares") {
        return Err(bad(CMD, "unidade deve ser polar ou retangular"));
    }
    let i = rede.indice(de).ok_or_else(|| bad(CMD, format!("barra {de} nao esta na rede energizada")))?;
    let j = rede.indice(para).ok_or_else(|| bad(CMD, format!("barra {para} nao esta na rede energizada")))?;
    let z = anacad_core::calc::zbus(&rede.y1, 0, 0.0).ok_or_else(|| bad(CMD, "matriz singular"))?;
    let zij = anacad_core::calc::impedancia_entre_barras(&z, i, j).ok_or_else(|| bad(CMD, "indice fora da Zbus"))?;
    Ok(json!({
        "de": de,
        "para": para,
        "unidade": unidade,
        "z_re": zij.re,
        "z_im": zij.im,
        "modulo": zij.modulo(),
        "angulo_graus": zij.im.atan2(zij.re).to_degrees(),
    }))
}

pub(super) fn run_zbus(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.zbus";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let rede = rede_do_banco(&banco, documento, p, CMD)?;
    if rede.y1.indices.is_empty() {
        return Ok(json!({"barras": [], "diagonal": []}));
    }
    let z = anacad_core::calc::zbus(&rede.y1, 0, 0.0).ok_or_else(|| bad(CMD, "matriz singular"))?;
    let barras: Vec<i64> = (0..rede.y1.indices.len()).filter_map(|k| rede.codigo(k)).collect();
    let diagonal: Vec<Value> = (0..rede.y1.indices.len())
        .filter(|k| !rede.ficticia(*k) && rede.indice_referencia() != Some(*k))
        .map(|k| json!({"barra": rede.codigo(k).unwrap_or(0), "z_re": z[k][k].re, "z_im": z[k][k].im, "modulo": z[k][k].modulo()}))
        .collect();
    Ok(json!({"barras": barras, "diagonal": diagonal, "referencia": rede.codigo_referencia()}))
}

pub(super) fn run_curto_simultaneo(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.curto.simultaneo";
    let db = str_param(p, "db").ok_or_else(|| bad(CMD, "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let sbase = f64_or(p, "sbase", 100.0);
    let barras = p.get("barras").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>()).unwrap_or_default();
    if barras.is_empty() {
        return Err(bad(CMD, "informe as barras"));
    }
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad(CMD, format!("{db}: {e}")))?;
    let rede = rede_do_banco(&banco, documento, p, CMD)?;
    let faltas: Vec<usize> = barras.iter().filter_map(|c| rede.indice(*c)).filter(|k| rede.indice_referencia() != Some(*k)).collect();
    if faltas.is_empty() {
        return Err(bad(CMD, "barras nao encontradas na rede energizada"));
    }
    let correntes = anacad_core::calc::faltas_simultaneas_3f(&rede.y1, &faltas, 0, 0.0).ok_or_else(|| bad(CMD, "matriz singular"))?;
    let mut saida = Vec::new();
    for (k, idx) in faltas.iter().enumerate() {
        let codigo = rede.codigo(*idx).unwrap_or(0);
        saida.push(json!({
            "barra": codigo,
            "i_pu": correntes[k].modulo(),
            "i_ka": correntes[k].modulo() * ka_por_pu(&banco, documento, codigo, sbase, CMD)?,
            "angulo_graus": correntes[k].im.atan2(correntes[k].re).to_degrees(),
        }));
    }
    Ok(json!({"correntes": saida}))
}

pub(super) fn run_teste_bloco(s: &mut Session, p: &Value) -> Result<Value> {
    let nome = str_param(p, "nome").unwrap_or("ANACAD_TESTE_BLOCO").trim().to_string();
    if nome.is_empty() {
        return Err(bad("anacad.teste.bloco", "nome vazio"));
    }
    if s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(&nome)) {
        return Err(bad("anacad.teste.bloco", format!("bloco ja existe: {nome}")));
    }
    let mut estado = p.get("semente").and_then(Value::as_u64).unwrap_or(0x5eed_1234);
    let mut proximo = || {
        estado = estado.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((estado >> 33) as f64) / ((1u64 << 31) as f64)
    };
    let entidades = 6usize;
    let doc = s.doc_mut()?;
    let mut bloco = Block::new(&nome);
    bloco.description = "Bloco aleatorio de teste (anacad)".to_string();
    for i in 0..entidades {
        let x = (proximo() - 0.5) * 40.0;
        let y = (proximo() - 0.5) * 40.0;
        let dx = (proximo() - 0.5) * 20.0;
        let dy = (proximo() - 0.5) * 20.0;
        if i % 2 == 0 {
            push_linha(doc, &mut bloco, (x, y), (x + dx, y + dy));
        } else {
            push_circulo(doc, &mut bloco, (x, y), 2.0 + proximo() * 4.0);
        }
    }
    doc.blocks.insert(nome.clone(), Arc::new(bloco));
    let handle = if p.get("inserir").and_then(Value::as_bool).unwrap_or(false) {
        Some(super::blocks::insert(s, &nome, Vec2::new(0.0, 0.0), 1.0, 0.0, &serde_json::Map::new())?)
    } else {
        None
    };
    Ok(json!({
        "bloco": nome,
        "entidades": entidades,
        "handle": handle.map(|h| h.hex()),
    }))
}
