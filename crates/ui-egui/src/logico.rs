//! Interface do modulo **logico**: o painel dockado e o comando
//! `ui.log.painel`.
//!
//! A UI e fina: os botoes chamam os comandos do motor (`app.run`, `log.*`)
//! e o estado da tela (banco, pagina, ultimo resultado) vive na memoria do egui,
//! como no painel do anacad.

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::CadApp;
use crate::domain::{DomainUi, UiCommand};

/// O menu que o modulo acrescenta a barra de menus.
pub const MENU: &str = "Lógico";

const ID_ESTADO: &str = "logico.estado";

/// Comandos so de interface: `(id, rotulo, caminho de menu, atalho)`.
pub const UI_COMMANDS: &[UiCommand] = &[(
    "ui.log.painel",
    "Painel Lógico",
    &["Lógico", "Painel"],
    None,
)];

#[derive(Clone, Default)]
struct Estado {
    arquivo: String,
    pagina: String,
    titulo1: String,
    unidade: String,
    lista: Vec<String>,
    mensagem: String,
}

/// O modulo como a casca o ve.
pub struct UiLogico;

impl DomainUi for UiLogico {
    fn id(&self) -> &'static str {
        "logico"
    }
    fn ui_commands(&self) -> &'static [UiCommand] {
        UI_COMMANDS
    }
    fn panel(&self, panel: crate::docking::Panel, app: &mut CadApp, ui: &mut egui::Ui) -> bool {
        if panel != crate::docking::Panel::Logico {
            return false;
        }
        egui::ScrollArea::vertical()
            .id_salt("dock-logico")
            .auto_shrink([false, false])
            .show(ui, |ui| painel(app, ui));
        true
    }
    fn route(&self, app: &mut CadApp, id: &str, _params: &Value) -> Option<Result<Value, String>> {
        (id == "ui.log.painel").then(|| abrir(app))
    }
}

/// O modulo registrado em [`crate::domain::builtin`].
pub static LOGICO_UI: UiLogico = UiLogico;

/// Abre (ou ativa) o painel dockado, como o QuickCalc faz.
pub fn abrir(app: &mut CadApp) -> Result<Value, String> {
    let panel = crate::docking::Panel::Logico;
    crate::docking::command(app, false, &json!({ "operation": "open", "panel": panel.id() }))
}

fn painel(app: &mut CadApp, ui: &mut Ui) {
    let mut estado: Estado = ui
        .ctx()
        .data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    ui.add_space(6.0);
    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);

    ui.label("Banco do projeto (.db)");
    ui.add(
        egui::TextEdit::singleline(&mut estado.arquivo)
            .desired_width(f32::INFINITY)
            .hint_text(r"C:\...\projeto.db"),
    );
    ui.horizontal(|ui| {
        if ui.button("Abrir / criar").clicked() {
            estado.mensagem = executar(
                app,
                "log.banco.abrir",
                json!({ "arquivo": estado.arquivo.trim() }),
                |v| match v.get("paginas").and_then(Value::as_u64) {
                    Some(total) => format!("banco aberto: {total} página(s)"),
                    None => "banco aberto".to_string(),
                },
            );
        }
        if ui.button("Listar páginas").clicked() {
            let arquivo = estado.arquivo.trim().to_string();
            match app.run("log.pagina.listar", json!({ "arquivo": arquivo })) {
                Ok(v) => {
                    estado.lista = v
                        .get("paginas")
                        .and_then(Value::as_array)
                        .map(|lista| {
                            lista
                                .iter()
                                .map(|p| {
                                    let chave = p.get("pagina").and_then(Value::as_str).unwrap_or("?");
                                    let titulo = p.get("titulo1").and_then(Value::as_str).unwrap_or("");
                                    if titulo.is_empty() {
                                        chave.to_string()
                                    } else {
                                        format!("{chave}  {titulo}")
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                    estado.mensagem = format!("{total} página(s) no banco");
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
    });

    ui.separator();
    ui.label("Página");
    ui.add(egui::TextEdit::singleline(&mut estado.pagina).desired_width(f32::INFINITY).hint_text("12A"));
    ui.label("Título 1");
    ui.add(egui::TextEdit::singleline(&mut estado.titulo1).desired_width(f32::INFINITY));
    ui.label("Unidade");
    ui.add(egui::TextEdit::singleline(&mut estado.unidade).desired_width(f32::INFINITY));
    if ui.button("Gravar página").clicked() {
        estado.mensagem = executar(
            app,
            "log.pagina.gravar",
            json!({
                "arquivo": estado.arquivo.trim(),
                "pagina": estado.pagina.trim(),
                "titulo1": estado.titulo1.trim(),
                "unidade": estado.unidade.trim(),
            }),
            |v| {
                let chave = v.get("pagina").and_then(Value::as_str).unwrap_or("?");
                let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                format!("página {chave} gravada ({total} no banco)")
            },
        );
    }

    if !estado.lista.is_empty() {
        ui.separator();
        for item in &estado.lista {
            ui.monospace(item);
        }
    }
    if !estado.mensagem.is_empty() {
        ui.separator();
        ui.label(&estado.mensagem);
    }

    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

/// Roda um comando do motor e resume o resultado (ou o erro) numa mensagem.
fn executar(
    app: &mut CadApp,
    cmd: &str,
    params: Value,
    resumo: impl FnOnce(&Value) -> String,
) -> String {
    match app.run(cmd, params) {
        Ok(v) => resumo(&v),
        Err(e) => format!("erro: {e}"),
    }
}
