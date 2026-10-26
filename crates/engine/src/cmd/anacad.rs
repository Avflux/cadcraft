//! Comandos do dominio anacad (M1/M2: diagnostico, bloco de teste e importacao).
//!
//! Este arquivo e parte do patch minimo descrito em
//! `anacad/docs/cadcraft-integracao.md`: o cadcraft continua sendo o upstream e
//! o patch e reaplicado por `anacad/scripts/cadcraft-patch.ps1`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use cadcraft_doc::{Block, Circle, Entity, EntityKind, HAlign, Handle, Line, LwPolyline, RawTag, Text, VAlign};
use cadcraft_geom::{PolyVertex, Vec2, Vec3};
use serde_json::{Value, json};

use super::machines::{SelOutcome, SelectPhase};
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
            .interactive(|_| Ok(Box::new(EditorM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.editar", "AnaCAD: Editar Linha (janela)", run_editar_linha)
            .menu(&[])
            .params("{db, documento?, handle, de?, para?, dados} -> {linha, handle}")
            .interactive(|_| Ok(Box::new(EditorM::default())))
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
        CommandSpec::new("anacad.equipamento.inserir", "AnaCAD: Inserir Equipamento", run_equipamento_inserir)
            .menu(&[])
            .params("{db, documento, tipo, x, y, codigo?} -> {equipamento, handle}")
            .enabled(always)
            .noundo()
            .interactive(|s| {
                if s.anacad_equip_tipo.is_empty() {
                    return Err(crate::EngineError::Other("escolha um equipamento".into()));
                }
                Ok(Box::new(EquipM { tipo: s.anacad_equip_tipo.clone() }))
            }),
        CommandSpec::new("anacad.equipamento.editar", "AnaCAD: Editar Equipamento", run_equipamento_editar)
            .menu(&[])
            .params("{db, documento, handle, dados} -> {equipamento, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.componentes", "AnaCAD: Componentes da Barra", run_barra_componentes)
            .menu(&[])
            .params("{db, documento, codigo} -> {componentes:[{id,nome}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.separar", "AnaCAD: Separar Barra", run_barra_separar)
            .menu(&[])
            .params("{db, documento, atual, nova, nome, componentes?, jumper?} -> {barra, componentes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.unir", "AnaCAD: Unir Barras", run_barra_unir)
            .menu(&[])
            .params("{db, documento, final, eliminada} -> {removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.seccionar", "AnaCAD: Seccionar Linha", run_linha_seccionar)
            .menu(&[])
            .params("{db, documento, de, para, nova, nome, percentual, reatores?} -> {barra, circuitos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.renumerar", "AnaCAD: Renumerar Barra", run_barra_renumerar)
            .menu(&[])
            .params("{db, documento, atual, nova} -> {barra, nova}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.localizar", "AnaCAD: Localizar Barra", run_barra_localizar)
            .menu(&[])
            .params("{db, documento, codigo} -> {codigo, x, y, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.terminal.listar", "AnaCAD: Listar Terminais", run_terminal_listar)
            .menu(&[])
            .params("{db, documento} -> {terminais:[{id,nome,x,y,handle}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.terminal.renomear", "AnaCAD: Renomear Terminal", run_terminal_renomear)
            .menu(&[])
            .params("{db, id, nome} -> {id, nome}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ponto.inserir", "AnaCAD: Inserir Ponto", run_ponto_inserir)
            .menu(&[])
            .params("{db, documento, tipo, x, y, nome?} -> {componente, handle}")
            .enabled(always)
            .noundo()
            .interactive(|s| {
                if s.anacad_equip_tipo.is_empty() {
                    return Err(crate::EngineError::Other("escolha o tipo de ponto".into()));
                }
                Ok(Box::new(PontoM { tipo: s.anacad_equip_tipo.clone() }))
            }),
        CommandSpec::new("anacad.barra.detalhar", "AnaCAD: Detalhar Barra", run_barra_detalhar)
            .menu(&[])
            .params("{db, documento, codigo} -> {barra, componentes, circuitos, vizinhos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.anotar.curto", "AnaCAD: Anotar Curto no Diagrama", run_anotar_curto)
            .menu(&[])
            .params("{db, documento?, limite?} -> {anotacoes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.texto.inserir", "AnaCAD: Texto Livre", run_texto_inserir)
            .menu(&[])
            .params("{x, y, texto, altura?, cor?} -> {handle}")
            .enabled(always)
            .noundo()
            .interactive(|s| Ok(Box::new(TextoM { texto: s.anacad_texto.clone(), altura: s.anacad_texto_altura, cor: s.anacad_texto_cor }))),
        CommandSpec::new("anacad.legenda.config.ler", "AnaCAD: Ler Opções de Legenda", run_legenda_config_ler)
            .menu(&[])
            .params("{db} -> {dados}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.config.gravar", "AnaCAD: Gravar Opções de Legenda", run_legenda_config_gravar)
            .menu(&[])
            .params("{db, dados} -> {ok}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.niveis.curto", "AnaCAD: Níveis de Curto", run_niveis_curto)
            .menu(&[])
            .params("{db, documento?, sbase?, referencia?} -> {niveis:[{barra,i3f_ka,i1f_ka,i2f_ka,xr}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equivalente", "AnaCAD: Equivalente de Rede", run_equivalente)
            .menu(&[])
            .params("{db, documento?, barras:[codigos], sbase?} -> {barras, pares, simetrica}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.zbus", "AnaCAD: Matriz Zbarra", run_zbus)
            .menu(&[])
            .params("{db, documento?, referencia?} -> {barras, diagonal}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.curto.simultaneo", "AnaCAD: Faltas Simultâneas", run_curto_simultaneo)
            .menu(&[])
            .params("{db, documento?, barras:[codigos], referencia?} -> {correntes}")
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
    let mutuas = ana.ler_dmut();
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ana.importar", format!("{db}: {e}")))?;
    let mut atualizadas = 0usize;
    let mut sem_barra = 0usize;
    let mut sem_vbase = 0usize;
    for barra in &barras {
        banco.atualizar_barra_ficticia(Some(documento), barra.codigo, barra.ficticia).map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
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
    let mut mutuas_gravadas = 0usize;
    for mutua in &mutuas {
        banco
            .inserir_mutua(&anacad_data::MutuaNova {
                documento: Some(documento),
                de1: mutua.de1,
                para1: mutua.para1,
                de2: mutua.de2,
                para2: mutua.para2,
                rm: mutua.rm,
                xm: mutua.xm,
            })
            .map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        mutuas_gravadas += 1;
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
        "mutuas": mutuas_gravadas,
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

fn ramos_abertos(p: &Value) -> Vec<(i64, i64)> {
    p.get("abrir")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|par| {
                    let par = par.as_array()?;
                    Some((par.first()?.as_i64()?, par.get(1)?.as_i64()?))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn ramo_aberto(abrir: &[(i64, i64)], de: i64, para: i64) -> bool {
    abrir.iter().any(|(a, b)| (*a == de && *b == para) || (*a == para && *b == de))
}

fn mutuas_do_banco(
    banco: &anacad_data::Banco,
    documento: i64,
    por_codigo: &BTreeMap<i64, i64>,
    componente: &BTreeSet<i64>,
) -> Result<Vec<anacad_core::calc::MutuaCalc>> {
    let mut saida = Vec::new();
    for m in banco.mutuas(documento).map_err(|e| bad("anacad.curto", e.to_string()))? {
        let (Some(de1), Some(para1), Some(de2), Some(para2)) =
            (por_codigo.get(&m.de1).copied(), por_codigo.get(&m.para1).copied(), por_codigo.get(&m.de2).copied(), por_codigo.get(&m.para2).copied())
        else {
            continue;
        };
        if !componente.contains(&de1) || !componente.contains(&para1) || !componente.contains(&de2) || !componente.contains(&para2) {
            continue;
        }
        saida.push(anacad_core::calc::MutuaCalc { de1, para1, de2, para2, rm: m.rm.unwrap_or(0.0), xm: m.xm.unwrap_or(0.0) });
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
    let abrir = ramos_abertos(p);
    let mut adjacencia: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let mut circuitos_mapped: Vec<anacad_core::calc::CircuitoCalc> = Vec::new();
    for c in &circuitos {
        let (Some(de), Some(para), Some(r1), Some(x1)) = (c.de, c.para, c.r1, c.x1) else { continue };
        if ramo_aberto(&abrir, de, para) {
            continue;
        }
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
            defasamento: c.defasamento.unwrap_or(0.0).to_radians(),
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
    let mutuas_calc = mutuas_do_banco(&banco, documento, &por_codigo, &componente)?;
    let ybus = anacad_core::calc::montar_ybus_com_shunts_e_mutuas(&entrada, &mutuas_calc, &shunts);
    let y0 = if tipo_falta == anacad_core::calc::TipoFalta::Monofasica {
        let circuitos0: Vec<anacad_core::calc::CircuitoCalc> = circuitos
            .iter()
            .filter_map(|c| {
                let (Some(de), Some(para)) = (c.de, c.para) else { return None };
                let (de, para) = (*por_codigo.get(&de)?, *por_codigo.get(&para)?);
                if !componente.contains(&de) || !componente.contains(&para) || ramo_aberto(&abrir, c.de?, c.para?) {
                    return None;
                }
                match (c.r0, c.x0) {
                    (Some(r0), Some(x0)) => Some(anacad_core::calc::CircuitoCalc {
                        de,
                        para,
                        r1: r0,
                        x1: x0,
                        tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
                        defasamento: 0.0,
                    }),
                    _ => match (c.r1, c.x1) {
                        (Some(r1), Some(x1)) => Some(anacad_core::calc::CircuitoCalc {
                            de,
                            para,
                            r1,
                            x1,
                            tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
                            defasamento: 0.0,
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
    let zf_pu = f64_or(p, "zf", 0.0);
    let (zth_result, i_pu_result, i_fonte_pu, fontes_len) = if zf_pu > 0.0 {
        let falta_zf = anacad_core::calc::curto_tipo_com_impedancia(
            &ybus,
            y0.as_ref(),
            falta,
            ref_idx,
            tipo_falta,
            1e9,
            anacad_core::calc::Complexo::novo(zf_pu, 0.0),
        )
        .ok_or_else(|| bad("anacad.curto", "matriz singular ou Zth nulo"))?;
        (falta_zf.zth, falta_zf.i_pu, 0.0, 0usize)
    } else if fontes.is_empty() {
        let mut esp = anacad_core::calc::esparsa_de_densa(&ybus);
        esp.somar(ref_idx, ref_idx, anacad_core::calc::Complexo::novo(1e9, 0.0));
        let fator = anacad_core::calc::fatorar_esparsa(&esp).ok_or_else(|| bad("anacad.curto", "fatoracao singular"))?;
        let z1 = fator.zth(falta).ok_or_else(|| bad("anacad.curto", "matriz singular ou Zth nulo"))?;
        let z0 = if tipo_falta == anacad_core::calc::TipoFalta::Monofasica {
            let y0 = y0.as_ref().ok_or_else(|| bad("anacad.curto", "sem sequencia zero"))?;
            let mut esp0 = anacad_core::calc::esparsa_de_densa(y0);
            esp0.somar(ref_idx, ref_idx, anacad_core::calc::Complexo::novo(1e9, 0.0));
            let fator0 = anacad_core::calc::fatorar_esparsa(&esp0).ok_or_else(|| bad("anacad.curto", "fatoracao zero singular"))?;
            Some(fator0.zth(falta).ok_or_else(|| bad("anacad.curto", "Z0 nulo"))?)
        } else {
            None
        };
        let falta_fat = anacad_core::calc::curto_com_zth(z1, z0, tipo_falta).ok_or_else(|| bad("anacad.curto", "matriz singular ou Zth nulo"))?;
        (falta_fat.zth, falta_fat.i_pu, 0.0, 0usize)
    } else if !fontes.is_empty() {
        let mut esp = anacad_core::calc::esparsa_de_densa(&ybus);
        esp.somar(ref_idx, ref_idx, anacad_core::calc::Complexo::novo(1e9, 0.0));
        let n = esp.linhas.len();
        let fator = anacad_core::calc::fatorar_esparsa(&esp).ok_or_else(|| bad("anacad.curto", "fatoracao singular"))?;
        let z1 = fator.zth(falta).ok_or_else(|| bad("anacad.curto", "matriz singular ou Zth nulo"))?;
        let z0 = if tipo_falta == anacad_core::calc::TipoFalta::Monofasica {
            let y0 = y0.as_ref().ok_or_else(|| bad("anacad.curto", "sem sequencia zero"))?;
            let mut esp0 = anacad_core::calc::esparsa_de_densa(y0);
            esp0.somar(ref_idx, ref_idx, anacad_core::calc::Complexo::novo(1e9, 0.0));
            let fator0 = anacad_core::calc::fatorar_esparsa(&esp0).ok_or_else(|| bad("anacad.curto", "fatoracao zero singular"))?;
            Some(fator0.zth(falta).ok_or_else(|| bad("anacad.curto", "Z0 nulo"))?)
        } else {
            None
        };
        let falta_fat = anacad_core::calc::curto_com_zth(z1, z0, tipo_falta).ok_or_else(|| bad("anacad.curto", "matriz singular ou Zth nulo"))?;
        let mut contribuicao = anacad_core::calc::Complexo::ZERO;
        for fonte in &fontes {
            let mut b = vec![anacad_core::calc::Complexo::ZERO; n];
            if fonte.barra < n {
                b[fonte.barra] = anacad_core::calc::Complexo::novo(1.0, 0.0);
                if let Some(x) = fator.resolver(&b)
                    && let Some(zfg) = x.get(falta).copied()
                {
                    contribuicao = contribuicao.soma(anacad_core::calc::Complexo::novo(fonte.i_pu, 0.0).multiplica(zfg.divide(z1)));
                }
            }
        }
        let base = anacad_core::calc::Complexo::novo(1.0, 0.0).divide(falta_fat.zth);
        let total = base.soma(contribuicao);
        (falta_fat.zth, total.modulo(), contribuicao.modulo(), fontes.len())
    } else {
        let resultado = anacad_core::calc::curto_tipo_com_fontes(&ybus, y0.as_ref(), falta, ref_idx, tipo_falta, 1e9, &fontes)
            .ok_or_else(|| bad("anacad.curto", "matriz singular ou Zth nulo"))?;
        (resultado.zth, resultado.i_pu, resultado.i_fonte_pu, fontes.len())
    };
    let vbase = banco.barra_vbase(documento, codigo).map_err(|e| bad("anacad.curto", e.to_string()))?.unwrap_or(0.0);
    let i_base_a = if vbase > 0.0 { sbase * 1e6 / (3.0f64.sqrt() * vbase * 1e3) } else { 0.0 };
    let prefalta = banco
        .dados_barra(documento, codigo)
        .map_err(|e| bad("anacad.curto", e.to_string()))?
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|d| d.get("preFaltaModulo").and_then(Value::as_f64))
        .filter(|v| *v > 0.0)
        .unwrap_or(1.0);
    let prefalta_angulo = banco
        .dados_barra(documento, codigo)
        .map_err(|e| bad("anacad.curto", e.to_string()))?
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|d| d.get("preFaltaAngulo").and_then(Value::as_f64))
        .unwrap_or(0.0);
    let i_pu = i_pu_result * prefalta;
    let i_ka = i_pu * i_base_a / 1e3;
    let contribuicoes_calc: Vec<Value> = if p.get("contribuicoes").and_then(Value::as_bool).unwrap_or(false) {
        anacad_core::calc::correntes_de_ramo(&ybus, &entrada.circuitos, falta, ref_idx, 1e9)
            .unwrap_or_default()
            .into_iter()
            .take(50)
            .map(|c| {
                let codigo_de = barras.iter().find(|b| b.id == c.de).and_then(|b| b.codigo).unwrap_or(0);
                let codigo_para = barras.iter().find(|b| b.id == c.para).and_then(|b| b.codigo).unwrap_or(0);
                json!({"de": codigo_de, "para": codigo_para, "i_pu": c.i_pu})
            })
            .collect()
    } else {
        Vec::new()
    };
    let z_angulo = zth_result.im.atan2(zth_result.re).to_degrees();
    let i_angulo = prefalta_angulo - z_angulo;
    let id = banco
        .inserir_resultado(
            Some(documento),
            "curto_circuito",
            &json!({ "barra": codigo, "referencia": referencia_codigo, "sbase": sbase, "tipo": tipo_texto }).to_string(),
            &json!({
                            "barra": codigo,
                            "tipo": tipo_texto,
                            "zth_re": zth_result.re,
                            "zth_im": zth_result.im,
                            "i_pu": i_pu,
                            "i_fonte_pu": i_fonte_pu,
                            "prefalta": prefalta,
                    "prefalta_angulo": prefalta_angulo,
                    "angulo_graus": i_angulo,
                            "prefalta_angulo": prefalta_angulo,
                            "angulo_graus": i_angulo,
                            "fontes": fontes_len,
                "zf_pu": zf_pu,
            "contribuicoes": contribuicoes_calc,
                            "zf_pu": zf_pu,
            "contribuicoes": contribuicoes_calc,
                        "contribuicoes": contribuicoes_calc,
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
        "zth_re": zth_result.re,
        "zth_im": zth_result.im,
        "i_pu": i_pu,
        "i_fonte_pu": i_fonte_pu,
        "prefalta": prefalta,
        "prefalta_angulo": prefalta_angulo,
        "angulo_graus": i_angulo,
        "fontes": fontes_len,
        "zf_pu": zf_pu,
        "contribuicoes": contribuicoes_calc,
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
                    Some(anacad_core::calc::CircuitoCalc {
                        de,
                        para,
                        r1,
                        x1,
                        tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
                        defasamento: c.defasamento.unwrap_or(0.0).to_radians(),
                    })
                }
                _ => None,
            })
            .collect(),
    };
    let shunts = shunts_do_banco(&banco, documento, &por_codigo)?;
    let componente: BTreeSet<i64> = entrada.barras.iter().copied().collect();
    let mutuas_calc = mutuas_do_banco(&banco, documento, &por_codigo, &componente)?;
    let ybus = anacad_core::calc::montar_ybus_com_shunts_e_mutuas(&entrada, &mutuas_calc, &shunts);
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
        let criado = run_criar_barra(s, &params)?;
        if let Some(handle) = criado.get("handle").and_then(Value::as_str) {
            s.anacad_edit_handle = Some(handle.to_string());
            if let Some(h) = Handle::parse_hex(handle) {
                s.set_selection(vec![h]);
            }
        }
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
        let criado = run_criar_linha(s, &params)?;
        if let Some(handle) = criado.get("handle").and_then(Value::as_str) {
            s.anacad_edit_handle = Some(handle.to_string());
            if let Some(h) = Handle::parse_hex(handle) {
                s.set_selection(vec![h]);
            }
        }
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

#[derive(Default)]
struct EditorM {
    sel: SelectPhase,
}

impl Interactive for EditorM {
    fn name(&self) -> &'static str {
        "ANACAD_EDITAR"
    }
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        self.sel = SelectPhase::begin(s);
        self.sel.single = true;
        Ok(Step::Continue)
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Selecione a polilinha", Accept::SELECT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        match self.sel.feed(s, &i)? {
            SelOutcome::More => Ok(Step::Continue),
            SelOutcome::Empty => Ok(Step::Done),
            SelOutcome::Done(hs) => {
                let Some(handle) = hs.first().copied() else { return Ok(Step::Done) };
                let ok = matches!(s.doc()?.entity(handle).map(|e| &e.kind), Some(EntityKind::LwPolyline(_)));
                if !ok {
                    return Err(crate::EngineError::Other("selecione uma LWPOLYLINE".into()));
                }
                s.anacad_edit_handle = Some(handle.hex());
                s.set_selection(vec![handle]);
                Ok(Step::Done)
            }
        }
    }
}

const EQUIPAMENTOS: &[(&str, &str)] = &[
    ("transformador", "Transformador"),
    ("gerador", "Gerador"),
    ("carga", "Carga"),
    ("shunt", "Shunt de Barra"),
    ("shunt_linha", "Shunt de Linha"),
    ("compensador_serie", "Compensador Série"),
    ("trafo_aterramento", "Trafo de Aterramento"),
    ("gerador_fonte", "Gerador Fonte de Corrente"),
    ("impedancia_mutua", "Impedância Mútua"),
    ("motor", "Motor de Indução"),
    ("compensador_estatico", "Compensador Estático"),
];

fn tipo_equipamento(tipo: &str) -> Option<(&'static str, &'static str)> {
    EQUIPAMENTOS.iter().copied().find(|(slug, nome)| *slug == tipo || *nome == tipo || slug.replace('_', " ") == tipo.to_lowercase())
}

fn push_linha(doc: &mut cadcraft_doc::Drawing, bloco: &mut Block, a: (f64, f64), b: (f64, f64)) {
    let h = doc.new_handle();
    bloco.entities.push(Entity::new(h, EntityKind::Line(Line { a: Vec3::new(a.0, a.1, 0.0), b: Vec3::new(b.0, b.1, 0.0) })));
}

fn push_circulo(doc: &mut cadcraft_doc::Drawing, bloco: &mut Block, c: (f64, f64), r: f64) {
    let h = doc.new_handle();
    bloco.entities.push(Entity::new(h, EntityKind::Circle(Circle { center: Vec3::new(c.0, c.1, 0.0), radius: r })));
}

fn bloco_equipamento(s: &mut Session, slug: &str) -> Result<String> {
    let nome = format!("ANACAD_EQ_{slug}");
    if s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(&nome)) {
        return Ok(nome);
    }
    let doc = s.doc_mut()?;
    let mut bloco = Block::new(&nome);
    bloco.description = format!("Equipamento {slug} (anacad)");
    match slug {
        "transformador" => {
            push_circulo(doc, &mut bloco, (-9.0, 0.0), 8.0);
            push_circulo(doc, &mut bloco, (9.0, 0.0), 8.0);
            push_linha(doc, &mut bloco, (-9.0, 0.0), (9.0, 0.0));
        }
        "gerador" => {
            push_circulo(doc, &mut bloco, (0.0, 0.0), 12.0);
            push_linha(doc, &mut bloco, (-12.0, 0.0), (12.0, 0.0));
            push_linha(doc, &mut bloco, (0.0, -12.0), (0.0, 12.0));
        }
        "carga" => {
            push_linha(doc, &mut bloco, (0.0, 12.0), (-10.0, -8.0));
            push_linha(doc, &mut bloco, (-10.0, -8.0), (10.0, -8.0));
            push_linha(doc, &mut bloco, (10.0, -8.0), (0.0, 12.0));
        }
        "shunt" | "shunt_linha" => {
            push_linha(doc, &mut bloco, (0.0, 12.0), (0.0, 4.0));
            push_linha(doc, &mut bloco, (0.0, 4.0), (-8.0, 0.0));
            push_linha(doc, &mut bloco, (-8.0, 0.0), (8.0, -4.0));
            push_linha(doc, &mut bloco, (8.0, -4.0), (0.0, -8.0));
        }
        "compensador_serie" => {
            push_linha(doc, &mut bloco, (-14.0, 0.0), (-10.0, 0.0));
            push_linha(doc, &mut bloco, (-10.0, 0.0), (-10.0, 8.0));
            push_linha(doc, &mut bloco, (-10.0, 8.0), (10.0, 8.0));
            push_linha(doc, &mut bloco, (10.0, 8.0), (10.0, -8.0));
            push_linha(doc, &mut bloco, (10.0, -8.0), (-10.0, -8.0));
            push_linha(doc, &mut bloco, (-10.0, -8.0), (-10.0, 0.0));
            push_linha(doc, &mut bloco, (10.0, 0.0), (14.0, 0.0));
        }
        "trafo_aterramento" => {
            push_circulo(doc, &mut bloco, (0.0, 4.0), 8.0);
            push_linha(doc, &mut bloco, (0.0, -4.0), (0.0, -12.0));
            push_linha(doc, &mut bloco, (-6.0, -12.0), (6.0, -12.0));
        }
        "gerador_fonte" => {
            push_circulo(doc, &mut bloco, (0.0, 0.0), 12.0);
            push_linha(doc, &mut bloco, (-12.0, 0.0), (8.0, 0.0));
            push_linha(doc, &mut bloco, (4.0, 4.0), (8.0, 0.0));
            push_linha(doc, &mut bloco, (4.0, -4.0), (8.0, 0.0));
        }
        "impedancia_mutua" => {
            push_linha(doc, &mut bloco, (-12.0, 0.0), (12.0, 0.0));
            push_linha(doc, &mut bloco, (-12.0, 6.0), (12.0, 6.0));
            push_linha(doc, &mut bloco, (-6.0, 0.0), (-6.0, 6.0));
            push_linha(doc, &mut bloco, (6.0, 0.0), (6.0, 6.0));
        }
        "motor" => {
            push_circulo(doc, &mut bloco, (0.0, 0.0), 12.0);
            push_linha(doc, &mut bloco, (-8.0, -8.0), (8.0, 8.0));
            push_linha(doc, &mut bloco, (-8.0, 8.0), (8.0, -8.0));
        }
        _ => {
            push_circulo(doc, &mut bloco, (0.0, 0.0), 12.0);
            push_linha(doc, &mut bloco, (-12.0, 0.0), (12.0, 0.0));
            push_linha(doc, &mut bloco, (0.0, -12.0), (0.0, 12.0));
        }
    }
    doc.blocks.insert(nome.clone(), Arc::new(bloco));
    Ok(nome)
}

fn equipamento_json(id: i64, tipo: &str, codigo: i64, handle: &str, x: f64, y: f64) -> Value {
    json!({"equipamento": id, "tipo": tipo, "codigo": codigo, "handle": handle, "x": x, "y": y})
}

fn run_equipamento_inserir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.inserir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tipo_txt = str_param(p, "tipo").ok_or_else(|| bad("anacad.equipamento.inserir", "`tipo` is required"))?;
    let (slug, nome_tipo) = tipo_equipamento(tipo_txt).ok_or_else(|| bad("anacad.equipamento.inserir", format!("tipo desconhecido: {tipo_txt}")))?;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.inserir", format!("{db}: {e}")))?;
    let codigo = match p.get("codigo").and_then(Value::as_i64) {
        Some(c) if c > 0 => c,
        _ => banco.proximo_codigo_equipamento(documento, nome_tipo).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?,
    };
    let id = banco.criar_equipamento(documento, nome_tipo, codigo, nome_tipo, x, y).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?;
    let bloco = bloco_equipamento(s, slug)?;
    let handle = super::blocks::insert(s, &bloco, Vec2::new(x, y), 1.0, 0.0, &serde_json::Map::new())?;
    banco.definir_handle("Equipamento", id, &handle.hex()).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?;
    marcar(
        s,
        handle,
        "ANACAD_EQ",
        &[("id", &id.to_string()), ("tipo", nome_tipo), ("codigo", &codigo.to_string()), ("x", &x.to_string()), ("y", &y.to_string())],
    )?;
    s.anacad_equip_handle = Some(handle.hex());
    s.set_selection(vec![handle]);
    Ok(equipamento_json(id, nome_tipo, codigo, &handle.hex(), x, y))
}

fn run_equipamento_editar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.editar", "`db` is required"))?;
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.equipamento.editar", "`handle` is required"))?;
    let handle = Handle::parse_hex(handle_txt).ok_or_else(|| bad("anacad.equipamento.editar", "handle invalido"))?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.editar", format!("{db}: {e}")))?;
    let nome = dados.get("nome").and_then(Value::as_str);
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.atualizar_dados_equipamento(&handle.hex(), &texto).map_err(|e| bad("anacad.equipamento.editar", e.to_string()))?;
    let pos = s.doc()?.entity(handle).and_then(|e| match &e.kind {
        EntityKind::Insert(ins) => Some((ins.insert.x, ins.insert.y)),
        _ => None,
    });
    if let Some((x, y)) = pos {
        banco.atualizar_equipamento_nome_pos(&handle.hex(), nome, x, y).map_err(|e| bad("anacad.equipamento.editar", e.to_string()))?;
    }
    let tipo = s
        .doc()?
        .entity(handle)
        .and_then(|e| {
            e.common
                .xdata
                .iter()
                .filter(|t| t.code == 1000)
                .map(|t| t.value.as_str())
                .collect::<Vec<_>>()
                .windows(2)
                .find(|w| w[0] == "tipo")
                .map(|w| w[1].to_string())
        })
        .unwrap_or_default();
    let codigo = s.doc()?.entity(handle).and_then(|e| {
        e.common
            .xdata
            .iter()
            .filter(|t| t.code == 1000)
            .map(|t| t.value.as_str())
            .collect::<Vec<_>>()
            .windows(2)
            .find(|w| w[0] == "codigo")
            .and_then(|w| w[1].parse::<i64>().ok())
    });
    let id = s
        .doc()?
        .entity(handle)
        .and_then(|e| {
            e.common
                .xdata
                .iter()
                .filter(|t| t.code == 1000)
                .map(|t| t.value.as_str())
                .collect::<Vec<_>>()
                .windows(2)
                .find(|w| w[0] == "id")
                .and_then(|w| w[1].parse::<i64>().ok())
        })
        .unwrap_or(0);
    marcar(s, handle, "ANACAD_EQ", &[("id", &id.to_string()), ("tipo", &tipo), ("codigo", &codigo.unwrap_or(0).to_string()), ("dados", &texto)])?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    banco.sincronizar_equipamento(documento, &tipo, &dados).map_err(|e| bad("anacad.equipamento.editar", e.to_string()))?;
    Ok(json!({"equipamento": id, "tipo": tipo, "handle": handle.hex(), "dados": dados}))
}

struct EquipM {
    tipo: String,
}

impl Interactive for EquipM {
    fn name(&self) -> &'static str {
        "ANACAD_EQ"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Ponto de inserção do equipamento", Accept::POINT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if s.anacad_db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de inserir".into()));
        }
        run_equipamento_inserir(s, &json!({"db": s.anacad_db, "documento": s.anacad_documento, "tipo": self.tipo, "x": p.x, "y": p.y}))?;
        Ok(Step::Done)
    }
    fn preview(&self, _s: &Session, _cursor: Vec2) -> Vec<EntityKind> {
        Vec::new()
    }
}

fn run_barra_componentes(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.componentes", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.componentes", "`codigo` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.componentes", format!("{db}: {e}")))?;
    let componentes: Vec<Value> = banco
        .componentes_da_barra(documento, codigo)
        .map_err(|e| bad("anacad.barra.componentes", e.to_string()))?
        .into_iter()
        .map(|(id, nome)| json!({"id": id, "nome": nome}))
        .collect();
    Ok(json!({"componentes": componentes}))
}

fn run_barra_separar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.separar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let atual = p.get("atual").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.separar", "`atual` is required"))?;
    let nova = p.get("nova").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.separar", "`nova` is required"))?;
    let nome = str_param(p, "nome").unwrap_or("Nova barra");
    let componentes: Vec<i64> =
        p.get("componentes").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    let jumper = bool_or(p, "jumper", false);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.separar", format!("{db}: {e}")))?;
    let id = banco.separar_barra(documento, atual, nova, nome, &componentes, jumper).map_err(|e| bad("anacad.barra.separar", e.to_string()))?;
    Ok(json!({"barra": id, "codigo": nova, "componentes": componentes.len(), "jumper": jumper}))
}

fn run_barra_unir(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.unir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let final_codigo = p.get("final").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.unir", "`final` is required"))?;
    let eliminada = p.get("eliminada").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.unir", "`eliminada` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.unir", format!("{db}: {e}")))?;
    let removidos = banco.unir_barras(documento, final_codigo, eliminada).map_err(|e| bad("anacad.barra.unir", e.to_string()))?;
    Ok(json!({"final": final_codigo, "eliminada": eliminada, "removidos": removidos}))
}

fn run_linha_seccionar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.seccionar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let de = p.get("de").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.seccionar", "`de` is required"))?;
    let para = p.get("para").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.seccionar", "`para` is required"))?;
    let nova = p.get("nova").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.seccionar", "`nova` is required"))?;
    let nome = str_param(p, "nome").unwrap_or("Barra de seccionamento");
    let percentual = f64_or(p, "percentual", 50.0);
    let reatores = bool_or(p, "reatores", false);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.seccionar", format!("{db}: {e}")))?;
    let barra = banco
        .seccionar_linha(&anacad_data::SeccionarLinha {
            documento,
            de_codigo: de,
            para_codigo: para,
            nova_codigo: nova,
            nome: nome.to_string(),
            percentual,
            eliminar_reatores: reatores,
        })
        .map_err(|e| bad("anacad.linha.seccionar", e.to_string()))?;
    Ok(json!({"barra": barra, "codigo": nova, "de": de, "para": para, "percentual": percentual}))
}

