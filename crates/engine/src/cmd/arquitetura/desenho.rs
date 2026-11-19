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
use cadcraft_doc::entity_bounds;
use cadcraft_geom::Vec3;
use serde_json::{Value, json};

use super::{bad, str_param};
use crate::{Result, Session};

use arquitetura_data::Pagina;
use arquitetura_data::xdata::{
    APP_EXPORTADO, Conexao, Exportado, Importado, RevisaoAutomatica, RevisaoCompleta, TextoAuxiliarCircuito, TSV_CABECALHO_EXPORTADO, TIPO_CIRCUITO, TIPO_REVISAO_SIMPLIFICADA, agrupar,
    self, APP, MARCADOR_PROJETO, MARCADOR_REVISAO, MARCADOR_REVISAO_AUTOMATICA, MARCADOR_TITULO, MARCADOR_UNIDADE, Par, Tag,
    TituloPagina,
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


/// Tipo (1..=4) do titulo em XData desta entidade, quando houver
/// (`LeXDataTitulo` da origem: so o marcador `TituloPagina` e o tipo
/// decidem; aqui via `TituloPagina::tipo_de` do `{arquitetura}_data`).
fn tipo_do_titulo(entidade: &cadcraft_doc::Entity) -> Option<u8> {
    pares_do(entidade, MARCADOR_TITULO).and_then(|pares| TituloPagina::tipo_de(&pares))
}

/// Entidades com XData ocupado (modulo atual ou de outra aplicacao do
/// legado): o `LerXDataVazio` do original e o contrario disto — um TEXT
/// com XData nao e texto livre.
fn tem_xdata_ocupado(entidade: &cadcraft_doc::Entity) -> bool {
    const APPS: &[&str] = &[
        APP,
        "Eletron",
        "DiagLog",
        "TextoLP",
        "AUXCONEXAO",
        "AUXINTERLIG",
        "AUXIMPORTADO",
        "DEPARA",
        "Definicao",
    ];
    entidade.common.xdata.iter().any(|r| {
        r.code == 1001 && APPS.iter().any(|a| r.value.eq_ignore_ascii_case(a))
    })
}


/// Revisoes simplificadas do desenho: uma por pagina (camada), com o texto
/// visivel do `Text` de titulo tipo 8 (`TIPO_REVISAO_SIMPLIFICADA`), como o
/// `Revisao.capturaRevisoesSimplificadas` do original. Paginas sem texto de
/// revisao entram com valor vazio (o `mRevSimplificada` nasce zerado).
pub fn revisoes_do_desenho(doc: &cadcraft_doc::Drawing) -> Vec<(String, String)> {
    let mut por_camada: BTreeMap<String, (String, String)> = BTreeMap::new();
    for pagina in paginas_do_desenho(doc) {
        por_camada.entry(pagina.pagina.to_lowercase()).or_insert_with(|| (pagina.pagina.clone(), String::new()));
    }
    for entidade in doc.model.iter() {
        if !matches!(&entidade.kind, EntityKind::Text(_)) {
            continue;
        }
        if tipo_do_titulo(entidade) != Some(TIPO_REVISAO_SIMPLIFICADA) {
            continue;
        }
        let valor = match &entidade.kind {
            EntityKind::Text(texto) => texto.value.clone(),
            _ => String::new(),
        };
        let chave = entidade.common.layer.to_lowercase();
        por_camada
            .entry(chave)
            .and_modify(|(_, v)| *v = valor.clone())
            .or_insert_with(|| (entidade.common.layer.clone(), valor));
    }
    por_camada.into_values().collect()
}

pub(super) fn run_revisao_ler(s: &mut Session, _p: &Value) -> Result<Value> {
    let revisoes: Vec<Value> = revisoes_do_desenho(s.doc()?)
        .iter()
        .map(|(pagina, revisao)| json!({ "pagina": pagina, "revisao": revisao }))
        .collect();
    Ok(json!({ "revisoes": revisoes, "total": revisoes.len(), "xdata": APP }))
}

pub(super) fn run_revisao_gravar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.revisao.gravar";
    let lista = p.get("revisoes").and_then(Value::as_array).ok_or_else(|| {
        bad(cmd, "informe as revisoes em `revisoes` (lista de {pagina, revisao})")
    })?;
    let mut atualizados = 0usize;
    let mut ignorados = 0usize;
    for item in lista {
        let pagina = item.get("pagina").and_then(Value::as_str).unwrap_or("").trim().to_string();
        if pagina.is_empty() {
            ignorados += 1;
            continue;
        }
        let revisao = item.get("revisao").and_then(Value::as_str).unwrap_or("").to_string();
        let alvo = textos_da_pagina(s, &pagina)
            .iter()
            .find(|(tipo, _)| *tipo == TIPO_REVISAO_SIMPLIFICADA)
            .map(|(_, h)| *h);
        match alvo {
            Some(handle) => {
                let bloco = raws(&xdata::de_pares(&TituloPagina::novo(TIPO_REVISAO_SIMPLIFICADA, revisao.clone()).para_pares()));
                s.doc_mut()?.modify_entity(handle, |e| {
                    e.common.xdata = bloco;
                    if let EntityKind::Text(texto) = &mut e.kind {
                        texto.value = revisao.clone();
                    }
                })?;
                atualizados += 1;
            }
            None => ignorados += 1,
        }
    }
    Ok(json!({ "atualizados": atualizados, "ignorados": ignorados, "total": lista.len(), "xdata": APP }))
}


