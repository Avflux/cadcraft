//! Painel AnaCAD: consulta e edição rápida do banco de projeto (`.db`).
//!
//! É um painel **dockado** do cadcraft (não uma janela flutuante): entra na
//! mesma prancheta de Layers/Properties, abre pelo menu `AnaCAD ▸ Painel`, pelo
//! botão da barra de ferramentas ou por `ui.anacad`.
//!
//! A UI é fina: os botões chamam os comandos do motor (`app.run`) e o estado da
//! tela (caminho do banco, documento, último resultado) vive na memória do egui.

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::CadApp;

const ID_ESTADO: &str = "anacad.estado";

#[derive(Clone)]
struct Estado {
    aba: String,
    db: String,
    documento: i64,
    resumo: String,
    barra_codigo: i64,
    barra_x: f64,
    barra_y: f64,
    linha_de: i64,
    linha_para: i64,
    legenda_numero: i64,
    legenda_x: f64,
    legenda_y: f64,
    legenda_handle: String,
    comp_handle: String,
    comp_dx: f64,
    comp_dy: f64,
    curto_barra: i64,
    curto_ref: i64,
    curto_tipo: String,
    ana_arquivo: String,
    lst_arquivo: String,
    mensagem: String,
}

impl Default for Estado {
    fn default() -> Self {
        Estado {
            aba: "Dados".into(),
            db: String::new(),
            documento: 1,
            resumo: String::from("Informe o caminho do .db e clique em Consultar."),
            barra_codigo: 0,
            barra_x: 0.0,
            barra_y: 0.0,
            linha_de: 0,
            linha_para: 0,
            legenda_numero: 0,
            legenda_x: 0.0,
            legenda_y: 0.0,
            legenda_handle: String::new(),
            comp_handle: String::new(),
            comp_dx: 0.0,
            comp_dy: 0.0,
            curto_barra: 0,
            curto_ref: 0,
            curto_tipo: "trifasica".into(),
            ana_arquivo: String::new(),
            lst_arquivo: String::new(),
            mensagem: String::new(),
        }
    }
}

/// Abre (ou ativa) o painel dockado, como o QuickCalc faz.
pub fn open(app: &mut CadApp) -> Result<Value, String> {
    let panel = crate::docking::Panel::Anacad;
    crate::docking::command(app, false, &json!({ "operation": "open", "panel": panel.id() }))
}

