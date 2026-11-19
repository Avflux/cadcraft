//! Comandos do dominio **eletrico** (`ele.*`): estado, banco do projeto
//! e paginas do desenho de formato.
//!
//! O dominio vive no repositorio irmao (`../eletrico`): este arquivo so liga os
//! comandos do cadcraft ao `eletrico_data`. A interface correspondente e
//! `crates/ui-egui/src/eletrico.rs`.

use std::any::Any;

use serde_json::{Value, json};

use super::{CommandSpec, always, bad, str_param};
use crate::domain::Domain;
use crate::{Result, Session};

mod desenho;

use desenho::{run_circuito_listar, run_circuito_mostrar, run_circuito_renomear, run_pagina_escrever, run_pagina_ler, run_pagina_sincronizar, run_projeto_gravar, run_projeto_ler, run_revisao_completa_gravar, run_revisao_completa_ler, run_revisao_gravar, run_revisao_ler,
    run_exportar_exportar, run_exportar_importar, run_exportar_remover, run_revisao_automatica_gravar, run_revisao_automatica_ler, run_titulos_textos_substituir, run_xdata_apagar, run_xdata_listar};

/// O banco que o desenho aponta, para a interface.
pub use desenho::banco_do_desenho;

/// O modulo como o motor o ve: seus comandos e o estado da sessao.
pub struct Modulo;

impl Domain for Modulo {
    fn id(&self) -> &'static str {
        "eletrico"
    }
    fn commands(&self) -> Vec<CommandSpec> {
        specs()
    }
    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(())
    }
}

/// O modulo registrado em [`crate::domain::builtin`].
pub static ELETRICO: Modulo = Modulo;

fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("ele.status", "Elétrico: Estado do Módulo", run_status)
            .menu(&["Elétrico", "Diagnóstico", "Estado do Módulo"])
            .params("{} -> {modulo, xdata, schema, comandos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.banco.abrir", "Elétrico: Abrir Banco", run_banco_abrir)
            .menu(&["Elétrico", "Arquivo", "Abrir Banco..."])
            .params("{arquivo} -> {arquivo, schema, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.pagina.gravar", "Elétrico: Gravar Página", run_pagina_gravar)
            .menu(&["Elétrico", "Páginas", "Gravar Página"])
            .params("{arquivo, pagina, titulo1?, titulo2?, titulo3?, titulo4?, unidade?, alternativo?} -> {pagina, pg_num, pg_alfa, pg_dec, total}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.pagina.listar", "Elétrico: Listar Páginas", run_pagina_listar)
            .menu(&["Elétrico", "Páginas", "Listar Páginas"])
            .params("{arquivo, limite?} -> {arquivo, total, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.projeto.gravar", "Elétrico: Gravar Vínculo do Projeto", run_projeto_gravar)
            .menu(&["Elétrico", "Projeto", "Gravar Vínculo..."])
            .params("{banco} -> {banco, handle, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.projeto.ler", "Elétrico: Ler Vínculo do Projeto", run_projeto_ler)
            .menu(&["Elétrico", "Projeto", "Ler Vínculo"])
            .params("{} -> {banco, encontrado, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.pagina.escrever", "Elétrico: Gravar Página no Desenho", run_pagina_escrever)
            .menu(&["Elétrico", "Páginas", "Gravar no Desenho"])
            .params("{pagina, titulo1?..titulo4?, unidade?, alternativo?, x?, y?, altura?, autor?, quando?, banco?} -> {pagina, criados, atualizados, removidos, banco, gravado_no_banco}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.pagina.ler", "Elétrico: Ler Páginas do Desenho", run_pagina_ler)
            .menu(&["Elétrico", "Páginas", "Ler do Desenho"])
            .params("{pagina?} -> {total, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.pagina.sincronizar", "Elétrico: Sincronizar Banco com o Desenho", run_pagina_sincronizar)
            .menu(&["Elétrico", "Páginas", "Sincronizar Banco"])
            .params("{banco?} -> {banco, gravadas, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.revisao.ler", "Elétrico: Ler Revisões do Desenho", run_revisao_ler)
            .menu(&["Elétrico", "Revisão", "Ler Revisões"])
            .params("{} -> {revisoes, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.revisao.gravar", "Elétrico: Gravar Revisões no Desenho", run_revisao_gravar)
            .menu(&["Elétrico", "Revisão", "Gravar Revisões"])
            .params("{revisoes} -> {atualizados, ignorados, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.revisao_completa.ler", "Elétrico: Ler Revisões Completas", run_revisao_completa_ler)
            .menu(&["Elétrico", "Revisão", "Ler Revisão Completa"])
            .params("{} -> {revisoes, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.revisao_completa.gravar", "Elétrico: Gravar Revisões Completas", run_revisao_completa_gravar)
            .menu(&["Elétrico", "Revisão", "Gravar Revisão Completa"])
            .params("{revisoes} -> {atualizados, ignorados, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.revisao_automatica.ler", "Elétrico: Ler Revisões Automáticas", run_revisao_automatica_ler)
            .menu(&["Elétrico", "Revisão", "Ler Revisões Automáticas"])
            .params("{} -> {revisoes, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.revisao_automatica.gravar", "Elétrico: Gravar Revisão Automática", run_revisao_automatica_gravar)
            .menu(&["Elétrico", "Revisão", "Gravar Revisão Automática"])
            .params("{revisao_antiga, revisao?, cor?, preencher?} -> {atualizados, titulos, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.exportar.exportar", "Elétrico: Exportar", run_exportar_exportar)
            .menu(&["Elétrico", "Exportar/Importar", "Exportar"])
            .params("{} -> {tsv, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.exportar.importar", "Elétrico: Importar", run_exportar_importar)
            .menu(&["Elétrico", "Exportar/Importar", "Importar"])
            .params("{tsv} -> {importados, erros, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.exportar.remover", "Elétrico: Remover Códigos de Exportação", run_exportar_remover)
            .menu(&["Elétrico", "Exportar/Importar", "Remover Códigos"])
            .params("{} -> {removidos, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.xdata.listar", "Elétrico: Listar XDatas", run_xdata_listar)
            .menu(&["Elétrico", "Projeto", "Listar XDatas"])
            .params("{incluir_conhecidos?} -> {xdatas, total, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.xdata.apagar", "Elétrico: Apagar XDatas", run_xdata_apagar)
            .menu(&["Elétrico", "Projeto", "Apagar XDatas"])
            .params("{apps} -> {apagados, apps, xdata}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.titulos_textos.substituir", "Elétrico: Editar Títulos (Textos)", run_titulos_textos_substituir)
            .menu(&["Elétrico", "Páginas", "Editar Títulos (Textos)"])
            .params("{paginas, titulo1?, titulo2?, titulo3?, titulo4?, texto?, mtexto?, de, para} -> {alterados, examinados, paginas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.circuito.listar", "Elétrico: Listar Circuitos", run_circuito_listar)
            .menu(&["Elétrico", "Circuitos", "Listar Circuitos"])
            .params("{} -> {paineis}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.circuito.renomear", "Elétrico: Renomear Circuito", run_circuito_renomear)
            .menu(&["Elétrico", "Circuitos", "Renomear Circuito"])
            .params("{potencial, nome} -> {potencial, nome, textos, linhas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("ele.circuito.mostrar", "Elétrico: Mostrar Circuito", run_circuito_mostrar)
            .menu(&["Elétrico", "Circuitos", "Mostrar Circuito"])
            .params("{handle} -> {handle, selecionado}")
            .enabled(always)
            .noundo(),
    ]
}