/// Revisoes completas do desenho: uma por pagina (camada), a do `Insert` de
/// maior posicao com XData `REVISAO` (`Revisao.capturaRevisoesCompletas`).
/// Pagina sem bloco de revisao entra com posicao 0 e campos vazios.
pub fn revisoes_completas_do_desenho(
    doc: &cadcraft_doc::Drawing,
) -> Vec<(String, RevisaoCompleta)> {
    let mut por_camada: BTreeMap<String, (String, RevisaoCompleta)> = BTreeMap::new();
    for pagina in paginas_do_desenho(doc) {
        por_camada
            .entry(pagina.pagina.to_lowercase())
            .or_insert_with(|| (pagina.pagina.clone(), RevisaoCompleta::novo(0)));
    }
    for entidade in doc.model.iter() {
        if !matches!(&entidade.kind, EntityKind::Insert(_)) {
            continue;
        }
        let Some(pares) = pares_do(entidade, MARCADOR_REVISAO) else { continue };
        let Ok(revisao) = RevisaoCompleta::de_pares(&pares) else { continue };
        let chave = entidade.common.layer.to_lowercase();
        por_camada
            .entry(chave)
            .and_modify(|(_, atual)| {
                if revisao.posicao > atual.posicao {
                    *atual = revisao.clone();
                }
            })
            .or_insert_with(|| (entidade.common.layer.clone(), revisao));
    }
    por_camada.into_values().collect()
}

pub(super) fn run_revisao_completa_ler(s: &mut Session, _p: &Value) -> Result<Value> {
    let revisoes: Vec<Value> = revisoes_completas_do_desenho(s.doc()?)
        .iter()
        .map(|(pagina, r)| {
            json!({
                "pagina": pagina,
                "posicao": r.posicao,
                "revisao": r.revisao,
                "descricao": r.descricao,
                "data": r.data,
                "elaborador": r.elaborador,
                "verificador": r.verificador,
                "aprovador": r.aprovador,
            })
        })
        .collect();
    Ok(json!({ "revisoes": revisoes, "total": revisoes.len(), "xdata": APP }))
}

pub(super) fn run_revisao_completa_gravar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.revisao_completa.gravar";
    let lista = p.get("revisoes").and_then(Value::as_array).ok_or_else(|| {
        bad(cmd, "informe as revisoes em `revisoes` (lista com pagina e os campos)")
    })?;
    let mut atualizados = 0usize;
    let mut ignorados = 0usize;
    for item in lista {
        let pagina = item.get("pagina").and_then(Value::as_str).unwrap_or("").trim().to_string();
        if pagina.is_empty() {
            ignorados += 1;
            continue;
        }
        let posicao_pedida = item.get("posicao").and_then(Value::as_i64);
        let alvo = {
            let doc = s.doc()?;
            doc.model
                .iter()
                .filter(|e| e.common.layer.eq_ignore_ascii_case(&pagina))
                .filter(|e| matches!(&e.kind, EntityKind::Insert(_)))
                .filter_map(|e| {
                    let pares = pares_do(e, MARCADOR_REVISAO)?;
                    let revisao = RevisaoCompleta::de_pares(&pares).ok()?;
                    Some((e.handle, revisao.posicao))
                })
                .max_by_key(|(_, posicao)| *posicao)
                .filter(|(_, posicao)| posicao_pedida.is_none_or(|pedida| *posicao as i64 == pedida))
                .map(|(handle, _)| handle)
        };
        match alvo {
            Some(handle) => {
                let doc = s.doc()?;
                let atual = doc
                    .model
                    .iter()
                    .find(|e| e.handle == handle)
                    .and_then(|e| pares_do(e, MARCADOR_REVISAO))
                    .and_then(|pares| RevisaoCompleta::de_pares(&pares).ok())
                    .unwrap_or_else(|| RevisaoCompleta::novo(0));
                let mut nova = atual.clone();
                nova.revisao = item.get("revisao").and_then(Value::as_str).unwrap_or("").to_string();
                nova.descricao = item.get("descricao").and_then(Value::as_str).unwrap_or("").to_string();
                nova.data = item.get("data").and_then(Value::as_str).unwrap_or("").to_string();
                nova.elaborador = item.get("elaborador").and_then(Value::as_str).unwrap_or("").to_string();
                nova.verificador = item.get("verificador").and_then(Value::as_str).unwrap_or("").to_string();
                nova.aprovador = item.get("aprovador").and_then(Value::as_str).unwrap_or("").to_string();
                let bloco = raws(&xdata::de_pares(&nova.para_pares()));
                s.doc_mut()?.modify_entity(handle, |e| e.common.xdata = bloco)?;
                atualizados += 1;
            }
            None => ignorados += 1,
        }
    }
    Ok(json!({ "atualizados": atualizados, "ignorados": ignorados, "total": lista.len(), "xdata": APP }))
}


/// Apps de XData conhecidos do legado (`DTXp66plP1w` do
/// `frmPesquisaXdatas`): o filtro padrao da janela esconde estes.
const XDATAS_CONHECIDOS: &[&str] = &[
    "ACAD", "ARANHA", "AUXBORNES", "AUXCONEXAO", "AUXDERIVACAO", "AUXINTERLIG",
    "CONEXAO", "DEFINICAO", "DEPARA", "DIAGLOG", "DISPOSITIVO", "DISP_FUNCAO",
    "DISTRIBUICAODEPOTENCIAIS", "ELETROSUL", "ELETRON", "EXPORTADO", "INTERLIGACAO",
    "LINHATEXTOLP", "MODELO", "TEXTOLP", "AUXIMPORTADO", "IMPORTADO", "LAYOUT",
    "MATERIAIS", "PLAQUETAS", "AUXEXPORTADO", "PADRAOINTERLIGACAOBORNE",
    "PADRAOINTERLIGACAOCABO", "PADRAOINTERLIGACAOCABOLINHA", "FIACAODWG", "DINTERLIG",
    "TOPOGRAFICO", "ARQUITETURA", "ELETRICO", "LOGICO", "ARQNET", "ELETRON4Z",
    "DIAGLOG4Z", "ARQNET4Z",
];

/// Apps de XData do desenho com a quantidade de entidades
/// (`clsApagaXdatas.BuscaxDatasNaoEletron`): o nome (em maiusculas) e o
/// valor do par 1001 de cada entidade com XData.
pub fn xdatas_do_desenho(doc: &cadcraft_doc::Drawing) -> Vec<(String, usize)> {
    let mut contagem: BTreeMap<String, usize> = BTreeMap::new();
    for entidade in doc.model.iter() {
        for tag in &entidade.common.xdata {
            if tag.code != 1001 {
                continue;
            }
            let nome = tag.value.trim().to_uppercase();
            if !nome.is_empty() {
                *contagem.entry(nome).or_default() += 1;
            }
            break;
        }
    }
    contagem.into_iter().collect()
}

