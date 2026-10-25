//! Comandos do dominio anacad (M1/M2: diagnostico, bloco de teste e importacao).
//!
//! Este arquivo e parte do patch minimo descrito em
//! `anacad/docs/cadcraft-integracao.md`: o cadcraft continua sendo o upstream e
//! o patch e reaplicado por `anacad/scripts/cadcraft-patch.ps1`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use cadcraft_doc::{Block, Circle, Entity, EntityKind, Handle, Line, RawTag};
use cadcraft_geom::{Vec2, Vec3};
use serde_json::{Value, json};

use super::*;
use crate::{Result, Session};

const BLOCO_TESTE: &str = "ANACAD_TESTE";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("anacad", "AnaCAD (diagnostico/importar)", run_anacad)
            .menu(&["AnaCAD", "Importar / Diagnostico"])
            .params("{file?: .lst para diagnostico, test_block?: bool, import?: .lst para importar, db?: .db, nome?: nome do documento} -> {...}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.consultar", "AnaCAD: Consultar Banco", run_consultar)
            .menu(&["AnaCAD", "Consultar Banco"])
            .params("{db, documento?: id} -> {documentos, documento?, componentes, linhas, ligacoes, contagens}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.mover", "AnaCAD: Mover Componente", run_mover)
            .menu(&["AnaCAD", "Mover Componente"])
            .params("{db, handle, dx?, dy?} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.apagar", "AnaCAD: Apagar Componente/Ligacao", run_apagar)
            .menu(&["AnaCAD", "Apagar Componente/Ligacao"])
            .params("{db, handle} -> {removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.comprimento", "AnaCAD: Calcular Comprimentos", run_comprimento)
            .menu(&["AnaCAD", "Calcular Comprimentos"])
            .params("{db, documento?} -> {resultado, ligacoes, comprimento_ligacoes, comprimento_linhas, comprimento_total}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana.importar", "AnaCAD: Importar .ANA (DBAR)", run_ana_importar)
            .menu(&["AnaCAD", "Importar .ANA (DBAR)"])
            .params("{arquivo, db, documento?} -> {barras, atualizadas, sem_barra, sem_vbase}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana", "AnaCAD: Diagnostico .ANA", run_ana)
            .menu(&["AnaCAD", "Diagnostico .ANA"])
            .params("{arquivo} -> {tipo, blocos: [{nome, tipo, linhas, cabecalho?}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.curto", "AnaCAD: Curto-Circuito (simplificado)", run_curto)
            .menu(&["AnaCAD", "Curto-Circuito (simplificado)"])
            .params("{db, documento?, barra (codigo), referencia (codigo)?, sbase?} -> {resultado, barra, referencia, zth, i_pu, vbase, i_ka}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.resultados", "AnaCAD: Resultados", run_resultados)
            .menu(&["AnaCAD", "Resultados"])
            .params("{db, documento?, limite?} -> {resultados: [{id, tipo, saida, criado_em}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ybus", "AnaCAD: Montar Ybus", run_ybus)
            .menu(&["AnaCAD", "Montar Ybus"])
            .params("{db, documento?} -> {resultado, barras, circuitos_usados, circuitos_sem_barra, pares_nao_nulos, simetrica, maior_diagonal}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.calcular", "AnaCAD: Calcular Topologia", run_calcular)
            .menu(&["AnaCAD", "Calcular Topologia"])
            .params("{db, documento?} -> {resultado, barras, linhas, ilhas, barras_isoladas, grau_max, grau_medio}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.validar", "AnaCAD: Validar Handles", run_validar)
            .menu(&["AnaCAD", "Validar Handles"])
            .params("{db} -> {componentes, componentes_ok, componentes_sem_entidade, ligacoes, ligacoes_ok, ligacoes_sem_entidade}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.criar", "AnaCAD: Criar Legenda", run_criar_legenda)
            .menu(&["AnaCAD", "Criar Legenda"])
            .params("{db, documento?, tipo?: X|Y, numero, x, y, canal?} -> {legenda, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.mover", "AnaCAD: Mover Legenda", run_mover_legenda)
            .menu(&["AnaCAD", "Mover Legenda"])
            .params("{db, handle, dx?, dy?} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.criar", "AnaCAD: Criar Barra", run_criar_barra)
            .menu(&["AnaCAD", "Criar Barra"])
            .params("{db, documento?, codigo, x?, y?, nome?} -> {barra, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.criar", "AnaCAD: Criar Linha", run_criar_linha)
            .menu(&["AnaCAD", "Criar Linha"])
            .params("{db, documento?, de (codigo da barra), para (codigo da barra)} -> {linha, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.exportar", "AnaCAD: Exportar .lst", run_exportar)
            .menu(&["AnaCAD", "Exportar .lst"])
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