fn run_barra_renumerar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.renumerar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let atual = p.get("atual").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.renumerar", "`atual` is required"))?;
    let nova = p.get("nova").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.renumerar", "`nova` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.renumerar", format!("{db}: {e}")))?;
    banco.renumerar_barra(documento, atual, nova).map_err(|e| bad("anacad.barra.renumerar", e.to_string()))?;
    Ok(json!({"atual": atual, "nova": nova}))
}

fn run_barra_localizar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.localizar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.localizar", "`codigo` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.localizar", format!("{db}: {e}")))?;
    let barra = banco
        .barra_por_codigo(documento, codigo)
        .map_err(|e| bad("anacad.barra.localizar", e.to_string()))?
        .ok_or_else(|| bad("anacad.barra.localizar", format!("barra {codigo} nao existe")))?;
    Ok(json!({"codigo": codigo, "x": barra.x, "y": barra.y, "handle": barra.handle}))
}

fn run_terminal_listar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.terminal.listar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.terminal.listar", format!("{db}: {e}")))?;
    let terminais: Vec<Value> = banco
        .terminais(documento)
        .map_err(|e| bad("anacad.terminal.listar", e.to_string()))?
        .into_iter()
        .map(|t| json!({"id": t.id, "nome": t.nome, "x": t.x, "y": t.y, "handle": t.handle}))
        .collect();
    Ok(json!({"terminais": terminais}))
}