pub(super) fn run_xdata_listar(s: &mut Session, p: &Value) -> Result<Value> {
    let incluir = p.get("incluir_conhecidos").and_then(Value::as_bool).unwrap_or(false);
    let xdatas: Vec<Value> = xdatas_do_desenho(s.doc()?)
        .into_iter()
        .filter(|(nome, _)| incluir || !XDATAS_CONHECIDOS.iter().any(|c| c.eq_ignore_ascii_case(nome)))
        .map(|(app, quantidade)| json!({ "app": app, "quantidade": quantidade }))
        .collect();
    Ok(json!({ "xdatas": xdatas, "total": xdatas.len(), "xdata": APP }))
}

pub(super) fn run_xdata_apagar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.xdata.apagar";
    let apps_val = p.get("apps").and_then(Value::as_array).ok_or_else(|| {
        bad(cmd, "informe os apps em `apps` (lista de nomes de XData)")
    })?;
    let apps: Vec<String> = apps_val
        .iter()
        .filter_map(Value::as_str)
        .map(|v| v.trim().to_uppercase())
        .filter(|v| !v.is_empty())
        .collect();
    if apps.is_empty() {
        return Err(bad(cmd, "selecione ao menos um XData para apagar"));
    }
    let mut apagados = 0usize;
    {
        let doc = s.doc()?;
        let alvos: Vec<Handle> = doc
            .model
            .iter()
            .filter(|e| {
                e.common.xdata.iter().any(|t| {
                    t.code == 1001 && apps.iter().any(|a| a.eq_ignore_ascii_case(&t.value))
                })
            })
            .map(|e| e.handle)
            .collect();
        for handle in alvos {
            s.doc_mut()?.modify_entity(handle, |e| e.common.xdata.clear())?;
            apagados += 1;
        }
    }
    Ok(json!({ "apagados": apagados, "apps": apps, "xdata": APP }))
}

