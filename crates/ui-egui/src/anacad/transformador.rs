//! Dados de Transformador.

use super::*;

/// Grupo "Tipo de Conexão" do transformador/gerador: rádios + Rn/Xn.
pub(super) fn grupo_conexao(ui: &mut Ui, titulo: &str, conexao: &mut String, rn: &mut f64, xn: &mut f64) {
    ui.group(|ui| {
        ui.label(egui::RichText::new(titulo).strong());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for c in ["Delta", "Estrela não aterrado", "Estrela aterrado"] {
                    ui.radio_value(conexao, c.to_string(), c);
                }
            });
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Resistência de Aterramento (Rn %):");
                    ui.add(egui::DragValue::new(rn).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Reatância de Aterramento (Xn %):");
                    ui.add(egui::DragValue::new(xn).speed(0.1));
                });
            });
        });
    });
}

/// Botão Alterar/Inserir da janela Dados de Transformador (176).
pub(super) fn aplicar_equipamento_trafo(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "codigo": estado.eq_codigo,
        "nome": estado.eq_nome,
        "tipo": "Transformador",
        "de": estado.eq_de,
        "para": estado.eq_para,
        "numeroCircuito": estado.eq_circuito,
        "area": estado.eq_area,
        "capacidade": estado.eq_capacidade,
        "circuitoEquivalente": estado.eq_circ_eq,
        "status": estado.eq_status,
        "R1": estado.eq_r1,
        "X1": estado.eq_x1,
        "R0": estado.eq_r0,
        "X0": estado.eq_x0,
        "conexao": estado.eq_conexao,
        "Rn": estado.eq_rn,
        "Xn": estado.eq_xn,
        "conexao2": estado.eq_conexao2,
        "Rn2": estado.eq_rn2,
        "Xn2": estado.eq_xn2,
        "defasamento": estado.eq_defasamento,
        "defasamentoExplicito": estado.eq_def_expl,
        "tap": estado.eq_tap,
        "unidades": estado.eq_unidades,
        "unidadesOperacao": estado.eq_unid_oper,
        "capacidadesDe": estado.eq_cap_de.to_vec(),
        "capacidadesPara": estado.eq_cap_para.to_vec(),
    });
    let cmd = json!({
        "db": estado.db,
        "documento": estado.documento,
        "handle": handle,
        "dados": dados,
    });
    estado.mensagem = match app.run("anacad.equipamento.editar", cmd) {
        Ok(_) => String::new(),
        Err(e) => format!("erro: {e}"),
    };
}

