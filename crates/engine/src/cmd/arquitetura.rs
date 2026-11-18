//! Comandos do dominio **arquitetura** (`arq.*`): estado, banco do projeto
//! e paginas do desenho de formato.
//!
//! O dominio vive no repositorio irmao (`../arquitetura`): este arquivo so liga os
//! comandos do cadcraft ao `arquitetura_data`. A interface correspondente e
//! `crates/ui-egui/src/arquitetura.rs`.

use std::any::Any;

use serde_json::{Value, json};

use super::{CommandSpec, always, bad, str_param};
use crate::domain::Domain;
use crate::{Result, Session};

/// O modulo como o motor o ve: seus comandos e o estado da sessao.
pub struct Modulo;

impl Domain for Modulo {
    fn id(&self) -> &'static str {
        "arquitetura"
    }
    fn commands(&self) -> Vec<CommandSpec> {
        specs()
    }
    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(())
    }
}

/// O modulo registrado em [`crate::domain::builtin`].
pub static ARQUITETURA: Modulo = Modulo;

fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("arq.status", "Arquitetura: Estado do Módulo", run_status)
            .menu(&["Arquitetura", "Diagnóstico", "Estado do Módulo"])
            .params("{} -> {modulo, xdata, schema, comandos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("arq.banco.abrir", "Arquitetura: Abrir Banco", run_banco_abrir)
            .menu(&["Arquitetura", "Arquivo", "Abrir Banco..."])
            .params("{arquivo} -> {arquivo, schema, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("arq.pagina.gravar", "Arquitetura: Gravar Página", run_pagina_gravar)
            .menu(&["Arquitetura", "Páginas", "Gravar Página"])
            .params("{arquivo, pagina, titulo1?, titulo2?, titulo3?, titulo4?, unidade?, alternativo?} -> {pagina, pg_num, pg_alfa, pg_dec, total}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("arq.pagina.listar", "Arquitetura: Listar Páginas", run_pagina_listar)
            .menu(&["Arquitetura", "Páginas", "Listar Páginas"])
            .params("{arquivo, limite?} -> {arquivo, total, paginas}")
            .enabled(always)
            .noundo(),
    ]
}

/// Abre (ou cria) o `.db` pedido no comando.
fn banco_do_comando(cmd: &str, p: &Value) -> Result<(String, arquitetura_data::Banco)> {
    let arquivo = str_param(p, "arquivo")
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| bad(cmd, "informe o arquivo .db em `arquivo`"))?;
    let banco = arquitetura_data::Banco::abrir(arquivo).map_err(|e| bad(cmd, e.to_string()))?;
    Ok((arquivo.to_string(), banco))
}

/// Campo de texto do comando, sem espacos nas pontas.
fn campo(p: &Value, chave: &str) -> String {
    str_param(p, chave).unwrap_or_default().trim().to_string()
}

fn pagina_para_json(pagina: &arquitetura_data::Pagina) -> Value {
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
        "modulo": "arquitetura",
        "xdata": arquitetura_data::XDATA_PREFIXO,
        "schema": arquitetura_data::SCHEMA_VERSAO,
        "comandos": [
            "arq.status",
            "arq.banco.abrir",
            "arq.pagina.gravar",
            "arq.pagina.listar",
        ],
    }))
}

fn run_banco_abrir(_s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.banco.abrir";
    let (arquivo, banco) = banco_do_comando(cmd, p)?;
    let schema = banco.versao().map_err(|e| bad(cmd, e.to_string()))?;
    let paginas = banco.paginas().map_err(|e| bad(cmd, e.to_string()))?.len();
    Ok(json!({ "arquivo": arquivo, "schema": schema, "paginas": paginas }))
}

fn run_pagina_gravar(_s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "arq.pagina.gravar";
    let (arquivo, banco) = banco_do_comando(cmd, p)?;
    let chave = campo(p, "pagina");
    if chave.is_empty() {
        return Err(bad(cmd, "informe a página (nome da camada) em `pagina`"));
    }
    let mut pagina = arquitetura_data::Pagina::nova(&chave);
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
    let cmd = "arq.pagina.listar";
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
        assert_eq!(specs.len(), 4);
        let mut ids: Vec<&str> = specs.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), specs.len(), "ids repetidos no catalogo");
        for id in ids {
            assert!(id.starts_with("arq."), "{id} fora do prefixo");
        }
    }

    #[test]
    fn pagina_do_comando_usa_o_core() {
        let pagina = arquitetura_data::Pagina::nova("12A");
        assert_eq!(pagina.pg_num, "12");
        assert_eq!(pagina.pg_alfa, "A");
        assert!((pagina.pg_dec - 12.65).abs() < 1e-9);
    }
}