pub(super) fn run_titulos_textos_substituir(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.titulos_textos.substituir";
    let paginas_val = p.get("paginas").and_then(Value::as_array).ok_or_else(|| {
        bad(cmd, "informe as paginas (camadas) em `paginas`")
    })?;
    let paginas: Vec<String> = paginas_val
        .iter()
        .filter_map(Value::as_str)
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    if paginas.is_empty() {
        return Err(bad(cmd, "selecione ao menos uma pagina para editar"));
    }
    let de = str_param(p, "de").unwrap_or_default().to_string();
    if de.is_empty() {
        return Err(bad(cmd, "informe o texto a substituir em `de`"));
    }
    let para = str_param(p, "para").unwrap_or_default().to_string();
    let filtros = [
        p.get("titulo1").and_then(Value::as_bool).unwrap_or(false),
        p.get("titulo2").and_then(Value::as_bool).unwrap_or(false),
        p.get("titulo3").and_then(Value::as_bool).unwrap_or(false),
        p.get("titulo4").and_then(Value::as_bool).unwrap_or(false),
    ];
    let b_texto = p.get("texto").and_then(Value::as_bool).unwrap_or(false);
    let b_mtexto = p.get("mtexto").and_then(Value::as_bool).unwrap_or(false);
    let camadas: Vec<String> = paginas.iter().map(|c| c.to_lowercase()).collect();

    type Alvo = (Handle, String);
    let mut alvos: Vec<Alvo> = Vec::new();
    let mut examinados = 0usize;
    {
        let doc = s.doc()?;
        for entidade in doc.model.iter() {
            if !camadas.iter().any(|c| c == &entidade.common.layer.to_lowercase()) {
                continue;
            }
            match &entidade.kind {
                EntityKind::Text(texto) => {
                    let original = texto.value.clone();
                    if original.trim().is_empty() {
                        continue;
                    }
                    let editar = match tipo_do_titulo(entidade) {
                        Some(tipo) => {
                            filtros.get((tipo - 1) as usize).copied().unwrap_or(false)
                        }
                        None => b_texto && !tem_xdata_ocupado(entidade),
                    };
                    if !editar {
                        continue;
                    }
                    examinados += 1;
                    let novo = original.replace(&de, &para);
                    if novo != original {
                        alvos.push((entidade.handle, novo));
                    }
                }
                EntityKind::MText(mtexto) => {
                    if !b_mtexto || tem_xdata_ocupado(entidade) {
                        continue;
                    }
                    let original = mtexto.contents.clone();
                    if original.trim().is_empty() {
                        continue;
                    }
                    examinados += 1;
                    let novo = original.replace(&de, &para);
                    if novo != original {
                        alvos.push((entidade.handle, novo));
                    }
                }
                _ => {}
            }
        }
    }
    let mut alterados = 0usize;
    for (handle, novo) in &alvos {
        let novo = novo.clone();
        s.doc_mut()?.modify_entity(*handle, |e| match &mut e.kind {
            EntityKind::Text(texto) => texto.value = novo,
            EntityKind::MText(mtexto) => mtexto.contents = novo,
            _ => {}
        })?;
        alterados += 1;
    }
    Ok(json!({
        "alterados": alterados,
        "examinados": examinados,
        "paginas": paginas,
        "xdata": APP,
    }))
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

/// Revisoes automaticas do desenho: distintas, com a quantidade de objetos
/// que a tem e a cor do primeiro (a grade do `frmRevisaoAutomatica` lista
/// uma linha por revisao; `EditaRevisoesAutomaticas` edita por revisao, nao
/// por objeto).
pub fn revisoes_automaticas_do_desenho(
    doc: &cadcraft_doc::Drawing,
) -> Vec<(String, i32, usize)> {
    let mut por_revisao: BTreeMap<String, (String, i32, usize)> = BTreeMap::new();
    for entidade in doc.model.iter() {
        let Some(pares) = pares_do(entidade, MARCADOR_REVISAO_AUTOMATICA) else { continue };
        let Ok(revisao) = RevisaoAutomatica::de_pares(&pares) else { continue };
        let entrada = por_revisao
            .entry(revisao.revisao.to_lowercase())
            .or_insert_with(|| (revisao.revisao.clone(), revisao.cor, 0));
        entrada.2 += 1;
    }
    por_revisao.into_values().collect()
}

pub(super) fn run_revisao_automatica_ler(s: &mut Session, _p: &Value) -> Result<Value> {
    let revisoes: Vec<Value> = revisoes_automaticas_do_desenho(s.doc()?)
        .iter()
        .map(|(revisao, cor, quantidade)| {
            json!({ "revisao": revisao, "cor": cor, "quantidade": quantidade })
        })
        .collect();
    Ok(json!({ "revisoes": revisoes, "total": revisoes.len(), "xdata": APP }))
}

pub(super) fn run_revisao_automatica_gravar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.revisao_automatica.gravar";
    let antiga = str_param(p, "revisao_antiga")
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| bad(cmd, "informe a revisao atual em `revisao_antiga`"))?
        .to_string();
    let nova = str_param(p, "revisao").unwrap_or_default().to_string();
    let cor = p
        .get("cor")
        .and_then(Value::as_i64)
        .unwrap_or(1)
        .clamp(1, 255) as i32;
    let preencher = p.get("preencher").and_then(Value::as_bool).unwrap_or(false);

    // Objetos com a revisao (o `EditaRevisoesAutomaticas` varre o ModelSpace).
    let alvos: Vec<Handle> = {
        let doc = s.doc()?;
        doc.model
            .iter()
            .filter(|e| {
                pares_do(e, MARCADOR_REVISAO_AUTOMATICA)
                    .and_then(|pares| RevisaoAutomatica::de_pares(&pares).ok())
                    .is_some_and(|r| r.revisao.eq_ignore_ascii_case(&antiga))
            })
            .map(|e| e.handle)
            .collect()
    };
    let bloco = raws(&xdata::de_pares(
        &RevisaoAutomatica::novo(cor, &nova).para_pares(),
    ));
    let mut atualizados = 0usize;
    for handle in &alvos {
        let nova_clonada = nova.clone();
        let bloco_clonado = bloco.clone();
        s.doc_mut()?.modify_entity(*handle, |e| {
            e.common.xdata = bloco_clonado;
            match &mut e.kind {
                // Nuvem (Polyline no original): a cor da entidade.
                EntityKind::LwPolyline(_) | EntityKind::Polyline3d(_) => {
                    e.common.color = cadcraft_color::Color::Index(cor as u8);
                }
                // Bloco com atributo REV (`AtualizaCorValorDaRevisao`).
                EntityKind::Insert(inserido) => {
                    for atributo in &mut inserido.attribs {
                        if atributo.tag.eq_ignore_ascii_case("REV") {
                            atributo.text.value = nova_clonada.clone();
                            atributo.props.color =
                                Some(cadcraft_color::Color::Index(cor as u8));
                        }
                    }
                }
                _ => {}
            }
        })?;
        atualizados += 1;
    }

    // `ckPreencher Revisão`: titulos tipo 8 com o mesmo texto sao renomeados
    // (`AtualizaRevSimplificadaDasPaginas`).
    let mut titulos = 0usize;
    if preencher {
        let textos: Vec<Handle> = {
            let doc = s.doc()?;
            doc.model
                .iter()
                .filter(|e| tipo_do_titulo(e) == Some(TIPO_REVISAO_SIMPLIFICADA))
                .filter(|e| match &e.kind {
                    EntityKind::Text(texto) => texto.value.eq_ignore_ascii_case(&antiga),
                    _ => false,
                })
                .map(|e| e.handle)
                .collect()
        };
        for handle in textos {
            let nova_clonada = nova.clone();
            s.doc_mut()?.modify_entity(handle, |e| {
                if let EntityKind::Text(texto) = &mut e.kind {
                    texto.value = nova_clonada;
                }
            })?;
            titulos += 1;
        }
    }
    Ok(json!({ "atualizados": atualizados, "titulos": titulos, "total": alvos.len(), "xdata": APP }))
}




/// Pares do XData da aplicacao `app` da entidade (o `EXPORTADO`/`IMPORTADO`
/// do legado tem o proprio nome como aplicacao, sem marcador).
fn pares_app(entidade: &cadcraft_doc::Entity, app: &str) -> Option<Vec<Par>> {
    xdata::le_aplicacao(&tags(&entidade.common.xdata), app)
}

/// Objetos com XData `EXPORTADO` do desenho, com a pagina (camada) e o
/// handle atual — como o `CapturaExportados` do original (sem o
/// enriquecimento por dispositivo, que e de fases futuras).
pub fn exportados_do_desenho(
    doc: &cadcraft_doc::Drawing,
) -> Vec<(Exportado, String, Handle)> {
    let mut saida = Vec::new();
    for entidade in doc.model.iter() {
        let Some(pares) = pares_app(entidade, APP_EXPORTADO) else { continue };
        let Ok(mut exportado) = Exportado::de_pares(&pares) else { continue };
        exportado.handle = entidade.handle.hex();
        saida.push((exportado, entidade.common.layer.clone(), entidade.handle));
    }
    saida
}

pub(super) fn run_exportar_exportar(s: &mut Session, _p: &Value) -> Result<Value> {
    let exportados = exportados_do_desenho(s.doc()?);
    let mut tsv = String::from(TSV_CABECALHO_EXPORTADO);
    for (exportado, pagina, _) in &exportados {
        tsv.push('\n');
        tsv.push_str(&exportado.linha_tsv(pagina));
    }
    Ok(json!({ "tsv": tsv, "total": exportados.len(), "xdata": APP }))
}

