//! XData da pagina no desenho.
//!
//! Modelo da origem (`ArqNet4/XData/XDataGeral.cs`): o titulo da pagina e um
//! `DBText` com o XData da aplicacao (`gravarXDataTitulo`) e a unidade/o
//! alternativo acompanham a camada (`GravarXDataUnidadeAlternativo`). O
//! cadcraft nao tem XData em camada, entao a unidade/alternativo vai num POINT
//! oculto da propria camada da pagina — mesmos quatro pares, outra entidade.
//!
//! O vinculo com o `.db` do projeto mora num POINT oculto na origem, como o
//! `ANACAD_CFG` do anacad e o `DirProjeto` do Eletron: trocar de arquivo troca o
//! banco, porque e o desenho que diz qual e o dele.

use std::collections::BTreeMap;

use cadcraft_doc::{EntityKind, HAlign, Handle, Point, RawTag, Text, VAlign};
use cadcraft_geom::Vec3;
use serde_json::{Value, json};

use super::{bad, str_param};
use crate::{Result, Session};

use arquitetura_data::Pagina;
use arquitetura_data::xdata::{
    self, APP, MARCADOR_PROJETO, MARCADOR_TITULO, MARCADOR_UNIDADE, Par, Tag, TituloPagina,
    UnidadeAlternativo,
};

/// Altura padrao do texto do titulo, em unidades do desenho.
const ALTURA_PADRAO: f64 = 2.5;
/// Espaco entre dois titulos, em alturas de texto.
const ENTRE_TITULOS: f64 = 1.5;

fn tags(xdata: &[RawTag]) -> Vec<Tag> {
    xdata.iter().map(|t| Tag { codigo: t.code, valor: t.value.clone() }).collect()
}

fn raws(pares: &[Tag]) -> Vec<RawTag> {
    pares.iter().map(|t| RawTag { code: t.codigo, value: t.valor.clone() }).collect()
}

/// Pares do bloco `marcador` no XData da entidade.
fn pares_do(entity: &cadcraft_doc::Entity, marcador: &str) -> Option<Vec<Par>> {
    xdata::le_bloco(&tags(&entity.common.xdata), marcador)
}

/// Campos `chave`/`valor` de um bloco, na ordem do XData (sem o nome da
/// aplicacao nem o marcador).
fn campos(pares: &[Par]) -> Vec<(String, String)> {
    let textos: Vec<String> =
        pares.iter().filter_map(|p| p.valor.texto().map(str::to_string)).skip(2).collect();
    textos
        .chunks(2)
        .filter(|par| par.len() == 2)
        .map(|par| (par[0].clone(), par[1].clone()))
        .collect()
}

/// O banco que o **desenho** aponta (`Projeto.banco`), vazio quando o desenho
/// ainda nao foi configurado.
pub fn banco_do_desenho(s: &Session) -> String {
    let Ok(doc) = s.doc() else { return String::new() };
    for entity in doc.model.iter() {
        let Some(pares) = pares_do(entity, MARCADOR_PROJETO) else { continue };
        let banco = campos(&pares)
            .into_iter()
            .find(|(chave, _)| chave.eq_ignore_ascii_case("banco"))
            .map(|(_, valor)| valor.trim().to_string())
            .unwrap_or_default();
        if !banco.is_empty() {
            return banco;
        }
    }
    String::new()
}

/// Entidade que guarda o vinculo do projeto (cria na origem quando falta).
fn ancora_projeto(s: &mut Session) -> Result<Handle> {
    let existente = s
        .doc()?
        .model
        .iter()
        .find(|e| pares_do(e, MARCADOR_PROJETO).is_some())
        .map(|e| e.handle);
    if let Some(handle) = existente {
        return Ok(handle);
    }
    let handle = s.add_entity(EntityKind::Point(Point { p: Vec3::ZERO, angle: 0.0 }))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        e.common.visible = false;
        e.common.layer = "0".to_string();
    })?;
    Ok(handle)
}

