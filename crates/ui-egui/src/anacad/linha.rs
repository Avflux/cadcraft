//! Dados de Linha de Transmissão.

use super::*;

/// Botão Alterar/Inserir da janela Dados de Linha (164).
pub(super) fn aplicar_linha(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "de": estado.linha_de,
        "para": estado.linha_para,
        "nome": estado.ed_nome,
        "area": estado.ed_area,
        "numeroCircuito": estado.ed_circuito,
        "potenciaNominal": estado.ed_potencia,
        "comprimento": estado.ed_comprimento,
        "R1": estado.ed_r1,
        "X1": estado.ed_x1,
        "S1": estado.ed_s1,
        "R0": estado.ed_r0,
        "X0": estado.ed_x0,
        "S0": estado.ed_s0,
        "capacidadesDe": estado.ed_cap_de.to_vec(),
        "capacidadesPara": estado.ed_cap_para.to_vec(),
        "circuitoEquivalente": estado.ed_circ_eq,
        "status": estado.ed_status,
    });
    let cmd = json!({
        "db": estado.db,
        "documento": estado.documento,
        "handle": handle,
        "de": estado.linha_de,
        "para": estado.linha_para,
        "dados": dados,
    });
    estado.mensagem = match app.run("anacad.linha.editar", cmd) {
        Ok(_) => String::new(),
        Err(e) => format!("erro: {e}"),
    };
}

