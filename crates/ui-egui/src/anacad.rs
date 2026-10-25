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
    db: String,
    documento: i64,
    barra_codigo: i64,
    barra_x: f64,
    barra_y: f64,
    barra_x2: f64,
    barra_y2: f64,
    linha_de: i64,
    linha_para: i64,
    legenda_numero: i64,
    legenda_x: f64,
    legenda_y: f64,
    comp_handle: String,
    comp_dx: f64,
    comp_dy: f64,
    curto_barra: i64,
    curto_ref: i64,
    curto_tipo: String,
    ana_arquivo: String,
    lst_arquivo: String,
    saida_arquivo: String,
    ed_nome: String,
    ed_area: String,
    ed_vbase: f64,
    ed_tipo: String,
    ed_capacidade: f64,
    ed_pre_mod: f64,
    ed_pre_ang: f64,
    ed_disjuntor: bool,
    ed_status: bool,
    ed_circuito: i64,
    ed_potencia: f64,
    ed_comprimento: f64,
    ed_r1: f64,
    ed_x1: f64,
    ed_s1: f64,
    ed_r0: f64,
    ed_x0: f64,
    ed_s0: f64,
    doc_pagina: String,
    doc_lista: Vec<(i64, String, String)>,
    doc_lista_db: String,
    mensagem: String,
}

impl Default for Estado {
    fn default() -> Self {
        Estado {
            db: String::new(),
            documento: 1,
            barra_codigo: 0,
            barra_x: 0.0,
            barra_y: 0.0,
            barra_x2: 0.0,
            barra_y2: 0.0,
            linha_de: 0,
            linha_para: 0,
            legenda_numero: 0,
            legenda_x: 0.0,
            legenda_y: 0.0,
            comp_handle: String::new(),
            comp_dx: 0.0,
            comp_dy: 0.0,
            curto_barra: 0,
            curto_ref: 0,
            curto_tipo: "trifasica".into(),
            ana_arquivo: String::new(),
            lst_arquivo: String::new(),
            saida_arquivo: String::new(),
            ed_nome: String::new(),
            ed_area: String::new(),
            ed_vbase: 0.0,
            ed_tipo: "Normal".into(),
            ed_capacidade: 0.0,
            ed_pre_mod: 1.0,
            ed_pre_ang: 0.0,
            ed_disjuntor: false,
            ed_status: true,
            ed_circuito: 0,
            ed_potencia: 0.0,
            ed_comprimento: 0.0,
            ed_r1: 0.0,
            ed_x1: 0.0,
            ed_s1: 0.0,
            ed_r0: 0.0,
            ed_x0: 0.0,
            ed_s0: 0.0,
            doc_pagina: String::new(),
            doc_lista: Vec::new(),
            doc_lista_db: String::new(),
            mensagem: String::new(),
        }
    }
}

/// Abre (ou ativa) o painel dockado, como o QuickCalc faz.
pub fn open(app: &mut CadApp) -> Result<Value, String> {
    let panel = crate::docking::Panel::Anacad;
    crate::docking::command(app, false, &json!({ "operation": "open", "panel": panel.id() }))
}

/// Conecta um banco (cria ou abre) e atualiza o status.
pub fn conectar_banco(app: &mut CadApp, cmd: &str, arquivo: &str) -> Result<Value, String> {
    let v = app.run(cmd, json!({ "arquivo": arquivo }))?;
    app.ui.anacad_db = arquivo.to_string();
    if let Some(doc) = v.get("primeiroDocumento").and_then(Value::as_i64) {
        app.ui.anacad_documento = doc;
    }
    let nome = v.get("nome").and_then(Value::as_str).unwrap_or(arquivo).to_string();
    app.status = Some((format!("Banco: {nome}"), crate::now_ms()));
    Ok(v)
}