/// Entidade que guarda unidade/alternativo da pagina.
fn ancora_unidade(s: &Session, camada: &str) -> Option<Handle> {
    let doc = s.doc().ok()?;
    doc.model
        .iter()
        .filter(|e| e.common.layer.eq_ignore_ascii_case(camada))
        .find(|e| pares_do(e, MARCADOR_UNIDADE).is_some())
        .map(|e| e.handle)
}

/// `(tipo, handle)` dos TEXT com XData de titulo na camada.
fn textos_da_pagina(s: &Session, camada: &str) -> Vec<(u8, Handle)> {
    let Ok(doc) = s.doc() else { return Vec::new() };
    doc.model
        .iter()
        .filter(|e| e.common.layer.eq_ignore_ascii_case(camada))
        .filter_map(|e| {
            let pares = pares_do(e, MARCADOR_TITULO)?;
            let titulo = TituloPagina::de_pares(&pares).ok()?;
            Some((titulo.tipo, e.handle))
        })
        .collect()
}

/// Paginas do desenho, uma por camada que tenha texto de titulo ou unidade.
pub fn paginas_do_desenho(doc: &cadcraft_doc::Drawing) -> Vec<Pagina> {
    let mut por_camada: BTreeMap<String, Pagina> = BTreeMap::new();
    for entity in doc.model.iter() {
        let chave = entity.common.layer.to_lowercase();
        if let Some(pares) = pares_do(entity, MARCADOR_TITULO) {
            let Ok(titulo) = TituloPagina::de_pares(&pares) else { continue };
            let pagina =
                por_camada.entry(chave).or_insert_with(|| Pagina::nova(&entity.common.layer));
            match titulo.tipo {
                1 => pagina.titulo1 = titulo.valor,
                2 => pagina.titulo2 = titulo.valor,
                3 => pagina.titulo3 = titulo.valor,
                4 => pagina.titulo4 = titulo.valor,
                _ => {}
            }
        } else if let Some(pares) = pares_do(entity, MARCADOR_UNIDADE) {
            let Ok(unidade) = UnidadeAlternativo::de_pares(&pares) else { continue };
            let pagina =
                por_camada.entry(chave).or_insert_with(|| Pagina::nova(&entity.common.layer));
            pagina.unidade = unidade.unidade;
            pagina.alternativo = unidade.alternativo;
        }
    }
    por_camada.into_values().collect()
}

/// Fonte do banco: o parametro do comando ou, sem ele, o XData do desenho.
fn banco_do_comando(s: &Session, p: &Value) -> String {
    str_param(p, "banco")
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| banco_do_desenho(s))
}

pub(super) fn run_projeto_gravar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.projeto.gravar";
    let banco = str_param(p, "banco")
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| bad(cmd, "informe o arquivo do banco em `banco`"))?
        .to_string();
    let handle = ancora_projeto(s)?;
    let pares = vec![
        Tag { codigo: 1001, valor: APP.to_string() },
        Tag { codigo: 1000, valor: MARCADOR_PROJETO.to_string() },
        Tag { codigo: 1000, valor: "versao".to_string() },
        Tag { codigo: 1000, valor: "1".to_string() },
        Tag { codigo: 1000, valor: "banco".to_string() },
        Tag { codigo: 1000, valor: banco.clone() },
    ];
    s.doc_mut()?.modify_entity(handle, |e| e.common.xdata = raws(&pares))?;
    Ok(json!({ "banco": banco, "handle": handle.hex(), "xdata": APP }))
}

pub(super) fn run_projeto_ler(s: &mut Session, _p: &Value) -> Result<Value> {
    let banco = banco_do_desenho(s);
    Ok(json!({ "banco": banco.clone(), "encontrado": !banco.is_empty(), "xdata": APP }))
}

