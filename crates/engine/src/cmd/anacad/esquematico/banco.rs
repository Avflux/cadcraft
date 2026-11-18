//! Banco e documentos do projeto: criar, abrir, listar e exportar a tela.

use super::*;

pub(in crate::cmd::anacad) fn run_banco_criar(s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.criar", "`arquivo` is required"))?;
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.criar", format!("{arquivo}: {e}")))?;
    // O banco aberto vale para os comandos seguintes (e para a cascata do apagar).
    set_banco(s, arquivo, 1);
    Ok(resumo_banco(&banco, arquivo)?)
}

pub(in crate::cmd::anacad) fn run_banco_abrir(s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.abrir", "`arquivo` is required"))?;
    if !std::path::Path::new(arquivo).exists() {
        return Err(bad("anacad.banco.abrir", format!("banco nao encontrado: {arquivo}")));
    }
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.abrir", format!("{arquivo}: {e}")))?;
    set_banco(s, arquivo, banco.documentos().map(|d| d.first().map(|d| d.id).unwrap_or(1)).unwrap_or(1));
    Ok(resumo_banco(&banco, arquivo)?)
}
