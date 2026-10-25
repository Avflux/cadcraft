//! Comandos do dominio anacad (M1/M2: diagnostico, bloco de teste e importacao).
//!
//! Este arquivo e parte do patch minimo descrito em
//! `anacad/docs/cadcraft-integracao.md`: o cadcraft continua sendo o upstream e
//! o patch e reaplicado por `anacad/scripts/cadcraft-patch.ps1`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use cadcraft_doc::{Block, Circle, Entity, EntityKind, Handle, Line, LwPolyline, RawTag};
use cadcraft_geom::{PolyVertex, Vec2, Vec3};
use serde_json::{Value, json};

use super::*;
use crate::{Accept, Input, Interactive, Prompt, Result, Session, Step};

const BLOCO_TESTE: &str = "ANACAD_TESTE";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("anacad", "AnaCAD (diagnostico/importar)", run_anacad)
            .menu(&[])
            .params("{file?: .lst para diagnostico, test_block?: bool, import?: .lst para importar, db?: .db, nome?: nome do documento} -> {...}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.consultar", "AnaCAD: Consultar Banco", run_consultar)
            .menu(&[])
            .params("{db, documento?: id} -> {documentos, documento?, componentes, linhas, ligacoes, contagens}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.mover", "AnaCAD: Mover Componente", run_mover)
            .menu(&[])
            .params("{db, handle, dx?, dy?} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.apagar", "AnaCAD: Apagar Componente/Ligacao", run_apagar)
            .menu(&[])
            .params("{db, handle} -> {removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.comprimento", "AnaCAD: Calcular Comprimentos", run_comprimento)
            .menu(&[])
            .params("{db, documento?} -> {resultado, ligacoes, comprimento_ligacoes, comprimento_linhas, comprimento_total}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.verificar", "AnaCAD: Verificar .LST", run_verificar)
            .menu(&["AnaCAD", "Arquivo", "Verificar .LST"])
            .params("{arquivo, db?} -> {igual, barras, componentes, linhas, ligacoes, bytes_exportados}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana.caso", "AnaCAD: Abrir .ANA como caso", run_ana_caso)
            .menu(&["AnaCAD", "Arquivo", "Abrir .ANA como caso"])
            .params("{arquivo, db, nome?} -> {db, documento, importacao}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana.importar", "AnaCAD: Importar .ANA (DBAR)", run_ana_importar)
            .menu(&[])
            .params("{arquivo, db, documento?} -> {barras, atualizadas, sem_barra, sem_vbase}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana", "AnaCAD: Diagnostico .ANA", run_ana)
            .menu(&["AnaCAD", "Arquivo", "Diagnóstico .ANA"])
            .params("{arquivo} -> {tipo, blocos: [{nome, tipo, linhas, cabecalho?}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.curto", "AnaCAD: Curto-Circuito (simplificado)", run_curto)
            .menu(&[])
            .params("{db, documento?, barra (codigo), referencia (codigo)?, sbase?} -> {resultado, barra, referencia, zth, i_pu, vbase, i_ka}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.referencia", "AnaCAD: Comparar com ANAFAS", run_referencia)
            .menu(&["AnaCAD", "Dados", "Comparar com ANAFAS"])
            .params("{arquivo (.OUT), db?, documento?, barra?} -> {referencias, comparacoes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.relatorio", "AnaCAD: Relatorio", run_relatorio)
            .menu(&[])
            .params("{db, documento?, arquivo?} -> {arquivo, bytes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.resultados", "AnaCAD: Resultados", run_resultados)
            .menu(&[])
            .params("{db, documento?, limite?} -> {resultados: [{id, tipo, saida, criado_em}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ybus", "AnaCAD: Montar Ybus", run_ybus)
            .menu(&[])
            .params("{db, documento?} -> {resultado, barras, circuitos_usados, circuitos_sem_barra, pares_nao_nulos, simetrica, maior_diagonal}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.calcular", "AnaCAD: Calcular Topologia", run_calcular)
            .menu(&[])
            .params("{db, documento?} -> {resultado, barras, linhas, ilhas, barras_isoladas, grau_max, grau_medio}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.validar", "AnaCAD: Validar Handles", run_validar)
            .menu(&["AnaCAD", "Dados", "Validar Handles"])
            .params("{db} -> {componentes, componentes_ok, componentes_sem_entidade, ligacoes, ligacoes_ok, ligacoes_sem_entidade}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.criar", "AnaCAD: Criar Legenda", run_criar_legenda)
            .menu(&[])
            .params("{db, documento?, tipo?: X|Y, numero, x, y, canal?} -> {legenda, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.mover", "AnaCAD: Mover Legenda", run_mover_legenda)
            .menu(&[])
            .params("{db, handle, dx?, dy?} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.criar", "AnaCAD: Criar Barra", run_criar_barra)
            .menu(&[])
            .params("{db, documento?, codigo, x?, y?, x2?, y2?, comprimento?, nome?} -> {barra, handle, x1, y1, x2, y2, largura}")
            .interactive(|_| Ok(Box::new(BarraM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.criar", "AnaCAD: Criar Linha", run_criar_linha)
            .menu(&[])
            .params("{db, documento?, de (codigo da barra), para (codigo da barra)} -> {linha, handle, x1, y1, x2, y2, largura}")
            .interactive(|_| Ok(Box::new(LinhaM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.atribuir", "AnaCAD: Atribuir Barra à Polilinha", run_atribuir_barra)
            .menu(&[])
            .params("{db, documento?, handle, codigo, nome?} -> {barra, handle, x1, y1, x2, y2, largura}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.atribuir", "AnaCAD: Atribuir Linha à Polilinha", run_atribuir_linha)
            .menu(&[])
            .params("{db, documento?, handle Polilinha, de, para} -> {linha, handle, x1, y1, x2, y2, largura}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.editar", "AnaCAD: Editar Barra (janela)", run_editar_barra)
            .menu(&[])
            .params("{db, documento?, handle, codigo?, nome?, dados} -> {barra, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.editar", "AnaCAD: Editar Linha (janela)", run_editar_linha)
            .menu(&[])
            .params("{db, documento?, handle, de?, para?, dados} -> {linha, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.banco.criar", "AnaCAD: Novo Banco", run_banco_criar)
            .menu(&[])
            .params("{arquivo} -> {arquivo, nome, versao, documentos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.banco.abrir", "AnaCAD: Abrir Banco", run_banco_abrir)
            .menu(&[])
            .params("{arquivo} -> {arquivo, nome, versao, documentos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.documento.listar", "AnaCAD: Listar Documentos", run_documento_listar)
            .menu(&[])
            .params("{db} -> {documentos: [{id, nome, pagina}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.documento.criar", "AnaCAD: Novo Documento", run_documento_criar)
            .menu(&[])
            .params("{db, nome, projeto?} -> {documento, projeto, nome, documentos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.exportar", "AnaCAD: Exportar .lst", run_exportar)
            .menu(&[])
            .params("{db, documento, saida} -> {saida, registros, mensagem}")
            .enabled(always)
            .noundo(),
    ]
}

fn run_anacad(s: &mut Session, p: &Value) -> Result<Value> {
    let mut resposta = json!({
        "versao": anacad_core::VERSAO,
        "cifrado": false,
        "registros": 0,
        "por_tipo": {},
    });

    if let Some(caminho) = str_param(p, "file") {
        let bytes = ler_arquivo(caminho)?;
        let (registros, cifrado) = anacad_core::ler_tela(&bytes);
        resposta = json!({
            "versao": anacad_core::VERSAO,
            "cifrado": cifrado,
            "registros": registros.len(),
            "por_tipo": contar_por_tipo(&registros),
        });
    }

    if let Some(caminho) = str_param(p, "import") {
        let bytes = ler_arquivo(caminho)?;
        let tela = anacad_core::ler_tela_tipada(&bytes).map_err(|e| bad("anacad", format!("{caminho}: {e}")))?;
        let nome = str_param(p, "nome")
            .map(str::to_string)
            .unwrap_or_else(|| std::path::Path::new(caminho).file_stem().and_then(|s| s.to_str()).unwrap_or("importado").to_string());
        resposta = json!({
            "versao": anacad_core::VERSAO,
            "tela": {
                "componentes": tela.componentes.len(),
                "linhas": tela.linhas.len(),
                "ligacoes": tela.ligacoes.len(),
                "legendas": tela.legendas_componente.len() + tela.legendas_ligacao.len(),
                "extensao": tela.extensao,
            },
        });
        if let Some(db) = str_param(p, "db") {
            let mut banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad", format!("{db}: {e}")))?;
            let resumo = banco.importar_screen(&tela, &nome, Some(caminho)).map_err(|e| bad("anacad", format!("{db}: {e}")))?;
            let desenho = desenhar_tela(s, &tela, &banco)?;
            resposta["banco"] = json!({
                "projeto": resumo.projeto,
                "documento": resumo.documento,
                "barras": resumo.barras,
                "componentes": resumo.componentes,
                "linhas": resumo.linhas,
                "ligacoes": resumo.ligacoes,
                "legendas": resumo.legendas,
            });
            resposta["desenho"] = desenho;
        }
    }

    if bool_or(p, "test_block", false) {
        let handle = bloco_de_teste(s)?;
        resposta["bloco"] = json!(handle);
    }

    Ok(resposta)
}

fn run_consultar(_s: &mut Session, p: &Value) -> Result<Value> {
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
        resposta["documento"] = json!(documento);
        resposta["componentes"] = componentes;
        resposta["linhas"] = linhas;
        resposta["ligacoes"] = ligacoes;
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

fn run_mover(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.mover", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.mover", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.mover", "handle invalido"))?;
    let dx = f64_or(p, "dx", 0.0);
    let dy = f64_or(p, "dy", 0.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.mover", format!("{db}: {e}")))?;
    let (x, y) = banco
        .posicao_por_handle(texto)
        .map_err(|e| bad("anacad.mover", e.to_string()))?
        .ok_or_else(|| bad("anacad.mover", "handle nao encontrado no banco"))?;
    let (x, y) = (x + dx, y + dy);
    banco.atualizar_posicao_por_handle(texto, x, y).map_err(|e| bad("anacad.mover", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::Insert(ins) = &mut e.kind {
            ins.insert.x = x;
            ins.insert.y = y;
        }
    })?;
    Ok(json!({ "handle": handle.hex(), "x": x, "y": y }))
}

fn run_apagar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.apagar", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.apagar", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.apagar", "handle invalido"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.apagar", format!("{db}: {e}")))?;
    let removidos = banco.apagar_por_handle(texto).map_err(|e| bad("anacad.apagar", e.to_string()))?;
    s.doc_mut()?.remove_entity(handle);
    Ok(json!({ "handle": texto, "removidos": removidos }))
}

fn run_comprimento(_s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_referencia(_s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_relatorio(_s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_verificar(_s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_ana_caso(s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_ana_importar(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.ana.importar", "`arquivo` is required"))?;
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ana.importar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let bytes = ler_arquivo(arquivo)?;
    let ana = anacad_core::ana::ArquivoAna::ler(&bytes).map_err(|e| bad("anacad.ana.importar", format!("{arquivo}: {e}")))?;
    let barras = ana.ler_dbar();
    let circuitos = ana.ler_dcir();
    let shunts = ana.ler_dshl();
    let geradores = ana.ler_deol();
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ana.importar", format!("{db}: {e}")))?;
    let mut atualizadas = 0usize;
    let mut sem_barra = 0usize;
    let mut sem_vbase = 0usize;
    for barra in &barras {
        let Some(vbase) = barra.vbase else {
            sem_vbase += 1;
            continue;
        };
        let linhas = banco.atualizar_barra_vbase(Some(documento), barra.codigo, vbase).map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        if linhas == 0 {
            sem_barra += 1;
        } else {
            atualizadas += linhas;
        }
    }
    let _ = banco.apagar_circuitos(Some(documento));
    banco.apagar_ana_extras(Some(documento)).map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
    let mut circuitos_gravados = 0usize;
    for circuito in &circuitos {
        banco
            .inserir_circuito(
                Some(documento),
                circuito.de,
                circuito.para,
                &circuito.tipo,
                circuito.r1,
                circuito.x1,
                circuito.r0,
                circuito.x0,
                circuito.tap,
                circuito.ia,
            )
            .map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        circuitos_gravados += 1;
    }
    let mut shunts_gravados = 0usize;
    for shunt in &shunts {
        banco
            .inserir_shunt(Some(documento), shunt.de, shunt.para, shunt.qpos, shunt.rn, shunt.xn)
            .map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        shunts_gravados += 1;
    }
    let mut geradores_gravados = 0usize;
    for gerador in &geradores {
        banco
            .inserir_gerador(Some(documento), gerador.nb, gerador.unidades, gerador.p_inic, gerador.i_max, gerador.v_min, gerador.v_max)
            .map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        geradores_gravados += 1;
    }
    Ok(json!({
        "arquivo": arquivo,
        "barras": barras.len(),
        "atualizadas": atualizadas,
        "sem_barra": sem_barra,
        "sem_vbase": sem_vbase,
        "circuitos": circuitos_gravados,
        "shunts": shunts_gravados,
        "geradores": geradores_gravados,
    }))
}

fn run_ana(_s: &mut Session, p: &Value) -> Result<Value> {
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

fn shunts_zero_do_banco(banco: &anacad_data::Banco, documento: i64, por_codigo: &BTreeMap<i64, i64>) -> Result<Vec<anacad_core::calc::ShuntCalc>> {
    let shunts = banco.shunts(documento, 100_000).map_err(|e| bad("anacad.shunt", e.to_string()))?;
    let mut saida = Vec::new();
    for shunt in &shunts {
        let Some(qpos) = shunt.qpos else { continue };
        if qpos == 0.0 {
            continue;
        }
        // Reator: Xpos = 100/|Qpos| pu; aterramento Xn em % (ex.: 26,5%).
        let xpos = 100.0 / qpos.abs();
        let xn = shunt.xn.unwrap_or(0.0) / 100.0;
        let z0 = xpos + 3.0 * xn;
        if z0 <= 0.0 {
            continue;
        }
        let y = anacad_core::calc::Complexo::novo(0.0, -1.0 / z0);
        if let Some(id) = shunt.de.and_then(|c| por_codigo.get(&c).copied()) {
            saida.push(anacad_core::calc::ShuntCalc { barra: id, y });
        }
    }
    Ok(saida)
}

fn run_curto(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.curto", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("barra").and_then(Value::as_i64).ok_or_else(|| bad("anacad.curto", "`barra` (codigo) is required"))?;
    let sbase = f64_or(p, "sbase", 100.0);
    let tipo_texto = str_param(p, "tipo").unwrap_or("trifasica");
    let tipo_falta =
        anacad_core::calc::TipoFalta::parse(tipo_texto).ok_or_else(|| bad("anacad.curto", "tipo deve ser trifasica, bifasica ou monofasica"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.curto", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.curto", e.to_string()))?;
    let circuitos = banco.circuitos(documento, 100_000).map_err(|e| bad("anacad.curto", e.to_string()))?;
    let por_codigo: BTreeMap<i64, i64> = barras.iter().filter_map(|b| b.codigo.map(|c| (c, b.id))).collect();
    let id_falta = *por_codigo.get(&codigo).ok_or_else(|| bad("anacad.curto", format!("barra {codigo} nao esta no documento")))?;
    let (referencia, referencia_codigo) = match p.get("referencia").and_then(Value::as_i64) {
        Some(codigo) => {
            let id = *por_codigo.get(&codigo).ok_or_else(|| bad("anacad.curto", format!("referencia {codigo} nao esta no documento")))?;
            (id, codigo)
        }
        None => {
            let barra = barras.iter().find(|b| b.id != id_falta).ok_or_else(|| bad("anacad.curto", "precisa de pelo menos duas barras"))?;
            (barra.id, barra.codigo.unwrap_or(0))
        }
    };
    let mut adjacencia: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let mut circuitos_mapped: Vec<anacad_core::calc::CircuitoCalc> = Vec::new();
    for c in &circuitos {
        let (Some(de), Some(para), Some(r1), Some(x1)) = (c.de, c.para, c.r1, c.x1) else { continue };
        let (Some(de), Some(para)) = (por_codigo.get(&de).copied(), por_codigo.get(&para).copied()) else {
            continue;
        };
        adjacencia.entry(de).or_default().push(para);
        adjacencia.entry(para).or_default().push(de);
        circuitos_mapped.push(anacad_core::calc::CircuitoCalc {
            de,
            para,
            r1,
            x1,
            tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
        });
    }
    let mut componente: BTreeSet<i64> = BTreeSet::new();
    let mut pilha = vec![id_falta];
    while let Some(no) = pilha.pop() {
        if !componente.insert(no) {
            continue;
        }
        if let Some(vizinhos) = adjacencia.get(&no) {
            pilha.extend(vizinhos.iter().copied());
        }
    }
    if !componente.contains(&referencia) {
        return Err(bad("anacad.curto", "a referencia esta em outra ilha da barra da falta"));
    }
    let entrada = anacad_core::calc::EntradaYbus {
        barras: barras.iter().map(|b| b.id).filter(|id| componente.contains(id)).collect(),
        circuitos: circuitos_mapped.into_iter().filter(|c| componente.contains(&c.de) && componente.contains(&c.para)).collect(),
    };
    let shunts = shunts_do_banco(&banco, documento, &por_codigo)?;
    let ybus = anacad_core::calc::montar_ybus_com_shunts(&entrada, &shunts);
    let y0 = if tipo_falta == anacad_core::calc::TipoFalta::Monofasica {
        let circuitos0: Vec<anacad_core::calc::CircuitoCalc> = circuitos
            .iter()
            .filter_map(|c| {
                let (Some(de), Some(para)) = (c.de, c.para) else { return None };
                let (de, para) = (*por_codigo.get(&de)?, *por_codigo.get(&para)?);
                if !componente.contains(&de) || !componente.contains(&para) {
                    return None;
                }
                match (c.r0, c.x0) {
                    (Some(r0), Some(x0)) => Some(anacad_core::calc::CircuitoCalc {
                        de,
                        para,
                        r1: r0,
                        x1: x0,
                        tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
                    }),
                    _ => match (c.r1, c.x1) {
                        (Some(r1), Some(x1)) => Some(anacad_core::calc::CircuitoCalc {
                            de,
                            para,
                            r1,
                            x1,
                            tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
                        }),
                        _ => None,
                    },
                }
            })
            .collect();
        let entrada0 = anacad_core::calc::EntradaYbus { barras: entrada.barras.clone(), circuitos: circuitos0 };
        let shunts0 = shunts_zero_do_banco(&banco, documento, &por_codigo)?;
        Some(anacad_core::calc::montar_ybus_com_shunts(&entrada0, &shunts0))
    } else {
        None
    };
    let pos = |id: i64| ybus.indices.iter().position(|b| *b == id);
    let (Some(falta), Some(ref_idx)) = (pos(id_falta), pos(referencia)) else {
        return Err(bad("anacad.curto", "barra nao entrou na Ybus"));
    };
    let geradores = banco.geradores(documento, 100_000).map_err(|e| bad("anacad.curto", e.to_string()))?;
    let mut fontes = Vec::new();
    for gerador in &geradores {
        let (Some(nb), Some(imax)) = (gerador.nb, gerador.i_max) else { continue };
        let unidades = gerador.unidades.unwrap_or(1.0);
        if imax <= 0.0 || unidades <= 0.0 {
            continue;
        }
        let Some(id) = por_codigo.get(&nb).copied() else {
            continue;
        };
        let Some(pos_gerador) = pos(id) else {
            continue;
        };
        let vbase = banco.barra_vbase(documento, nb).map_err(|e| bad("anacad.curto", e.to_string()))?.unwrap_or(0.0);
        if vbase <= 0.0 {
            continue;
        }
        let i_base_a = sbase * 1e6 / (3.0f64.sqrt() * vbase * 1e3);
        if i_base_a <= 0.0 {
            continue;
        }
        fontes.push(anacad_core::calc::FonteCalc { barra: pos_gerador, i_pu: imax * unidades / i_base_a });
    }
    let resultado = anacad_core::calc::curto_tipo_com_fontes(&ybus, y0.as_ref(), falta, ref_idx, tipo_falta, 1e9, &fontes)
        .ok_or_else(|| bad("anacad.curto", "matriz singular ou Zth nulo"))?;
    let vbase = banco.barra_vbase(documento, codigo).map_err(|e| bad("anacad.curto", e.to_string()))?.unwrap_or(0.0);
    let i_base_a = if vbase > 0.0 { sbase * 1e6 / (3.0f64.sqrt() * vbase * 1e3) } else { 0.0 };
    let i_ka = resultado.i_pu * i_base_a / 1e3;
    let id = banco
        .inserir_resultado(
            Some(documento),
            "curto_circuito",
            &json!({ "barra": codigo, "referencia": referencia_codigo, "sbase": sbase, "tipo": tipo_texto }).to_string(),
            &json!({
                "barra": codigo,
                "tipo": tipo_texto,
                "zth_re": resultado.zth.re,
                "zth_im": resultado.zth.im,
                "i_pu": resultado.i_pu,
                "i_fonte_pu": resultado.i_fonte_pu,
                "fontes": fontes.len(),
                "vbase": vbase,
                "i_ka": i_ka,
            })
            .to_string(),
        )
        .map_err(|e| bad("anacad.curto", e.to_string()))?;
    Ok(json!({
        "resultado": id,
        "barra": codigo,
        "referencia": referencia_codigo,
        "tipo": tipo_texto,
        "zth_re": resultado.zth.re,
        "zth_im": resultado.zth.im,
        "i_pu": resultado.i_pu,
        "i_fonte_pu": resultado.i_fonte_pu,
        "fontes": fontes.len(),
        "vbase": vbase,
        "i_ka": i_ka,
    }))
}

fn run_resultados(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.resultados", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let limite = p.get("limite").and_then(Value::as_i64).unwrap_or(10);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.resultados", format!("{db}: {e}")))?;
    let resultados = serde_json::to_value(banco.resultados(documento, limite).map_err(|e| bad("anacad.resultados", e.to_string()))?)
        .map_err(|e| bad("anacad.resultados", e.to_string()))?;
    Ok(json!({ "db": db, "documento": documento, "resultados": resultados }))
}

fn shunts_do_banco(banco: &anacad_data::Banco, documento: i64, por_codigo: &BTreeMap<i64, i64>) -> Result<Vec<anacad_core::calc::ShuntCalc>> {
    let shunts = banco.shunts(documento, 100_000).map_err(|e| bad("anacad.shunt", e.to_string()))?;
    let mut saida = Vec::new();
    for shunt in &shunts {
        let Some(qpos) = shunt.qpos else { continue };
        let b = anacad_core::calc::susceptancia_de_qpos(qpos);
        let y = anacad_core::calc::Complexo::novo(0.0, b);
        // O DSHL é o shunt do terminal `De` (BF) da linha — ex.: reator
        // aterrado no terminal 20008 da LT 20008-20007.
        if let Some(id) = shunt.de.and_then(|c| por_codigo.get(&c).copied()) {
            saida.push(anacad_core::calc::ShuntCalc { barra: id, y });
        }
    }
    Ok(saida)
}

fn run_ybus(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ybus", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ybus", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.ybus", e.to_string()))?;
    let circuitos = banco.circuitos(documento, 100_000).map_err(|e| bad("anacad.ybus", e.to_string()))?;
    let por_codigo: BTreeMap<i64, i64> = barras.iter().filter_map(|b| b.codigo.map(|codigo| (codigo, b.id))).collect();
    let entrada = anacad_core::calc::EntradaYbus {
        barras: barras.iter().map(|b| b.id).collect(),
        circuitos: circuitos
            .iter()
            .filter_map(|c| match (c.de, c.para, c.r1, c.x1) {
                (Some(de), Some(para), Some(r1), Some(x1)) => {
                    let de = *por_codigo.get(&de)?;
                    let para = *por_codigo.get(&para)?;
                    Some(anacad_core::calc::CircuitoCalc { de, para, r1, x1, tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0) })
                }
                _ => None,
            })
            .collect(),
    };
    let shunts = shunts_do_banco(&banco, documento, &por_codigo)?;
    let ybus = anacad_core::calc::montar_ybus_com_shunts(&entrada, &shunts);
    let resumo = ybus.resumo();
    let id = banco
        .inserir_resultado(
            Some(documento),
            "ybus",
            &json!({ "circuitos": circuitos.len(), "base_mva": 100 }).to_string(),
            &json!({
                "barras": resumo.barras,
                "circuitos": resumo.circuitos,
                "circuitos_usados": resumo.circuitos_usados,
                "circuitos_sem_barra": resumo.circuitos_sem_barra,
                "pares_nao_nulos": resumo.pares_nao_nulos,
                "simetrica": resumo.simetrica,
                "maior_diagonal": resumo.maior_diagonal,
            })
            .to_string(),
        )
        .map_err(|e| bad("anacad.ybus", e.to_string()))?;
    Ok(json!({
        "resultado": id,
        "barras": resumo.barras,
        "circuitos_usados": resumo.circuitos_usados,
        "circuitos_sem_barra": resumo.circuitos_sem_barra,
        "pares_nao_nulos": resumo.pares_nao_nulos,
        "simetrica": resumo.simetrica,
        "maior_diagonal": resumo.maior_diagonal,
    }))
}

fn run_calcular(_s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_validar(s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_criar_legenda(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tipo = str_param(p, "tipo").unwrap_or("X").to_ascii_uppercase();
    if tipo != "X" && tipo != "Y" {
        return Err(bad("anacad.legenda.criar", "`tipo` deve ser X ou Y"));
    }
    let numero = p.get("numero").and_then(Value::as_i64).unwrap_or(0) as i32;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let canal = str_param(p, "canal");
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.criar", format!("{db}: {e}")))?;
    let id = banco.criar_legenda(documento, &tipo, numero, canal, numero, 0, x, y).map_err(|e| bad("anacad.legenda.criar", e.to_string()))?;
    let altura = s.doc()?.header.f64("TEXTSIZE", 20.0);
    let estilo = s.doc()?.header.str("TEXTSTYLE", "Standard");
    let kind = EntityKind::Text(cadcraft_doc::Text {
        insert: Vec3::new(x, y, 0.0),
        align_pt: None,
        height: altura,
        value: numero.to_string(),
        rotation: 0.0,
        width_factor: 1.0,
        oblique: 0.0,
        style: estilo,
        halign: cadcraft_doc::HAlign::Left,
        valign: cadcraft_doc::VAlign::Baseline,
    });
    let handle = s.add_entity(kind)?;
    marcar(s, handle, "ANACAD_LEGENDA", &[("id", &id.to_string()), ("tipo", &tipo), ("numero", &numero.to_string())])?;
    banco.definir_handle("Legenda", id, &handle.hex()).map_err(|e| bad("anacad.legenda.criar", e.to_string()))?;
    Ok(json!({ "legenda": id, "handle": handle.hex() }))
}

fn run_mover_legenda(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.mover", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.legenda.mover", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.legenda.mover", "handle invalido"))?;
    let dx = f64_or(p, "dx", 0.0);
    let dy = f64_or(p, "dy", 0.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.mover", format!("{db}: {e}")))?;
    let legenda = banco
        .legenda_por_handle(texto)
        .map_err(|e| bad("anacad.legenda.mover", e.to_string()))?
        .ok_or_else(|| bad("anacad.legenda.mover", "handle nao esta no banco"))?;
    let (x, y) = (legenda.x.unwrap_or(0.0) + dx, legenda.y.unwrap_or(0.0) + dy);
    banco.atualizar_legenda_posicao(texto, x, y).map_err(|e| bad("anacad.legenda.mover", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::Text(t) = &mut e.kind {
            t.insert.x = x;
            t.insert.y = y;
        }
    })?;
    Ok(json!({ "handle": handle.hex(), "x": x, "y": y }))
}

fn run_criar_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let nome = str_param(p, "nome");
    let comprimento = f64_or(p, "comprimento", 1.0).max(0.01);
    let x2 = p.get("x2").and_then(Value::as_f64).unwrap_or(x + comprimento);
    let y2 = p.get("y2").and_then(Value::as_f64).unwrap_or(y);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.criar", format!("{db}: {e}")))?;
    let codigo = match p.get("codigo").and_then(Value::as_i64) {
        Some(c) if c > 0 => c,
        _ => banco.proximo_codigo_barra(documento).map_err(|e| bad("anacad.barra.criar", e.to_string()))?,
    };
    let centro = ((x + x2) / 2.0, (y + y2) / 2.0);
    let id = banco.criar_barra(documento, codigo, nome, centro.0, centro.1).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;

    // Barra = LWPOLYLINE de 2 vértices (início/fim), largura 0,3.
    let poly = LwPolyline {
        vertices: vec![PolyVertex { p: Vec2::new(x, y), ..PolyVertex::default() }, PolyVertex { p: Vec2::new(x2, y2), ..PolyVertex::default() }],
        closed: false,
        const_width: 0.3,
        elevation: 0.0,
        plinegen: false,
    };
    let handle = s.add_entity(EntityKind::LwPolyline(poly))?;
    marcar(
        s,
        handle,
        "ANACAD_BARRA",
        &[
            ("id", &id.to_string()),
            ("codigo", &codigo.to_string()),
            ("x1", &x.to_string()),
            ("y1", &y.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0.3"),
        ],
    )?;
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    Ok(json!({
        "barra": id,
        "codigo": codigo,
        "handle": handle.hex(),
        "x1": x, "y1": y, "x2": x2, "y2": y2,
        "largura": 0.3,
        "tipo": "LWPOLYLINE",
    }))
}

fn run_criar_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let de = p.get("de").and_then(Value::as_i64);
    let para = p.get("para").and_then(Value::as_i64);
    let mut x1 = f64_or(p, "x1", 0.0);
    let mut y1 = f64_or(p, "y1", 0.0);
    let mut x2 = f64_or(p, "x2", 0.0);
    let mut y2 = f64_or(p, "y2", 0.0);
    let mut barra_de = None;
    let mut barra_para = None;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.criar", format!("{db}: {e}")))?;
    if let (Some(de), Some(para)) = (de, para) {
        barra_de = Some(
            banco
                .barra_por_codigo(documento, de)
                .map_err(|e| bad("anacad.linha.criar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.criar", format!("barra {de} nao existe no documento")))?,
        );
        barra_para = Some(
            banco
                .barra_por_codigo(documento, para)
                .map_err(|e| bad("anacad.linha.criar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.criar", format!("barra {para} nao existe no documento")))?,
        );
        if let (Some(b1), Some(b2)) = (barra_de.as_ref(), barra_para.as_ref()) {
            x1 = b1.x.unwrap_or(x1);
            y1 = b1.y.unwrap_or(y1);
            x2 = b2.x.unwrap_or(x2);
            y2 = b2.y.unwrap_or(y2);
        }
    }
    let id = banco
        .criar_linha(documento, barra_de.as_ref().map(|b| b.id), barra_para.as_ref().map(|b| b.id))
        .map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    let poly = LwPolyline {
        vertices: vec![PolyVertex { p: Vec2::new(x1, y1), ..PolyVertex::default() }, PolyVertex { p: Vec2::new(x2, y2), ..PolyVertex::default() }],
        closed: false,
        const_width: 0.0,
        elevation: 0.0,
        plinegen: false,
    };
    let handle = s.add_entity(EntityKind::LwPolyline(poly))?;
    let mut campos = vec![
        ("id", id.to_string()),
        ("x1", x1.to_string()),
        ("y1", y1.to_string()),
        ("x2", x2.to_string()),
        ("y2", y2.to_string()),
        ("largura", "0".to_string()),
    ];
    if let Some(de) = de {
        campos.push(("de", de.to_string()));
    }
    if let Some(para) = para {
        campos.push(("para", para.to_string()));
    }
    let refs: Vec<(&str, &str)> = campos.iter().map(|(k, v)| (*k, v.as_str())).collect();
    marcar(s, handle, "ANACAD_LINHA", &refs)?;
    banco.definir_handle("Linha", id, &handle.hex()).map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    Ok(json!({
        "linha": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": 0.0,
        "tipo": "LWPOLYLINE",
    }))
}

fn ler_polilinha(s: &Session, texto: &str) -> Result<(Handle, LwPolyline)> {
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.polilinha", "handle invalido"))?;
    let poly = {
        let doc = s.doc()?;
        let entidade = doc.entity(handle).ok_or_else(|| bad("anacad.polilinha", "entidade nao encontrada"))?;
        match &entidade.kind {
            EntityKind::LwPolyline(p) => p.clone(),
            _ => return Err(bad("anacad.polilinha", "entidade nao e LWPOLYLINE")),
        }
    };
    if poly.vertices.len() < 2 {
        return Err(bad("anacad.polilinha", "polilinha precisa de 2 vertices"));
    }
    Ok((handle, poly))
}

fn run_editar_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.editar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.editar", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.editar", format!("{db}: {e}")))?;
    let codigo = p
        .get("codigo")
        .and_then(Value::as_i64)
        .or_else(|| dados.get("codigo").and_then(Value::as_i64))
        .ok_or_else(|| bad("anacad.barra.editar", "`codigo` is required"))?;
    let nome = dados.get("nome").and_then(Value::as_str);
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    let id = match banco.barra_por_codigo(documento, codigo).map_err(|e| bad("anacad.barra.editar", e.to_string()))? {
        Some(b) => {
            banco.atualizar_barra_nome_pos(b.id, nome, (x1 + x2) / 2.0, (y1 + y2) / 2.0).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
            b.id
        }
        None => {
            banco.criar_barra(documento, codigo, nome, (x1 + x2) / 2.0, (y1 + y2) / 2.0).map_err(|e| bad("anacad.barra.editar", e.to_string()))?
        }
    };
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.atualizar_dados_barra(&handle.hex(), &texto).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.3;
        }
    })?;
    marcar(
        s,
        handle,
        "ANACAD_BARRA",
        &[
            ("id", &id.to_string()),
            ("codigo", &codigo.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0.3"),
            ("dados", &texto),
        ],
    )?;
    Ok(json!({ "barra": id, "handle": handle.hex(), "dados": dados }))
}

fn run_editar_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.editar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.linha.editar", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let de = p.get("de").and_then(Value::as_i64).or_else(|| dados.get("de").and_then(Value::as_i64));
    let para = p.get("para").and_then(Value::as_i64).or_else(|| dados.get("para").and_then(Value::as_i64));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.editar", format!("{db}: {e}")))?;
    let barra_de = match de {
        Some(c) => Some(
            banco
                .barra_por_codigo(documento, c)
                .map_err(|e| bad("anacad.linha.editar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.editar", format!("barra {c} nao existe")))?,
        ),
        None => None,
    };
    let barra_para = match para {
        Some(c) => Some(
            banco
                .barra_por_codigo(documento, c)
                .map_err(|e| bad("anacad.linha.editar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.editar", format!("barra {c} nao existe")))?,
        ),
        None => None,
    };
    let id = banco
        .linha_id_por_handle(&handle.hex())
        .map_err(|e| bad("anacad.linha.editar", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.editar", "linha nao encontrada pelo handle"))?;
    banco
        .atualizar_barras_linha(&handle.hex(), barra_de.as_ref().map(|b| b.id), barra_para.as_ref().map(|b| b.id))
        .map_err(|e| bad("anacad.linha.editar", e.to_string()))?;
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.atualizar_dados_linha(&handle.hex(), &texto).map_err(|e| bad("anacad.linha.editar", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.0;
        }
    })?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    marcar(
        s,
        handle,
        "ANACAD_LINHA",
        &[
            ("id", &id.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0"),
            ("dados", &texto),
        ],
    )?;
    Ok(json!({ "linha": id, "handle": handle.hex(), "dados": dados }))
}

fn run_atribuir_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.atribuir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.atribuir", "`handle` is required"))?;
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.atribuir", "`codigo` is required"))?;
    let nome = str_param(p, "nome");
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.3;
        }
    })?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.atribuir", format!("{db}: {e}")))?;
    let id = banco.criar_barra(documento, codigo, nome, (x1 + x2) / 2.0, (y1 + y2) / 2.0).map_err(|e| bad("anacad.barra.atribuir", e.to_string()))?;
    marcar(
        s,
        handle,
        "ANACAD_BARRA",
        &[
            ("id", &id.to_string()),
            ("codigo", &codigo.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", &poly.const_width.to_string()),
        ],
    )?;
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.atribuir", e.to_string()))?;
    Ok(json!({
        "barra": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": poly.const_width,
    }))
}

fn run_atribuir_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.atribuir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.linha.atribuir", "`handle` is required"))?;
    let de = p.get("de").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.atribuir", "`de` is required"))?;
    let para = p.get("para").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.atribuir", "`para` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.0;
        }
    })?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.atribuir", format!("{db}: {e}")))?;
    let barra_de = banco
        .barra_por_codigo(documento, de)
        .map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.atribuir", format!("barra {de} nao existe")))?;
    let barra_para = banco
        .barra_por_codigo(documento, para)
        .map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.atribuir", format!("barra {para} nao existe")))?;
    let id = banco.criar_linha(documento, Some(barra_de.id), Some(barra_para.id)).map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?;
    marcar(
        s,
        handle,
        "ANACAD_LINHA",
        &[
            ("id", &id.to_string()),
            ("de", &de.to_string()),
            ("para", &para.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", &poly.const_width.to_string()),
        ],
    )?;
    banco.definir_handle("Linha", id, &handle.hex()).map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?;
    Ok(json!({
        "linha": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": poly.const_width,
    }))
}