pub(super) fn run_pagina_escrever(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.pagina.escrever";
    let chave = str_param(p, "pagina")
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| bad(cmd, "informe a pagina (nome da camada) em `pagina`"))?
        .to_string();
    let altura =
        p.get("altura").and_then(Value::as_f64).unwrap_or(ALTURA_PADRAO).clamp(0.1, 100.0);
    let x = p.get("x").and_then(Value::as_f64).unwrap_or(0.0);
    let y = p.get("y").and_then(Value::as_f64).unwrap_or(0.0);
    let autor = str_param(p, "autor").unwrap_or_default().trim().to_string();
    let quando = str_param(p, "quando").unwrap_or_default().trim().to_string();
    let titulos = [(1u8, "titulo1"), (2, "titulo2"), (3, "titulo3"), (4, "titulo4")];

    s.doc_mut()?.ensure_layer(&chave);
    let existentes = textos_da_pagina(s, &chave);
    let (mut criados, mut atualizados, mut removidos) = (0u32, 0u32, 0u32);
    for (indice, (tipo, campo)) in titulos.iter().enumerate() {
        let valor = super::campo(p, campo);
        let alvo = existentes.iter().find(|(t, _)| t == tipo).map(|(_, h)| *h);
        if valor.is_empty() {
            if let Some(handle) = alvo {
                s.doc_mut()?.remove_entity(handle);
                removidos += 1;
            }
            continue;
        }
        let titulo =
            TituloPagina::novo(*tipo, valor.clone()).com_autor(autor.clone(), quando.clone());
        let bloco = raws(&xdata::de_pares(&titulo.para_pares()));
        let pos = Vec3::new(x, y - indice as f64 * altura * ENTRE_TITULOS, 0.0);
        match alvo {
            Some(handle) => {
                let camada = chave.clone();
                s.doc_mut()?.modify_entity(handle, |e| {
                    e.common.xdata = bloco;
                    e.common.layer = camada;
                    if let EntityKind::Text(texto) = &mut e.kind {
                        texto.value = valor;
                        texto.insert = pos;
                        texto.height = altura;
                    }
                })?;
                atualizados += 1;
            }
            None => {
                let texto = Text {
                    insert: pos,
                    align_pt: None,
                    height: altura,
                    value: valor,
                    rotation: 0.0,
                    width_factor: 1.0,
                    oblique: 0.0,
                    style: "Standard".to_string(),
                    halign: HAlign::Left,
                    valign: VAlign::Baseline,
                };
                let handle = s.add_entity(EntityKind::Text(texto))?;
                let camada = chave.clone();
                s.doc_mut()?.modify_entity(handle, |e| {
                    e.common.layer = camada;
                    e.common.xdata = bloco;
                })?;
                criados += 1;
            }
        }
    }

    let unidade = super::campo(p, "unidade");
    let alternativo = super::campo(p, "alternativo");
    let vazio = unidade.is_empty() && alternativo.is_empty();
    let bloco = raws(&xdata::de_pares(
        &UnidadeAlternativo { unidade: unidade.clone(), alternativo: alternativo.clone() }
            .para_pares(),
    ));
    match ancora_unidade(s, &chave) {
        Some(handle) if vazio => {
            s.doc_mut()?.remove_entity(handle);
        }
        Some(handle) => {
            s.doc_mut()?.modify_entity(handle, |e| e.common.xdata = bloco)?;
        }
        None if !vazio => {
            let handle = s.add_entity(EntityKind::Point(Point { p: Vec3::ZERO, angle: 0.0 }))?;
            let camada = chave.clone();
            s.doc_mut()?.modify_entity(handle, |e| {
                e.common.layer = camada;
                e.common.visible = false;
                e.common.xdata = bloco;
            })?;
        }
        None => {}
    }

    let banco = banco_do_comando(s, p);
    let mut gravado_no_banco = false;
    let mut banco_erro = String::new();
    if !banco.is_empty() {
        let mut pagina = Pagina::nova(&chave);
        pagina.titulo1 = super::campo(p, "titulo1");
        pagina.titulo2 = super::campo(p, "titulo2");
        pagina.titulo3 = super::campo(p, "titulo3");
        pagina.titulo4 = super::campo(p, "titulo4");
        pagina.unidade = unidade;
        pagina.alternativo = alternativo;
        // O desenho ja foi gravado: um  inacessivel vira aviso, nao erro.
        let mut erro = String::new();
        match arquitetura_data::Banco::abrir(&banco) {
            Ok(conexao) => match conexao.salvar_pagina(&pagina) {
                Ok(()) => gravado_no_banco = true,
                Err(e) => erro = e.to_string(),
            },
            Err(e) => erro = e.to_string(),
        }
        banco_erro = erro;
    }

    Ok(json!({
        "pagina": chave,
        "criados": criados,
        "atualizados": atualizados,
        "removidos": removidos,
        "banco": banco,
        "gravado_no_banco": gravado_no_banco,
        "banco_erro": banco_erro,
        "xdata": APP,
    }))
}

