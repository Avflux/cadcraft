//! Dados do Equipamento (janela comum de inserção/edição).

use super::*;

/// Janela `editar_equipamento` do diálogo AnaCAD.
pub(super) fn janela_editar_equipamento(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    let handle = estado.comp_handle.clone();
    if estado.ed_handle != handle {
        if let Some(dados) = dados_entidade(app, &handle) {
            preencher_equipamento(estado, &dados);
        }
        estado.ed_handle = handle.clone();
    }
    let conexoes = ["Delta", "Estrela não aterrado", "Estrela aterrado"];
    ui.separator();
    ui.label(egui::RichText::new("Identificação").strong());
    ui.horizontal(|ui| {
        ui.label("equipamento");
        ui.monospace(if estado.eq_tipo.is_empty() { "-" } else { &estado.eq_tipo });
        ui.label("código");
        ui.add(egui::DragValue::new(&mut estado.eq_codigo).speed(1.0));
        ui.checkbox(&mut estado.eq_status, "Ligado");
    });
    ui.horizontal(|ui| {
        ui.label("nome");
        ui.add(egui::TextEdit::singleline(&mut estado.eq_nome).desired_width(220.0));
        ui.label("grupo");
        campo_grupo(ui, &mut estado.eq_grupo, &estado.grupos_opcoes);
        ui.label("área");
        campo_area(ui, &mut estado.eq_area, &estado.areas_opcoes);
    });
    let tipo = estado.eq_tipo.as_str();
    if tipo == "Transformador" {
        ui.separator();
        ui.label(egui::RichText::new("Enrolamentos").strong());
        ui.horizontal(|ui| {
            ui.label("primário");
            campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
            ui.label("secundário");
            campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
            ui.label("circuito");
            ui.add(egui::DragValue::new(&mut estado.eq_circuito).speed(1.0));
            ui.label("capacidade MVA");
            ui.add(egui::DragValue::new(&mut estado.eq_capacidade).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("R1%");
            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
            ui.label("X1%");
            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
            ui.label("R0%");
            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
            ui.label("X0%");
            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("conexão primário");
            egui::ComboBox::from_id_salt("eq.con1").selected_text(&estado.eq_conexao).show_ui(ui, |ui| {
                for c in conexoes {
                    ui.selectable_value(&mut estado.eq_conexao, c.to_string(), c);
                }
            });
            ui.label("Rn%");
            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
            ui.label("Xn%");
            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("conexão secundário");
            egui::ComboBox::from_id_salt("eq.con2").selected_text(&estado.eq_conexao2).show_ui(ui, |ui| {
                for c in conexoes {
                    ui.selectable_value(&mut estado.eq_conexao2, c.to_string(), c);
                }
            });
            ui.label("Rn%");
            ui.add(egui::DragValue::new(&mut estado.eq_rn2).speed(1.0));
            ui.label("Xn%");
            ui.add(egui::DragValue::new(&mut estado.eq_xn2).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("defasamento°");
            ui.add(egui::DragValue::new(&mut estado.eq_defasamento).speed(1.0));
            ui.label("tap");
            ui.add(egui::DragValue::new(&mut estado.eq_tap).speed(0.001));
            ui.label("unidades");
            ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
        });
    } else if tipo == "Gerador" || tipo == "Gerador Fonte de Corrente" || tipo == "Motor de Indução" {
        ui.separator();
        ui.label(egui::RichText::new("Máquina").strong());
        ui.horizontal(|ui| {
            ui.label("barra");
            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
            ui.label("potência MVA");
            ui.add(egui::DragValue::new(&mut estado.eq_pot).speed(1.0));
            ui.label("unidades");
            ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
        });
        ui.horizontal(|ui| {
            ui.label("R1%");
            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
            ui.label("X1%");
            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
            ui.label("R0%");
            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
            ui.label("X0%");
            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("conexão");
            egui::ComboBox::from_id_salt("eq.con").selected_text(&estado.eq_conexao).show_ui(ui, |ui| {
                for c in conexoes {
                    ui.selectable_value(&mut estado.eq_conexao, c.to_string(), c);
                }
            });
            ui.label("Rn%");
            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
            ui.label("Xn%");
            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("P MW");
            ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
            ui.label("Q Mvar");
            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
        });
    } else if tipo == "Carga" {
        ui.separator();
        ui.label(egui::RichText::new("Carga").strong());
        ui.horizontal(|ui| {
            ui.label("barra");
            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
            ui.label("P MW");
            ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
            ui.label("Q Mvar");
            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
            ui.label("unidades");
            ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
        });
        ui.horizontal(|ui| {
            ui.label("conexão");
            egui::ComboBox::from_id_salt("eq.con").selected_text(&estado.eq_conexao).show_ui(ui, |ui| {
                for c in conexoes {
                    ui.selectable_value(&mut estado.eq_conexao, c.to_string(), c);
                }
            });
        });
    } else if tipo == "Compensador Série" || tipo == "Impedância Mútua" {
        ui.separator();
        ui.label(egui::RichText::new(if tipo == "Impedância Mútua" { "Linhas acopladas" } else { "Circuito série" }).strong());
        ui.horizontal(|ui| {
            ui.label("de");
            campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
            ui.label("para");
            campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("R1%");
            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
            ui.label("X1%");
            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
            ui.label("R0%");
            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
            ui.label("X0%");
            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
        });
    } else if tipo == "Trafo de Aterramento" {
        ui.separator();
        ui.label(egui::RichText::new("Aterramento").strong());
        ui.horizontal(|ui| {
            ui.label("barra");
            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
            ui.label("R0%");
            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
            ui.label("X0%");
            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("Rn%");
            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
            ui.label("Xn%");
            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
        });
    } else if tipo == "Compensador Estático" {
        ui.separator();
        ui.label(egui::RichText::new("Compensador estático").strong());
        ui.horizontal(|ui| {
            ui.label("barra");
            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
            ui.label("Q Mvar");
            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("R1%");
            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
            ui.label("X1%");
            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
            ui.label("R0%");
            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
            ui.label("X0%");
            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
        });
    } else if tipo.contains("Shunt") {
        ui.separator();
        ui.label(egui::RichText::new("Shunt").strong());
        ui.horizontal(|ui| {
            ui.label("de");
            campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
            ui.label("para");
            campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
            ui.label("Q Mvar");
            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("Rn%");
            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
            ui.label("Xn%");
            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
        });
    } else {
        ui.separator();
        ui.label(egui::RichText::new("Conexão").strong());
        let usa_par = matches!(tipo, "Compensador Série" | "Impedância Mútua");
        if usa_par {
            ui.horizontal(|ui| {
                ui.label("de");
                campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
                ui.label("para");
                campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("barra");
                campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
            });
        }
        ui.horizontal(|ui| {
            ui.label("potência MVA");
            ui.add(egui::DragValue::new(&mut estado.eq_pot).speed(1.0));
            ui.label("R1%");
            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
            ui.label("X1%");
            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("R0%");
            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
            ui.label("X0%");
            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("P MW");
            ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
            ui.label("Q Mvar");
            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
        });
    }
    ui.separator();
    if ui.button("Aplicar").clicked() {
        let dados = json!({
            "tipo": estado.eq_tipo, "codigo": estado.eq_codigo, "nome": estado.eq_nome,
            "grupo": estado.eq_grupo, "area": estado.eq_area,
            "de": estado.eq_de, "para": estado.eq_para, "barra": estado.eq_barra,
            "numeroCircuito": estado.eq_circuito, "capacidade": estado.eq_capacidade,
            "potencia": estado.eq_pot, "R1": estado.eq_r1, "X1": estado.eq_x1,
            "R0": estado.eq_r0, "X0": estado.eq_x0, "tap": estado.eq_tap,
            "defasamento": estado.eq_defasamento,
            "conexao": estado.eq_conexao, "conexao2": estado.eq_conexao2,
            "Rn": estado.eq_rn, "Xn": estado.eq_xn, "Rn2": estado.eq_rn2, "Xn2": estado.eq_xn2,
            "P": estado.eq_p, "Q": estado.eq_q, "unidades": estado.eq_unidades,
            "status": estado.eq_status,
        });
        let cmd = json!({"db": estado.db, "handle": handle, "dados": dados});
        match app.run("anacad.equipamento.editar", cmd) {
            Ok(_) => {
                *fechar = true;
                *notificacao = Some(format!("equipamento {} atualizado", estado.eq_codigo));
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
}
