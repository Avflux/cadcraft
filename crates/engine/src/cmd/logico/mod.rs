//! Comandos do dominio **logico** (`log.*`): estado, banco do projeto
//! e paginas do desenho de formato.
//!
//! O dominio vive no repositorio irmao (`../logico`): este arquivo so liga os
//! comandos do cadcraft ao `logico_data`. A interface correspondente e
//! `crates/ui-egui/src/logico.rs`.

use std::any::Any;

use serde_json::{Value, json};

use super::{CommandSpec, always, bad, str_param};
use crate::domain::Domain;
use crate::{Result, Session};

mod desenho;

use desenho::{run_circuito_listar, run_circuito_mostrar, run_circuito_renomear, run_pagina_escrever, run_pagina_ler, run_pagina_sincronizar, run_projeto_gravar, run_projeto_ler, run_revisao_completa_gravar, run_revisao_completa_ler, run_revisao_gravar, run_revisao_ler,
    run_revisao_automatica_gravar, run_revisao_automatica_ler, run_titulos_textos_substituir, run_xdata_apagar, run_xdata_listar};

/// O banco que o desenho aponta, para a interface.
pub use desenho::banco_do_desenho;

/// O modulo como o motor o ve: seus comandos e o estado da sessao.
pub struct Modulo;

impl Domain for Modulo {
    fn id(&self) -> &'static str {
        "logico"
    }
    fn commands(&self) -> Vec<CommandSpec> {
        specs()
    }
    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(())
    }
}

/// O modulo registrado em [`crate::domain::builtin`].
pub static LOGICO: Modulo = Modulo;

fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("log.status", "Lógico: Estado do Módulo", run_status)
            .menu(&["Lógico", "Diagnóstico", "Estado do Módulo"])
            .params("{} -> {modulo, xdata, schema, comandos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.banco.abrir", "Lógico: Abrir Banco", run_banco_abrir)
            .menu(&["Lógico", "Arquivo", "Abrir Banco..."])
            .params("{arquivo} -> {arquivo, schema, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.pagina.gravar", "Lógico: Gravar Página", run_pagina_gravar)
            .menu(&["Lógico", "Páginas", "Gravar Página"])
            .params("{arquivo, pagina, titulo1?, titulo2?, titulo3?, titulo4?, unidade?, alternativo?} -> {pagina, pg_num, pg_alfa, pg_dec, total}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.pagina.listar", "Lógico: Listar Páginas", run_pagina_listar)
            .menu(&["Lógico", "Páginas", "Listar Páginas"])
            .params("{arquivo, limite?} -> {arquivo, total, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.projeto.gravar", "Lógico: Gravar Vínculo do Projeto", run_projeto_gravar)
            .menu(&["Lógico", "Projeto", "Gravar Vínculo..."])
            .params("{banco} -> {banco, handle, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.projeto.ler", "Lógico: Ler Vínculo do Projeto", run_projeto_ler)
            .menu(&["Lógico", "Projeto", "Ler Vínculo"])
            .params("{} -> {banco, encontrado, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.pagina.escrever", "Lógico: Gravar Página no Desenho", run_pagina_escrever)
            .menu(&["Lógico", "Páginas", "Gravar no Desenho"])
            .params("{pagina, titulo1?..titulo4?, unidade?, alternativo?, x?, y?, altura?, autor?, quando?, banco?} -> {pagina, criados, atualizados, removidos, banco, gravado_no_banco}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.pagina.ler", "Lógico: Ler Páginas do Desenho", run_pagina_ler)
            .menu(&["Lógico", "Páginas", "Ler do Desenho"])
            .params("{pagina?} -> {total, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.pagina.sincronizar", "Lógico: Sincronizar Banco com o Desenho", run_pagina_sincronizar)
            .menu(&["Lógico", "Páginas", "Sincronizar Banco"])
            .params("{banco?} -> {banco, gravadas, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.revisao.ler", "Lógico: Ler Revisões do Desenho", run_revisao_ler)
            .menu(&["Lógico", "Revisão", "Ler Revisões"])
            .params("{} -> {revisoes, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.revisao.gravar", "Lógico: Gravar Revisões no Desenho", run_revisao_gravar)
            .menu(&["Lógico", "Revisão", "Gravar Revisões"])
            .params("{revisoes} -> {atualizados, ignorados, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.revisao_completa.ler", "Lógico: Ler Revisões Completas", run_revisao_completa_ler)
            .menu(&["Lógico", "Revisão", "Ler Revisão Completa"])
            .params("{} -> {revisoes, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.revisao_completa.gravar", "Lógico: Gravar Revisões Completas", run_revisao_completa_gravar)
            .menu(&["Lógico", "Revisão", "Gravar Revisão Completa"])
            .params("{revisoes} -> {atualizados, ignorados, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.revisao_automatica.ler", "Lógico: Ler Revisões Automáticas", run_revisao_automatica_ler)
            .menu(&["Lógico", "Revisão", "Ler Revisões Automáticas"])
            .params("{} -> {revisoes, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.revisao_automatica.gravar", "Lógico: Gravar Revisão Automática", run_revisao_automatica_gravar)
            .menu(&["Lógico", "Revisão", "Gravar Revisão Automática"])
            .params("{revisao_antiga, revisao?, cor?, preencher?} -> {atualizados, titulos, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.xdata.listar", "Lógico: Listar XDatas", run_xdata_listar)
            .menu(&["Lógico", "Projeto", "Listar XDatas"])
            .params("{incluir_conhecidos?} -> {xdatas, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.xdata.apagar", "Lógico: Apagar XDatas", run_xdata_apagar)
            .menu(&["Lógico", "Projeto", "Apagar XDatas"])
            .params("{apps} -> {apagados, apps, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.titulos_textos.substituir", "Lógico: Editar Títulos (Textos)", run_titulos_textos_substituir)
            .menu(&["Lógico", "Páginas", "Editar Títulos (Textos)"])
            .params("{paginas, titulo1?, titulo2?, titulo3?, titulo4?, texto?, mtexto?, de, para} -> {alterados, examinados, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.circuito.listar", "Lógico: Listar Circuitos", run_circuito_listar)
            .menu(&["Lógico", "Circuitos", "Listar Circuitos"])
            .params("{} -> {paineis}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.circuito.renomear", "Lógico: Renomear Circuito", run_circuito_renomear)
            .menu(&["Lógico", "Circuitos", "Renomear Circuito"])
            .params("{potencial, nome} -> {potencial, nome, textos, linhas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("log.circuito.mostrar", "Lógico: Mostrar Circuito", run_circuito_mostrar)
            .menu(&["Lógico", "Circuitos", "Mostrar Circuito"])
            .params("{handle} -> {handle, selecionado}")
            .enabled(always)
            .noundo(),
    ]
}

/// Abre (ou cria) o `.db` pedido no comando.
fn banco_do_comando(cmd: &str, p: &Value) -> Result<(String, logico_data::Banco)> {
    let arquivo = str_param(p, "arquivo")
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| bad(cmd, "informe o arquivo .db em `arquivo`"))?;
    let banco = logico_data::Banco::abrir(arquivo).map_err(|e| bad(cmd, e.to_string()))?;
    Ok((arquivo.to_string(), banco))
}

/// Campo de texto do comando, sem espacos nas pontas.
fn campo(p: &Value, chave: &str) -> String {
    str_param(p, chave).unwrap_or_default().trim().to_string()
}

fn pagina_para_json(pagina: &logico_data::Pagina) -> Value {
    json!({
        "pagina": pagina.pagina,
        "pg_num": pagina.pg_num,
        "pg_alfa": pagina.pg_alfa,
        "pg_dec": pagina.pg_dec,
        "titulo1": pagina.titulo1,
        "titulo2": pagina.titulo2,
        "titulo3": pagina.titulo3,
        "titulo4": pagina.titulo4,
        "unidade": pagina.unidade,
        "alternativo": pagina.alternativo,
    })
}

