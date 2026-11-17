//! Banco e documentos do projeto: criar, abrir, listar e exportar a tela.

use super::*;

pub(in crate::cmd::anacad) fn run_banco_criar(s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.criar", "`arquivo` is required"))?;
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.criar", format!("{arquivo}: {e}")))?;
    // O banco aberto vale para os comandos seguintes (e para a cascata do apagar).
    set_banco(s, arquivo, 1);
    resumo_banco(&banco, arquivo)
}

pub(in crate::cmd::anacad) fn run_banco_abrir(s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.abrir", "`arquivo` is required"))?;
    if !std::path::Path::new(arquivo).exists() {
        return Err(bad("anacad.banco.abrir", format!("banco nao encontrado: {arquivo}")));
    }
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.abrir", format!("{arquivo}: {e}")))?;
    set_banco(s, arquivo, banco.documentos().map(|d| d.first().map(|d| d.id).unwrap_or(1)).unwrap_or(1));
    resumo_banco(&banco, arquivo)
}

pub(in crate::cmd::anacad) fn run_documento_listar(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_documento_criar(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_exportar(_s: &mut Session, p: &Value) -> Result<Value> {
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