#[derive(Default)]
struct BarraM {
    pts: Vec<Vec2>,
}

impl Interactive for BarraM {
    fn name(&self) -> &'static str {
        "ANACAD_BARRA"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if self.pts.is_empty() {
            Prompt::new("Primeiro ponto da barra", Accept::POINT)
        } else {
            Prompt::new("Segundo ponto da barra", Accept::POINT).base(self.pts[0])
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if self.pts.is_empty() {
            self.pts.push(p);
            s.last_point = p;
            return Ok(Step::Continue);
        }
        let a = self.pts[0];
        if a.near(p, 1e-12) {
            return Err(crate::EngineError::Other("os dois pontos da barra coincidem".into()));
        }
        let db = s.anacad_db.clone();
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de criar a barra".into()));
        }
        let params = json!({
            "db": db, "documento": s.anacad_documento,
            "x": a.x, "y": a.y, "x2": p.x, "y2": p.y,
        });
        run_criar_barra(s, &params)?;
        Ok(Step::Done)
    }
    fn preview(&self, _s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        if let Some(a) = self.pts.first() {
            let mut pl = LwPolyline {
                vertices: vec![PolyVertex { p: *a, ..PolyVertex::default() }, PolyVertex { p: cursor, ..PolyVertex::default() }],
                closed: false,
                const_width: 0.3,
                elevation: 0.0,
                plinegen: false,
            };
            pl.const_width = 0.3;
            return vec![EntityKind::LwPolyline(pl)];
        }
        Vec::new()
    }
}