fn run_terminal_renomear(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.terminal.renomear", "`db` is required"))?;
    let id = p.get("id").and_then(Value::as_i64).ok_or_else(|| bad("anacad.terminal.renomear", "`id` is required"))?;
    let nome = str_param(p, "nome").ok_or_else(|| bad("anacad.terminal.renomear", "`nome` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.terminal.renomear", format!("{db}: {e}")))?;
    let n = banco.renomear_terminal(id, nome).map_err(|e| bad("anacad.terminal.renomear", e.to_string()))?;
    Ok(json!({"id": id, "nome": nome, "linhas": n}))
}

fn tipo_ponto(tipo: &str) -> Option<(i64, &'static str)> {
    match tipo {
        "terminal" => Some((20, "Terminal")),
        "monitoracao" => Some((21, "Ponto de Monitoração")),
        "quebra" => Some((29, "Ponto de Quebra")),
        _ => None,
    }
}

fn run_ponto_inserir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ponto.inserir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tipo_txt = p.get("tipo").and_then(Value::as_str).unwrap_or("terminal");
    let (tipo, nome_tipo) = tipo_ponto(tipo_txt).ok_or_else(|| bad("anacad.ponto.inserir", "tipo desconhecido"))?;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let nome = str_param(p, "nome").unwrap_or(nome_tipo);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ponto.inserir", format!("{db}: {e}")))?;
    let id = banco.criar_componente(documento, tipo, None, nome, x, y).map_err(|e| bad("anacad.ponto.inserir", e.to_string()))?;
    let bloco = garantir_bloco_simbolo(s, tipo as i32)?;
    let handle = super::blocks::insert(s, &bloco, Vec2::new(x, y), 1.0, 0.0, &serde_json::Map::new())?;
    banco.definir_handle("Componente", id, &handle.hex()).map_err(|e| bad("anacad.ponto.inserir", e.to_string()))?;
    marcar(s, handle, "ANACAD_COMP", &[("id", &id.to_string()), ("tipo", &tipo.to_string()), ("codigo", "0"), ("nome", nome)])?;
    s.set_selection(vec![handle]);
    Ok(json!({"componente": id, "tipo": tipo, "handle": handle.hex(), "nome": nome}))
}

