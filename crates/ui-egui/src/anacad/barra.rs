//! Dados de Barra.

use super::*;

/// Botão Alterar/Inserir da janela Dados de Barra (149).
pub(super) fn aplicar_barra(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "codigo": estado.barra_codigo,
        "nome": estado.ed_nome,
        "area": estado.ed_area,
        "vbase": estado.ed_vbase,
        "tipo": estado.ed_tipo,
        // AT só vale em barra mid-point (fictícia de transformador), cf. DBAR.
        "autotrafo": estado.ed_autotrafo && estado.ed_tipo == "Fictícia de transformador",
        "capacidadeInterrupcao": estado.ed_capacidade,
        "capacidadesAmarre": [estado.ed_cap_am[0], estado.ed_cap_am[1]],
        "preFaltaModulo": estado.ed_pre_mod,
        "preFaltaAngulo": estado.ed_pre_ang,
        "disjuntorSE": estado.ed_disjuntor,
        "status": estado.ed_status,
    });
    let cmd = json!({
        "db": estado.db,
        "documento": estado.documento,
        "handle": handle,
        "codigo": estado.barra_codigo,
        "dados": dados,
    });
    estado.mensagem = match app.run("anacad.barra.editar", cmd) {
        Ok(_) => String::new(),
        Err(e) => format!("erro: {e}"),
    };
}

