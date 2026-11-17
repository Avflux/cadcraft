//! Ferramentas de desenho do diagrama (criar, renumerar, localizar, seccionar, separar, unir, legendas, textos).

use super::*;

/// Janela `criar_barra` do diálogo AnaCAD.
pub(super) fn janela_criar_barra(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
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

/// Janela `criar_linha` do diálogo AnaCAD.
pub(super) fn janela_criar_linha(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
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

/// Janela `criar_legenda` do diálogo AnaCAD.
pub(super) fn janela_criar_legenda(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
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

/// Janela `opcoes_legenda` do diálogo AnaCAD.
pub(super) fn janela_opcoes_legenda(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    if !estado.leg_carregada {
        if let Ok(v) = app.run("anacad.legenda.config.ler", json!({"db": estado.db}))
            && let Some(d) = v.get("dados")
        {
            if let Some(s) = d.get("moduloCorrente").and_then(Value::as_str) {
                estado.leg_modulo_corrente = s.into();
            }
            if let Some(s) = d.get("anguloCorrente").and_then(Value::as_str) {
                estado.leg_angulo_corrente = s.into();
            }
            if let Some(s) = d.get("moduloTensao").and_then(Value::as_str) {
                estado.leg_modulo_tensao = s.into();
            }
            if let Some(s) = d.get("anguloTensao").and_then(Value::as_str) {
                estado.leg_angulo_tensao = s.into();
            }
            if let Some(b) = d.get("barrasNome").and_then(Value::as_bool) {
                estado.leg_barras_nome = b;
            }
            if let Some(b) = d.get("equipNumero").and_then(Value::as_bool) {
                estado.leg_equip_numero = b;
            }
            if let Some(b) = d.get("trafoTap").and_then(Value::as_bool) {
                estado.leg_trafo_tap = b;
            }
            if let Some(b) = d.get("trafoDefasamento").and_then(Value::as_bool) {
                estado.leg_trafo_defasamento = b;
            }
        }
        estado.leg_carregada = true;
    }
    ui.label(egui::RichText::new("Unidades de Corrente").strong());
    ui.horizontal(|ui| {
        ui.label("Módulo");
        ui.add(egui::TextEdit::singleline(&mut estado.leg_modulo_corrente).desired_width(60.0));
        ui.label("Ângulo");
        ui.add(egui::TextEdit::singleline(&mut estado.leg_angulo_corrente).desired_width(60.0));
    });
    ui.label(egui::RichText::new("Unidades de Tensão").strong());
    ui.horizontal(|ui| {
        ui.label("Módulo");
        ui.add(egui::TextEdit::singleline(&mut estado.leg_modulo_tensao).desired_width(60.0));
        ui.label("Ângulo");
        ui.add(egui::TextEdit::singleline(&mut estado.leg_angulo_tensao).desired_width(60.0));
    });
    ui.separator();
    ui.checkbox(&mut estado.leg_barras_nome, "Identificar barras por nome");
    ui.checkbox(&mut estado.leg_equip_numero, "Identificar equipamentos por número");
    ui.checkbox(&mut estado.leg_trafo_tap, "Mostrar tap de transformadores");
    ui.checkbox(&mut estado.leg_trafo_defasamento, "Mostrar defasamento de transformadores");
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Restaurar padrão").clicked() {
            estado.leg_modulo_corrente = "kA".into();
            estado.leg_angulo_corrente = "grau".into();
            estado.leg_modulo_tensao = "kV".into();
            estado.leg_angulo_tensao = "grau".into();
            estado.leg_barras_nome = true;
            estado.leg_equip_numero = true;
            estado.leg_trafo_tap = true;
            estado.leg_trafo_defasamento = true;
        }
        if ui.button("Salvar").clicked() {
            let dados = json!({
                "moduloCorrente": estado.leg_modulo_corrente,
                "anguloCorrente": estado.leg_angulo_corrente,
                "moduloTensao": estado.leg_modulo_tensao,
                "anguloTensao": estado.leg_angulo_tensao,
                "barrasNome": estado.leg_barras_nome,
                "equipNumero": estado.leg_equip_numero,
                "trafoTap": estado.leg_trafo_tap,
                "trafoDefasamento": estado.leg_trafo_defasamento,
            });
            match app.run("anacad.legenda.config.gravar", json!({"db": estado.db, "dados": dados})) {
                Ok(_) => {
                    *fechar = true;
                    *notificacao = Some("opções de legenda salvas".to_string());
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
    });
}

/// Janela `texto_livre` do diálogo AnaCAD.
pub(super) fn janela_texto_livre(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.label("Texto:");
    ui.add(egui::TextEdit::multiline(&mut estado.texto_conteudo).desired_width(420.0).desired_rows(4));
    ui.horizontal(|ui| {
        ui.label("Altura");
        ui.add(egui::DragValue::new(&mut estado.texto_altura).range(0.1..=500.0).speed(0.1));
        ui.label("Cor ACI");
        ui.add(egui::DragValue::new(&mut estado.texto_cor).range(1..=255));
    });
    ui.separator();
    if ui.button("Inserir no desenho").clicked() {
        cadcraft_engine::cmd::anacad::set_texto(&mut app.session, estado.texto_conteudo.clone(), estado.texto_altura, estado.texto_cor);
        app.ui.anacad_dialogo.clear();
        app.start("anacad.texto.inserir");
    }
}

/// Janela `detalhar_barra` do diálogo AnaCAD.
pub(super) fn janela_detalhar_barra(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.horizontal(|ui| {
        ui.label("Código da barra");
        ui.add(egui::DragValue::new(&mut estado.det_codigo).speed(1.0));
        if ui.button("Atualizar").clicked() {
            match app.run("anacad.barra.detalhar", json!({"db": estado.db, "documento": estado.documento, "codigo": estado.det_codigo})) {
                Ok(v) => {
                    let componentes = v.get("componentes").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
                    let circuitos = v.get("circuitos").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
                    let vizinhos = v
                        .get("vizinhos")
                        .and_then(Value::as_array)
                        .map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                        .unwrap_or_default();
                    estado.det_resumo =
                        format!("barra {}\ncomponentes: {componentes}\ncircuitos: {circuitos}\nvizinhos: {vizinhos:?}", estado.det_codigo);
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
    });
    ui.separator();
    ui.monospace(&estado.det_resumo);
    if !estado.mensagem.is_empty() {
        ui.separator();
        ui.label(&estado.mensagem);
    }
}

/// Janela `renomear_terminal | localizar_terminal` do diálogo AnaCAD.
pub(super) fn janela_terminal(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, acao: &str, fechar: &mut bool, notificacao: &mut Option<String>) {
    if !estado.term_carregada {
        estado.term_lista = carregar_terminais(app, &estado.db, estado.documento);
        estado.term_carregada = true;
        if let Some((id, nome, _, _)) = estado.term_lista.first() {
            estado.term_id = *id;
            estado.term_nome = nome.clone();
        }
    }
    ui.horizontal(|ui| {
        if ui.button("Atualizar lista").clicked() {
            estado.term_lista = carregar_terminais(app, &estado.db, estado.documento);
        }
        ui.label(format!("{} terminal(is)", estado.term_lista.len()));
    });
    let mut escolhido = None;
    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
        for (id, nome, _, _) in &estado.term_lista {
            if ui.selectable_label(estado.term_id == *id, format!("#{id} {nome}")).clicked() {
                escolhido = Some((*id, nome.clone()));
            }
        }
    });
    if let Some((id, nome)) = escolhido {
        estado.term_id = id;
        estado.term_nome = nome;
    }
    if acao == "renomear_terminal" {
        ui.horizontal(|ui| {
            ui.label("Novo nome");
            ui.add(egui::TextEdit::singleline(&mut estado.term_nome).desired_width(240.0));
        });
        if ui.button("Renomear").clicked() {
            match app.run("anacad.terminal.renomear", json!({"db": estado.db, "id": estado.term_id, "nome": estado.term_nome})) {
                Ok(_) => {
                    estado.term_carregada = false;
                    *fechar = true;
                    *notificacao = Some("terminal renomeado".to_string());
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
    } else if ui.button("Localizar").clicked()
        && let Some((_, _, x, y)) = estado.term_lista.iter().find(|(id, _, _, _)| *id == estado.term_id)
    {
        let _ = app.run("zoom", json!({"mode": "center", "center": [x, y], "height": 120.0}));
        *fechar = true;
        *notificacao = Some("terminal localizado".to_string());
    }
}

/// Janela `renumerar_barra` do diálogo AnaCAD.
pub(super) fn janela_renumerar_barra(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label("Código atual");
        ui.add(egui::DragValue::new(&mut estado.ren_atual).speed(1.0));
        ui.label("Novo código");
        ui.add(egui::DragValue::new(&mut estado.ren_nova).speed(1.0));
    });
    ui.separator();
    if ui.button("Renumerar").clicked() {
        let cmd = json!({"db": estado.db, "documento": estado.documento,
        "atual": estado.ren_atual, "nova": estado.ren_nova});
        match app.run("anacad.barra.renumerar", cmd) {
            Ok(_) => {
                *fechar = true;
                *notificacao = Some(format!("barra {} renumerada para {}", estado.ren_atual, estado.ren_nova));
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
}

/// Janela `localizar_barra` do diálogo AnaCAD.
pub(super) fn janela_localizar_barra(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label("Código da barra");
        ui.add(egui::DragValue::new(&mut estado.loc_codigo).speed(1.0));
    });
    ui.separator();
    if ui.button("Localizar").clicked() {
        match app.run("anacad.barra.localizar", json!({"db": estado.db, "documento": estado.documento, "codigo": estado.loc_codigo})) {
            Ok(v) => {
                let x = v.get("x").and_then(Value::as_f64).unwrap_or(0.0);
                let y = v.get("y").and_then(Value::as_f64).unwrap_or(0.0);
                let _ = app.run("zoom", json!({"mode": "center", "center": [x, y], "height": 120.0}));
                *fechar = true;
                *notificacao = Some(format!("barra {} localizada", estado.loc_codigo));
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
}

/// Janela `seccionar_linha` do diálogo AnaCAD.
pub(super) fn janela_seccionar_linha(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label("Barra De");
        ui.add(egui::DragValue::new(&mut estado.sec_de).speed(1.0));
        ui.label("Barra Para");
        ui.add(egui::DragValue::new(&mut estado.sec_para).speed(1.0));
    });
    ui.horizontal(|ui| {
        ui.label("Nova barra");
        ui.add(egui::DragValue::new(&mut estado.sec_nova).speed(1.0));
        ui.label("Nome");
        ui.add(egui::TextEdit::singleline(&mut estado.sec_nome).desired_width(220.0));
    });
    ui.horizontal(|ui| {
        ui.label("Percentual a partir da barra De (%)");
        ui.add(egui::DragValue::new(&mut estado.sec_percentual).range(0.1..=99.9).speed(1.0));
        ui.checkbox(&mut estado.sec_reatores, "Eliminar reatores");
    });
    ui.separator();
    if ui.button("Executar").clicked() {
        let cmd = json!({
            "db": estado.db, "documento": estado.documento,
            "de": estado.sec_de, "para": estado.sec_para,
            "nova": estado.sec_nova, "nome": estado.sec_nome,
            "percentual": estado.sec_percentual, "reatores": estado.sec_reatores,
        });
        match app.run("anacad.linha.seccionar", cmd) {
            Ok(_) => {
                *fechar = true;
                *notificacao = Some(format!("linha seccionada em {}", estado.sec_nova));
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
}

/// Janela `separar_barra` do diálogo AnaCAD.
pub(super) fn janela_separar_barra(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label("Barra atual");
        ui.add(egui::DragValue::new(&mut estado.top_atual).speed(1.0));
        ui.label("Nova barra");
        ui.add(egui::DragValue::new(&mut estado.top_nova).speed(1.0));
    });
    ui.horizontal(|ui| {
        ui.label("Nome da nova");
        ui.add(egui::TextEdit::singleline(&mut estado.top_nome).desired_width(220.0));
        ui.checkbox(&mut estado.top_jumper, "JUMPER entre as barras");
    });
    if estado.top_comp_barra != estado.top_atual {
        estado.top_componentes = carregar_componentes_barra(app, &estado.db, estado.documento, estado.top_atual);
        estado.top_comp_barra = estado.top_atual;
    }
    ui.separator();
    ui.label("Componentes da barra atual (marque os que vão para a nova):");
    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        for (id, nome, marcado) in &mut estado.top_componentes {
            ui.checkbox(marcado, format!("#{id} {nome}"));
        }
    });
    ui.separator();
    if ui.button("Separar").clicked() {
        let componentes: Vec<i64> = estado.top_componentes.iter().filter(|(_, _, marcado)| *marcado).map(|(id, _, _)| *id).collect();
        let cmd = json!({
            "db": estado.db, "documento": estado.documento,
            "atual": estado.top_atual, "nova": estado.top_nova,
            "nome": estado.top_nome, "componentes": componentes,
            "jumper": estado.top_jumper,
        });
        match app.run("anacad.barra.separar", cmd) {
            Ok(v) => {
                *fechar = true;
                *notificacao =
                    Some(format!("barra {} separada ({} componentes)", estado.top_nova, v.get("componentes").and_then(Value::as_i64).unwrap_or(0)));
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
}

/// Janela `unir_barras` do diálogo AnaCAD.
pub(super) fn janela_unir_barras(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label("Barra final");
        ui.add(egui::DragValue::new(&mut estado.top_final).speed(1.0));
        ui.label("Barra eliminada");
        ui.add(egui::DragValue::new(&mut estado.top_eliminada).speed(1.0));
    });
    ui.separator();
    if ui.button("Unir").clicked() {
        let cmd = json!({
            "db": estado.db, "documento": estado.documento,
            "final": estado.top_final, "eliminada": estado.top_eliminada,
        });
        match app.run("anacad.barra.unir", cmd) {
            Ok(_) => {
                *fechar = true;
                *notificacao = Some(format!("barra {} unida à {}", estado.top_eliminada, estado.top_final));
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
}