/// Conteúdo do painel (chamado pelo dock).
pub fn panel(app: &mut CadApp, ui: &mut Ui) {
    let mut estado: Estado = ui.ctx().data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    if app.ui.anacad_tab != estado.aba {
        estado.aba = app.ui.anacad_tab.clone();
    }

    egui::ScrollArea::vertical().id_salt("anacad-conteudo").auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        ui.horizontal(|ui| {
            ui.label("Banco .db:");
            ui.add(egui::TextEdit::singleline(&mut estado.db).desired_width(f32::INFINITY));
        });
        ui.horizontal(|ui| {
            ui.label("Documento:");
            ui.add(egui::DragValue::new(&mut estado.documento).range(0..=i64::MAX));
            if ui.button("Consultar").clicked() {
                estado.resumo = consultar(app, &estado.db, estado.documento);
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            for aba in ["Dados", "Arquivo", "Edição", "Cálculos"] {
                if ui.selectable_label(estado.aba == aba, aba).clicked() {
                    estado.aba = aba.to_string();
                    app.ui.anacad_tab = estado.aba.clone();
                }
            }
        });
        ui.separator();

        match estado.aba.as_str() {
            "Arquivo" => {
                ui.label("Importar .LST:");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut estado.lst_arquivo).desired_width(300.0).hint_text("caminho do .lst"));
                    if ui.button("Importar .LST").clicked() {
                        estado.mensagem = importar_lst(app, &estado.db, &estado.lst_arquivo);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                });
                ui.label("Importar .ANA:");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut estado.ana_arquivo).desired_width(300.0).hint_text("caminho do .ANA"));
                    if ui.button("Importar .ANA").clicked() {
                        estado.mensagem = importar_ana(app, &estado.db, estado.documento, &estado.ana_arquivo);
                    }
                });
                ui.separator();
                if ui.button("Gerar relatório Markdown").clicked() {
                    estado.mensagem = gerar_relatorio(app, &estado.db, estado.documento);
                }
            }
            "Edição" => {
                ui.label("Nova barra:");
                ui.horizontal(|ui| {
                    ui.label("código");
                    ui.add(egui::DragValue::new(&mut estado.barra_codigo).speed(1.0));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut estado.barra_x).speed(1.0));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut estado.barra_y).speed(1.0));
                    if ui.button("Criar barra").clicked() {
                        estado.mensagem = criar_barra(app, &estado.db, estado.documento, estado.barra_codigo, estado.barra_x, estado.barra_y);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                });
                ui.label("Nova linha:");
                ui.horizontal(|ui| {
                    ui.label("de");
                    ui.add(egui::DragValue::new(&mut estado.linha_de).speed(1.0));
                    ui.label("para");
                    ui.add(egui::DragValue::new(&mut estado.linha_para).speed(1.0));
                    if ui.button("Criar linha").clicked() {
                        estado.mensagem = criar_linha(app, &estado.db, estado.documento, estado.linha_de, estado.linha_para);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                });
                ui.separator();
                ui.label("Nova legenda (X):");
                ui.horizontal(|ui| {
                    ui.label("número");
                    ui.add(egui::DragValue::new(&mut estado.legenda_numero).speed(1.0));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut estado.legenda_x).speed(1.0));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut estado.legenda_y).speed(1.0));
                    if ui.button("Criar legenda").clicked() {
                        estado.mensagem = criar_legenda(app, &estado.db, estado.documento, estado.legenda_numero, estado.legenda_x, estado.legenda_y);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("handle");
                    ui.add(egui::TextEdit::singleline(&mut estado.legenda_handle).desired_width(90.0));
                    ui.label("dx");
                    ui.add(egui::DragValue::new(&mut estado.comp_dx).speed(1.0));
                    ui.label("dy");
                    ui.add(egui::DragValue::new(&mut estado.comp_dy).speed(1.0));
                    if ui.button("Mover legenda").clicked() {
                        estado.mensagem = mover_legenda(app, &estado.db, &estado.legenda_handle, estado.comp_dx, estado.comp_dy);
                    }
                });
                ui.separator();
                ui.label("Componente:");
                ui.horizontal(|ui| {
                    ui.label("handle");
                    ui.add(egui::TextEdit::singleline(&mut estado.comp_handle).desired_width(90.0));
                    ui.label("dx");
                    ui.add(egui::DragValue::new(&mut estado.comp_dx).speed(1.0));
                    ui.label("dy");
                    ui.add(egui::DragValue::new(&mut estado.comp_dy).speed(1.0));
                    if ui.button("Mover").clicked() {
                        estado.mensagem = mover_componente(app, &estado.db, &estado.comp_handle, estado.comp_dx, estado.comp_dy);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                    if ui.button("Apagar").clicked() {
                        estado.mensagem = apagar_componente(app, &estado.db, &estado.comp_handle);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                });
            }
            "Cálculos" => {
                ui.horizontal(|ui| {
                    if ui.button("Calcular topologia").clicked() {
                        estado.mensagem = calcular(app, &estado.db, estado.documento);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                    if ui.button("Calcular comprimentos").clicked() {
                        estado.mensagem = calcular_comprimentos(app, &estado.db, estado.documento);
                        estado.resumo = consultar(app, &estado.db, estado.documento);
                    }
                    if ui.button("Montar Ybus").clicked() {
                        estado.mensagem = montar_ybus(app, &estado.db, estado.documento);
                    }
                });
                ui.label("Curto-circuito (barra infinita):");
                ui.horizontal(|ui| {
                    ui.label("barra");
                    ui.add(egui::DragValue::new(&mut estado.curto_barra).speed(1.0));
                    ui.label("referência");
                    ui.add(egui::DragValue::new(&mut estado.curto_ref).speed(1.0));
                    ui.label("tipo");
                    egui::ComboBox::from_id_salt("anacad.curto.tipo").selected_text(&estado.curto_tipo).show_ui(ui, |ui| {
                        for tipo in ["trifasica", "bifasica", "monofasica"] {
                            ui.selectable_value(&mut estado.curto_tipo, tipo.to_string(), tipo);
                        }
                    });
                });
                ui.horizontal(|ui| {
                    if ui.button("Calcular curto").clicked() {
                        estado.mensagem = calcular_curto(app, &estado.db, estado.documento, estado.curto_barra, estado.curto_ref, &estado.curto_tipo);
                    }
                    if ui.button("Resultados").clicked() {
                        estado.mensagem = listar_resultados(app, &estado.db, estado.documento);
                    }
                });
            }
            _ => {
                ui.label("Consulta do banco:");
                ui.label("Informe o .db e clique em Consultar. Use as abas acima para importar, editar e calcular.");
            }
        }

        if !estado.mensagem.is_empty() {
            ui.separator();
            ui.label(&estado.mensagem);
        }
        ui.separator();
        ui.label("Resumo:");
        ui.monospace(&estado.resumo);
    });

    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