/// Diálogos das ferramentas abertos pelo tool set.
pub fn dialogo(app: &mut CadApp, ctx: &egui::Context) {
    if app.ui.anacad_dialogo.is_empty() {
        return;
    }
    let acao = app.ui.anacad_dialogo.clone();
    let titulo = match acao.as_str() {
        "banco_novo" => "Novo banco",
        "banco_abrir" => "Abrir banco",
        "documento_novo" => "Novo documento",
        "criar_barra" => "Criar barra (LWPOLYLINE 0,3)",
        "criar_linha" => "Criar linha (LWPOLYLINE 0,0)",
        "criar_legenda" => "Criar legenda (X)",
        "editar_barra" => "Dados de Barra",
        "editar_linha" => "Dados de Linha de Transmissão",
        "atribuir_barra" => "Atribuir barra a polilinha",
        "atribuir_linha" => "Atribuir linha a polilinha",
        "mover" => "Mover por handle",
        "apagar" => "Apagar por handle",
        "importar_lst" => "Importar .LST",
        "importar_ana" => "Importar .ANA",
        "exportar" => "Exportar .LST",
        "curto" => "Curto-circuito",
        "consultar" => "Consultar banco",
        "topologia" => "Calcular topologia",
        "comprimentos" => "Calcular comprimentos",
        "ybus" => "Montar Ybus",
        "resultados" => "Resultados",
        "relatorio" => "Relatório",
        _ => return,
    };
    let mut estado: Estado = ctx.data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    estado.db = app.ui.anacad_db.clone();
    estado.documento = app.ui.anacad_documento;
    let mut aberto = true;
    egui::Window::new(titulo).id(egui::Id::new("anacad.dialogo")).collapsible(false).resizable(false).open(&mut aberto).show(ctx, |ui| {
        match acao.as_str() {
            "banco_novo" | "banco_abrir" => {
                let cmd = if acao == "banco_novo" { "anacad.banco.criar" } else { "anacad.banco.abrir" };
                let tem_picker = if acao == "banco_novo" { app.services.pick_db_save.is_some() } else { app.services.pick_db_open.is_some() };
                if !estado.db.trim().is_empty() {
                    ui.label("Selecionado:");
                    ui.monospace(&estado.db);
                }
                if tem_picker {
                    let rotulo = if acao == "banco_novo" { "Escolher pasta/nome e criar..." } else { "Escolher banco para abrir..." };
                    if ui.button(rotulo).clicked() {
                        let escolhido = if acao == "banco_novo" {
                            app.services.pick_db_save.as_ref().and_then(|f| f("anacad.db"))
                        } else {
                            app.services.pick_db_open.as_ref().and_then(|f| f())
                        };
                        if let Some(arquivo) = escolhido {
                            estado.db = arquivo.clone();
                            match conectar_banco(app, cmd, &arquivo) {
                                Ok(_) => estado.mensagem = "banco conectado".to_string(),
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Arquivo .db");
                        ui.add(egui::TextEdit::singleline(&mut estado.db).desired_width(340.0));
                    });
                    let rotulo = if acao == "banco_novo" { "Criar banco" } else { "Abrir banco" };
                    if ui.button(rotulo).clicked() {
                        match conectar_banco(app, cmd, &estado.db.clone()) {
                            Ok(_) => estado.mensagem = "banco conectado".to_string(),
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                if !estado.mensagem.is_empty() {
                    ui.separator();
                    ui.label(&estado.mensagem);
                }
            }
            "documento_novo" => {
                if estado.db.trim().is_empty() {
                    ui.label("Abra ou crie um banco primeiro.");
                } else {
                    if estado.doc_lista_db != estado.db {
                        estado.doc_lista = carregar_documentos(app, &estado.db);
                        estado.doc_lista_db = estado.db.clone();
                    }
                    if ui.button("Atualizar lista").clicked() {
                        estado.doc_lista = carregar_documentos(app, &estado.db);
                    }
                    ui.separator();
                    ui.label("Documentos existentes:");
                    let mut escolhido = None;
                    for (id, nome, pagina) in &estado.doc_lista {
                        let pagina = if pagina.is_empty() { String::new() } else { format!(" — pág. {pagina}") };
                        let rotulo = format!("#{id}  {nome}{pagina}");
                        if ui.selectable_label(estado.documento == *id, rotulo).clicked() {
                            escolhido = Some(*id);
                        }
                    }
                    if let Some(id) = escolhido {
                        estado.documento = id;
                        app.ui.anacad_documento = id;
                        estado.mensagem = format!("documento ativo: #{id}");
                    }
                    ui.separator();
                    ui.label("Novo documento:");
                    ui.horizontal(|ui| {
                        ui.label("Nome");
                        ui.add(egui::TextEdit::singleline(&mut estado.ed_nome).desired_width(200.0));
                        ui.label("Página");
                        ui.add(egui::TextEdit::singleline(&mut estado.doc_pagina).desired_width(70.0));
                    });
                    if ui.button("Criar documento").clicked() {
                        match app.run("anacad.documento.criar", json!({ "db": estado.db, "nome": estado.ed_nome, "pagina": estado.doc_pagina })) {
                            Ok(v) => {
                                if let Some(doc) = v.get("documento").and_then(Value::as_i64) {
                                    app.ui.anacad_documento = doc;
                                    estado.documento = doc;
                                }
                                estado.mensagem = format!("documento criado: {}", estado.ed_nome);
                                estado.doc_lista = carregar_documentos(app, &estado.db);
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                if !estado.mensagem.is_empty() {
                    ui.separator();
                    ui.label(&estado.mensagem);
                }
            }
            "criar_barra" => {
                ui.horizontal(|ui| {
                    ui.label("início x");
                    ui.add(egui::DragValue::new(&mut estado.barra_x).speed(1.0));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut estado.barra_y).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("fim x");
                    ui.add(egui::DragValue::new(&mut estado.barra_x2).speed(1.0));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut estado.barra_y2).speed(1.0));
                });
                if ui.button("Criar barra").clicked() {
                    let x2 = if estado.barra_x2 == 0.0 && estado.barra_y2 == 0.0 { estado.barra_x + 1.0 } else { estado.barra_x2 };
                    let y2 = if estado.barra_x2 == 0.0 && estado.barra_y2 == 0.0 { estado.barra_y } else { estado.barra_y2 };
                    estado.mensagem = criar_barra(app, &estado.db, estado.documento, 0, estado.barra_x, estado.barra_y, x2, y2);
                }
            }
            "criar_linha" => {
                ui.horizontal(|ui| {
                    ui.label("início x");
                    ui.add(egui::DragValue::new(&mut estado.barra_x).speed(1.0));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut estado.barra_y).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("fim x");
                    ui.add(egui::DragValue::new(&mut estado.barra_x2).speed(1.0));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut estado.barra_y2).speed(1.0));
                });
                if ui.button("Criar linha").clicked() {
                    let cmd = json!({"db": estado.db, "documento": estado.documento,
                        "x1": estado.barra_x, "y1": estado.barra_y,
                        "x2": estado.barra_x2, "y2": estado.barra_y2});
                    estado.mensagem = match app.run("anacad.linha.criar", cmd) {
                        Ok(v) => format!("linha criada: {}", v.get("handle").and_then(Value::as_str).unwrap_or("-")),
                        Err(e) => format!("erro: {e}"),
                    };
                }
            }
            "criar_legenda" => {
                ui.horizontal(|ui| {
                    ui.label("número");
                    ui.add(egui::DragValue::new(&mut estado.legenda_numero).speed(1.0));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut estado.legenda_x).speed(1.0));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut estado.legenda_y).speed(1.0));
                });
                if ui.button("Criar legenda").clicked() {
                    estado.mensagem = criar_legenda(app, &estado.db, estado.documento, estado.legenda_numero, estado.legenda_x, estado.legenda_y);
                }
            }
            "editar_barra" => {
                let handle = if !estado.comp_handle.trim().is_empty() {
                    estado.comp_handle.clone()
                } else {
                    app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                };
                ui.horizontal(|ui| {
                    ui.label("handle");
                    ui.add(egui::TextEdit::singleline(&mut estado.comp_handle).desired_width(90.0));
                    ui.label("código");
                    ui.add(egui::DragValue::new(&mut estado.barra_codigo).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("nome");
                    ui.add(egui::TextEdit::singleline(&mut estado.ed_nome).desired_width(160.0));
                    ui.label("área");
                    ui.add(egui::TextEdit::singleline(&mut estado.ed_area).desired_width(120.0));
                });
                ui.horizontal(|ui| {
                    ui.label("base tensão kV");
                    ui.add(egui::DragValue::new(&mut estado.ed_vbase).speed(1.0));
                    ui.label("capac. interrupção kA");
                    ui.add(egui::DragValue::new(&mut estado.ed_capacidade).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("tipo");
                    egui::ComboBox::from_id_salt("anacad.ed.tipo").selected_text(&estado.ed_tipo).show_ui(ui, |ui| {
                        for t in ["Normal", "Fictícia de transformador", "Auxiliar", "Autotrafo"] {
                            ui.selectable_value(&mut estado.ed_tipo, t.to_string(), t);
                        }
                    });
                    ui.checkbox(&mut estado.ed_status, "Ligado");
                    ui.checkbox(&mut estado.ed_disjuntor, "Disjuntor na SE");
                });
                ui.horizontal(|ui| {
                    ui.label("pré-falta módulo pu");
                    ui.add(egui::DragValue::new(&mut estado.ed_pre_mod).speed(1.0));
                    ui.label("ângulo°");
                    ui.add(egui::DragValue::new(&mut estado.ed_pre_ang).speed(1.0));
                });
                if ui.button("Aplicar").clicked() {
                    let dados = json!({"codigo": estado.barra_codigo, "nome": estado.ed_nome, "area": estado.ed_area,
                        "vbase": estado.ed_vbase, "tipo": estado.ed_tipo, "capacidadeInterrupcao": estado.ed_capacidade,
                        "preFaltaModulo": estado.ed_pre_mod, "preFaltaAngulo": estado.ed_pre_ang,
                        "disjuntorSE": estado.ed_disjuntor, "status": estado.ed_status});
                    let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle,
                        "codigo": estado.barra_codigo, "dados": dados});
                    estado.mensagem = match app.run("anacad.barra.editar", cmd) {
                        Ok(_) => "barra atualizada".to_string(),
                        Err(e) => format!("erro: {e}"),
                    };
                }
            }
            "editar_linha" => {
                let handle = if !estado.comp_handle.trim().is_empty() {
                    estado.comp_handle.clone()
                } else {
                    app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                };
                ui.horizontal(|ui| {
                    ui.label("handle");
                    ui.add(egui::TextEdit::singleline(&mut estado.comp_handle).desired_width(90.0));
                    ui.label("de");
                    ui.add(egui::DragValue::new(&mut estado.linha_de).speed(1.0));
                    ui.label("para");
                    ui.add(egui::DragValue::new(&mut estado.linha_para).speed(1.0));
                    ui.label("circuito");
                    ui.add(egui::DragValue::new(&mut estado.ed_circuito).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("nome");
                    ui.add(egui::TextEdit::singleline(&mut estado.ed_nome).desired_width(160.0));
                    ui.label("área");
                    ui.add(egui::TextEdit::singleline(&mut estado.ed_area).desired_width(120.0));
                });
                ui.horizontal(|ui| {
                    ui.label("potência MVA");
                    ui.add(egui::DragValue::new(&mut estado.ed_potencia).speed(1.0));
                    ui.label("comprimento km");
                    ui.add(egui::DragValue::new(&mut estado.ed_comprimento).speed(1.0));
                    ui.checkbox(&mut estado.ed_status, "Ligado");
                });
                ui.horizontal(|ui| {
                    ui.label("R1%");
                    ui.add(egui::DragValue::new(&mut estado.ed_r1).speed(1.0));
                    ui.label("X1%");
                    ui.add(egui::DragValue::new(&mut estado.ed_x1).speed(1.0));
                    ui.label("S1 Mvar");
                    ui.add(egui::DragValue::new(&mut estado.ed_s1).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("R0%");
                    ui.add(egui::DragValue::new(&mut estado.ed_r0).speed(1.0));
                    ui.label("X0%");
                    ui.add(egui::DragValue::new(&mut estado.ed_x0).speed(1.0));
                    ui.label("S0 Mvar");
                    ui.add(egui::DragValue::new(&mut estado.ed_s0).speed(1.0));
                });
                if ui.button("Aplicar").clicked() {
                    let dados = json!({"de": estado.linha_de, "para": estado.linha_para, "nome": estado.ed_nome,
                        "area": estado.ed_area, "numeroCircuito": estado.ed_circuito,
                        "potenciaNominal": estado.ed_potencia, "comprimento": estado.ed_comprimento,
                        "R1": estado.ed_r1, "X1": estado.ed_x1, "S1": estado.ed_s1,
                        "R0": estado.ed_r0, "X0": estado.ed_x0, "S0": estado.ed_s0, "status": estado.ed_status});
                    let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle,
                        "de": estado.linha_de, "para": estado.linha_para, "dados": dados});
                    estado.mensagem = match app.run("anacad.linha.editar", cmd) {
                        Ok(_) => "linha atualizada".to_string(),
                        Err(e) => format!("erro: {e}"),
                    };
                }
            }
            "atribuir_barra" | "atribuir_linha" | "mover" | "apagar" => {
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
            "importar_lst" | "importar_ana" => {
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
            "exportar" => {
                ui.horizontal(|ui| {
                    ui.label("saída .lst");
                    ui.add(egui::TextEdit::singleline(&mut estado.saida_arquivo).desired_width(320.0));
                });
                if ui.button("Exportar").clicked() {
                    estado.mensagem =
                        match app.run("anacad.exportar", json!({ "db": estado.db, "documento": estado.documento, "saida": estado.saida_arquivo })) {
                            Ok(_) => "tela exportada".to_string(),
                            Err(e) => format!("erro: {e}"),
                        };
                }
            }
            "curto" => {
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
                if ui.button("Calcular curto").clicked() {
                    estado.mensagem = calcular_curto(app, &estado.db, estado.documento, estado.curto_barra, estado.curto_ref, &estado.curto_tipo);
                }
            }
            _ => {
                if ui.button("Executar").clicked() {
                    estado.mensagem = match acao.as_str() {
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
        }
        if !estado.mensagem.is_empty() {
            ui.separator();
            ui.label(&estado.mensagem);
        }
    });
    if !aberto {
        app.ui.anacad_dialogo.clear();
    }
    app.ui.anacad_db = estado.db.clone();
    app.ui.anacad_documento = estado.documento;
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

/// Conteúdo do painel (chamado pelo dock).
pub fn panel(app: &mut CadApp, ui: &mut Ui) {
    let mut estado: Estado = ui.ctx().data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    if !app.ui.anacad_db.is_empty() && app.ui.anacad_db != estado.db {
        estado.db = app.ui.anacad_db.clone();
    }
    estado.documento = app.ui.anacad_documento.max(1);
    egui::ScrollArea::vertical().id_salt("anacad-conteudo").auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        ui.label("Status");
        ui.label("Banco conectado:");
        if estado.db.trim().is_empty() {
            ui.monospace("(nenhum)");
        } else {
            let nome = std::path::Path::new(&estado.db).file_name().and_then(|n| n.to_str()).unwrap_or(estado.db.as_str()).to_string();
            ui.monospace(&nome).on_hover_text(&estado.db);
            ui.small("conectado");
        }
        if !estado.mensagem.is_empty() {
            ui.separator();
            ui.label(&estado.mensagem);
        }
    });
    app.ui.anacad_db = estado.db.clone();
    app.ui.anacad_documento = estado.documento;
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

fn atribuir_barra(app: &mut CadApp, db: &str, documento: i64, handle: &str, codigo: i64) -> String {
    match app.run("anacad.barra.atribuir", json!({ "db": db, "documento": documento, "handle": handle, "codigo": codigo })) {
        Ok(v) => format!("barra atribuída: #{} (handle {})", v.get("barra").and_then(Value::as_i64).unwrap_or(0), handle),
        Err(e) => format!("erro: {e}"),
    }
}

fn atribuir_linha(app: &mut CadApp, db: &str, documento: i64, handle: &str, de: i64, para: i64) -> String {
    match app.run("anacad.linha.atribuir", json!({ "db": db, "documento": documento, "handle": handle, "de": de, "para": para })) {
        Ok(v) => format!("linha atribuída: #{} (handle {})", v.get("linha").and_then(Value::as_i64).unwrap_or(0), handle),
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

fn criar_barra(app: &mut CadApp, db: &str, documento: i64, codigo: i64, x: f64, y: f64, x2: f64, y2: f64) -> String {
    match app.run("anacad.barra.criar", json!({ "db": db, "documento": documento, "codigo": codigo, "x": x, "y": y, "x2": x2, "y2": y2 })) {
        Ok(v) => format!(
            "barra {} criada (handle {})",
            v.get("barra").and_then(Value::as_i64).unwrap_or(0),
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

fn carregar_documentos(app: &mut CadApp, db: &str) -> Vec<(i64, String, String)> {
    match app.run("anacad.documento.listar", json!({ "db": db })) {
        Ok(v) => v
            .get("documentos")
            .and_then(Value::as_array)
            .map(|itens| {
                itens
                    .iter()
                    .filter_map(|d| {
                        Some((
                            d.get("id")?.as_i64()?,
                            d.get("nome")?.as_str()?.to_string(),
                            d.get("pagina").and_then(Value::as_str).unwrap_or("").to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => Vec::new(),
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
