//! Painel dockado do AnaCAD e entrega do contexto (banco/documento) ao motor.

use super::*;

/// Hand the UI's connected database and active document to the engine before starting a command
/// that reads them (the interactive `anacad.*` commands take them from the session).
pub(crate) fn contexto(app: &mut CadApp) {
    cadcraft_engine::cmd::anacad::set_banco(&mut app.session, &app.ui.anacad_db, app.ui.anacad_documento);
}

pub fn panel(app: &mut CadApp, ui: &mut Ui) {
    let mut estado: Estado = ui.ctx().data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    // O painel só mostra o banco do desenho: quem o troca é a Configuração do
    // Projeto, e o valor vem do `ANACAD_CFG`, relido a cada troca de aba. Sem
    // este espelho o painel devolveria à sessão o banco do desenho anterior.
    estado.db = app.ui.anacad_db.clone();
    estado.documento = app.ui.anacad_documento.max(1);
    // A barra do módulo mora aqui, não numa linha da barra do CAD, e fica fixa
    // no alto da janela: barra, linha e equipamentos abrem suas janelas daqui.
    crate::anacad::contrib::ferramentas(app, ui);
    ui.separator();
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
    app.ui.anacad_documento = estado.documento;
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}