/// Janela `editar_barra` do diálogo AnaCAD.
pub(super) fn janela_editar_barra(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    let handle = if !estado.comp_handle.trim().is_empty() {
        estado.comp_handle.clone()
    } else {
        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
    };
    estado.comp_handle = handle.clone();
    let chave = format!("b|{}|{}|{}", estado.db, estado.documento, handle);
    if estado.ed_listas_em != chave {
        if estado.ed_handle != handle {
            if let Some(dados) = dados_entidade(app, &handle) {
                preencher_estado(estado, &dados);
            }
            estado.ed_handle = handle.clone();
        }
        recarregar_listas_dados(app, estado, &handle, "barra");
        estado.ed_listas_em = chave;
    }
    let mut selecionar = false;
    let largura = ui.available_width().max(440.0);
    // Cabeçalho: handle à esquerda e a "mão" (selecionar no desenho)
    // no fim da linha, como nas ferramentas do Eletron (`Select_32`).
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if crate::icons::button(ui, crate::icons::Icon::SelectObjects, 20.0, "Selecionar no desenho", false)
                .on_hover_text("Selecionar a barra no desenho")
                .clicked()
            {
                selecionar = true;
            }
        });
    });
    // Conteúdo rola dentro de uma altura fixa; o rodapé de ações fica
    // fixo no fim da janela. Não usar `ui.available_height()` aqui: numa
    // janela de tamanho automático ele vale a altura da tela, e a janela
    // crescia até o limite do app.
    egui::ScrollArea::vertical().max_height(470.0).auto_shrink([false, false]).show(ui, |ui| {
        let largura_grupo = (largura - 18.0).max(380.0);
        ui.group(|ui| {
            ui.set_min_width(largura_grupo);
            ui.label(egui::RichText::new("Identificação").strong());
            egui::Grid::new("anacad.barra.identificacao").num_columns(4).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label("Número:");
                campo_numero(ui, "anacad.barra.numero", &mut estado.barra_codigo, &estado.barras_opcoes);
                ui.label("Nome:");
                campo_nome_editavel(ui, "anacad.barra.nome", &mut estado.ed_nome, &mut estado.barra_codigo, &estado.barras_opcoes);
                ui.end_row();
                ui.label("Área:");
                campo_area(ui, &mut estado.ed_area, &estado.areas_opcoes);
                ui.checkbox(&mut estado.ed_status, "Ligado");
                ui.end_row();
            });
        });
        ui.horizontal_top(|ui| {
            // `ui.group` herda o layout do pai (aqui horizontal), então o
            // empilhamento dos rádios precisa de um `vertical` explícito;
            // sem ele os itens saem lado a lado e as legendas se sobrepõem.
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Tipo").strong());
                    ui.radio_value(&mut estado.ed_tipo, "Normal".to_string(), "Normal");
                    ui.radio_value(&mut estado.ed_tipo, "Fictícia de transformador".to_string(), "Fictícia de transformador");
                    ui.radio_value(&mut estado.ed_tipo, "Auxiliar".to_string(), "Auxiliar");
                    // Manual do Usuário (DBAR): o indicador AT (autotrafo)
                    // só deve ser preenchido em barra "mid-point", isto é,
                    // fictícia de transformador.
                    let ficticia = estado.ed_tipo == "Fictícia de transformador";
                    if !ficticia {
                        estado.ed_autotrafo = false;
                    }
                    ui.add_enabled(ficticia, egui::Checkbox::new(&mut estado.ed_autotrafo, "Autotrafo"))
                        .on_disabled_hover_text("só pode ser selecionado em barra fictícia de transformador (mid-point)");
                });
            });
            // "Base de Tensão (kV):" fica na coluna da direita, logo
            // acima e com a mesma largura do grupo "Tensão Pré-Falta"
            // (ANAFAS 149: coluna x=139, 140 DLU de largura).
            ui.vertical(|ui| {
                ui.set_width(240.0);
                ui.horizontal(|ui| {
                    ui.label("Base de Tensão (kV):");
                    ui.add_sized([90.0, 20.0], egui::DragValue::new(&mut estado.ed_vbase).speed(0.1));
                });
                ui.group(|ui| {
                    ui.set_min_width(220.0);
                    ui.label(egui::RichText::new("Tensão Pré-Falta").strong());
                    egui::Grid::new("anacad.barra.prefalta").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                        ui.label("Módulo (pu):");
                        ui.add_sized([90.0, 20.0], egui::DragValue::new(&mut estado.ed_pre_mod).speed(0.01));
                        ui.end_row();
                        ui.label("Ângulo (°):");
                        ui.add_sized([90.0, 20.0], egui::DragValue::new(&mut estado.ed_pre_ang).speed(0.1));
                        ui.end_row();
                    });
                });
            });
        });
        ui.group(|ui| {
            ui.set_min_width(largura_grupo);
            ui.checkbox(&mut estado.ed_disjuntor, "Existe disjuntor na SE");
            ui.horizontal(|ui| {
                ui.label("Menor Capacidade de Interrupção (kA):");
                ui.add_sized([80.0, 20.0], egui::DragValue::new(&mut estado.ed_capacidade).speed(0.1));
            });
            ui.label("Capacidades de Interrupção Simétrica dos disjuntores de amarre (kA):");
            ui.horizontal(|ui| {
                ui.add_sized([80.0, 20.0], egui::DragValue::new(&mut estado.ed_cap_am[0]).speed(0.1));
                ui.add_sized([80.0, 20.0], egui::DragValue::new(&mut estado.ed_cap_am[1]).speed(0.1));
            });
        });
        ui.group(|ui| {
            ui.set_min_width(largura_grupo);
            ui.label(egui::RichText::new("Barras Vizinhas").strong());
            egui::Grid::new("anacad.barra.vizinhas").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label("Número:");
                combo_codigos(ui, "anacad.barra.vizinho.numero", &mut estado.viz_codigo, &estado.vizinhos);
                ui.end_row();
                ui.label("Nome:");
                ui.horizontal(|ui| {
                    combo_codigos(ui, "anacad.barra.vizinho.nome", &mut estado.viz_codigo, &estado.vizinhos);
                    if ui.add_enabled(estado.viz_codigo != 0, egui::Button::new("Mostrar")).clicked() {
                        let cmd = json!({
                            "db": estado.db,
                            "documento": estado.documento,
                            "codigo": estado.viz_codigo,
                        });
                        estado.mensagem = match app.run("anacad.barra.localizar", cmd) {
                            Ok(v) => format!(
                                "barra {} em x={:.3} y={:.3}",
                                estado.viz_codigo,
                                v.get("x").and_then(Value::as_f64).unwrap_or(0.0),
                                v.get("y").and_then(Value::as_f64).unwrap_or(0.0),
                            ),
                            Err(e) => format!("erro: {e}"),
                        };
                    }
                    ui.add_enabled(false, egui::Button::new("Trocar"))
                        .on_disabled_hover_text("comportamento do ANAFAS (149) não levantado; pendência na paridade §4");
                });
                ui.end_row();
            });
        });
        if !estado.mensagem.is_empty() {
            ui.add_space(2.0);
            ui.label(&estado.mensagem);
        }
    });
    // Rodapé de ações (ANAFAS 149): alinhado ao fim da janela.
    ui.separator();
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Fechar").clicked() {
                *fechar = true;
            }
            let existe = estado.ed_existe;
            if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
                let cmd = json!({
                    "db": estado.db,
                    "documento": estado.documento,
                    "handle": handle,
                    "codigo": estado.barra_codigo,
                });
                match app.run("anacad.barra.dados.excluir", cmd) {
                    Ok(_) => {
                        estado.ed_existe = false;
                        estado.ed_listas_em.clear();
                        estado.mensagem = format!("dados elétricos da barra {} removidos", estado.barra_codigo);
                    }
                    Err(e) => estado.mensagem = format!("erro: {e}"),
                }
            }
            if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
                aplicar_barra(app, estado, &handle);
                if !estado.mensagem.starts_with("erro") {
                    *notificacao = Some(format!("barra {} atualizada", estado.barra_codigo));
                    *fechar = true;
                }
            }
            if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
                aplicar_barra(app, estado, &handle);
                if !estado.mensagem.starts_with("erro") {
                    *notificacao = Some(format!("barra {} inserida", estado.barra_codigo));
                    *fechar = true;
                }
            }
        });
    });
    if selecionar {
        app.ui.anacad_dialogo.clear();
        app.ui.anacad_handle.clear();
        app.start("anacad.barra.editar");
    }
}
