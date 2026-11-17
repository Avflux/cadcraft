//! Operações diretas sobre o banco (importar, exportar, atribuir, mover, apagar, consultar).

use super::*;

pub(super) fn calcular(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.calcular", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "topologia: {} barra(s), {} ilha(s), {} isolada(s), grau máx {}",
            v.get("barras").and_then(Value::as_i64).unwrap_or(0),
            v.get("ilhas").and_then(Value::as_i64).unwrap_or(0),
            v.get("barras_isoladas").and_then(Value::as_i64).unwrap_or(0),
            v.get("grau_max").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn importar_lst(app: &mut CadApp, db: &str, arquivo: &str) -> String {
    if arquivo.trim().is_empty() {
        return "informe o caminho do .lst".to_string();
    }
    match app.run("anacad", json!({ "import": arquivo, "db": db })) {
        Ok(v) => format!(
            "LST: {} barras, {} linhas, {} ligações",
            v.pointer("/banco/barras").and_then(Value::as_i64).unwrap_or(0),
            v.pointer("/banco/linhas").and_then(Value::as_i64).unwrap_or(0),
            v.pointer("/banco/ligacoes").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn importar_ana(app: &mut CadApp, db: &str, documento: i64, arquivo: &str) -> String {
    if arquivo.trim().is_empty() {
        return "informe o caminho do .ANA".to_string();
    }
    match app.run("anacad.ana.importar", json!({ "arquivo": arquivo, "db": db, "documento": documento })) {
        Ok(v) => format!(
            "ANA: {} barras, {} circuitos, {} shunts, {} geradores",
            v.get("barras").and_then(Value::as_i64).unwrap_or(0),
            v.get("circuitos").and_then(Value::as_i64).unwrap_or(0),
            v.get("shunts").and_then(Value::as_i64).unwrap_or(0),
            v.get("geradores").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn atribuir_barra(app: &mut CadApp, db: &str, documento: i64, handle: &str, codigo: i64) -> String {
    match app.run("anacad.barra.atribuir", json!({ "db": db, "documento": documento, "handle": handle, "codigo": codigo })) {
        Ok(v) => format!("barra atribuída: #{} (handle {})", v.get("barra").and_then(Value::as_i64).unwrap_or(0), handle),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn atribuir_linha(app: &mut CadApp, db: &str, documento: i64, handle: &str, de: i64, para: i64) -> String {
    match app.run("anacad.linha.atribuir", json!({ "db": db, "documento": documento, "handle": handle, "de": de, "para": para })) {
        Ok(v) => format!("linha atribuída: #{} (handle {})", v.get("linha").and_then(Value::as_i64).unwrap_or(0), handle),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn montar_ybus(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.ybus", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "Ybus: {} barras, {} circuitos, {} pares, simétrica={}",
            v.get("barras").and_then(Value::as_i64).unwrap_or(0),
            v.get("circuitos_usados").and_then(Value::as_i64).unwrap_or(0),
            v.get("pares_nao_nulos").and_then(Value::as_i64).unwrap_or(0),
            v.get("simetrica").and_then(Value::as_bool).unwrap_or(false)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn gerar_relatorio(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.relatorio", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "relatório: {} ({} bytes)",
            v.get("arquivo").and_then(Value::as_str).unwrap_or("-"),
            v.get("bytes").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn listar_resultados(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.resultados", json!({ "db": db, "documento": documento, "limite": 5 })) {
        Ok(v) => {
            let lista = v.get("resultados").and_then(Value::as_array);
            match lista {
                Some(itens) if !itens.is_empty() => itens
                    .iter()
                    .map(|r| {
                        format!("#{} {}", r.get("id").and_then(Value::as_i64).unwrap_or(0), r.get("tipo").and_then(Value::as_str).unwrap_or("-"))
                    })
                    .collect::<Vec<_>>()
                    .join(" · "),
                _ => "nenhum resultado".to_string(),
            }
        }
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn calcular_comprimentos(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.comprimento", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "comprimentos: ligações {:.1}, linhas {:.1}, total {:.1}",
            v.get("comprimento_ligacoes").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("comprimento_linhas").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("comprimento_total").and_then(Value::as_f64).unwrap_or(0.0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn criar_legenda(app: &mut CadApp, db: &str, documento: i64, numero: i64, x: f64, y: f64) -> String {
    match app.run("anacad.legenda.criar", json!({ "db": db, "documento": documento, "tipo": "X", "numero": numero, "x": x, "y": y })) {
        Ok(v) => format!(
            "legenda {} criada (handle {})",
            v.get("legenda").and_then(Value::as_i64).unwrap_or(0),
            v.get("handle").and_then(Value::as_str).unwrap_or("-")
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn mover_componente(app: &mut CadApp, db: &str, handle: &str, dx: f64, dy: f64) -> String {
    match app.run("anacad.mover", json!({ "db": db, "handle": handle, "dx": dx, "dy": dy })) {
        Ok(v) => format!(
            "componente movido para ({}, {})",
            v.get("x").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("y").and_then(Value::as_f64).unwrap_or(0.0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn apagar_componente(app: &mut CadApp, db: &str, handle: &str) -> String {
    match app.run("anacad.apagar", json!({ "db": db, "handle": handle })) {
        Ok(v) => format!("apagado ({} linha(s))", v.get("removidos").and_then(Value::as_i64).unwrap_or(0)),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn criar_barra(app: &mut CadApp, db: &str, documento: i64, codigo: i64, x: f64, y: f64, x2: f64, y2: f64) -> String {
    match app.run("anacad.barra.criar", json!({ "db": db, "documento": documento, "codigo": codigo, "x": x, "y": y, "x2": x2, "y2": y2 })) {
        Ok(v) => format!(
            "barra {} criada (handle {})",
            v.get("barra").and_then(Value::as_i64).unwrap_or(0),
            v.get("handle").and_then(Value::as_str).unwrap_or("-")
        ),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn consultar(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.consultar", json!({ "db": db, "documento": documento })) {
        Ok(v) => resumir(&v),
        Err(e) => format!("erro: {e}"),
    }
}

pub(super) fn carregar_componentes_barra(app: &mut CadApp, db: &str, documento: i64, codigo: i64) -> Vec<(i64, String, bool)> {
    match app.run("anacad.barra.componentes", json!({"db": db, "documento": documento, "codigo": codigo})) {
        Ok(v) => v
            .get("componentes")
            .and_then(Value::as_array)
            .map(|itens| itens.iter().filter_map(|c| Some((c.get("id")?.as_i64()?, c.get("nome")?.as_str()?.to_string(), false))).collect())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(super) fn carregar_terminais(app: &mut CadApp, db: &str, documento: i64) -> Vec<(i64, String, f64, f64)> {
    match app.run("anacad.terminal.listar", json!({"db": db, "documento": documento})) {
        Ok(v) => v
            .get("terminais")
            .and_then(Value::as_array)
            .map(|itens| {
                itens
                    .iter()
                    .filter_map(|t| {
                        Some((
                            t.get("id")?.as_i64()?,
                            t.get("nome")?.as_str()?.to_string(),
                            t.get("x").and_then(Value::as_f64).unwrap_or(0.0),
                            t.get("y").and_then(Value::as_f64).unwrap_or(0.0),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Janela `atribuir_barra | atribuir_linha | mover | apagar` do diálogo AnaCAD.
pub(super) fn janela_por_handle(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, acao: &str) {
    ui.horizontal(|ui| {
        ui.label("handle");
        ui.add(egui::TextEdit::singleline(&mut estado.comp_handle).desired_width(90.0));
    });
    if acao == "atribuir_barra" {
        ui.horizontal(|ui| {
            ui.label("código");
            ui.add(egui::DragValue::new(&mut estado.barra_codigo).speed(1.0));
        });
        if ui.button("Atribuir barra").clicked() {
            estado.mensagem = atribuir_barra(app, &estado.db, estado.documento, &estado.comp_handle, estado.barra_codigo);
        }
    } else if acao == "atribuir_linha" {
        ui.horizontal(|ui| {
            ui.label("de");
            ui.add(egui::DragValue::new(&mut estado.linha_de).speed(1.0));
            ui.label("para");
            ui.add(egui::DragValue::new(&mut estado.linha_para).speed(1.0));
        });
        if ui.button("Atribuir linha").clicked() {
            estado.mensagem = atribuir_linha(app, &estado.db, estado.documento, &estado.comp_handle, estado.linha_de, estado.linha_para);
        }
    } else if acao == "mover" {
        ui.horizontal(|ui| {
            ui.label("dx");
            ui.add(egui::DragValue::new(&mut estado.comp_dx).speed(1.0));
            ui.label("dy");
            ui.add(egui::DragValue::new(&mut estado.comp_dy).speed(1.0));
        });
        if ui.button("Mover").clicked() {
            estado.mensagem = mover_componente(app, &estado.db, &estado.comp_handle, estado.comp_dx, estado.comp_dy);
        }
    } else if ui.button("Apagar").clicked() {
        estado.mensagem = apagar_componente(app, &estado.db, &estado.comp_handle);
    }
}

/// Janela `importar_lst | importar_ana` do diálogo AnaCAD.
pub(super) fn janela_importar(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, acao: &str) {
    let arquivo = if acao == "importar_lst" { &mut estado.lst_arquivo } else { &mut estado.ana_arquivo };
    ui.horizontal(|ui| {
        ui.label("arquivo");
        ui.add(egui::TextEdit::singleline(arquivo).desired_width(320.0));
    });
    if ui.button("Importar").clicked() {
        estado.mensagem = if acao == "importar_lst" {
            importar_lst(app, &estado.db, &estado.lst_arquivo)
        } else {
            importar_ana(app, &estado.db, estado.documento, &estado.ana_arquivo)
        };
    }
}

/// Janela `exportar_ana` do diálogo AnaCAD.
pub(super) fn janela_exportar_ana(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.horizontal(|ui| {
        ui.label("saída .ANA");
        ui.add(egui::TextEdit::singleline(&mut estado.ana_arquivo).desired_width(320.0));
    });
    if ui.button("Salvar .ANA").clicked() {
        estado.mensagem =
            match app.run("anacad.ana.exportar", json!({ "db": estado.db, "documento": estado.documento, "arquivo": estado.ana_arquivo })) {
                Ok(_) => "arquivo .ANA gravado".to_string(),
                Err(e) => format!("erro: {e}"),
            };
    }
}

/// Janela `exportar` do diálogo AnaCAD.
pub(super) fn janela_exportar(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.horizontal(|ui| {
        ui.label("saída .lst");
        ui.add(egui::TextEdit::singleline(&mut estado.saida_arquivo).desired_width(320.0));
    });
    if ui.button("Exportar").clicked() {
        estado.mensagem = match app.run("anacad.exportar", json!({ "db": estado.db, "documento": estado.documento, "saida": estado.saida_arquivo })) {
            Ok(_) => "tela exportada".to_string(),
            Err(e) => format!("erro: {e}"),
        };
    }
}

/// Janela `_` do diálogo AnaCAD.
pub(super) fn janela_executar(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, acao: &str) {
    if ui.button("Executar").clicked() {
        estado.mensagem = match acao {
            "consultar" => consultar(app, &estado.db, estado.documento),
            "topologia" => calcular(app, &estado.db, estado.documento),
            "comprimentos" => calcular_comprimentos(app, &estado.db, estado.documento),
            "ybus" => montar_ybus(app, &estado.db, estado.documento),
            "resultados" => listar_resultados(app, &estado.db, estado.documento),
            "relatorio" => gerar_relatorio(app, &estado.db, estado.documento),
            _ => String::new(),
        };
    }
}