struct PontoM {
    tipo: String,
}

impl Interactive for PontoM {
    fn name(&self) -> &'static str {
        "ANACAD_PONTO"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Ponto de inserção", Accept::POINT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if s.anacad_db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de inserir".into()));
        }
        run_ponto_inserir(s, &json!({"db": s.anacad_db, "documento": s.anacad_documento, "tipo": self.tipo, "x": p.x, "y": p.y}))?;
        Ok(Step::Done)
    }
}

fn run_barra_detalhar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.detalhar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.detalhar", "`codigo` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.detalhar", format!("{db}: {e}")))?;
    let barra = banco
        .barra_por_codigo(documento, codigo)
        .map_err(|e| bad("anacad.barra.detalhar", e.to_string()))?
        .ok_or_else(|| bad("anacad.barra.detalhar", format!("barra {codigo} nao existe")))?;
    let componentes: Vec<Value> = banco
        .componentes_da_barra(documento, codigo)
        .map_err(|e| bad("anacad.barra.detalhar", e.to_string()))?
        .into_iter()
        .map(|(id, nome)| json!({"id": id, "nome": nome}))
        .collect();
    let circuitos: Vec<Value> = banco
        .circuitos_da_barra(documento, codigo)
        .map_err(|e| bad("anacad.barra.detalhar", e.to_string()))?
        .into_iter()
        .map(|c| json!({"de": c.de, "para": c.para, "tipo": c.tipo, "R1": c.r1, "X1": c.x1}))
        .collect();
    let mut vizinhos: Vec<i64> = Vec::new();
    for c in &circuitos {
        for k in ["de", "para"] {
            if let Some(v) = c.get(k).and_then(Value::as_i64)
                && v != codigo
                && !vizinhos.contains(&v)
            {
                vizinhos.push(v);
            }
        }
    }
    Ok(
        json!({"barra": {"codigo": codigo, "nome": barra.nome, "x": barra.x, "y": barra.y}, "componentes": componentes, "circuitos": circuitos, "vizinhos": vizinhos}),
    )
}