#[derive(Default)]
struct LinhaM {
    pts: Vec<Vec2>,
}

impl Interactive for LinhaM {
    fn name(&self) -> &'static str {
        "ANACAD_LINHA"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if self.pts.is_empty() {
            Prompt::new("Primeiro ponto da linha", Accept::POINT)
        } else {
            Prompt::new("Segundo ponto da linha", Accept::POINT).base(self.pts[0])
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if self.pts.is_empty() {
            self.pts.push(p);
            s.last_point = p;
            return Ok(Step::Continue);
        }
        let a = self.pts[0];
        if a.near(p, 1e-12) {
            return Err(crate::EngineError::Other("os dois pontos da linha coincidem".into()));
        }
        let db = s.anacad_db.clone();
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de criar a linha".into()));
        }
        let params = json!({
            "db": db, "documento": s.anacad_documento,
            "x1": a.x, "y1": a.y, "x2": p.x, "y2": p.y,
        });
        run_criar_linha(s, &params)?;
        Ok(Step::Done)
    }
    fn preview(&self, _s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        if let Some(a) = self.pts.first() {
            let pl = LwPolyline {
                vertices: vec![PolyVertex { p: *a, ..PolyVertex::default() }, PolyVertex { p: cursor, ..PolyVertex::default() }],
                closed: false,
                const_width: 0.0,
                elevation: 0.0,
                plinegen: false,
            };
            return vec![EntityKind::LwPolyline(pl)];
        }
        Vec::new()
    }
}