pub(super) fn run_pagina_ler(s: &mut Session, p: &Value) -> Result<Value> {
    let filtro = str_param(p, "pagina").map(str::trim).filter(|v| !v.is_empty());
    let mut paginas = paginas_do_desenho(s.doc()?);
    if let Some(chave) = filtro {
        paginas.retain(|pagina| pagina.pagina.eq_ignore_ascii_case(chave));
    }
    let lista: Vec<Value> = paginas.iter().map(super::pagina_para_json).collect();
    Ok(json!({ "total": lista.len(), "paginas": lista }))
}

pub(super) fn run_pagina_sincronizar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.pagina.sincronizar";
    let banco = banco_do_comando(s, p);
    if banco.is_empty() {
        return Err(bad(
            cmd,
            "o desenho nao tem banco: grave o vinculo com `arq.projeto.gravar`",
        ));
    }
    let paginas = {
        let doc = s.doc()?;
        paginas_do_desenho(doc)
    };
    let conexao = arquitetura_data::Banco::abrir(&banco).map_err(|e| bad(cmd, e.to_string()))?;
    for pagina in &paginas {
        conexao.salvar_pagina(pagina).map_err(|e| bad(cmd, e.to_string()))?;
    }
    let chaves: Vec<&str> = paginas.iter().map(|p| p.pagina.as_str()).collect();
    Ok(json!({ "banco": banco, "gravadas": paginas.len(), "paginas": chaves }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sessao() -> Session {
        Session::new()
    }

    #[test]
    fn escreve_e_le_a_pagina_no_desenho() -> Result<()> {
        let mut s = sessao();
        let parametros = json!({
            "pagina": "12A",
            "titulo1": "DIAGRAMA UNIFILAR",
            "titulo2": "SE-01",
            "unidade": "SE-01",
            "alternativo": "ALT",
        });
        let saida = run_pagina_escrever(&mut s, &parametros)?;
        assert_eq!(saida["criados"], 2);
        assert_eq!(saida["gravado_no_banco"], false);

        let ler = run_pagina_ler(&mut s, &json!({}))?;
        assert_eq!(ler["total"], 1);
        let pagina = &ler["paginas"][0];
        assert_eq!(pagina["pagina"], "12A");
        assert_eq!(pagina["titulo1"], "DIAGRAMA UNIFILAR");
        assert_eq!(pagina["titulo2"], "SE-01");
        assert_eq!(pagina["unidade"], "SE-01");
        assert_eq!(pagina["alternativo"], "ALT");
        Ok(())
    }

    #[test]
    fn escrever_de_novo_atualiza_em_vez_de_duplicar() -> Result<()> {
        let mut s = sessao();
        let mut parametros = json!({ "pagina": "12A", "titulo1": "A", "unidade": "U" });
        run_pagina_escrever(&mut s, &parametros)?;
        parametros["titulo1"] = json!("B");
        let saida = run_pagina_escrever(&mut s, &parametros)?;
        assert_eq!(saida["criados"], 0);
        assert_eq!(saida["atualizados"], 1);
        let texto = run_pagina_ler(&mut s, &json!({ "pagina": "12A" }))?;
        assert_eq!(texto["paginas"][0]["titulo1"], "B");
        assert_eq!(texto["total"], 1);
        Ok(())
    }

    #[test]
    fn titulo_vazio_apaga_o_texto() -> Result<()> {
        let mut s = sessao();
        run_pagina_escrever(&mut s, &json!({ "pagina": "3", "titulo1": "A", "titulo2": "B" }))?;
        let saida = run_pagina_escrever(&mut s, &json!({ "pagina": "3", "titulo1": "A" }))?;
        assert_eq!(saida["removidos"], 1);
        let ler = run_pagina_ler(&mut s, &json!({ "pagina": "3" }))?;
        assert_eq!(ler["paginas"][0]["titulo2"], "");
        Ok(())
    }

    #[test]
    fn xdata_no_desenho_segue_o_layout_do_dominio() -> Result<()> {
        let mut s = sessao();
        run_pagina_escrever(&mut s, &json!({ "pagina": "7", "titulo1": "T", "unidade": "U" }))?;
        let doc = s.doc()?;
        let texto = doc
            .model
            .iter()
            .find(|e| pares_do(e, MARCADOR_TITULO).is_some())
            .ok_or_else(|| bad("teste", "texto do titulo ausente"))?;
        assert_eq!(texto.common.layer, "7");
        assert_eq!(texto.common.xdata[0].code, 1001);
        assert_eq!(texto.common.xdata[0].value, APP);
        assert_eq!(texto.common.xdata[1].value, MARCADOR_TITULO);
        let ponto = doc
            .model
            .iter()
            .find(|e| pares_do(e, MARCADOR_UNIDADE).is_some())
            .ok_or_else(|| bad("teste", "ancora da unidade ausente"))?;
        assert!(!ponto.common.visible);
        assert_eq!(ponto.common.layer, "7");
        Ok(())
    }

    #[test]
    fn projeto_gravado_no_desenho_e_lido_de_volta() -> Result<()> {
        let mut s = sessao();
        assert!(banco_do_desenho(&s).is_empty());
        let saida = run_projeto_gravar(&mut s, &json!({ "banco": "C:/tmp/p.db" }))?;
        assert_eq!(saida["banco"], "C:/tmp/p.db");
        assert_eq!(banco_do_desenho(&s), "C:/tmp/p.db");
        assert_eq!(run_projeto_ler(&mut s, &json!({}))?["encontrado"], true);

        // Gravar de novo atualiza o mesmo ponto, sem criar outro.
        run_projeto_gravar(&mut s, &json!({ "banco": "C:/tmp/q.db" }))?;
        assert_eq!(banco_do_desenho(&s), "C:/tmp/q.db");
        let pontos = s
            .doc()?
            .model
            .iter()
            .filter(|e| pares_do(e, MARCADOR_PROJETO).is_some())
            .count();
        assert_eq!(pontos, 1);
        Ok(())
    }

    /// O teste da troca de arquivo: grava, salva em DXF, abre de novo e a pagina
    /// continua la — o XData viaja com o desenho.
    #[test]
    fn pagina_sobrevive_a_ida_e_volta_em_dxf() -> Result<()> {
        let mut s = sessao();
        let esperado = std::env::temp_dir().join("m4b_prova.db").to_string_lossy().to_string();
        run_projeto_gravar(&mut s, &json!({ "banco": esperado }))?;
        run_pagina_escrever(
            &mut s,
            &json!({ "pagina": "12A", "titulo1": "DIAGRAMA", "unidade": "SE-01" }),
        )?;
        let dxf = cadcraft_io::write_dxf(s.doc()?);
        let reaberto = cadcraft_io::read_dxf(dxf.as_bytes()).map_err(|e| bad("teste", e.to_string()))?;

        let paginas = paginas_do_desenho(&reaberto);
        assert_eq!(paginas.len(), 1);
        assert_eq!(paginas[0].pagina, "12A");
        assert_eq!(paginas[0].titulo1, "DIAGRAMA");
        assert_eq!(paginas[0].unidade, "SE-01");

        let banco = reaberto
            .model
            .iter()
            .find_map(|e| {
                let pares = pares_do(e, MARCADOR_PROJETO)?;
                campos(&pares).into_iter().find(|(k, _)| k == "banco").map(|(_, v)| v)
            })
            .ok_or_else(|| bad("teste", "vinculo do projeto nao sobreviveu ao DXF"))?;
        assert_eq!(banco, esperado);
        let _ = std::fs::remove_file(&esperado);
        Ok(())
    }
}
