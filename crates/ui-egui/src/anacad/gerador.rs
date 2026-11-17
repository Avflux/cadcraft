//! Dados de Gerador.

use super::*;

/// Conteúdo do painel (chamado pelo dock).
/// Botão Alterar/Inserir da janela Dados de Gerador (163).
pub(super) fn aplicar_equipamento_gerador(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "codigo": estado.eq_codigo,
        "nome": estado.eq_nome,
        "tipo": "Gerador",
        "barra": estado.eq_barra,
        "grupo": estado.eq_grupo,
        "area": estado.eq_area,
        "status": estado.eq_status,
        "tipoGerador": estado.eq_tipo_gerador,
        "sincrono": estado.eq_sincrono,
        "circuitoEquivalente": estado.eq_circ_eq,
        "potencia": estado.eq_pot,
        "barraAlta": estado.eq_barra_alta,
        "associarTrafoElevador": estado.eq_assoc_trafo,
        "numeroCircuito": estado.eq_circuito,
        "R1": estado.eq_r1,
        "X1": estado.eq_x1,
        "R0": estado.eq_r0,
        "X0": estado.eq_x0,
        "conexao": estado.eq_conexao,
        "Rn": estado.eq_rn,
        "Xn": estado.eq_xn,
        "P": estado.eq_p,
        "Q": estado.eq_q,
        "unidades": estado.eq_unidades,
        "unidadesOperacao": estado.eq_unid_oper,
        "unidadesPorTransformador": estado.eq_unid_por_trafo,
        "capacidadesDe": estado.eq_cap_de.to_vec(),
        "agregadorCompensador": estado.eq_agreg_comp,
        "agregadorTrafo3": estado.eq_agreg_trafo3,
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

/// Janela `editar_gerador` do diálogo AnaCAD.
pub(super) fn janela_editar_gerador(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    let handle = if !estado.comp_handle.trim().is_empty() {
        estado.comp_handle.clone()
    } else {
        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
    };
    estado.comp_handle = handle.clone();
    let chave = format!("g|{}|{}|{}", estado.db, estado.documento, handle);
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
    egui::ScrollArea::vertical().max_height(600.0).auto_shrink([false, false]).show(ui, |ui| {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Identificação").strong());
            ui.horizontal(|ui| {
                ui.label("Barra:");
                campo_conexao(ui, "anacad.ger.barra", &mut estado.eq_barra, &estado.barras_opcoes);
                ui.label("Nome:");
                combo_conexao(ui, "anacad.ger.barra.nome", &mut estado.eq_barra, &estado.barras_opcoes);
                ui.label("Área:");
                campo_area(ui, &mut estado.eq_area, &estado.areas_opcoes);
                ui.checkbox(&mut estado.eq_status, "Ligado");
            });
            ui.horizontal(|ui| {
                ui.label("Grupo:");
                campo_grupo(ui, &mut estado.eq_grupo, &estado.grupos_opcoes);
                ui.label("Nome:");
                ui.add(egui::TextEdit::singleline(&mut estado.eq_nome).desired_width(120.0));
                ui.label("Tipo:");
                ui.add(egui::TextEdit::singleline(&mut estado.eq_tipo_gerador).desired_width(80.0));
            });
            // Os dois checks vão para a linha de baixo: na mesma
            // linha do Grupo/Nome/Tipo o "Circuito Equivalente"
            // ficava fora da janela (640 px).
            ui.horizontal(|ui| {
                ui.checkbox(&mut estado.eq_sincrono, "Síncrono");
                ui.checkbox(&mut estado.eq_circ_eq, "Circuito Equivalente");
            });
        });
        ui.horizontal_top(|ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Potência Nominal (MVA):");
                    ui.add(egui::DragValue::new(&mut estado.eq_pot).speed(1.0));
                });
            });
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Barra de Alta:");
                    campo_numero(ui, "anacad.ger.barra_alta", &mut estado.eq_barra_alta, &estado.barras_opcoes);
                });
                ui.checkbox(&mut estado.eq_assoc_trafo, "Associar a Transformador Elevador");
                ui.horizontal(|ui| {
                    ui.label("Num. Circuito:");
                    ui.add(egui::DragValue::new(&mut estado.eq_circuito).speed(1.0));
                });
            });
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Sequência Positiva").strong());
            ui.horizontal(|ui| {
                ui.label("Resistência (R1 %):");
                ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(0.1));
                ui.label("Reatância Subtransitória (X''d %):");
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
            });
        });
        grupo_conexao(ui, "Tipo de Conexão", &mut estado.eq_conexao, &mut estado.eq_rn, &mut estado.eq_xn);
        ui.group(|ui| {
            ui.label(egui::RichText::new("Despacho").strong());
            ui.horizontal(|ui| {
                ui.label("Geração Ativa (MW):");
                ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
                ui.label("Geração Reativa (MVar):");
                ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
            });
        });
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Número de Unidades:");
                ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
                ui.label("Número de Unidades em Operação:");
                ui.add(egui::DragValue::new(&mut estado.eq_unid_oper).range(0..=9999));
                ui.label("Unidades por Transformador:");
                ui.add(egui::DragValue::new(&mut estado.eq_unid_por_trafo).range(0..=9999));
            });
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
            ui.horizontal(|ui| {
                ui.label("1:");
                ui.add(egui::DragValue::new(&mut estado.eq_cap_de[0]).speed(0.1));
                ui.label("2:");
                ui.add(egui::DragValue::new(&mut estado.eq_cap_de[1]).speed(0.1));
                ui.label("3:");
                ui.add(egui::DragValue::new(&mut estado.eq_cap_de[2]).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Agregador para Compensador Síncrono:");
                ui.add(egui::TextEdit::singleline(&mut estado.eq_agreg_comp).desired_width(80.0));
            });
            ui.horizontal(|ui| {
                ui.label("Agregador para Transformador de 3 Enrolamentos:");
                ui.add(egui::TextEdit::singleline(&mut estado.eq_agreg_trafo3).desired_width(80.0));
            });
        });
    });
    // O rodapé fica fora da rolagem, como no Transformador:
    // dentro dela os botões ficavam escondidos no fim do conteúdo.
    ui.separator();
    ui.horizontal(|ui| {
        let existe = estado.eq_existe;
        if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
            aplicar_equipamento_gerador(app, estado, &handle);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some("dados do gerador inseridos".to_string());
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
            aplicar_equipamento_gerador(app, estado, &handle);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some("gerador atualizado".to_string());
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
            let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle});
            match app.run("anacad.equipamento.dados.excluir", cmd) {
                Ok(_) => {
                    estado.eq_existe = false;
                    estado.mensagem = "dados elétricos do gerador removidos".to_string();
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