fn resumo_banco(banco: &anacad_data::Banco, arquivo: &str) -> Result<Value> {
    let documentos = banco.documentos().map_err(|e| bad("anacad.banco", e.to_string()))?;
    let nome = std::path::Path::new(arquivo).file_name().and_then(|n| n.to_str()).unwrap_or(arquivo).to_string();
    Ok(json!({
        "arquivo": arquivo,
        "nome": nome,
        "versao": banco.versao().map_err(|e| bad("anacad.banco", e.to_string()))?,
        "documentos": documentos.len(),
        "primeiroDocumento": documentos.first().map(|d| d.id),
    }))
}

fn run_banco_criar(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.criar", "`arquivo` is required"))?;
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.criar", format!("{arquivo}: {e}")))?;
    resumo_banco(&banco, arquivo)
}

fn run_banco_abrir(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.abrir", "`arquivo` is required"))?;
    if !std::path::Path::new(arquivo).exists() {
        return Err(bad("anacad.banco.abrir", format!("banco nao encontrado: {arquivo}")));
    }
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.abrir", format!("{arquivo}: {e}")))?;
    resumo_banco(&banco, arquivo)
}

fn run_documento_listar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.documento.listar", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.documento.listar", format!("{db}: {e}")))?;
    let documentos: Vec<Value> = banco
        .documentos()
        .map_err(|e| bad("anacad.documento.listar", e.to_string()))?
        .into_iter()
        .map(|d| json!({"id": d.id, "nome": d.nome, "pagina": d.pagina}))
        .collect();
    Ok(json!({ "documentos": documentos }))
}