fn calcular(app: &mut CadApp, db: &str, documento: i64) -> String {
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

fn importar_lst(app: &mut CadApp, db: &str, arquivo: &str) -> String {
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

fn importar_ana(app: &mut CadApp, db: &str, documento: i64, arquivo: &str) -> String {
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

fn montar_ybus(app: &mut CadApp, db: &str, documento: i64) -> String {
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

fn calcular_curto(app: &mut CadApp, db: &str, documento: i64, barra: i64, referencia: i64, tipo: &str) -> String {
    match app.run("anacad.curto", json!({ "db": db, "documento": documento, "barra": barra, "referencia": referencia, "tipo": tipo })) {
        Ok(v) => format!(
            "curto {} em {}: Zth = {:.5}+j{:.5}, {:.2} pu, {:.2} kA",
            tipo,
            v.get("barra").and_then(Value::as_i64).unwrap_or(0),
            v.get("zth_re").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("zth_im").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("i_pu").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("i_ka").and_then(Value::as_f64).unwrap_or(0.0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn gerar_relatorio(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.relatorio", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "relatório: {} ({} bytes)",
            v.get("arquivo").and_then(Value::as_str).unwrap_or("-"),
            v.get("bytes").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn listar_resultados(app: &mut CadApp, db: &str, documento: i64) -> String {
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

fn calcular_comprimentos(app: &mut CadApp, db: &str, documento: i64) -> String {
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

fn criar_legenda(app: &mut CadApp, db: &str, documento: i64, numero: i64, x: f64, y: f64) -> String {
    match app.run("anacad.legenda.criar", json!({ "db": db, "documento": documento, "tipo": "X", "numero": numero, "x": x, "y": y })) {
        Ok(v) => format!(
            "legenda {} criada (handle {})",
            v.get("legenda").and_then(Value::as_i64).unwrap_or(0),
            v.get("handle").and_then(Value::as_str).unwrap_or("-")
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn mover_legenda(app: &mut CadApp, db: &str, handle: &str, dx: f64, dy: f64) -> String {
    match app.run("anacad.legenda.mover", json!({ "db": db, "handle": handle, "dx": dx, "dy": dy })) {
        Ok(v) => format!(
            "legenda movida para ({}, {})",
            v.get("x").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("y").and_then(Value::as_f64).unwrap_or(0.0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn mover_componente(app: &mut CadApp, db: &str, handle: &str, dx: f64, dy: f64) -> String {
    match app.run("anacad.mover", json!({ "db": db, "handle": handle, "dx": dx, "dy": dy })) {
        Ok(v) => format!(
            "componente movido para ({}, {})",
            v.get("x").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("y").and_then(Value::as_f64).unwrap_or(0.0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn apagar_componente(app: &mut CadApp, db: &str, handle: &str) -> String {
    match app.run("anacad.apagar", json!({ "db": db, "handle": handle })) {
        Ok(v) => format!("apagado ({} linha(s))", v.get("removidos").and_then(Value::as_i64).unwrap_or(0)),
        Err(e) => format!("erro: {e}"),
    }
}

fn criar_barra(app: &mut CadApp, db: &str, documento: i64, codigo: i64, x: f64, y: f64) -> String {
    match app.run("anacad.barra.criar", json!({ "db": db, "documento": documento, "codigo": codigo, "x": x, "y": y })) {
        Ok(v) => format!(
            "barra {} criada (handle {})",
            v.get("barra").and_then(Value::as_i64).unwrap_or(0),
            v.get("handle").and_then(Value::as_str).unwrap_or("-")
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn criar_linha(app: &mut CadApp, db: &str, documento: i64, de: i64, para: i64) -> String {
    match app.run("anacad.linha.criar", json!({ "db": db, "documento": documento, "de": de, "para": para })) {
        Ok(v) => format!(
            "linha {} criada (handle {})",
            v.get("linha").and_then(Value::as_i64).unwrap_or(0),
            v.get("handle").and_then(Value::as_str).unwrap_or("-")
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn consultar(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.consultar", json!({ "db": db, "documento": documento })) {
        Ok(v) => resumir(&v),
        Err(e) => format!("erro: {e}"),
    }
}

fn resumir(v: &Value) -> String {
    let mut s = String::new();
    if let Some(docs) = v.get("documentos").and_then(Value::as_array) {
        s.push_str("Documentos:\n");
        for d in docs {
            let id = d.get("id").and_then(Value::as_i64).unwrap_or(0);
            let nome = d.get("nome").and_then(Value::as_str).unwrap_or("");
            s.push_str(&format!("  {id} - {nome}\n"));
        }
    }
    if let Some(c) = v.get("contagens") {
        s.push_str("Contagens: ");
        for chave in ["barras", "componentes", "linhas", "ligacoes", "legendas"] {
            if let Some(n) = c.get(chave).and_then(Value::as_i64) {
                s.push_str(&format!("{chave}={n} "));
            }
        }
        s.push('\n');
    }
    if let Some(comps) = v.get("componentes").and_then(Value::as_array) {
        s.push_str(&format!("Componentes ({} listados):\n", comps.len()));
        for c in comps.iter().take(12) {
            let id = c.get("id").and_then(Value::as_i64).unwrap_or(0);
            let tipo = c.get("tipo").and_then(Value::as_i64).unwrap_or(0);
            let x = c.get("x").and_then(Value::as_f64).unwrap_or(0.0);
            let y = c.get("y").and_then(Value::as_f64).unwrap_or(0.0);
            let h = c.get("handle").and_then(Value::as_str).unwrap_or("-");
            s.push_str(&format!("  #{id} tipo {tipo} em ({x:.0},{y:.0}) handle {h}\n"));
        }
    }
    if s.is_empty() {
        s.push_str("sem dados");
    }
    s
}