/// Janela `editar_transformador` do diálogo AnaCAD.
pub(super) fn janela_editar_transformador(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    let handle = if !estado.comp_handle.trim().is_empty() {
        estado.comp_handle.clone()
    } else {
        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
    };
    estado.comp_handle = handle.clone();
    let chave = format!("t|{}|{}|{}", estado.db, estado.documento, handle);
    if estado.ed_listas_em != chave {
        if estado.ed_handle != handle {
            if let Some(dados) = dados_entidade(app, &handle) {
                preencher_equipamento(estado, &dados);
            }
            estado.ed_handle = handle.clone();
        }
        carregar_barras_opcoes(app, estado);
        estado.ed_listas_em = chave;
    }
    egui::ScrollArea::vertical().max_height(520.0).auto_shrink([false, false]).show(ui, |ui| {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Identificação").strong());
            egui::Grid::new("anacad.trafo.identificacao").num_columns(4).show(ui, |ui| {
                ui.label("Primário:");
                campo_conexao(ui, "anacad.trafo.de", &mut estado.eq_de, &estado.barras_opcoes);
                ui.label("Nome:");
                combo_conexao(ui, "anacad.trafo.de.nome", &mut estado.eq_de, &estado.barras_opcoes);
                ui.end_row();
                ui.label("Secundário:");
                campo_conexao(ui, "anacad.trafo.para", &mut estado.eq_para, &estado.barras_opcoes);
                ui.label("Nome:");
                combo_conexao(ui, "anacad.trafo.para.nome", &mut estado.eq_para, &estado.barras_opcoes);
                ui.end_row();
                ui.label("Número do Circuito:");
                ui.add(egui::DragValue::new(&mut estado.eq_circuito).speed(1.0));
                ui.label("Nome:");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut estado.eq_nome).desired_width(110.0));
                    ui.checkbox(&mut estado.eq_status, "Ligado");
                });
                ui.end_row();
                ui.label("Área:");
                campo_area(ui, &mut estado.eq_area, &estado.areas_opcoes);
                ui.horizontal(|ui| {
                    ui.label("Capacidade (MVA):");
                    ui.add(egui::DragValue::new(&mut estado.eq_capacidade).speed(0.1));
                });
                ui.checkbox(&mut estado.eq_circ_eq, "Circuito Equivalente");
                ui.end_row();
            });
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Sequência Positiva").strong());
            ui.horizontal(|ui| {
                ui.label("Resistência (R1 %):");
                ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(0.1));
                ui.label("Reatância (X1 %):");
                ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(0.1));
            });
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Sequência Zero").strong());
            ui.horizontal(|ui| {
                ui.label("Resistência (R0 %):");
                ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(0.1));
                ui.label("Reatância (X0 %):");
                ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(0.1));
                ui.add_enabled(false, egui::Button::new("Converter"))
                    .on_disabled_hover_text("conversão de trafo/shunt (790/791) sem fórmula no reverso; paridade §4");
            });
        });
        grupo_conexao(ui, "Tipo de Conexão do Primário", &mut estado.eq_conexao, &mut estado.eq_rn, &mut estado.eq_xn);
        grupo_conexao(ui, "Tipo de Conexão do Secundário", &mut estado.eq_conexao2, &mut estado.eq_rn2, &mut estado.eq_xn2);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Defasamento (°)");
                ui.add(egui::DragValue::new(&mut estado.eq_defasamento).speed(0.1));
                ui.checkbox(&mut estado.eq_def_expl, "Explícito");
            });
            ui.label(egui::RichText::new("Obs: def. do secundário em relação ao primário.").small().weak());
        });
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Tap (pu):");
                ui.add(egui::DragValue::new(&mut estado.eq_tap).speed(0.001));
            });
        });
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Número de Unidades:");
                ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
                ui.label("Número de Unidades em Operação:");
                ui.add(egui::DragValue::new(&mut estado.eq_unid_oper).range(0..=9999));
            });
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
            // De e Para em linhas separadas: os seis campos não
            // cabem numa linha só da janela (520 px) e o "Para"
            // ficava cortado.
            ui.horizontal(|ui| {
                ui.label("De");
                for (i, v) in estado.eq_cap_de.iter_mut().enumerate() {
                    ui.label(format!("{}:", i + 1));
                    ui.add(egui::DragValue::new(v).speed(0.1));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Para");
                for (i, v) in estado.eq_cap_para.iter_mut().enumerate() {
                    ui.label(format!("{}:", i + 1));
                    ui.add(egui::DragValue::new(v).speed(0.1));
                }
            });
        });
    });

    ui.separator();
    ui.horizontal(|ui| {
        let existe = estado.eq_existe;
        if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
            aplicar_equipamento_trafo(app, estado, &handle);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some("dados do transformador inseridos".to_string());
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
            aplicar_equipamento_trafo(app, estado, &handle);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some("transformador atualizado".to_string());
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
            let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle});
            match app.run("anacad.equipamento.dados.excluir", cmd) {
                Ok(_) => {
                    estado.eq_existe = false;
                    estado.mensagem = "dados elétricos do transformador removidos".to_string();
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
        if ui.button("Fechar").clicked() {
            *fechar = true;
        }
    });
    ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
}
