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
),
    (
        "ui.log.titulos",
        "Editar Títulos",
        &["Lógico", "Editar Títulos"],
        None,
    ),
];

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
pub struct UiLogico;

impl DomainUi for UiLogico {
    fn id(&self) -> &'static str {
        "logico"
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
    fn dialogs(&self, app: &mut CadApp, ctx: &egui::Context) {
        janela_titulos(app, ctx);
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
        match id {
            "ui.log.painel" => Some(abrir(app)),
            "ui.log.titulos" => Some(abrir_titulos(app)),
            _ => None,
        }
    }
}

/// O modulo registrado em [`crate::domain::builtin`].
pub static LOGICO_UI: UiLogico = UiLogico;

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
    estado.arquivo = cadcraft_engine::cmd::logico::banco_do_desenho(&app.session);
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

    ui.horizontal(|ui| {
        if ui.button("Vincular ao desenho").clicked() {
            let arquivo = estado.arquivo.trim().to_string();
            estado.mensagem = executar(
                app,
                "log.projeto.gravar",
                json!({ "banco": arquivo }),
                |v| {
                    let handle = v.get("handle").and_then(Value::as_str).unwrap_or("?");
                    format!("banco vinculado ao desenho ({handle})")
                },
            );
        }
        if ui.button("Ler do desenho").clicked() {
            match app.run("log.pagina.ler", json!({})) {
                Ok(v) => {
                    estado.lista = lista_do_json(&v);
                    let total = v.get("total").and_then(Value::as_u64).unwrap_or(0);
                    estado.mensagem = format!("{total} página(s) no desenho");
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
        if ui.button("Editar Títulos").clicked() {
            let _ = abrir_titulos(app);
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
            estado.mensagem = gravar_pagina(app, &estado, "log.pagina.gravar");
        }
        if ui.button("Gravar no desenho").clicked() {
            estado.mensagem = gravar_pagina(app, &estado, "log.pagina.escrever");
        }
    });
    if ui.button("Sincronizar banco com o desenho").clicked() {
        estado.mensagem = executar(
            app,
            "log.pagina.sincronizar",
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

const ID_JANELA: &str = "logico.janela_titulos";

/// Uma linha da grade de títulos: uma página e seus campos editáveis.
#[derive(Clone, Default)]
struct LinhaTitulos {
    pagina: String,
    titulo1: String,
    titulo2: String,
    titulo3: String,
    titulo4: String,
    unidade: String,
    alternativo: String,
}

/// Estado da janela **Editar Títulos**, na memória do egui (como o painel).
#[derive(Clone, Default)]
struct JanelaTitulos {
    lista: Vec<LinhaTitulos>,
    carregada: bool,
    pagina: String,
    titulo1: String,
    titulo2: String,
    titulo3: String,
    titulo4: String,
    unidade: String,
    alternativo: String,
    de: String,
    para: String,
    mensagem: String,
}

/// Abre a janela **Editar Títulos** (`frmEditaTitulos` no original).
pub fn abrir_titulos(app: &mut CadApp) -> Result<Value, String> {
    app.ui.logico_janela_titulos = true;
    Ok(json!({ "janela": "titulos" }))
}

/// Texto de um campo do resultado de um comando.
fn campo_json(v: &Value, chave: &str) -> String {
    v.get(chave).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Lê as páginas do desenho para a grade (e pede a janela para redesenhar).
fn carregar_lista(app: &mut CadApp, janela: &mut JanelaTitulos) {
    janela.carregada = true;
    match app.run("log.pagina.ler", json!({})) {
        Ok(v) => {
            janela.lista = v
                .get("paginas")
                .and_then(Value::as_array)
                .map(|paginas| {
                    paginas
                        .iter()
                        .map(|p| LinhaTitulos {
                            pagina: campo_json(p, "pagina"),
                            titulo1: campo_json(p, "titulo1"),
                            titulo2: campo_json(p, "titulo2"),
                            titulo3: campo_json(p, "titulo3"),
                            titulo4: campo_json(p, "titulo4"),
                            unidade: campo_json(p, "unidade"),
                            alternativo: campo_json(p, "alternativo"),
                        })
                        .collect()
                })
                .unwrap_or_default();
            janela.mensagem.clear();
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Parâmetros de um `log.pagina.escrever` com os campos da linha.
fn linha_para_json(janela: &JanelaTitulos) -> Value {
    json!({
        "pagina": janela.pagina.trim(),
        "titulo1": janela.titulo1.trim(),
        "titulo2": janela.titulo2.trim(),
        "titulo3": janela.titulo3.trim(),
        "titulo4": janela.titulo4.trim(),
        "unidade": janela.unidade.trim(),
        "alternativo": janela.alternativo.trim(),
    })
}

/// Grava a página da janela no desenho (e espelha no `.db`) e relê a grade.
fn gravar_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    if janela.pagina.trim().is_empty() {
        janela.mensagem = "informe a página (nome da camada)".to_string();
        return;
    }
    match app.run("log.pagina.escrever", linha_para_json(janela)) {
        Ok(v) => {
            let criados = v.get("criados").and_then(Value::as_u64).unwrap_or(0);
            let atualizados = v.get("atualizados").and_then(Value::as_u64).unwrap_or(0);
            let nome = janela.pagina.trim().to_string();
            carregar_lista(app, janela);
            janela.mensagem = format!("página {nome}: {criados} criado(s), {atualizados} atualizado(s)");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Limpa os títulos da página (unidade e alternativo ficam como estão).
fn limpar_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    if janela.pagina.trim().is_empty() {
        janela.mensagem = "informe a página (nome da camada)".to_string();
        return;
    }
    let nome = janela.pagina.trim().to_string();
    janela.titulo1.clear();
    janela.titulo2.clear();
    janela.titulo3.clear();
    janela.titulo4.clear();
    match app.run("log.pagina.escrever", linha_para_json(janela)) {
        Ok(_) => {
            carregar_lista(app, janela);
            janela.mensagem = format!("títulos de {nome} limpos");
        }
        Err(e) => janela.mensagem = format!("erro: {e}"),
    }
}

/// Substituir De→Para em todos os títulos de todas as páginas (como o
/// `DeToolStripTextBox`/`ParaToolStripTextBox` do original).
fn substituir_janela(app: &mut CadApp, janela: &mut JanelaTitulos) {
    let de = janela.de.trim().to_string();
    let para = janela.para.trim().to_string();
    if de.is_empty() {
        janela.mensagem = "informe o texto a substituir em `de`".to_string();
        return;
    }
    let mut contadas = 0u32;
    for linha in janela.lista.clone() {
        let titulos = [
            linha.titulo1.replace(&de, &para),
            linha.titulo2.replace(&de, &para),
            linha.titulo3.replace(&de, &para),
            linha.titulo4.replace(&de, &para),
        ];
        if titulos[0] != linha.titulo1
            || titulos[1] != linha.titulo2
            || titulos[2] != linha.titulo3
            || titulos[3] != linha.titulo4
        {
            let parametros = json!({
                "pagina": linha.pagina,
                "titulo1": titulos[0],
                "titulo2": titulos[1],
                "titulo3": titulos[2],
                "titulo4": titulos[3],
                "unidade": linha.unidade,
                "alternativo": linha.alternativo,
            });
            if app.run("log.pagina.escrever", parametros).is_ok() {
                contadas += 1;
            }
        }
    }
    carregar_lista(app, janela);
    janela.mensagem = format!("{contadas} página(s) substituída(s)");
}

/// A janela **Editar Títulos** (`frmEditaTitulos` do original): grade de
/// páginas com título 1–4, unidade e alternativo, botões de cima espelhando o
/// menu de contexto (`Atualizar lista`; `Copiar`/`Colar`/`Exportar`/`Importar`
/// ainda não), `Substituir` com `De`/`Para` e `Limpar`, e a edição da página
/// selecionada com `Gravar` — tudo sobre os comandos do M4b.
fn janela_titulos(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ui.logico_janela_titulos {
        return;
    }
    let mut janela: JanelaTitulos = ctx
        .data_mut(|d| d.get_temp_mut_or_default::<JanelaTitulos>(egui::Id::new(ID_JANELA)).clone());
    if !janela.carregada {
        carregar_lista(app, &mut janela);
    }
    let mut aberta = true;
    egui::Window::new("Editar Títulos")
        .id(egui::Id::new(ID_JANELA))
        .open(&mut aberta)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Atualizar lista").clicked() {
                    carregar_lista(app, &mut janela);
                }
                if ui.button("Limpar página").clicked() {
                    limpar_janela(app, &mut janela);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Substituir");
                ui.add(egui::TextEdit::singleline(&mut janela.de).hint_text("de").desired_width(120.0));
                ui.label("para");
                ui.add(egui::TextEdit::singleline(&mut janela.para).hint_text("para").desired_width(120.0));
                if ui.button("Substituir").clicked() {
                    substituir_janela(app, &mut janela);
                }
            });

            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                egui::Grid::new("grade_titulos").striped(true).show(ui, |ui| {
                    for cabecalho in
                        ["Página", "Título 1", "Título 2", "Título 3", "Título 4", "Unidade", "Alternativo"]
                    {
                        ui.label(cabecalho);
                    }
                    ui.end_row();
                    for linha in janela.lista.clone() {
                        let selecionada = linha.pagina == janela.pagina;
                        if ui.selectable_label(selecionada, &linha.pagina).clicked() {
                            janela.pagina = linha.pagina.clone();
                            janela.titulo1 = linha.titulo1.clone();
                            janela.titulo2 = linha.titulo2.clone();
                            janela.titulo3 = linha.titulo3.clone();
                            janela.titulo4 = linha.titulo4.clone();
                            janela.unidade = linha.unidade.clone();
                            janela.alternativo = linha.alternativo.clone();
                        }
                        ui.label(&linha.titulo1);
                        ui.label(&linha.titulo2);
                        ui.label(&linha.titulo3);
                        ui.label(&linha.titulo4);
                        ui.label(&linha.unidade);
                        ui.label(&linha.alternativo);
                        ui.end_row();
                    }
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Página");
                ui.add(egui::TextEdit::singleline(&mut janela.pagina).desired_width(140.0).hint_text("12A"));
                if ui.button("Gravar").clicked() {
                    gravar_janela(app, &mut janela);
                }
            });
            ui.horizontal(|ui| {
                for (rotulo, valor) in [
                    ("Título 1", &mut janela.titulo1),
                    ("Título 2", &mut janela.titulo2),
                    ("Título 3", &mut janela.titulo3),
                    ("Título 4", &mut janela.titulo4),
                ] {
                    ui.label(rotulo);
                    ui.add(egui::TextEdit::singleline(valor).desired_width(160.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Unidade");
                ui.add(egui::TextEdit::singleline(&mut janela.unidade).desired_width(160.0));
                ui.label("Alternativo");
                ui.add(egui::TextEdit::singleline(&mut janela.alternativo).desired_width(160.0));
            });
            if !janela.mensagem.is_empty() {
                ui.label(&janela.mensagem);
            }
        });
    if !aberta {
        app.ui.logico_janela_titulos = false;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_JANELA), janela));
}
