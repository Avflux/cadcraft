//! Comandos do dominio anacad (M1/M2: diagnostico, bloco de teste e importacao).
//!
//! Este arquivo e parte do patch minimo descrito em
//! `anacad/docs/cadcraft-integracao.md`: o cadcraft continua sendo o upstream e
//! o patch e reaplicado por `anacad/scripts/cadcraft-patch.ps1`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use cadcraft_doc::{Attrib, Block, Circle, Entity, EntityKind, HAlign, Handle, Line, LwPolyline, RawTag, Text, VAlign};
use cadcraft_geom::{PolyVertex, Vec2, Vec3};
use serde_json::{Value, json};

use super::machines::{SelOutcome, SelectPhase};
use super::*;
use crate::{Accept, Input, Interactive, Prompt, Result, Session, Step};

mod ana;
mod analise;
mod esquematico;

use ana::*;
use analise::*;
use esquematico::*;

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
        CommandSpec::new("anacad.curto", "AnaCAD: Curto-Circuito", run_curto)
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
        CommandSpec::new("anacad.barra.dados.excluir", "AnaCAD: Excluir Dados Elétricos da Barra", run_barra_dados_excluir)
            .menu(&[])
            .params("{db, documento?, handle, codigo?} -> {barra, handle, removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.dados.excluir", "AnaCAD: Excluir Dados Elétricos da Linha", run_linha_dados_excluir)
            .menu(&[])
            .params("{db, documento?, handle} -> {linha, handle, removidos}")
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
            .enabled(equipamento_escolhido)
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
        CommandSpec::new("anacad.equipamento.dados.excluir", "AnaCAD: Excluir Dados Elétricos do Equipamento", run_equipamento_dados_excluir)
            .menu(&[])
            .params("{db, documento?, handle} -> {equipamento, handle, removidos}")
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
            .enabled(ponto_escolhido)
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
        CommandSpec::new("anacad.evolucao.niveis", "AnaCAD: Evolução de Níveis", run_evolucao_niveis)
            .menu(&[])
            .params("{db, documento?, comparar, sbase?} -> {barras:[{barra,i3f_base_ka,i3f_novo_ka,delta_ka,delta_pct}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.macro", "AnaCAD: Estudo Macro (barras × contingências)", run_macro)
            .menu(&[])
            .params("{db, documento?, sbase?, barras?, casos:[{nome?, abrir?:[[de,para]]}]} -> {casos:[{nome, abrir, niveis}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.superacao", "AnaCAD: Superação de Disjuntores", run_superacao)
            .menu(&[])
            .params("{db, documento?, sbase?} -> {barras:[{barra,i3f_ka,capacidade_ka,margem,supera}], superam, sem_capacidade}")
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
        CommandSpec::new("anacad.impedancia.barras", "AnaCAD: Impedância entre Barras", run_impedancia_barras)
            .menu(&[])
            .params("{db, documento?, de, para} -> {de, para, z_re, z_im, modulo, angulo_graus}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.curto.simultaneo", "AnaCAD: Faltas Simultâneas", run_curto_simultaneo)
            .menu(&[])
            .params("{db, documento?, barras:[codigos], referencia?} -> {correntes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana.exportar", "AnaCAD: Salvar .ANA", run_ana_exportar)
            .menu(&[])
            .params("{db, documento?, arquivo} -> {arquivo, bytes, barras, circuitos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.bloco", "AnaCAD: Definir Bloco do Equipamento", run_equipamento_bloco)
            .menu(&[])
            .params("{db, tipo, bloco?} -> {tipo, slug, bloco}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.teste.bloco", "AnaCAD: Bloco de Teste", run_teste_bloco)
            .menu(&[])
            .params("{nome?, inserir?, semente?} -> {bloco, handle, entidades}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.criar", "AnaCAD: Conectar Terminais", run_conexao_criar)
            .menu(&[])
            .params("{db, documento?, de_handle, de_terminal?, para_handle, para_terminal?, potencial?, desenhar?} -> {conexao, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.validar", "AnaCAD: Validar Conexões", run_conexao_validar)
            .menu(&[])
            .params("{db, documento?, tolerancia?} -> {conexoes:[{id,ok,...}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.topologia", "AnaCAD: Topologia das Conexões", run_conexao_topologia)
            .menu(&[])
            .params("{db, documento?, tolerancia?, apenas_validas?} -> {componentes, nos, arestas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.mover", "AnaCAD: Mover Equipamento", run_equipamento_mover)
            .menu(&[])
            .params("{db, handle, dx, dy} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.transformar", "AnaCAD: Rotacionar/Escalar Equipamento", run_equipamento_transformar)
            .menu(&[])
            .params("{db, handle, rotacao?, escala?} -> {handle, rotacao, escala}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.sincronizar", "AnaCAD: Sincronizar Equipamentos", run_equipamento_sincronizar)
            .menu(&[])
            .params("{db, documento?} -> {atualizados, ausentes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.comparar", "AnaCAD: Comparar Topologia CAD × Banco", run_conexao_comparar)
            .menu(&[])
            .params("{db, documento?} -> {cad, banco, faltando_em_cad, extras_cad}")
            .enabled(always)
            .noundo(),
        CommandSpec::new(
            "anacad.equipamento.sincronizar_conexoes",
            "AnaCAD: Sincronizar De/Para pelas Conexões",
            run_equipamento_sincronizar_conexoes,
        )
        .menu(&[])
        .params("{db, documento?} -> {atualizados, ignoradas}")
        .enabled(always)
        .noundo(),
        CommandSpec::new("anacad.exportar", "AnaCAD: Exportar .lst", run_exportar)
            .menu(&[])
            .params("{db, documento, saida} -> {saida, registros, mensagem}")
            .enabled(always)
            .noundo(),
    ]
}