fn run_ana_importar(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.ana.importar", "`arquivo` is required"))?;
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ana.importar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let bytes = ler_arquivo(arquivo)?;
    let ana = anacad_core::ana::ArquivoAna::ler_texto(&bytes).map_err(|e| bad("anacad.ana.importar", format!("{arquivo}: {e}")))?;
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
    let arquivo = anacad_core::ana::ArquivoAna::ler_texto(&bytes).map_err(|e| bad("anacad.ana", format!("{caminho}: {e}")))?;
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
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.criar", "`codigo` is required"))?;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let nome = str_param(p, "nome");
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.criar", format!("{db}: {e}")))?;
    let id = banco.criar_barra(documento, codigo, nome, x, y).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    let bloco = garantir_bloco(s, "ANACAD_BARRA", 30.0)?;
    let handle = super::blocks::insert(s, &bloco, Vec2::new(x, y), 1.0, 0.0, &serde_json::Map::new())?;
    marcar(s, handle, "ANACAD_BARRA", &[("id", &id.to_string()), ("codigo", &codigo.to_string())])?;
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    Ok(json!({ "barra": id, "handle": handle.hex() }))
}

fn run_criar_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let de = p.get("de").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.criar", "`de` is required"))?;
    let para = p.get("para").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.criar", "`para` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.criar", format!("{db}: {e}")))?;
    let barra_de = banco
        .barra_por_codigo(documento, de)
        .map_err(|e| bad("anacad.linha.criar", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.criar", format!("barra {de} nao existe no documento")))?;
    let barra_para = banco
        .barra_por_codigo(documento, para)
        .map_err(|e| bad("anacad.linha.criar", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.criar", format!("barra {para} nao existe no documento")))?;
    let (x1, y1) = (barra_de.x.unwrap_or(0.0), barra_de.y.unwrap_or(0.0));
    let (x2, y2) = (barra_para.x.unwrap_or(0.0), barra_para.y.unwrap_or(0.0));
    let id = banco.criar_linha(documento, Some(barra_de.id), Some(barra_para.id)).map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    let kind = EntityKind::Line(Line { a: Vec3::new(x1, y1, 0.0), b: Vec3::new(x2, y2, 0.0) });
    let handle = s.add_entity(kind)?;
    marcar(s, handle, "ANACAD_LINHA", &[("id", &id.to_string()), ("de", &de.to_string()), ("para", &para.to_string())])?;
    banco.definir_handle("Linha", id, &handle.hex()).map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    Ok(json!({ "linha": id, "handle": handle.hex() }))
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

fn garantir_bloco(s: &mut Session, nome: &str, raio: f64) -> Result<String> {
    let existe = s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(nome));
    if !existe {
        let doc = s.doc_mut()?;
        let mut bloco = Block::new(nome);
        bloco.description = format!("{nome} (anacad)");
        let circulo = doc.new_handle();
        bloco.entities.push(Entity::new(circulo, EntityKind::Circle(Circle { center: Vec3::ZERO, radius: raio })));
        let h = doc.new_handle();
        bloco.entities.push(Entity::new(h, EntityKind::Line(Line { a: Vec3::new(-raio, 0.0, 0.0), b: Vec3::new(raio, 0.0, 0.0) })));
        let v = doc.new_handle();
        bloco.entities.push(Entity::new(v, EntityKind::Line(Line { a: Vec3::new(0.0, -raio, 0.0), b: Vec3::new(0.0, raio, 0.0) })));
        doc.blocks.insert(nome.to_string(), Arc::new(bloco));
    }
    Ok(nome.to_string())
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