pub(super) fn run_exportar_importar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.exportar.importar";
    let texto = str_param(p, "tsv").ok_or_else(|| {
        bad(cmd, "informe o texto TSV em `tsv` (como o Exportar gera)")
    })?;
    let mut importados = 0usize;
    let mut erros = 0usize;
    let mut linha = 0usize;
    for linha_txt in texto.lines() {
        let linha_txt = linha_txt.trim();
        if linha_txt.is_empty() || linha_txt == TSV_CABECALHO_EXPORTADO {
            continue;
        }
        match Importado::de_linha_tsv(linha_txt) {
            Ok(importado) => {
                // Sem a biblioteca de blocos de dispositivos (fases futuras),
                // o objeto importado e um texto com o tipo, na camada da
                // pagina — o XData IMPORTADO e o que a paridade carrega.
                linha += 1;
                let camada = if importado.pagina.trim().is_empty() {
                    "0".to_string()
                } else {
                    importado.pagina.clone()
                };
                let rotulo = if importado.tipo.trim().is_empty() {
                    "IMP".to_string()
                } else {
                    importado.tipo.clone()
                };
                let texto_novo = Text {
                    insert: cadcraft_geom::Vec3::new(0.0, (linha as f64) * 3.0, 0.0),
                    align_pt: None,
                    height: ALTURA_PADRAO,
                    value: rotulo,
                    rotation: 0.0,
                    width_factor: 1.0,
                    oblique: 0.0,
                    style: "Standard".to_string(),
                    halign: HAlign::Left,
                    valign: VAlign::Baseline,
                };
                let handle = s.add_entity(EntityKind::Text(texto_novo))?;
                let bloco = raws(&xdata::de_pares(&importado.para_pares()));
                s.doc_mut()?.modify_entity(handle, |e| {
                    e.common.layer = camada;
                    e.common.xdata = bloco;
                })?;
                importados += 1;
            }
            Err(_) => erros += 1,
        }
    }
    Ok(json!({ "importados": importados, "erros": erros, "xdata": APP }))
}

pub(super) fn run_exportar_remover(s: &mut Session, _p: &Value) -> Result<Value> {
    let alvos: Vec<Handle> = exportados_do_desenho(s.doc()?)
        .into_iter()
        .map(|(_, _, handle)| handle)
        .collect();
    let mut removidos = 0usize;
    for handle in alvos {
        s.doc_mut()?.modify_entity(handle, |e| e.common.xdata = Vec::new())?;
        removidos += 1;
    }
    Ok(json!({ "removidos": removidos, "xdata": APP }))
}


#[cfg(test)]
mod tests {
    use super::*;

    fn sessao() -> Session {
        Session::new()
    }


    #[test]
    fn exportar_exportar_importar_remover_no_desenho() -> Result<()> {
        let mut s = sessao();
        // Dois objetos com XData EXPORTADO (o carimbo do legado).
        for (indice, tipo) in [(1i32, "B"), (2, "E")] {
            let texto = Text {
                insert: Vec3::ZERO,
                align_pt: None,
                height: 2.5,
                value: tipo.to_string(),
                rotation: 0.0,
                width_factor: 1.0,
                oblique: 0.0,
                style: "Standard".to_string(),
                halign: HAlign::Left,
                valign: VAlign::Baseline,
            };
            let handle = s.add_entity(EntityKind::Text(texto))?;
            let bloco = raws(&xdata::de_pares(
                &Exportado::novo(indice, 7, "", tipo, 3).para_pares(),
            ));
            s.doc_mut()?.modify_entity(handle, |e| {
                e.common.layer = "12A".to_string();
                e.common.xdata = bloco;
            })?;
        }

        // Exportar: TSV com cabecalho e uma linha por objeto.
        let saida = run_exportar_exportar(&mut s, &json!({}))?;
        assert_eq!(saida["total"], 2);
        let tsv = saida["tsv"]
            .as_str()
            .ok_or_else(|| bad("teste", "tsv ausente"))?;
        assert!(tsv.starts_with("Indice\tCodigo\tHandle\tTipo\tDWG\tPagina"));
        assert!(tsv.contains("\tB\t3\t12A"));
        assert!(tsv.contains("\tE\t3\t12A"));

        // Importar esse TSV: cria dois textos com XData IMPORTADO.
        let importado = run_exportar_importar(&mut s, &json!({ "tsv": tsv }))?;
        assert_eq!(importado["importados"], 2);
        assert_eq!(importado["erros"], 0);
        let criados: Vec<_> = s
            .doc()?
            .model
            .iter()
            .filter(|e| pares_app(e, xdata::APP_IMPORTADO).is_some())
            .collect();
        assert_eq!(criados.len(), 2);
        assert_eq!(criados[0].common.layer, "12A");

        // Remover: limpa o EXPORTADO dos dois objetos originais.
        let removido = run_exportar_remover(&mut s, &json!({}))?;
        assert_eq!(removido["removidos"], 2);
        let depois = run_exportar_exportar(&mut s, &json!({}))?;
        assert_eq!(depois["total"], 0);
        assert!(run_exportar_importar(&mut s, &json!({})).is_err());
        Ok(())
    }