fn run_documento_criar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.documento.criar", "`db` is required"))?;
    let nome = str_param(p, "nome").ok_or_else(|| bad("anacad.documento.criar", "`nome` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.documento.criar", format!("{db}: {e}")))?;
    let projeto = match p.get("projeto").and_then(Value::as_i64) {
        Some(id) => id,
        None => match banco.projetos().map_err(|e| bad("anacad.documento.criar", e.to_string()))?.first() {
            Some((id, _)) => *id,
            None => banco.criar_projeto("AnaCAD").map_err(|e| bad("anacad.documento.criar", e.to_string()))?,
        },
    };
    let pagina = str_param(p, "pagina");
    let documento = banco.inserir_documento(projeto, nome, None, pagina, 0).map_err(|e| bad("anacad.documento.criar", e.to_string()))?;
    let documentos = banco.documentos().map_err(|e| bad("anacad.documento.criar", e.to_string()))?.len();
    Ok(json!({ "documento": documento, "projeto": projeto, "nome": nome, "documentos": documentos }))
}

fn run_exportar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.exportar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let saida = str_param(p, "saida").ok_or_else(|| bad("anacad.exportar", "`saida` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.exportar", format!("{db}: {e}")))?;
    let tela = banco.exportar_screen(documento).map_err(|e| bad("anacad.exportar", e.to_string()))?;
    let texto = tela.escrever_texto();
    let registros = anacad_core::ler_registros(texto.as_bytes()).len();
    escrever_arquivo(saida, &anacad_core::cifrar(texto.as_bytes()))?;
    Ok(json!({ "saida": saida, "registros": registros, "mensagem": "tela exportada" }))
}