fn run_anotar_curto(s: &mut Session, p: &Value) -> Result<Value> {
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

fn run_texto_inserir(s: &mut Session, p: &Value) -> Result<Value> {
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let texto = str_param(p, "texto").ok_or_else(|| bad("anacad.texto.inserir", "`texto` is required"))?;
    let altura = f64_or(p, "altura", 2.5);
    let cor = p.get("cor").and_then(Value::as_u64).unwrap_or(7) as u8;
    let entidade = EntityKind::Text(Text {
        insert: Vec3::new(x, y, 0.0),
        align_pt: None,
        height: altura,
        value: texto.to_string(),
        rotation: 0.0,
        width_factor: 1.0,
        oblique: 0.0,
        style: "Standard".into(),
        halign: HAlign::Left,
        valign: VAlign::Baseline,
    });
    let handle = s.add_entity(entidade)?;
    s.doc_mut()?.modify_entity(handle, |e| e.common.color = cadcraft_doc::color::Color::Index(cor))?;
    marcar(s, handle, "ANACAD_TEXTO", &[("texto", texto), ("altura", &altura.to_string()), ("cor", &cor.to_string())])?;
    Ok(json!({"handle": handle.hex(), "texto": texto}))
}

struct TextoM {
    texto: String,
    altura: f64,
    cor: u8,
}

impl Interactive for TextoM {
    fn name(&self) -> &'static str {
        "ANACAD_TEXTO"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Ponto do texto", Accept::POINT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if self.texto.trim().is_empty() {
            return Err(crate::EngineError::Other("informe o texto".into()));
        }
        run_texto_inserir(s, &json!({"x": p.x, "y": p.y, "texto": self.texto, "altura": self.altura, "cor": self.cor}))?;
        Ok(Step::Done)
    }
}