fn run_status(_s: &mut Session, _p: &Value) -> Result<Value> {
    Ok(json!({
        "modulo": "logico",
        "xdata": logico_data::XDATA_PREFIXO,
        "schema": logico_data::SCHEMA_VERSAO,
        "comandos": [
            "log.status",
            "log.banco.abrir",
            "log.pagina.gravar",
            "log.pagina.listar",
            "log.projeto.gravar",
            "log.projeto.ler",
            "log.pagina.escrever",
            "log.pagina.ler",
            "log.pagina.sincronizar",
            "log.revisao_completa.ler",
            "log.revisao_completa.gravar",
            "log.revisao_automatica.ler",
            "log.revisao_automatica.gravar",
            "log.xdata.listar",
            "log.xdata.apagar",
            "log.titulos_textos.substituir",
            "log.circuito.listar",
            "log.circuito.renomear",
            "log.circuito.mostrar",
        ],
    }))
}

fn run_banco_abrir(_s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "log.banco.abrir";
    let (arquivo, banco) = banco_do_comando(cmd, p)?;
    let schema = banco.versao().map_err(|e| bad(cmd, e.to_string()))?;
    let paginas = banco.paginas().map_err(|e| bad(cmd, e.to_string()))?.len();
    Ok(json!({ "arquivo": arquivo, "schema": schema, "paginas": paginas }))
}

fn run_pagina_gravar(_s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "log.pagina.gravar";
    let (arquivo, banco) = banco_do_comando(cmd, p)?;
    let chave = campo(p, "pagina");
    if chave.is_empty() {
        return Err(bad(cmd, "informe a página (nome da camada) em `pagina`"));
    }
    let mut pagina = logico_data::Pagina::nova(&chave);
    pagina.titulo1 = campo(p, "titulo1");
    pagina.titulo2 = campo(p, "titulo2");
    pagina.titulo3 = campo(p, "titulo3");
    pagina.titulo4 = campo(p, "titulo4");
    pagina.unidade = campo(p, "unidade");
    pagina.alternativo = campo(p, "alternativo");
    banco.salvar_pagina(&pagina).map_err(|e| bad(cmd, e.to_string()))?;
    let total = banco.paginas().map_err(|e| bad(cmd, e.to_string()))?.len();
    let mut valor = pagina_para_json(&pagina);
    if let Some(mapa) = valor.as_object_mut() {
        mapa.insert("arquivo".to_string(), json!(arquivo));
        mapa.insert("total".to_string(), json!(total));
    }
    Ok(valor)
}

fn run_pagina_listar(_s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "log.pagina.listar";
    let (arquivo, banco) = banco_do_comando(cmd, p)?;
    let limite = p.get("limite").and_then(Value::as_u64).unwrap_or(200) as usize;
    let todas = banco.paginas().map_err(|e| bad(cmd, e.to_string()))?;
    let total = todas.len();
    let paginas: Vec<Value> = todas.iter().take(limite).map(pagina_para_json).collect();
    Ok(json!({ "arquivo": arquivo, "total": total, "paginas": paginas }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comandos_do_modulo_sao_do_prefixo() {
        let specs = specs();
        assert_eq!(specs.len(), 21);
        let mut ids: Vec<&str> = specs.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), specs.len(), "ids repetidos no catalogo");
        for id in ids {
            assert!(id.starts_with("log."), "{id} fora do prefixo");
        }
    }

    #[test]
    fn pagina_do_comando_usa_o_core() {
        let pagina = logico_data::Pagina::nova("12A");
        assert_eq!(pagina.pg_num, "12");
        assert_eq!(pagina.pg_alfa, "A");
        assert!((pagina.pg_dec - 12.65).abs() < 1e-9);
    }
}