#[cfg(not(target_arch = "wasm32"))]
fn escrever_arquivo(caminho: &str, bytes: &[u8]) -> Result<()> {
    std::fs::write(caminho, bytes).map_err(|e| bad("anacad.exportar", format!("{caminho}: {e}")))
}

#[cfg(target_arch = "wasm32")]
fn escrever_arquivo(_caminho: &str, _bytes: &[u8]) -> Result<()> {
    Err(bad("anacad.exportar", "exportar nao esta disponivel no build web"))
}

fn contar_por_tipo(registros: &[anacad_core::Registro]) -> BTreeMap<String, u64> {
    let mut por_tipo: BTreeMap<String, u64> = BTreeMap::new();
    for registro in registros {
        *por_tipo.entry((registro.tipo as char).to_string()).or_insert(0) += 1;
    }
    por_tipo
}

#[cfg(not(target_arch = "wasm32"))]
fn ler_arquivo(caminho: &str) -> Result<Vec<u8>> {
    std::fs::read(caminho).map_err(|e| bad("anacad", format!("{caminho}: {e}")))
}

#[cfg(target_arch = "wasm32")]
fn ler_arquivo(_caminho: &str) -> Result<Vec<u8>> {
    Err(bad("anacad", "ler um .lst nao esta disponivel no build web"))
}