fn run_legenda_config_ler(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.config.ler", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.config.ler", format!("{db}: {e}")))?;
    let dados = banco
        .ler_config("legenda_opcoes")
        .map_err(|e| bad("anacad.legenda.config.ler", e.to_string()))?
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .unwrap_or_else(|| json!({}));
    Ok(json!({"dados": dados}))
}

fn run_legenda_config_gravar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.config.gravar", "`db` is required"))?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.config.gravar", format!("{db}: {e}")))?;
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.gravar_config("legenda_opcoes", &texto).map_err(|e| bad("anacad.legenda.config.gravar", e.to_string()))?;
    Ok(json!({"ok": true}))
}

fn run_niveis_curto(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.niveis.curto", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let sbase = f64_or(p, "sbase", 100.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.niveis.curto", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.niveis.curto", e.to_string()))?;
    let circuitos = banco.circuitos(documento, 100_000).map_err(|e| bad("anacad.niveis.curto", e.to_string()))?;
    let por_codigo: BTreeMap<i64, i64> = barras.iter().filter_map(|b| b.codigo.map(|c| (c, b.id))).collect();
    let mut ids: Vec<i64> = barras.iter().map(|b| b.id).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Ok(json!({"niveis": []}));
    }
    let mut positivos = Vec::new();
    let mut zeros = Vec::new();
    for c in &circuitos {
        let (Some(de), Some(para)) = (c.de, c.para) else { continue };
        let (Some(de), Some(para)) = (por_codigo.get(&de).copied(), por_codigo.get(&para).copied()) else { continue };
        if let (Some(r1), Some(x1)) = (c.r1, c.x1) {
            positivos.push(anacad_core::calc::CircuitoCalc {
                de,
                para,
                r1,
                x1,
                tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
                defasamento: c.defasamento.unwrap_or(0.0).to_radians(),
            });
        }
        if let (Some(r0), Some(x0)) = (c.r0, c.x0) {
            zeros.push(anacad_core::calc::CircuitoCalc { de, para, r1: r0, x1: x0, tap: 1.0, defasamento: 0.0 });
        }
    }
    let entrada1 = anacad_core::calc::EntradaYbus { barras: ids.clone(), circuitos: positivos };
    let entrada0 = if zeros.is_empty() { None } else { Some(anacad_core::calc::EntradaYbus { barras: ids.clone(), circuitos: zeros }) };
    let y1 = anacad_core::calc::montar_ybus(&entrada1);
    let y0 = entrada0.as_ref().map(anacad_core::calc::montar_ybus);
    let ref_idx = match p.get("referencia").and_then(Value::as_i64) {
        Some(code) => por_codigo.get(&code).and_then(|id| y1.indices.iter().position(|b| b == id)).unwrap_or(0),
        None => 0,
    };
    let referencia = y1.indices.get(ref_idx).copied().unwrap_or(ids[0]);
    let mut esp1 = anacad_core::calc::montar_ybus_esparsa(&entrada1);
    esp1.somar(ref_idx, ref_idx, anacad_core::calc::Complexo::novo(1e9, 0.0));
    let fat1 = anacad_core::calc::fatorar_esparsa(&esp1);
    let mut esp0 = entrada0.as_ref().map(anacad_core::calc::montar_ybus_esparsa);
    let fat0 = esp0.as_mut().map(|e| {
        e.somar(ref_idx, ref_idx, anacad_core::calc::Complexo::novo(1e9, 0.0));
        anacad_core::calc::fatorar_esparsa(e)
    });
    let ficticias: BTreeSet<i64> = barras.iter().filter(|b| b.ficticia).map(|b| b.id).collect();
    let mut niveis = Vec::new();
    for id in &ids {
        if ficticias.contains(id) {
            continue;
        }
        if *id == referencia {
            continue;
        }
        let Some(idx) = y1.indices.iter().position(|b| b == id) else { continue };
        let z1 = fat1
            .as_ref()
            .and_then(|f| f.zth(idx))
            .or_else(|| anacad_core::calc::zth_esparso(&entrada1, *id, referencia, 1e9))
            .or_else(|| anacad_core::calc::zth(&y1, idx, ref_idx, 1e9));
        let Some(z1) = z1 else { continue };
        let modulo1 = z1.modulo();
        if modulo1 < 1e-12 {
            continue;
        }
        let i3 = 1.0 / modulo1;
        let i2 = (3.0f64).sqrt() / (2.0 * modulo1);
        let mut i1 = None;
        if let Some(y0) = &y0
            && let Some(idx0) = y0.indices.iter().position(|b| b == id)
            && let Some(z0) = fat0
                .as_ref()
                .and_then(|f| f.as_ref().and_then(|f| f.zth(idx0)))
                .or_else(|| entrada0.as_ref().and_then(|e| anacad_core::calc::zth_esparso(e, *id, referencia, 1e9)))
                .or_else(|| anacad_core::calc::zth(y0, idx0, ref_idx, 1e9))
        {
            let z = anacad_core::calc::Complexo { re: 2.0 * z1.re + z0.re, im: 2.0 * z1.im + z0.im };
            if z.modulo() > 1e-12 {
                i1 = Some(3.0 / z.modulo());
            }
        }
        let codigo = barras.iter().find(|b| b.id == *id).and_then(|b| b.codigo).unwrap_or(0);
        let vbase = banco.barra_vbase(documento, codigo).map_err(|e| bad("anacad.niveis.curto", e.to_string()))?.unwrap_or(0.0);
        let i_base_a = if vbase > 0.0 { sbase * 1e6 / ((3.0f64).sqrt() * vbase * 1e3) } else { 0.0 };
        let k = i_base_a / 1e3;
        niveis.push(json!({
            "barra": codigo,
            "i3f_ka": i3 * k,
            "i2f_ka": i2 * k,
            "i1f_ka": i1.map(|v| v * k),
            "xr": if z1.re.abs() > 1e-12 { (z1.im / z1.re).abs() } else { 0.0 },
            "vbase": vbase,
        }));
    }
    Ok(json!({"niveis": niveis, "referencia": referencia}))
}