/// Janela `editar_linha` do diálogo AnaCAD.
pub(super) fn janela_editar_linha(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool, notificacao: &mut Option<String>) {
    let handle = if !estado.comp_handle.trim().is_empty() {
        estado.comp_handle.clone()
    } else {
        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
    };
    estado.comp_handle = handle.clone();
    let chave = format!("l|{}|{}|{}", estado.db, estado.documento, handle);
    if estado.ed_listas_em != chave {
        if estado.ed_handle != handle {
            if let Some(dados) = dados_entidade(app, &handle) {
                preencher_estado(estado, &dados);
            }
            estado.ed_handle = handle.clone();
        }
        recarregar_listas_dados(app, estado, &handle, "linha");
        estado.ed_listas_em = chave;
    }
    let mut selecionar = false;
    let largura = ui.available_width().max(560.0);
    let largura_grupo = (largura - 20.0).max(520.0);
    // No ANAFAS as barras De/Para não se trocam depois de
    // preenchidas: elas vêm das extremidades do traço.
    let travado = estado.ed_existe;
    // Cabeçalho com o handle e a "mão" (selecionar no desenho), como na
    // janela Dados de Barra (149): o rodapé fica só com as ações.
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if crate::icons::button(ui, crate::icons::Icon::SelectObjects, 20.0, "Selecionar no desenho", false)
                .on_hover_text("Selecionar a linha no desenho")
                .clicked()
            {
                selecionar = true;
            }
        });
    });
    // Os campos têm largura fixa (`add_sized`) e os grupos largura
    // mínima: sem isso o conteúdo passa da janela e as legendas saem
    // cortadas. Sem área rolável, como no DIALOGEX do ANAFAS: o
    // formulário inteiro cabe na janela.
    ui.group(|ui| {
        ui.set_min_width(largura_grupo);
        ui.label(egui::RichText::new("Identificação").strong());
        egui::Grid::new("anacad.linha.identificacao").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label("Barra De:");
            campo_barra_da_linha(ui, "anacad.linha.de", &mut estado.linha_de, &estado.barras_opcoes, !travado);
            ui.label("Nome:");
            combo_nome_da_linha(ui, "anacad.linha.de.nome", &mut estado.linha_de, &estado.barras_opcoes, !travado);
            ui.end_row();
            ui.label("Barra Para:");
            campo_barra_da_linha(ui, "anacad.linha.para", &mut estado.linha_para, &estado.barras_opcoes, !travado);
            ui.label("Nome:");
            combo_nome_da_linha(ui, "anacad.linha.para.nome", &mut estado.linha_para, &estado.barras_opcoes, !travado);
            ui.end_row();
            ui.label("Número do Circuito:");
            // Lista suspensa editável: número livre + os circuitos
            // que já existem entre o De e o Para (ANAFAS recarrega
            // o combo de circuitos para o par de barras).
            ui.horizontal(|ui| {
                campo_circuito(ui, &mut estado.ed_circuito, estado.linha_de, estado.linha_para, &estado.circuitos_par);
            });
            ui.label("Área:");
            ui.horizontal(|ui| {
                campo_area(ui, &mut estado.ed_area, &estado.areas_opcoes);
                ui.checkbox(&mut estado.ed_status, "Ligado");
            });
            ui.end_row();
            ui.label("Nome:");
            ui.add_sized([180.0, 20.0], egui::TextEdit::singleline(&mut estado.ed_nome));
            ui.checkbox(&mut estado.ed_circ_eq, "Circuito Equivalente");
            ui.end_row();
        });
    });
    ui.group(|ui| {
        ui.set_min_width(largura_grupo);
        ui.horizontal(|ui| {
            ui.label("Potência Nominal (MVA):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_potencia).speed(1.0));
            ui.label("Comprimento (km):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_comprimento).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.set_min_width(largura_grupo);
        ui.label(egui::RichText::new("Sequência Positiva").strong());
        egui::Grid::new("anacad.linha.seq1").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label("Resistência (R1 %):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_r1).speed(0.1));
            ui.label("Reatância (X1 %):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_x1).speed(0.1));
            ui.end_row();
            ui.label("Susceptância (S1 Mvar):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_s1).speed(0.1));
            ui.end_row();
        });
    });
    ui.group(|ui| {
        ui.set_min_width(largura_grupo);
        ui.label(egui::RichText::new("Sequência Zero").strong());
        egui::Grid::new("anacad.linha.seq0").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label("Resistência (R0 %):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_r0).speed(0.1));
            ui.label("Reatância (X0 %):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_x0).speed(0.1));
            ui.end_row();
            ui.label("Susceptância (S0 Mvar):");
            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_s0).speed(0.1));
            ui.end_row();
        });
    });
    ui.group(|ui| {
        ui.set_min_width(largura_grupo);
        ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
        // De e Para em linhas separadas: as seis capacidades lado a
        // lado não cabem na largura do diálogo.
        ui.horizontal(|ui| {
            ui.label("De:");
            for (i, v) in estado.ed_cap_de.iter_mut().enumerate() {
                ui.label(format!("{}:", i + 1));
                ui.add_sized([80.0, 20.0], egui::DragValue::new(v).speed(0.1));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Para:");
            for (i, v) in estado.ed_cap_para.iter_mut().enumerate() {
                ui.label(format!("{}:", i + 1));
                ui.add_sized([80.0, 20.0], egui::DragValue::new(v).speed(0.1));
            }
        });
    });
    ui.separator();
    ui.horizontal(|ui| {
        let existe = estado.ed_existe;
        if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
            aplicar_linha(app, estado, &handle);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some(format!("linha {handle} inserida"));
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
            aplicar_linha(app, estado, &handle);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some(format!("linha {handle} atualizada"));
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
            let cmd = json!({
                "db": estado.db,
                "documento": estado.documento,
                "handle": handle,
            });
            match app.run("anacad.linha.dados.excluir", cmd) {
                Ok(_) => {
                    estado.ed_existe = false;
                    estado.ed_listas_em.clear();
                    estado.mensagem = "dados elétricos da linha removidos".to_string();
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
        if ui.button("Fechar").clicked() {
            *fechar = true;
        }
    });
    if selecionar {
        app.ui.anacad_dialogo.clear();
        app.ui.anacad_handle.clear();
        app.start("anacad.linha.editar");
    }
}