/// Abre (ou cria) o `.db` pedido no comando.
fn banco_do_comando(cmd: &str, p: &Value) -> Result<(String, eletrico_data::Banco)> {
    let arquivo = str_param(p, "arquivo")
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| bad(cmd, "informe o arquivo .db em `arquivo`"))?;
    let banco = eletrico_data::Banco::abrir(arquivo).map_err(|e| bad(cmd, e.to_string()))?;
    Ok((arquivo.to_string(), banco))
}

/// Campo de texto do comando, sem espacos nas pontas.
fn campo(p: &Value, chave: &str) -> String {
    str_param(p, chave).unwrap_or_default().trim().to_string()
}

fn pagina_para_json(pagina: &eletrico_data::Pagina) -> Value {
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
        "modulo": "eletrico",
        "xdata": eletrico_data::XDATA_PREFIXO,
        "schema": eletrico_data::SCHEMA_VERSAO,
        "comandos": [
            "ele.status",
            "ele.banco.abrir",
            "ele.pagina.gravar",
            "ele.pagina.listar",
            "ele.projeto.gravar",
            "ele.projeto.ler",
            "ele.pagina.escrever",
            "ele.pagina.ler",
            "ele.pagina.sincronizar",
            "ele.revisao_completa.ler",
            "ele.revisao_completa.gravar",
            "ele.revisao_automatica.ler",
            "ele.revisao_automatica.gravar",
            "ele.exportar.exportar",
            "ele.exportar.importar",
            "ele.exportar.remover",
            "ele.xdata.listar",
            "ele.xdata.apagar",
            "ele.titulos_textos.substituir",
            "ele.circuito.listar",
            "ele.circuito.renomear",
            "ele.circuito.mostrar",
        ],
    }))
}

fn run_banco_abrir(_s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "ele.banco.abrir";
    let (arquivo, banco) = banco_do_comando(cmd, p)?;
    let schema = banco.versao().map_err(|e| bad(cmd, e.to_string()))?;
    let paginas = banco.paginas().map_err(|e| bad(cmd, e.to_string()))?.len();
    Ok(json!({ "arquivo": arquivo, "schema": schema, "paginas": paginas }))
}

fn run_pagina_gravar(_s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "ele.pagina.gravar";
    let (arquivo, banco) = banco_do_comando(cmd, p)?;
    let chave = campo(p, "pagina");
    if chave.is_empty() {
        return Err(bad(cmd, "informe a página (nome da camada) em `pagina`"));
    }
    let mut pagina = eletrico_data::Pagina::nova(&chave);
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
    let cmd = "ele.pagina.listar";
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
        assert_eq!(specs.len(), 24);
        let mut ids: Vec<&str> = specs.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), specs.len(), "ids repetidos no catalogo");
        for id in ids {
            assert!(id.starts_with("ele."), "{id} fora do prefixo");
        }
    }

    #[test]
    fn pagina_do_comando_usa_o_core() {
        let pagina = eletrico_data::Pagina::nova("12A");
        assert_eq!(pagina.pg_num, "12");
        assert_eq!(pagina.pg_alfa, "A");
        assert!((pagina.pg_dec - 12.65).abs() < 1e-9);
    }
}