/// Grava a menor capacidade de interrupção por barra (kA) nos dados da barra:
/// mínimo entre `DJ_BF`/`DJ_BT` dos circuitos que tocam a barra, com o `DISJUN`
/// do `DBAR` como alternativa (manual §4.3.1). Usada pelo estudo de superação.
fn gravar_capacidades_das_barras(
    banco: &anacad_data::Banco,
    documento: i64,
    barras: &[anacad_core::ana::BarraAna],
    circuitos: &[anacad_core::ana::CircuitoAna],
) -> usize {
    use std::collections::BTreeMap;
    let mut minimos: BTreeMap<i64, f64> = BTreeMap::new();
    for circuito in circuitos {
        for (barra, capacidade) in [(circuito.de, circuito.dj_bf), (circuito.para, circuito.dj_bt)] {
            if let Some(valor) = capacidade.filter(|v| *v > 0.0) {
                minimos
                    .entry(barra)
                    .and_modify(|atual| {
                        if valor < *atual {
                            *atual = valor;
                        }
                    })
                    .or_insert(valor);
            }
        }
    }
    for barra in barras {
        if let Some(valor) = barra.disjun.filter(|v| *v > 0.0) {
            minimos
                .entry(barra.codigo)
                .and_modify(|atual| {
                    if valor < *atual {
                        *atual = valor;
                    }
                })
                .or_insert(valor);
        }
    }
    let mut dados: std::collections::BTreeMap<i64, serde_json::Map<String, serde_json::Value>> = std::collections::BTreeMap::new();
    for barra in barras {
        let mut campos = serde_json::Map::new();
        if let Some(modulo) = barra.vpre {
            campos.insert("preFaltaModulo".into(), json!(modulo));
        }
        if let Some(angulo) = barra.ang {
            campos.insert("preFaltaAngulo".into(), json!(angulo));
        }
        if !campos.is_empty() {
            dados.entry(barra.codigo).or_default().extend(campos);
        }
    }
    for (codigo, capacidade) in minimos {
        dados.entry(codigo).or_default().insert("capacidadeInterrupcao".into(), json!(capacidade));
    }
    let mut gravadas = 0;
    for (codigo, campos) in dados {
        let texto = serde_json::Value::Object(campos).to_string();
        if banco.atualizar_dados_barra_codigo(documento, codigo, &texto).unwrap_or(0) > 0 {
            gravadas += 1;
        }
    }
    gravadas
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

/// Rede de cálculo do documento, montada no `anacad-core` (igual para todos
/// os comandos): terra como referência, `abrir` e `referencia` dos parâmetros.
fn rede_do_banco(banco: &anacad_data::Banco, documento: i64, p: &Value, cmd: &str) -> Result<anacad_core::rede::Rede> {
    let caso = banco.caso(documento).map_err(|e| bad(cmd, e.to_string()))?;
    Ok(caso.rede(&anacad_core::rede::Opcoes { abrir: ramos_abertos(p), referencia: p.get("referencia").and_then(Value::as_i64) }))
}

/// kA por pu de corrente na barra `codigo` (0 sem `VBase`).
fn ka_por_pu(banco: &anacad_data::Banco, documento: i64, codigo: i64, sbase: f64, cmd: &str) -> Result<f64> {
    let vbase = banco.barra_vbase(documento, codigo).map_err(|e| bad(cmd, e.to_string()))?.unwrap_or(0.0);
    Ok(if vbase > 0.0 { sbase / (3.0f64.sqrt() * vbase) } else { 0.0 })
}

fn fatorar(y: &anacad_core::calc::Ybus, cmd: &str) -> Result<anacad_core::calc::FatoracaoEsparsa> {
    anacad_core::calc::fatorar_esparsa(&anacad_core::calc::esparsa_de_densa(y)).ok_or_else(|| bad(cmd, "matriz singular"))
}

fn polar(z: anacad_core::calc::Complexo) -> Value {
    json!({"modulo": z.modulo(), "angulo": z.im.atan2(z.re).to_degrees()})
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

fn push_attdef(doc: &mut cadcraft_doc::Drawing, bloco: &mut Block, tag: &str, prompt: &str, default: &str, at: (f64, f64), height: f64) {
    let h = doc.new_handle();
    bloco.entities.push(Entity::new(
        h,
        EntityKind::AttDef(Attrib {
            tag: tag.to_ascii_uppercase(),
            text: Text {
                insert: Vec3::new(at.0, at.1, 0.0),
                align_pt: None,
                height,
                value: default.to_string(),
                rotation: 0.0,
                width_factor: 1.0,
                oblique: 0.0,
                style: "Standard".to_string(),
                halign: HAlign::Left,
                valign: VAlign::Baseline,
            },
            invisible: false,
            constant: false,
            prompt: prompt.to_string(),
            props: Default::default(),
            multiline: None,
            modes: Default::default(),
        }),
    ));
}

fn push_linha(doc: &mut cadcraft_doc::Drawing, bloco: &mut Block, a: (f64, f64), b: (f64, f64)) {
    let h = doc.new_handle();
    bloco.entities.push(Entity::new(h, EntityKind::Line(Line { a: Vec3::new(a.0, a.1, 0.0), b: Vec3::new(b.0, b.1, 0.0) })));
}

fn push_circulo(doc: &mut cadcraft_doc::Drawing, bloco: &mut Block, c: (f64, f64), r: f64) {
    let h = doc.new_handle();
    bloco.entities.push(Entity::new(h, EntityKind::Circle(Circle { center: Vec3::new(c.0, c.1, 0.0), radius: r })));
}

fn chave_bloco_equipamento(slug: &str) -> String {
    format!("equipamento_bloco.{slug}")
}

fn bloco_personalizado(s: &Session, banco: &anacad_data::Banco, slug: &str) -> Option<String> {
    let nome = banco.ler_config(&chave_bloco_equipamento(slug)).ok().flatten()?;
    if nome.trim().is_empty() {
        return None;
    }
    let existe = s.doc().ok()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(nome.trim()));
    existe.then(|| nome.trim().to_string())
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
    push_attdef(doc, &mut bloco, "CODIGO", "Codigo", "", (-14.0, -18.0), 2.0);
    push_attdef(doc, &mut bloco, "NOME", "Nome", "", (-4.0, -18.0), 2.0);
    push_attdef(doc, &mut bloco, "TENSAO", "Tensao", "", (8.0, -18.0), 2.0);
    push_attdef(doc, &mut bloco, "T1", "Terminal De", "DE", (-14.0, 14.0), 2.0);
    push_attdef(doc, &mut bloco, "T2", "Terminal Para", "PARA", (8.0, 14.0), 2.0);
    doc.blocks.insert(nome.clone(), Arc::new(bloco));
    Ok(nome)
}

fn equipamento_json(id: i64, tipo: &str, codigo: i64, handle: &str, x: f64, y: f64, bloco: &str, atributos: &[Value]) -> Value {
    json!({"equipamento": id, "tipo": tipo, "codigo": codigo, "handle": handle, "x": x, "y": y, "bloco": bloco, "atributos": atributos})
}

/// `anacad.equipamento.inserir` só começa com o tipo escolhido (o comando é
/// interativo e não tem o que inserir sem ele); sem tipo fica desabilitado.
fn equipamento_escolhido(s: &Session) -> std::result::Result<(), String> {
    if s.anacad_equip_tipo.is_empty() { Err("escolha um equipamento".into()) } else { Ok(()) }
}

/// `anacad.ponto.inserir` só começa com o tipo de ponto escolhido.
fn ponto_escolhido(s: &Session) -> std::result::Result<(), String> {
    if s.anacad_equip_tipo.is_empty() { Err("escolha o tipo de ponto".into()) } else { Ok(()) }
}

fn tipo_ponto(tipo: &str) -> Option<(i64, &'static str)> {
    match tipo {
        "terminal" => Some((20, "Terminal")),
        "monitoracao" => Some((21, "Ponto de Monitoração")),
        "quebra" => Some((29, "Ponto de Quebra")),
        _ => None,
    }
}

/// Níveis de curto por barra de um documento (base do relatório `RE1NCC`),
/// reaproveitado pelo comando de níveis e pelo estudo 4.6 (evolução).
fn niveis_do_documento(banco: &anacad_data::Banco, documento: i64, sbase: f64, cmd: &str, p: &Value) -> Result<(Vec<Value>, i64, Vec<i64>)> {
    let rede = rede_do_banco(banco, documento, p, cmd)?;
    if rede.y1.indices.is_empty() {
        return Ok((Vec::new(), rede.codigo_referencia(), rede.desenergizadas.clone()));
    }
    let fat1 = fatorar(&rede.y1, cmd)?;
    let fat0 = anacad_core::calc::fatorar_esparsa(&anacad_core::calc::esparsa_de_densa(&rede.y0));
    let mut niveis = Vec::new();
    for k in 0..rede.y1.indices.len() {
        if rede.ficticia(k) || rede.indice_referencia() == Some(k) {
            continue;
        }
        let (Some(codigo), Some(z1)) = (rede.codigo(k), fat1.zth(k)) else {
            continue;
        };
        let z0 = fat0.as_ref().and_then(|f| f.zth(k));
        let Some(n) = anacad_core::calc::nivel_curto(z1, z0) else {
            continue;
        };
        let ka = ka_por_pu(banco, documento, codigo, sbase, cmd)?;
        let prefalta = banco
            .dados_barra(documento, codigo)
            .ok()
            .flatten()
            .and_then(|texto| serde_json::from_str::<Value>(&texto).ok())
            .and_then(|d| d.get("preFaltaModulo").and_then(Value::as_f64))
            .filter(|v| *v > 0.0)
            .unwrap_or(1.0);
        // Pré-falta do formato `A`: corrente ∝ V e MVA ∝ V².
        let v2 = prefalta * prefalta;
        niveis.push(json!({
            "barra": codigo,
            "prefalta": prefalta,
            "i3f_ka": n.i3f * ka * prefalta,
            "i2f_ka": n.i2f * ka * prefalta,
            "i2ft_ka": n.i2ft.map(|v| v * ka * prefalta),
            "i1f_ka": n.i1f.map(|v| v * ka * prefalta),
            "i3f_mva": n.i3f * sbase * v2,
            "i1f_mva": n.i1f.map(|v| v * sbase * v2),
            "xr": n.xr1,
            "vbase": banco.barra_vbase(documento, codigo).map_err(|e| bad(cmd, e.to_string()))?.unwrap_or(0.0),
        }));
    }
    Ok((niveis, rede.codigo_referencia(), rede.desenergizadas.clone()))
}

fn ana_num(v: Option<f64>, dec: usize) -> String {
    match v {
        Some(v) if v.is_finite() => format!("{v:.dec$}"),
        _ => String::new(),
    }
}

fn ana_coloca(linha: &mut [char], inicio: usize, texto: &str) {
    for (i, ch) in texto.chars().enumerate() {
        if inicio + i < linha.len() {
            linha[inicio + i] = ch;
        }
    }
}

fn ana_linha(campos: &[(usize, String)]) -> String {
    let mut linha = vec![' '; 96];
    for (inicio, texto) in campos {
        ana_coloca(&mut linha, *inicio, texto);
    }
    linha.into_iter().collect::<String>().trim_end().to_string()
}

fn terminal_da_entidade(s: &Session, handle: Handle, terminal: Option<&str>) -> Result<(String, String, f64, f64)> {
    let ent = s.doc()?.entity(handle).ok_or_else(|| bad("anacad.conexao.criar", "entidade nao encontrada"))?;
    match &ent.kind {
        EntityKind::Insert(ins) => {
            let tag = terminal.unwrap_or("T1").to_ascii_uppercase();
            let attr = ins
                .attribs
                .iter()
                .find(|a| a.tag.eq_ignore_ascii_case(&tag))
                .ok_or_else(|| bad("anacad.conexao.criar", format!("terminal {tag} nao encontrado")))?;
            Ok(("Equipamento".to_string(), tag, attr.text.insert.x, attr.text.insert.y))
        }
        EntityKind::LwPolyline(poly) => {
            let tag = terminal.unwrap_or("INICIO").to_ascii_uppercase();
            let p = if tag == "FIM" || tag == "T2" {
                poly.vertices.last().map(|v| v.p).unwrap_or(Vec2::ZERO)
            } else {
                poly.vertices.first().map(|v| v.p).unwrap_or(Vec2::ZERO)
            };
            Ok(("Linha".to_string(), tag, p.x, p.y))
        }
        _ => Err(bad("anacad.conexao.criar", "entidade nao suportada (use bloco ou polilinha)")),
    }
}

fn distancia_ponto_linha(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let den = dx * dx + dy * dy;
    if den < 1e-18 {
        return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / den).clamp(0.0, 1.0);
    let qx = a.0 + t * dx;
    let qy = a.1 + t * dy;
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}

fn endpoints_da_linha(s: &Session, handle: Handle) -> Option<((f64, f64), (f64, f64))> {
    let ent = s.doc().ok()?.entity(handle)?;
    let EntityKind::LwPolyline(poly) = &ent.kind else { return None };
    let a = poly.vertices.first()?.p;
    let b = poly.vertices.last()?.p;
    Some(((a.x, a.y), (b.x, b.y)))
}

fn codigo_da_barra(s: &Session, handle: Handle) -> Option<i64> {
    let ent = s.doc().ok()?.entity(handle)?;
    let tags: Vec<&str> = ent.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
    if !ent.common.xdata.iter().any(|t| t.code == 1001 && t.value == "ANACAD_BARRA") {
        return None;
    }
    tags.windows(2).find(|w| w[0] == "codigo").and_then(|w| w[1].parse::<i64>().ok())
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