    #[test]
    fn revisao_automatica_ler_e_gravar_no_desenho() -> Result<()> {
        let mut s = sessao();
        run_pagina_escrever(&mut s, &json!({ "pagina": "12A", "titulo1": "DIAGRAMA" }))?;
        // Duas nuvens com a revisao "A" e uma com "B".
        let mut nuvem = |valor: &str| -> Result<()> {
            let polilinha = cadcraft_doc::LwPolyline {
                vertices: vec![cadcraft_geom::PolyVertex {
                    p: cadcraft_geom::Vec2::new(0.0, 0.0),
                    bulge: 0.0,
                    start_width: 0.0,
                    end_width: 0.0,
                }],
                closed: true,
                const_width: 0.0,
                elevation: 0.0,
                plinegen: false,
            };
            let handle = s.add_entity(EntityKind::LwPolyline(polilinha))?;
            let bloco = raws(&xdata::de_pares(
                &RevisaoAutomatica::novo(2, valor).para_pares(),
            ));
            s.doc_mut()?.modify_entity(handle, |e| {
                e.common.layer = "12A".to_string();
                e.common.xdata = bloco;
            })?;
            Ok(())
        };
        nuvem("A")?;
        nuvem("A")?;
        nuvem("B")?;
        // Titulo tipo 8 com o texto "A" (para o `preencher`).
        let texto = Text {
            insert: Vec3::ZERO,
            align_pt: None,
            height: 2.5,
            value: "A".to_string(),
            rotation: 0.0,
            width_factor: 1.0,
            oblique: 0.0,
            style: "Standard".to_string(),
            halign: HAlign::Left,
            valign: VAlign::Baseline,
        };
        let handle = s.add_entity(EntityKind::Text(texto))?;
        let bloco = raws(&xdata::de_pares(
            &TituloPagina::novo(TIPO_REVISAO_SIMPLIFICADA, "A").para_pares(),
        ));
        s.doc_mut()?.modify_entity(handle, |e| {
            e.common.layer = "12A".to_string();
            e.common.xdata = bloco;
        })?;

        let ler = run_revisao_automatica_ler(&mut s, &json!({}))?;
        assert_eq!(ler["total"], 2);
        assert_eq!(ler["revisoes"][0]["revisao"], "A");
        assert_eq!(ler["revisoes"][0]["cor"], 2);
        assert_eq!(ler["revisoes"][0]["quantidade"], 2);
        assert_eq!(ler["revisoes"][1]["revisao"], "B");

        // Editar a revisao "A" (o `EditaRevisoesAutomaticas`): o XData e a
        // cor mudam, e o titulo tipo 8 acompanha (`preencher`).
        let gravado = run_revisao_automatica_gravar(
            &mut s,
            &json!({ "revisao_antiga": "A", "revisao": "C", "cor": 5, "preencher": true }),
        )?;
        assert_eq!(gravado["atualizados"], 2);
        assert_eq!(gravado["titulos"], 1);
        let depois = run_revisao_automatica_ler(&mut s, &json!({}))?;
        assert_eq!(depois["total"], 2);
        assert_eq!(depois["revisoes"][0]["revisao"], "B");
        assert_eq!(depois["revisoes"][1]["revisao"], "C");
        assert_eq!(depois["revisoes"][1]["cor"], 5);
        let entidade = s
            .doc()?
            .model
            .iter()
            .find(|e| e.common.layer == "12A"
                && matches!(&e.kind, EntityKind::LwPolyline(_)))
            .ok_or_else(|| bad("teste", "nuvem"))?;
        assert_eq!(entidade.common.color, cadcraft_color::Color::Index(5));
        let simples = run_revisao_ler(&mut s, &json!({}))?;
        assert_eq!(simples["revisoes"][0]["revisao"], "C");
        assert!(run_revisao_automatica_gravar(&mut s, &json!({ "revisao": "D" })).is_err());
        Ok(())
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
    fn titulos_textos_substitui_filtros_e_paginas() -> Result<()> {
        let mut s = sessao();
        run_pagina_escrever(&mut s, &json!({ "pagina": "12A", "titulo1": "DIAGRAMA SE-01" }))?;
        // Texto livre na camada da pagina (sem XData) e um fora dela.
        for (camada, valor) in [("12A", "Nota SE-01 livre"), ("99", "Nota SE-01 fora")] {
            let texto = Text {
                insert: Vec3::ZERO,
                align_pt: None,
                height: 2.5,
                value: valor.to_string(),
                rotation: 0.0,
                width_factor: 1.0,
                oblique: 0.0,
                style: "Standard".to_string(),
                halign: HAlign::Left,
                valign: VAlign::Baseline,
            };
            let handle = s.add_entity(EntityKind::Text(texto))?;
            let camada = camada.to_string();
            s.doc_mut()?.modify_entity(handle, |e| e.common.layer = camada)?;
        }

        let saida = run_titulos_textos_substituir(
            &mut s,
            &json!({ "paginas": ["12A"], "titulo1": true, "texto": true, "de": "SE-01", "para": "SE-02" }),
        )?;
        assert_eq!(saida["alterados"], 2);
        assert_eq!(saida["examinados"], 2);
        assert_eq!(saida["xdata"], APP);

        // So o texto visivel muda: o XData (fonte do `pagina.ler`) segue intacto,
        // como no original, que edita o `Text` e nao o XData do titulo.
        let ler = run_pagina_ler(&mut s, &json!({ "pagina": "12A" }))?;
        assert_eq!(ler["paginas"][0]["titulo1"], "DIAGRAMA SE-01");
        let valores: Vec<String> = s
            .doc()?
            .model
            .iter()
            .filter_map(|e| match &e.kind {
                EntityKind::Text(t) => Some(t.value.clone()),
                _ => None,
            })
            .collect();
        assert!(valores.iter().any(|v| v == "DIAGRAMA SE-02"));
        assert!(!valores.iter().any(|v| v == "DIAGRAMA SE-01"));
        assert!(valores.iter().any(|v| v == "Nota SE-02 livre"));
        assert!(valores.iter().any(|v| v == "Nota SE-01 fora"));

        // Sem filtro marcado nada muda; sem pagina ou sem `de` e erro.
        let nada = run_titulos_textos_substituir(
            &mut s,
            &json!({ "paginas": ["12A"], "de": "SE-02", "para": "SE-03" }),
        )?;
        assert_eq!(nada["alterados"], 0);
        assert!(run_titulos_textos_substituir(&mut s, &json!({ "paginas": [], "de": "x" })).is_err());
        assert!(run_titulos_textos_substituir(&mut s, &json!({ "paginas": ["12A"], "de": "" })).is_err());
        Ok(())
    }


    #[test]
    fn revisao_ler_e_gravar_no_desenho() -> Result<()> {
        let mut s = sessao();
        run_pagina_escrever(&mut s, &json!({ "pagina": "12A", "titulo1": "DIAGRAMA" }))?;
        // Texto de revisao simplificada (titulo tipo 8) na camada da pagina.
        let texto = Text {
            insert: Vec3::ZERO,
            align_pt: None,
            height: 2.5,
            value: "A".to_string(),
            rotation: 0.0,
            width_factor: 1.0,
            oblique: 0.0,
            style: "Standard".to_string(),
            halign: HAlign::Left,
            valign: VAlign::Baseline,
        };
        let handle = s.add_entity(EntityKind::Text(texto))?;
        let bloco = raws(&xdata::de_pares(&TituloPagina::novo(TIPO_REVISAO_SIMPLIFICADA, "A").para_pares()));
        s.doc_mut()?.modify_entity(handle, |e| {
            e.common.layer = "12A".to_string();
            e.common.xdata = bloco;
        })?;

        let ler = run_revisao_ler(&mut s, &json!({}))?;
        assert_eq!(ler["total"], 1);
        assert_eq!(ler["revisoes"][0]["pagina"], "12A");
        assert_eq!(ler["revisoes"][0]["revisao"], "A");

        // Gravar atualiza o texto visivel e o XData; pagina sem texto de
        // revisao e ignorada (o original so atualiza o que ja existe).
        let gravado = run_revisao_gravar(
            &mut s,
            &json!({ "revisoes": [
                { "pagina": "12A", "revisao": "B" },
                { "pagina": "99", "revisao": "C" },
            ] }),
        )?;
        assert_eq!(gravado["atualizados"], 1);
        assert_eq!(gravado["ignorados"], 1);
        let depois = run_revisao_ler(&mut s, &json!({}))?;
        assert_eq!(depois["revisoes"][0]["revisao"], "B");
        // O titulo 1 nao foi tocado.
        let pagina = run_pagina_ler(&mut s, &json!({ "pagina": "12A" }))?;
        assert_eq!(pagina["paginas"][0]["titulo1"], "DIAGRAMA");
        assert!(run_revisao_gravar(&mut s, &json!({})).is_err());
        Ok(())
    }


    #[test]
    fn revisao_completa_ler_e_gravar_no_desenho() -> Result<()> {
        let mut s = sessao();
        run_pagina_escrever(&mut s, &json!({ "pagina": "12A", "titulo1": "DIAGRAMA" }))?;
        // Bloco de revisao (Insert com XData REVISAO) na camada da pagina.
        let bloco_inserido = cadcraft_doc::Insert {
            block: "Revisao_Corpo0".to_string(),
            insert: Vec3::ZERO,
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: 0.0,
            attribs: Vec::new(),
            cols: 1,
            rows: 1,
            col_spacing: 0.0,
            row_spacing: 0.0,
        };
        let handle = s.add_entity(EntityKind::Insert(bloco_inserido))?;
        let revisao_antiga = RevisaoCompleta {
            posicao: 1,
            revisao: "A".to_string(),
            descricao: "EMISSAO".to_string(),
            ..RevisaoCompleta::default()
        };
        let xdata_antigo = raws(&xdata::de_pares(&revisao_antiga.para_pares()));
        s.doc_mut()?.modify_entity(handle, |e| {
            e.common.layer = "12A".to_string();
            e.common.xdata = xdata_antigo;
        })?;

        let ler = run_revisao_completa_ler(&mut s, &json!({}))?;
        assert_eq!(ler["total"], 1);
        assert_eq!(ler["revisoes"][0]["pagina"], "12A");
        assert_eq!(ler["revisoes"][0]["posicao"], 1);
        assert_eq!(ler["revisoes"][0]["revisao"], "A");
        assert_eq!(ler["revisoes"][0]["descricao"], "EMISSAO");

        // Gravar atualiza os campos no Insert existente; pagina sem bloco
        // de revisao e ignorada (sem criacao de bloco, como documentado).
        let gravado = run_revisao_completa_gravar(
            &mut s,
            &json!({ "revisoes": [
                { "pagina": "12A", "revisao": "B", "descricao": "ALTERACAO", "elaborador": "ANA" },
                { "pagina": "99", "revisao": "C" },
            ] }),
        )?;
        assert_eq!(gravado["atualizados"], 1);
        assert_eq!(gravado["ignorados"], 1);
        let depois = run_revisao_completa_ler(&mut s, &json!({}))?;
        assert_eq!(depois["revisoes"][0]["revisao"], "B");
        assert_eq!(depois["revisoes"][0]["descricao"], "ALTERACAO");
        assert_eq!(depois["revisoes"][0]["elaborador"], "ANA");
        assert_eq!(depois["revisoes"][0]["posicao"], 1);
        assert!(run_revisao_completa_gravar(&mut s, &json!({})).is_err());
        Ok(())
    }


    #[test]
    fn xdata_listar_e_apagar_no_desenho() -> Result<()> {
        let mut s = sessao();
        run_pagina_escrever(&mut s, &json!({ "pagina": "12A", "titulo1": "DIAGRAMA" }))?;
        // Entidade com XData de outro aplicativo.
        let texto = Text {
            insert: Vec3::ZERO,
            align_pt: None,
            height: 2.5,
            value: "NOTA".to_string(),
            rotation: 0.0,
            width_factor: 1.0,
            oblique: 0.0,
            style: "Standard".to_string(),
            halign: HAlign::Left,
            valign: VAlign::Baseline,
        };
        let handle = s.add_entity(EntityKind::Text(texto))?;
        s.doc_mut()?.modify_entity(handle, |e| {
            e.common.xdata = vec![
                RawTag { code: 1001, value: "OUTROAPP".to_string() },
                RawTag { code: 1000, value: "algo".to_string() },
            ];
        })?;

        // Sem incluir conhecidos, so aparece o app fora da lista.
        let lista = run_xdata_listar(&mut s, &json!({}))?;
        assert_eq!(lista["total"], 1);
        assert_eq!(lista["xdatas"][0]["app"], "OUTROAPP");
        assert_eq!(lista["xdatas"][0]["quantidade"], 1);
        // Incluindo, tambem aparece o app do modulo.
        let tudo = run_xdata_listar(&mut s, &json!({ "incluir_conhecidos": true }))?;
        assert!(tudo["xdatas"].as_array().is_some_and(|a| a.iter().any(|x| x["app"] == APP)));

        let saida = run_xdata_apagar(&mut s, &json!({ "apps": ["outroapp"] }))?;
        assert_eq!(saida["apagados"], 1);
        let depois = run_xdata_listar(&mut s, &json!({}))?;
        assert_eq!(depois["total"], 0);
        // O XData do modulo segue intacto.
        let pagina = run_pagina_ler(&mut s, &json!({ "pagina": "12A" }))?;
        assert_eq!(pagina["paginas"][0]["titulo1"], "DIAGRAMA");
        assert!(run_xdata_apagar(&mut s, &json!({ "apps": [] })).is_err());
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
/// As conexoes de circuitos do desenho: polylines (LWPOLYLINE/POLYLINE)
/// com XData `CONEXAO` de tipo circuito, já com pagina e entidade.
fn conexoes_do_desenho(doc: &cadcraft_doc::Drawing) -> Vec<Conexao> {
    let mut conexoes = Vec::new();
    for entity in doc.model.iter() {
        if !matches!(
            entity.kind,
            EntityKind::LwPolyline(_) | EntityKind::Polyline3d(_)
        ) {
            continue;
        }
        let Ok(mut conexao) = Conexao::de_tags(&tags(&entity.common.xdata)) else { continue };
        if conexao.tipo != TIPO_CIRCUITO {
            continue;
        }
        conexao.pagina = entity.common.layer.clone();
        conexao.entidade = entity.handle.hex();
        conexoes.push(conexao);
    }
    conexoes
}

/// Agrupamento da janela: papeis com circuitos (potencial, nome e paginas).
pub fn circuitos_do_desenho(s: &Session) -> Value {
    let Ok(doc) = s.doc() else { return json!({"paineis": []}) };
    let conexoes = conexoes_do_desenho(doc);
    let paineis = agrupar(&conexoes, &std::collections::BTreeMap::new());
    json!({
        "paineis": paineis
            .iter()
            .map(|painel| json!({
                "numero": painel.numero,
                "nome": painel.nome,
                "circuitos": painel
                    .circuitos
                    .iter()
                    .map(|circuito| json!({
                        "painel": painel.numero,
                        "nome_painel": painel.nome,
                        "potencial": circuito.potencial,
                        "nome": circuito.nome,
                        "paginas": circuito
                            .paginas
                            .iter()
                            .map(|p| json!({"pagina": p.pagina, "handle": p.handle}))
                            .collect::<Vec<Value>>(),
                        "paginas_txt": circuito
                            .paginas
                            .iter()
                            .map(|p| p.pagina.clone())
                            .collect::<Vec<String>>()
                            .join("; "),
                    }))
                    .collect::<Vec<Value>>(),
            }))
            .collect::<Vec<Value>>(),
    })
}

pub(super) fn run_circuito_listar(s: &mut Session, _p: &Value) -> Result<Value> {
    Ok(circuitos_do_desenho(s))
}

pub(super) fn run_circuito_renomear(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.circuito.renomear";
    let potencial = p
        .get("potencial")
        .and_then(Value::as_i64)
        .ok_or_else(|| bad(cmd, "informe o potencial do circuito em `potencial`"))?;
    let nome = str_param(p, "nome")
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| bad(cmd, "informe o nome novo em `nome`"))?
        .to_string();
    let doc = s.doc()?;
    let textos: Vec<Handle> = doc
        .model
        .iter()
        .filter(|e| matches!(e.kind, EntityKind::Text(_)))
        .filter(|e| {
            TextoAuxiliarCircuito::de_tags(&tags(&e.common.xdata))
                .is_ok_and(|aux| aux.potencial as i64 == potencial && aux.tipo == 1)
        })
        .map(|e| e.handle)
        .collect();
    let linhas: Vec<Handle> = doc
        .model
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                EntityKind::LwPolyline(_) | EntityKind::Polyline3d(_)
            )
        })
        .filter(|e| {
            Conexao::de_tags(&tags(&e.common.xdata))
                .is_ok_and(|c| c.potencial as i64 == potencial && c.tipo == TIPO_CIRCUITO)
        })
        .map(|e| e.handle)
        .collect();
    let (total_textos, total_linhas) = (textos.len(), linhas.len());
    for handle in textos {
        let nome = nome.clone();
        s.doc_mut()?.modify_entity(handle, |e| {
            if let EntityKind::Text(texto) = &mut e.kind {
                texto.value = nome;
            }
        })?;
    }
    for handle in linhas {
        s.doc_mut()?.modify_entity(handle, |e| {
            if let Ok(mut conexao) = Conexao::de_tags(&tags(&e.common.xdata)) {
                conexao.nome = nome.clone();
                e.common.xdata = raws(&conexao.para_tags());
            }
        })?;
    }
    let textos = total_textos;
    let linhas = total_linhas;
    Ok(json!({
        "potencial": potencial,
        "nome": nome,
        "textos": textos,
        "linhas": linhas,
    }))
}

pub(super) fn run_circuito_mostrar(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.circuito.mostrar";
    let handle_txt = str_param(p, "handle")
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| bad(cmd, "informe o handle da polyline em `handle`"))?;
    let handle = Handle::parse_hex(handle_txt)
        .ok_or_else(|| bad(cmd, "handle invalido"))?;
    let (centro, altura) = {
        let doc = s.doc()?;
        let entidade =
            doc.entity(handle).ok_or_else(|| bad(cmd, "entidade nao encontrada"))?;
        let b = entity_bounds(doc, entidade, 0);
        if b.is_empty() {
            return Err(bad(cmd, "entidade sem geometria"));
        }
        (b.center(), b)
    };
    s.set_selection(vec![handle]);
    let (w, h) = s.viewport_px;
    let st = s.state_mut()?;
    let aspecto = st.zoom_aspect(w / h.max(1.0));
    let altura = (altura.height().max(altura.width() / aspecto.max(1e-6)) * 1.05).max(1e-6);
    st.set_zoom_frame(crate::View { center: centro, height: altura });
    Ok(json!({"handle": handle.hex(), "selecionado": true}))
}