/// Cria (uma vez) o bloco `ANACAD_TESTE` com uma linha, insere e marca o INSERT
/// com XData do app `ANACAD`; devolve o handle do INSERT (hex).
fn bloco_de_teste(s: &mut Session) -> Result<String> {
    let existe = s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(BLOCO_TESTE));
    if !existe {
        let doc = s.doc_mut()?;
        let mut bloco = Block::new(BLOCO_TESTE);
        bloco.description = "Bloco de teste do anacad (M1)".into();
        let h = doc.new_handle();
        let linha = Entity::new(h, EntityKind::Line(Line { a: Vec3::new(0.0, 0.0, 0.0), b: Vec3::new(10.0, 0.0, 0.0) }));
        bloco.entities.push(linha);
        doc.blocks.insert(BLOCO_TESTE.to_string(), Arc::new(bloco));
    }

    let handle = super::blocks::insert(s, BLOCO_TESTE, Vec2::new(0.0, 0.0), 1.0, 0.0, &serde_json::Map::new())?;
    marcar(s, handle, "ANACAD", &[("versao", "1"), ("bloco", BLOCO_TESTE)])?;
    Ok(handle.hex())
}

/// Desenha os componentes como INSERTs de símbolos e as ligações como linhas;
/// registra os handles nas tabelas do banco.
fn desenhar_tela(s: &mut Session, tela: &anacad_core::Screen, banco: &anacad_data::Banco) -> Result<Value> {
    let mut simbolos = 0usize;
    let mut componentes = 0usize;
    for c in &tela.componentes {
        let bloco = garantir_bloco_simbolo(s, c.tipo)?;
        let handle = super::blocks::insert(s, &bloco, Vec2::new(c.px, c.py), 1.0, 0.0, &serde_json::Map::new())?;
        marcar(s, handle, "ANACAD_COMP", &[("id", &c.numero.to_string()), ("tipo", &c.tipo.to_string()), ("barra", &c.refs[0].to_string())])?;
        let _ = banco.definir_handle("Componente", c.numero as i64, &handle.hex());
        componentes += 1;
    }
    simbolos += tela.componentes.iter().map(|c| c.tipo).collect::<std::collections::BTreeSet<_>>().len();

    let mut trechos = 0usize;
    for u in &tela.ligacoes {
        let mut primeiro: Option<Handle> = None;
        for par in u.quebras.windows(2) {
            let (Some(a), Some(b)) = (par.first(), par.get(1)) else { continue };
            let kind = EntityKind::Line(Line { a: Vec3::new(a.0, a.1, 0.0), b: Vec3::new(b.0, b.1, 0.0) });
            let handle = s.add_entity(kind)?;
            marcar(s, handle, "ANACAD_LIG", &[("id", &u.numero.to_string())])?;
            if primeiro.is_none() {
                primeiro = Some(handle);
            }
            trechos += 1;
        }
        if let Some(handle) = primeiro {
            let _ = banco.definir_handle("Ligacao", u.numero as i64, &handle.hex());
        }
    }

    Ok(json!({
        "simbolos": simbolos,
        "componentes": componentes,
        "trechos_ligacao": trechos,
    }))
}

