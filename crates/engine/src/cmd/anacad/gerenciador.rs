//! Gerenciador de Dados: a árvore de dados do caso (recurso 333/336 do ANAFAS)
//! lida para a janela de grade.
//!
//! A fonte é sempre o banco conectado; o desenho não entra aqui.
//! `anacad.dados.tabelas` dá os totais de cada item e `anacad.dados.tabela`
//! devolve as colunas (os campos da janela `Dados de …` do elemento) e as linhas
//! de um item: Barra, Linha, Transformador, CapacitorSérie, Gerador,
//! GeradorEólico, ShuntBarra, ShuntLinha, TrafoAterramento, CompensadorEstático,
//! Mutua, Carga e Área.

use super::*;

pub(super) fn run_dados_tabelas(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.dados.tabelas", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.dados.tabelas", format!("{db}: {e}")))?;
    let mut itens = Vec::new();
    for item in anacad_data::ITENS_GERENCIADOR {
        let total =
            banco.gerenciador(item, documento.unwrap_or(1), 5_000).map(|t| t.linhas.len()).map_err(|e| bad("anacad.dados.tabelas", e.to_string()))?;
        itens.push(json!({ "item": item, "total": total }));
    }
    Ok(json!({ "db": db, "documento": documento, "itens": itens }))
}

pub(super) fn run_dados_tabela(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.dados.tabela", "`db` is required"))?;
    let item = str_param(p, "item").ok_or_else(|| bad("anacad.dados.tabela", "`item` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let limite = p.get("limite").and_then(Value::as_i64).unwrap_or(500);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.dados.tabela", format!("{db}: {e}")))?;
    let tabela = banco.gerenciador(item, documento, limite).map_err(|e| bad("anacad.dados.tabela", e.to_string()))?;
    Ok(json!({
        "db": db,
        "item": tabela.nome,
        "colunas": tabela.colunas,
        "linhas": tabela.linhas,
    }))
}
