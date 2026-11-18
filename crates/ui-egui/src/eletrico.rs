//! Interface do modulo **eletrico**: o painel dockado e o comando
//! `ui.ele.painel`.
//!
//! A UI e fina: os botoes chamam os comandos do motor (`app.run`, `ele.*`)
//! e o estado da tela (banco, pagina, ultimo resultado) vive na memoria do egui,
//! como no painel do anacad.

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::CadApp;
use crate::domain::{DomainUi, UiCommand};

/// O menu que o modulo acrescenta a barra de menus.
pub const MENU: &str = "Elétrico";

const ID_ESTADO: &str = "eletrico.estado";

/// Comandos so de interface: `(id, rotulo, caminho de menu, atalho)`.
pub const UI_COMMANDS: &[UiCommand] = &[(
    "ui.ele.painel",
    "Painel Elétrico",
    &["Elétrico", "Painel"],
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
    /// Desenho de que o painel ja leu o banco (troca de arquivo = troca de banco).
    carregado: String,
}

/// O modulo como a casca o ve.
pub struct UiEletrico;

impl DomainUi for UiEletrico {
    fn id(&self) -> &'static str {
        "eletrico"
    }
    fn ui_commands(&self) -> &'static [UiCommand] {
        UI_COMMANDS
    }
    /// O banco do desenho, relido a cada troca de aba: e o XData do proprio
    /// arquivo que decide qual `.db` o painel usa, como o `LerXDataConfiguracao`
    /// do original.
    fn begin_frame(&self, app: &mut CadApp, ctx: &egui::Context) {
        sincronizar_desenho(app, ctx);
    }
    fn panel(&self, panel: crate::docking::Panel, app: &mut CadApp, ui: &mut egui::Ui) -> bool {
        if panel != crate::docking::Panel::Eletrico {
            return false;
        }
        egui::ScrollArea::vertical()
            .id_salt("dock-eletrico")
            .auto_shrink([false, false])
            .show(ui, |ui| painel(app, ui));
        true
    }
    fn route(&self, app: &mut CadApp, id: &str, _params: &Value) -> Option<Result<Value, String>> {
        (id == "ui.ele.painel").then(|| abrir(app))
    }
}

/// O modulo registrado em [`crate::domain::builtin`].
pub static ELETRICO_UI: UiEletrico = UiEletrico;

/// Identidade do desenho ativo (caminho, ou aba e titulo quando ainda nao foi salvo).
fn chave_do_desenho(app: &CadApp) -> String {
    match app.session.docs.get(app.session.active) {
        Some(d) => d
            .path
            .clone()
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| format!("#{} {}", app.session.active, d.title)),
        None => String::new(),
    }
}

/// Troca de arquivo, troca de banco: relê o vinculo do desenho uma vez por aba.
fn sincronizar_desenho(app: &mut CadApp, ctx: &egui::Context) {
    let desenho = chave_do_desenho(app);
    if desenho.is_empty() {
        return;
    }
    let mut estado: Estado =
        ctx.data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    if estado.carregado == desenho {
        return;
    }
    estado.carregado = desenho;
    estado.arquivo = cadcraft_engine::cmd::eletrico::banco_do_desenho(&app.session);
    estado.lista.clear();
    estado.mensagem = if estado.arquivo.is_empty() {
        "desenho sem banco: vincule um .db".to_string()
    } else {
        String::new()
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

/// Abre (ou ativa) o painel dockado, como o QuickCalc faz.
pub fn abrir(app: &mut CadApp) -> Result<Value, String> {
    let panel = crate::docking::Panel::Eletrico;
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
                "ele.banco.abrir",
                json!({ "arquivo": estado.arquivo.trim() }),
                |v| match v.get("paginas").and_then(Value::as_u64) {
                    Some(total) => format!("banco aberto: {total} página(s)"),
                    None => "banco aberto".to_string(),
                },
            );
        }
        if ui.button("Listar páginas").clicked() {
            let arquivo = estado.arquivo.trim().to_string();
            match app.run("ele.pagina.listar", json!({ "arquivo": arquivo })) {
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

    ui.horizontal(|ui| {
        if ui.button("Vincular ao desenho").clicked() {
            let arquivo = estado.arquivo.trim().to_string();
            estado.mensagem = executar(
                app,
                "ele.projeto.gravar",
                json!({ "banco": arquivo }),
                |v| {
                    let handle = v.get("handle").and_then(Value::as_str).unwrap_or("?");
                    format!("banco vinculado ao desenho ({handle})")
                },
            );
        }
        if ui.button("Ler do desenho").clicked() {
            match app.run("ele.pagina.ler", json!({})) {
                Ok(v) => {
                    estado.lista = lista_do_json(&v);
                    let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                    estado.mensagem = format!("{total} página(s) no desenho");
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
    ui.horizontal(|ui| {
        if ui.button("Gravar página").clicked() {
            estado.mensagem = gravar_pagina(app, &estado, "ele.pagina.gravar");
        }
        if ui.button("Gravar no desenho").clicked() {
            estado.mensagem = gravar_pagina(app, &estado, "ele.pagina.escrever");
        }
    });
    if ui.button("Sincronizar banco com o desenho").clicked() {
        estado.mensagem = executar(
            app,
            "ele.pagina.sincronizar",
            json!({ "banco": estado.arquivo.trim() }),
            |v| {
                let gravadas = v.get("gravadas").and_then(Value::as_u64).unwrap_or(0);
                format!("banco sincronizado: {gravadas} página(s)")
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

/// Roda `pagina.gravar` ou `pagina.escrever` com os campos do painel.
fn gravar_pagina(app: &mut CadApp, estado: &Estado, cmd: &str) -> String {
    executar(
        app,
        cmd,
        json!({
            "arquivo": estado.arquivo.trim(),
            "banco": estado.arquivo.trim(),
            "pagina": estado.pagina.trim(),
            "titulo1": estado.titulo1.trim(),
            "unidade": estado.unidade.trim(),
        }),
        |v| {
            let chave = v.get("pagina").and_then(Value::as_str).unwrap_or("?");
            match v.get("gravado_no_banco").and_then(Value::as_bool) {
                Some(true) => format!("página {chave} gravada no desenho e no banco"),
                Some(false) => format!("página {chave} gravada no desenho (sem banco)"),
                None => {
                    let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                    format!("página {chave} gravada ({total} no banco)")
                }
            }
        },
    )
}

/// Lista de paginas de um resultado de comando, como o painel mostra.
fn lista_do_json(v: &Value) -> Vec<String> {
    v.get("paginas")
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
        .unwrap_or_default()
}