fn garantir_bloco_simbolo(s: &mut Session, tipo: i32) -> Result<String> {
    let nome = format!("ANACAD_SIMB_{tipo}");
    let existe = s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(&nome));
    if !existe {
        let doc = s.doc_mut()?;
        let mut bloco = Block::new(&nome);
        bloco.description = format!("Simbolo do tipo {tipo} (anacad)");
        let circulo = doc.new_handle();
        bloco.entities.push(Entity::new(circulo, EntityKind::Circle(Circle { center: Vec3::ZERO, radius: 16.0 })));
        let horizontal = doc.new_handle();
        bloco.entities.push(Entity::new(horizontal, EntityKind::Line(Line { a: Vec3::new(-24.0, 0.0, 0.0), b: Vec3::new(24.0, 0.0, 0.0) })));
        let vertical = doc.new_handle();
        bloco.entities.push(Entity::new(vertical, EntityKind::Line(Line { a: Vec3::new(0.0, -24.0, 0.0), b: Vec3::new(0.0, 24.0, 0.0) })));
        doc.blocks.insert(nome.clone(), Arc::new(bloco));
    }
    Ok(nome)
}

fn marcar(s: &mut Session, handle: Handle, app: &str, campos: &[(&str, &str)]) -> Result<()> {
    let mut tags = vec![RawTag { code: 1001, value: app.to_string() }];
    for (chave, valor) in campos {
        tags.push(RawTag { code: 1000, value: (*chave).to_string() });
        tags.push(RawTag { code: 1000, value: (*valor).to_string() });
    }
    s.doc_mut()?.modify_entity(handle, |e| {
        e.common.xdata = tags;
    })?;
    Ok(())
}