fn run_equivalente(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equivalente", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let manter: Vec<i64> = p.get("barras").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    if manter.is_empty() {
        return Err(bad("anacad.equivalente", "informe as barras a manter"));
    }
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equivalente", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.equivalente", e.to_string()))?;
    let circuitos = banco.circuitos(documento, 100_000).map_err(|e| bad("anacad.equivalente", e.to_string()))?;
    let por_codigo: BTreeMap<i64, i64> = barras.iter().filter_map(|b| b.codigo.map(|c| (c, b.id))).collect();
    let mut ids: Vec<i64> = barras.iter().map(|b| b.id).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut circuitos_calc = Vec::new();
    for c in &circuitos {
        let (Some(de), Some(para), Some(r1), Some(x1)) = (c.de, c.para, c.r1, c.x1) else { continue };
        let (Some(de), Some(para)) = (por_codigo.get(&de).copied(), por_codigo.get(&para).copied()) else { continue };
        circuitos_calc.push(anacad_core::calc::CircuitoCalc {
            de,
            para,
            r1,
            x1,
            tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
            defasamento: c.defasamento.unwrap_or(0.0).to_radians(),
        });
    }
    let ybus = anacad_core::calc::montar_ybus(&anacad_core::calc::EntradaYbus { barras: ids, circuitos: circuitos_calc });
    let mut manter_ids = Vec::new();
    for codigo in &manter {
        if let Some(id) = por_codigo.get(codigo) {
            manter_ids.push(*id);
        }
    }
    let eq =
        anacad_core::calc::reduzir_ybus(&ybus, &manter_ids).ok_or_else(|| bad("anacad.equivalente", "nao foi possivel reduzir (matriz singular)"))?;
    let resumo = eq.resumo();
    let barras_equivalentes: Vec<i64> = eq.indices.iter().map(|id| barras.iter().find(|b| b.id == *id).and_then(|b| b.codigo).unwrap_or(0)).collect();
    let validacao_equivalente = if p.get("validar").and_then(Value::as_bool).unwrap_or(false) && manter_ids.len() > 1 {
        let ref_original = manter_ids.first().and_then(|id| ybus.indices.iter().position(|b| b == id)).unwrap_or(0);
        let ref_eq = 0usize;
        let mut amostras = Vec::new();
        let mut maior = 0.0f64;
        for (k, id) in manter_ids.iter().enumerate() {
            if k == 0 {
                continue;
            }
            let Some(pos_o) = ybus.indices.iter().position(|b| b == id) else { continue };
            let Some(pos_e) = eq.indices.iter().position(|b| b == id) else { continue };
            let (Some(z_o), Some(z_e)) = (anacad_core::calc::zth(&ybus, pos_o, ref_original, 1e9), anacad_core::calc::zth(&eq, pos_e, ref_eq, 1e9))
            else {
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
    let entrada = json!({"barras": manter, "documento": documento}).to_string();
    let saida = json!({
        "barras": barras_equivalentes,
        "pares_nao_nulos": resumo.pares_nao_nulos,
        "simetrica": resumo.simetrica,
        "maior_diagonal": resumo.maior_diagonal,
    })
    .to_string();
    let id = banco.inserir_resultado(Some(documento), "equivalente", &entrada, &saida).map_err(|e| bad("anacad.equivalente", e.to_string()))?;
    Ok(json!({
        "resultado": id,
        "barras": barras_equivalentes,
        "pares_nao_nulos": resumo.pares_nao_nulos,
        "simetrica": resumo.simetrica,
        "maior_diagonal": resumo.maior_diagonal,
        "validacao": validacao_equivalente,
    }))
}

fn run_zbus(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.zbus", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.zbus", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.zbus", e.to_string()))?;
    let circuitos = banco.circuitos(documento, 100_000).map_err(|e| bad("anacad.zbus", e.to_string()))?;
    let por_codigo: BTreeMap<i64, i64> = barras.iter().filter_map(|b| b.codigo.map(|c| (c, b.id))).collect();
    let mut ids: Vec<i64> = barras.iter().filter(|b| !b.ficticia).map(|b| b.id).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut circuitos_calc = Vec::new();
    for c in &circuitos {
        let (Some(de), Some(para), Some(r1), Some(x1)) = (c.de, c.para, c.r1, c.x1) else { continue };
        let (Some(de), Some(para)) = (por_codigo.get(&de).copied(), por_codigo.get(&para).copied()) else { continue };
        if !ids.contains(&de) || !ids.contains(&para) {
            continue;
        }
        circuitos_calc.push(anacad_core::calc::CircuitoCalc {
            de,
            para,
            r1,
            x1,
            tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
            defasamento: c.defasamento.unwrap_or(0.0).to_radians(),
        });
    }
    if ids.is_empty() {
        return Ok(json!({"barras": [], "diagonal": []}));
    }
    let entrada = anacad_core::calc::EntradaYbus { barras: ids.clone(), circuitos: circuitos_calc };
    let ybus = anacad_core::calc::montar_ybus(&entrada);
    let ref_idx = match p.get("referencia").and_then(Value::as_i64) {
        Some(code) => por_codigo.get(&code).and_then(|id| ybus.indices.iter().position(|b| b == id)).unwrap_or(0),
        None => 0,
    };
    let z = anacad_core::calc::zbus(&ybus, ref_idx, 1e9).ok_or_else(|| bad("anacad.zbus", "matriz singular"))?;
    let codigo_de = |id: i64| barras.iter().find(|b| b.id == id).and_then(|b| b.codigo).unwrap_or(0);
    let barras_codigo: Vec<i64> = ybus.indices.iter().map(|id| codigo_de(*id)).collect();
    let diagonal: Vec<Value> = ybus
        .indices
        .iter()
        .enumerate()
        .skip(if ref_idx == 0 { 1 } else { 0 })
        .filter(|(i, _)| *i != ref_idx)
        .map(|(i, _)| json!({"barra": barras_codigo[i], "z_re": z[i][i].re, "z_im": z[i][i].im, "modulo": z[i][i].modulo()}))
        .collect();
    Ok(json!({"barras": barras_codigo, "diagonal": diagonal, "referencia": codigo_de(ybus.indices[ref_idx])}))
}

fn run_curto_simultaneo(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.curto.simultaneo", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let sbase = f64_or(p, "sbase", 100.0);
    let barras = p.get("barras").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>()).unwrap_or_default();
    if barras.is_empty() {
        return Err(bad("anacad.curto.simultaneo", "informe as barras"));
    }
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.curto.simultaneo", format!("{db}: {e}")))?;
    let info = banco.barras(documento, 100_000).map_err(|e| bad("anacad.curto.simultaneo", e.to_string()))?;
    let circuitos = banco.circuitos(documento, 100_000).map_err(|e| bad("anacad.curto.simultaneo", e.to_string()))?;
    let por_codigo: BTreeMap<i64, i64> = info.iter().filter_map(|b| b.codigo.map(|c| (c, b.id))).collect();
    let mut ids: Vec<i64> = info.iter().map(|b| b.id).collect();
    ids.sort_unstable();
    ids.dedup();
    let componente: BTreeSet<i64> = ids.iter().copied().collect();
    let mut circuitos_calc = Vec::new();
    for c in &circuitos {
        let (Some(de), Some(para), Some(r1), Some(x1)) = (c.de, c.para, c.r1, c.x1) else { continue };
        let (Some(de), Some(para)) = (por_codigo.get(&de).copied(), por_codigo.get(&para).copied()) else { continue };
        if !componente.contains(&de) || !componente.contains(&para) {
            continue;
        }
        circuitos_calc.push(anacad_core::calc::CircuitoCalc {
            de,
            para,
            r1,
            x1,
            tap: c.tap.filter(|t| *t > 0.0).map(|t| t / 1000.0).unwrap_or(1.0),
            defasamento: c.defasamento.unwrap_or(0.0).to_radians(),
        });
    }
    let entrada = anacad_core::calc::EntradaYbus { barras: ids.clone(), circuitos: circuitos_calc };
    let shunts = shunts_do_banco(&banco, documento, &por_codigo)?;
    let mutuas = mutuas_do_banco(&banco, documento, &por_codigo, &componente)?;
    let ybus = anacad_core::calc::montar_ybus_com_shunts_e_mutuas(&entrada, &mutuas, &shunts);
    let ref_idx = match p.get("referencia").and_then(Value::as_i64) {
        Some(code) => por_codigo.get(&code).and_then(|id| ybus.indices.iter().position(|b| b == id)).unwrap_or(0),
        None => 0,
    };
    let faltas: Vec<usize> = barras.iter().filter_map(|code| por_codigo.get(code).and_then(|id| ybus.indices.iter().position(|b| b == id))).collect();
    if faltas.is_empty() {
        return Err(bad("anacad.curto.simultaneo", "barras nao encontradas"));
    }
    let correntes =
        anacad_core::calc::faltas_simultaneas_3f(&ybus, &faltas, ref_idx, 1e9).ok_or_else(|| bad("anacad.curto.simultaneo", "matriz singular"))?;
    let mut saida = Vec::new();
    for (k, idx) in faltas.iter().enumerate() {
        let id = ybus.indices[*idx];
        let codigo = info.iter().find(|b| b.id == id).and_then(|b| b.codigo).unwrap_or(0);
        let vbase = banco.barra_vbase(documento, codigo).map_err(|e| bad("anacad.curto.simultaneo", e.to_string()))?.unwrap_or(0.0);
        let i_base_a = if vbase > 0.0 { sbase * 1e6 / ((3.0f64).sqrt() * vbase * 1e3) } else { 0.0 };
        saida.push(json!({
            "barra": codigo,
            "i_pu": correntes[k].modulo(),
            "i_ka": correntes[k].modulo() * i_base_a / 1e3,
            "angulo_graus": correntes[k].im.atan2(correntes[k].re).to_degrees(),
        }));
    }
    Ok(json!({"correntes": saida}))
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

#[cfg(test)]
mod testes_editor {
    use super::*;

    #[test]
    fn editor_guarda_handle_da_polilinha() {
        let mut s = Session::new();
        let poly = LwPolyline {
            vertices: vec![
                PolyVertex { p: Vec2::new(0.0, 0.0), ..PolyVertex::default() },
                PolyVertex { p: Vec2::new(10.0, 0.0), ..PolyVertex::default() },
            ],
            closed: false,
            const_width: 0.3,
            elevation: 0.0,
            plinegen: false,
        };
        let h = s.add_entity(EntityKind::LwPolyline(poly)).unwrap();
        let mut m = EditorM::default();
        m.begin(&mut s).unwrap();
        let step = m.input(&mut s, Input::Pick(vec![h])).unwrap();
        assert_eq!(step, Step::Done);
        assert_eq!(s.anacad_edit_handle.as_deref(), Some(h.hex().as_str()));
    }
}
