//! What the module adds to the shared shell: its menu entries, its Tool Sets tab, its Tool Bar
//! row, the dockable palette of its tools, and the router for its `ui.anacad.*` commands.
//!
//! It lives here, not in `menus.rs`/`palettes.rs`/`chrome.rs`, so those core files keep a single
//! line per extension point and the module stays one folder.

use egui::{Rect, Sense, Stroke, pos2, vec2};
use serde_json::{Value, json};

use crate::CadApp;
use crate::icons::{self, Icon};
use crate::palettes::{Tool, command_known};
use crate::theme::Tokens;

/// The menu this module adds to the menu bar.
pub const MENU: &str = "AnaCAD";
/// The Tool Sets tab this module adds.
pub const TAB: &str = "AnaCAD";

/// UI-only commands: (id, label, menu path, shortcut).
pub const UI_COMMANDS: &[(&str, &str, &[&str], Option<&str>)] = &[
    ("ui.anacad", "Painel AnaCAD", &["AnaCAD", "Toolbar", "Painel AnaCAD"], None),
    ("ui.anacad.toolset", "Ferramentas AnaCAD", &["AnaCAD", "Toolbar", "Ferramentas AnaCAD"], None),
    ("ui.anacad.banco.conectar", "", &[], None),
    ("ui.anacad.equip.transformador", "Transformador", &["AnaCAD", "Equipamentos", "Transformador"], None),
    ("ui.anacad.equip.gerador", "Gerador", &["AnaCAD", "Equipamentos", "Gerador"], None),
    ("ui.anacad.equip.carga", "Carga", &["AnaCAD", "Equipamentos", "Carga"], None),
    ("ui.anacad.equip.shunt", "Shunt de Barra", &["AnaCAD", "Equipamentos", "Shunt de Barra"], None),
    ("ui.anacad.equip.shunt_linha", "Shunt de Linha", &["AnaCAD", "Equipamentos", "Shunt de Linha"], None),
    ("ui.anacad.equip.compensador_serie", "Compensador Série", &["AnaCAD", "Equipamentos", "Compensador Série"], None),
    ("ui.anacad.equip.trafo_aterramento", "Trafo de Aterramento", &["AnaCAD", "Equipamentos", "Trafo de Aterramento"], None),
    ("ui.anacad.equip.gerador_fonte", "Gerador Fonte de Corrente", &["AnaCAD", "Equipamentos", "Gerador Fonte de Corrente"], None),
    ("ui.anacad.equip.impedancia_mutua", "Impedância Mútua", &["AnaCAD", "Equipamentos", "Impedância Mútua"], None),
    ("ui.anacad.equip.motor", "Motor de Indução", &["AnaCAD", "Equipamentos", "Motor de Indução"], None),
    ("ui.anacad.equip.compensador_estatico", "Compensador Estático", &["AnaCAD", "Equipamentos", "Compensador Estático"], None),
    ("ui.anacad.texto.livre", "Texto Livre...", &["AnaCAD", "Inserir", "Texto Livre..."], None),
    ("ui.anacad.inserir.terminal", "Terminal...", &["AnaCAD", "Inserir", "Terminal..."], None),
    ("ui.anacad.inserir.monitoracao", "Ponto de Monitoração...", &["AnaCAD", "Inserir", "Ponto de Monitoração..."], None),
    ("ui.anacad.inserir.quebra", "Ponto de Quebra...", &["AnaCAD", "Inserir", "Ponto de Quebra..."], None),
    ("ui.anacad.legenda.opcoes", "Opções de Legenda...", &["AnaCAD", "Exibir", "Opções de Legenda..."], None),
    ("ui.anacad.anotar.curto", "Níveis de Curto no Diagrama", &["AnaCAD", "Exibir", "Níveis de Curto no Diagrama"], None),
    ("ui.anacad.exibir.normal", "Desenho Normal", &["AnaCAD", "Exibir", "Desenho Normal"], None),
    ("ui.anacad.exibir.nao_desenhados", "Elementos Não Desenhados", &["AnaCAD", "Exibir", "Elementos Não Desenhados"], None),
    ("ui.anacad.exibir.mutuas", "Linhas com Mútuas", &["AnaCAD", "Exibir", "Linhas com Mútuas"], None),
    ("ui.anacad.exibir.esquematico", "Esquemático", &["AnaCAD", "Exibir", "Esquemático"], None),
    ("ui.anacad.exibir.unifilar", "Unifilar", &["AnaCAD", "Exibir", "Unifilar"], None),
    ("ui.anacad.detalhar.barra", "Detalhar Barra...", &["AnaCAD", "Topologia", "Detalhar Barra..."], None),
    ("ui.anacad.renomear.terminal", "Renomear Terminal...", &["AnaCAD", "Topologia", "Renomear Terminal..."], None),
    ("ui.anacad.localizar.terminal", "Localizar Terminal...", &["AnaCAD", "Topologia", "Localizar Terminal..."], None),
    ("ui.anacad.renumerar.barra", "Renumerar Barra...", &["AnaCAD", "Topologia", "Renumerar Barra..."], None),
    ("ui.anacad.localizar.barra", "Localizar Barra...", &["AnaCAD", "Topologia", "Localizar Barra..."], None),
    ("ui.anacad.seccionar.linha", "Seccionar Linha...", &["AnaCAD", "Topologia", "Seccionar Linha..."], None),
    ("ui.anacad.separar.barra", "Separar Barra...", &["AnaCAD", "Topologia", "Separar Barra..."], None),
    ("ui.anacad.unir.barras", "Unir Barras...", &["AnaCAD", "Topologia", "Unir Barras..."], None),
    ("ui.anacad.banco.novo", "Novo Banco...", &["AnaCAD", "Banco", "Novo Banco..."], None),
    ("ui.anacad.banco.abrir", "Abrir Banco...", &["AnaCAD", "Banco", "Abrir Banco..."], None),
    ("ui.anacad.documento.novo", "Novo Documento...", &["AnaCAD", "Banco", "Novo Documento..."], None),
    ("ui.anacad.importar.lst", "Importar .LST", &["AnaCAD", "Arquivo", "Importar .LST"], None),
    ("ui.anacad.importar.ana", "Importar .ANA", &["AnaCAD", "Arquivo", "Importar .ANA"], None),
    ("ui.anacad.exportar", "Exportar .LST", &["AnaCAD", "Arquivo", "Exportar .LST"], None),
    ("ui.anacad.ana.exportar", "Salvar .ANA", &["AnaCAD", "Arquivo", "Salvar .ANA"], None),
    ("ui.anacad.relatorio", "Relatório", &["AnaCAD", "Arquivo", "Relatório"], None),
    ("ui.anacad.consultar", "Consultar", &["AnaCAD", "Dados", "Consultar"], None),
    ("ui.anacad.resultados", "Resultados", &["AnaCAD", "Dados", "Resultados"], None),
    ("ui.anacad.topologia", "Topologia", &["AnaCAD", "Cálculos", "Topologia"], None),
    ("ui.anacad.comprimentos", "Comprimentos", &["AnaCAD", "Cálculos", "Comprimentos"], None),
    ("ui.anacad.ybus", "Ybus", &["AnaCAD", "Cálculos", "Ybus"], None),
    ("ui.anacad.equivalente", "Equivalente de Rede...", &["AnaCAD", "Cálculos", "Equivalente de Rede..."], None),
    ("ui.anacad.curto.simultaneo", "Faltas Simultâneas...", &["AnaCAD", "Cálculos", "Faltas Simultâneas..."], None),
    ("ui.anacad.zbus", "Matriz Zbarra...", &["AnaCAD", "Cálculos", "Matriz Zbarra..."], None),
    ("ui.anacad.niveis.curto", "Níveis de Curto...", &["AnaCAD", "Cálculos", "Níveis de Curto..."], None),
    ("ui.anacad.curto", "Curto-circuito", &["AnaCAD", "Cálculos", "Curto-circuito"], None),
    ("ui.anacad.atribuir.barra", "Atribuir Barra", &["AnaCAD", "Edição", "Atribuir Barra"], None),
    ("ui.anacad.atribuir.linha", "Atribuir Linha", &["AnaCAD", "Edição", "Atribuir Linha"], None),
    ("ui.anacad.editar.barra", "Editar Barra", &["AnaCAD", "Edição", "Editar Barra"], None),
    ("ui.anacad.impedancia", "Impedância entre Barras", &["AnaCAD", "Cálculos", "Impedância entre Barras"], None),
    ("ui.anacad.superacao", "Superação de Disjuntores", &["AnaCAD", "Cálculos", "Superação de Disjuntores"], None),
    ("ui.anacad.editar.linha", "Editar Linha", &["AnaCAD", "Edição", "Editar Linha"], None),
    ("ui.anacad.mover", "Mover", &["AnaCAD", "Edição", "Mover"], None),
    ("ui.anacad.apagar", "Apagar", &["AnaCAD", "Edição", "Apagar"], None),
    ("ui.anacad.criar.barra", "Criar Barra", &["AnaCAD", "Edição", "Criar Barra"], None),
    ("ui.anacad.criar.linha", "Criar Linha", &["AnaCAD", "Edição", "Criar Linha"], None),
    ("ui.anacad.criar.legenda", "Criar Legenda", &["AnaCAD", "Edição", "Criar Legenda"], None),
];

/// The Tool Bar row this module adds (before Help).
pub const TOOLBAR: &[(Icon, &str, &str, Option<&str>)] = &[
    (Icon::AnacadTransformador, "ui.anacad", "AnaCAD", None),
    (Icon::Import, "ui.anacad.importar.lst", "AnaCAD: Importar", None),
    (Icon::Properties, "ui.anacad.consultar", "AnaCAD: Consultar", None),
    (Icon::Table, "ui.anacad.topologia", "AnaCAD: Topologia", None),
    (Icon::AnacadBarra, "ui.anacad.criar.barra", "AnaCAD: Criar Barra", None),
];

/// The module's Tool Sets tab: (group name, large tools, small tools).
pub const TOOLSET: &[(&str, &[Tool], &[Tool])] = &[
    (
        "Unifilar",
        &[
            (Icon::AnacadBarra, "ui.anacad.criar.barra", "Barra"),
            (Icon::AnacadLinha, "ui.anacad.criar.linha", "Linha"),
            (Icon::AnacadTransformador, "ui.anacad.equip.transformador", "Transformador"),
            (Icon::AnacadShunt, "ui.anacad.equip.shunt", "Shunt de Barra"),
            (Icon::AnacadGerador, "ui.anacad.equip.gerador", "Gerador"),
            (Icon::AnacadEquivalente, "ui.anacad.equip.gerador_fonte", "Equivalente de Rede"),
            (Icon::AnacadCarga, "ui.anacad.equip.carga", "Carga"),
            (Icon::AnacadShuntLinha, "ui.anacad.equip.shunt_linha", "Shunt de Linha"),
            (Icon::AnacadTrafoAterramento, "ui.anacad.equip.trafo_aterramento", "Trafo de Aterramento"),
            (Icon::AnacadInjecao, "ui.anacad.equip.compensador_estatico", "Compensador Estático / Injeção"),
            (Icon::AnacadCompensadorSerie, "ui.anacad.equip.compensador_serie", "Compensador Série"),
            (Icon::AnacadAcoplador, "ui.anacad.unir.barras", "Acoplador / Unir Barras"),
            (Icon::AnacadPmn, "ui.anacad.inserir.monitoracao", "Ponto de Monitoração (PMN)"),
            (Icon::AnacadTerminal, "ui.anacad.inserir.terminal", "Terminal"),
            (Icon::AnacadQuebra, "ui.anacad.inserir.quebra", "Ponto de Quebra"),
            (Icon::AnacadTexto, "ui.anacad.texto.livre", "Texto Livre"),
            (Icon::AnacadLegenda, "ui.anacad.criar.legenda", "Legenda"),
        ],
        &[],
    ),
    (
        "Exibir",
        &[
            (Icon::Properties, "ui.anacad.exibir.normal", "Desenho Normal"),
            (Icon::Layers, "ui.anacad.exibir.esquematico", "Esquemático"),
            (Icon::AnacadMutua, "ui.anacad.exibir.mutuas", "Linhas com Mútuas"),
            (Icon::AnacadLinha, "ui.anacad.exibir.unifilar", "Unifilar"),
            (Icon::Annotation, "ui.anacad.anotar.curto", "Anotar Curto"),
            (Icon::AnacadLegenda, "ui.anacad.legenda.opcoes", "Opções de Legenda"),
        ],
        &[],
    ),
    (
        "Inserir",
        &[
            (Icon::AnacadTerminal, "ui.anacad.inserir.terminal", "Terminal"),
            (Icon::AnacadPmn, "ui.anacad.inserir.monitoracao", "Ponto de Monitoração"),
            (Icon::AnacadQuebra, "ui.anacad.inserir.quebra", "Ponto de Quebra"),
            (Icon::AnacadTexto, "ui.anacad.texto.livre", "Texto Livre"),
        ],
        &[],
    ),
    (
        "Topologia",
        &[
            (Icon::Break, "ui.anacad.separar.barra", "Separar Barra"),
            (Icon::Line, "ui.anacad.seccionar.linha", "Seccionar Linha"),
            (Icon::Text, "ui.anacad.renumerar.barra", "Renumerar Barra"),
            (Icon::ZoomWindow, "ui.anacad.localizar.barra", "Localizar Barra"),
            (Icon::Text, "ui.anacad.renomear.terminal", "Renomear Terminal"),
            (Icon::ZoomWindow, "ui.anacad.localizar.terminal", "Localizar Terminal"),
            (Icon::Properties, "ui.anacad.detalhar.barra", "Detalhar Barra"),
            (Icon::AnacadAcoplador, "ui.anacad.unir.barras", "Unir Barras"),
        ],
        &[],
    ),
    (
        "Equipamentos",
        &[
            (Icon::AnacadTransformador, "ui.anacad.equip.transformador", "Transformador"),
            (Icon::AnacadGerador, "ui.anacad.equip.gerador", "Gerador"),
            (Icon::AnacadCarga, "ui.anacad.equip.carga", "Carga"),
            (Icon::AnacadShunt, "ui.anacad.equip.shunt", "Shunt de Barra"),
            (Icon::AnacadShuntLinha, "ui.anacad.equip.shunt_linha", "Shunt de Linha"),
            (Icon::AnacadCompensadorSerie, "ui.anacad.equip.compensador_serie", "Compensador Série"),
        ],
        &[
            (Icon::AnacadTrafoAterramento, "ui.anacad.equip.trafo_aterramento", "Trafo de Aterramento"),
            (Icon::AnacadEquivalente, "ui.anacad.equip.gerador_fonte", "Gerador Fonte de Corrente"),
            (Icon::AnacadMutua, "ui.anacad.equip.impedancia_mutua", "Impedância Mútua"),
            (Icon::AnacadMotor, "ui.anacad.equip.motor", "Motor de Indução"),
            (Icon::AnacadInjecao, "ui.anacad.equip.compensador_estatico", "Compensador Estático"),
        ],
    ),
    (
        "Banco",
        &[(Icon::Plus, "ui.anacad.banco.novo", "Novo Banco"), (Icon::Import, "ui.anacad.banco.abrir", "Abrir Banco")],
        &[(Icon::List, "ui.anacad.documento.novo", "Novo Documento")],
    ),
    (
        "Arquivo",
        &[(Icon::Import, "ui.anacad.importar.lst", "Importar .LST"), (Icon::Attach, "ui.anacad.importar.ana", "Importar .ANA")],
        &[(Icon::Export, "ui.anacad.exportar", "Exportar .LST"), (Icon::List, "ui.anacad.relatorio", "Relatório")],
    ),
    ("Dados", &[(Icon::Properties, "ui.anacad.consultar", "Consultar banco")], &[(Icon::List, "ui.anacad.resultados", "Resultados")]),
    (
        "Cálculos",
        &[
            (Icon::Table, "ui.anacad.topologia", "Topologia"),
            (Icon::Measure, "ui.anacad.comprimentos", "Comprimentos"),
            (Icon::Layers, "ui.anacad.ybus", "Ybus"),
            (Icon::List, "ui.anacad.curto", "Curto-circuito"),
            (Icon::Table, "ui.anacad.niveis.curto", "Níveis de Curto"),
            (Icon::Layers, "ui.anacad.equivalente", "Equivalente de Rede"),
            (Icon::Table, "ui.anacad.zbus", "Matriz Zbarra"),
            (Icon::List, "ui.anacad.curto.simultaneo", "Faltas Simultâneas"),
        ],
        &[],
    ),
    (
        "Edição",
        &[
            (Icon::AnacadBarra, "ui.anacad.criar.barra", "Criar Barra"),
            (Icon::AnacadLinha, "ui.anacad.criar.linha", "Criar Linha"),
            (Icon::AnacadLegenda, "ui.anacad.criar.legenda", "Criar Legenda"),
        ],
        &[
            (Icon::Properties, "ui.anacad.editar.barra", "Editar Barra"),
            (Icon::Properties, "ui.anacad.editar.linha", "Editar Linha"),
            (Icon::Move, "ui.anacad.atribuir.barra", "Atribuir Barra"),
            (Icon::Properties, "ui.anacad.atribuir.linha", "Atribuir Linha"),
            (Icon::Move, "ui.anacad.mover", "Mover"),
            (Icon::Erase, "ui.anacad.apagar", "Apagar"),
        ],
    ),
];

/// The dockable palette with only this module's tools (same format as Tool Sets).
pub fn tools(app: &mut CadApp, ui: &mut egui::Ui) {
    let t = Tokens::get();
    egui::Frame::NONE.fill(t.panel).show(ui, |ui| {
        let mut clicked: Option<&'static str> = None;
        let mut toggle_group = None;
        let mut tool_visible: Option<(String, bool)> = None;
        let mut restore: Vec<String> = Vec::new();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for (name, large, small) in TOOLSET {
                let collapsed = app.ui.collapsed_groups.iter().any(|g| g == name);
                let (hr, hresp) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::click());
                let pp = ui.painter();
                icons::paint(
                    pp,
                    Rect::from_center_size(pos2(hr.left() + 11.0, hr.center().y), vec2(11.0, 11.0)),
                    if collapsed { Icon::ChevronRight } else { Icon::ChevronDown },
                    false,
                );
                pp.text(
                    pos2(hr.left() + 22.0, hr.center().y),
                    egui::Align2::LEFT_CENTER,
                    crate::i18n::t(name),
                    egui::FontId::proportional(12.5),
                    t.text,
                );
                let gear = Rect::from_center_size(pos2(hr.right() - 12.0, hr.center().y), vec2(13.0, 13.0));
                let gresp = ui.interact(gear, ui.id().with(("anacad_gear", name)), Sense::click());
                icons::paint(pp, Rect::from_center_size(gear.center(), vec2(11.0, 11.0)), Icon::Gear, false);
                icons::describe_control(ui, &hresp, name, Some(!collapsed), false);
                icons::describe_control(ui, &gresp, &format!("{name} options"), None, false);
                let gresp = gresp.on_hover_text(crate::tl!("Show and hide the tools of this category"));
                if hresp.clicked() {
                    toggle_group = Some(name.to_string());
                }
                let hidden = |cmd: &str| app.ui.hidden_tools.iter().any(|h| h == cmd);
                egui::Popup::from_toggle_button_response(&gresp).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
                    ui.label(crate::tl!("Show in Tool Sets"));
                    for (_, cmd, tip) in large.iter().chain(small.iter()) {
                        let mut shown = !hidden(cmd);
                        if ui.checkbox(&mut shown, *tip).changed() {
                            tool_visible = Some((cmd.to_string(), shown));
                        }
                    }
                    if ui.selectable_label(false, crate::tl!("Show All")).clicked() {
                        restore = large.iter().chain(small.iter()).map(|(_, cmd, _)| cmd.to_string()).collect();
                    }
                });
                if !collapsed {
                    ui.add_space(4.0);
                    if !large.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                            for (icon, cmd, tip) in large.iter() {
                                if hidden(cmd) {
                                    continue;
                                }
                                ui.add_enabled_ui(command_known(cmd), |ui| {
                                    if icons::button(ui, *icon, 40.0, tip, false).clicked() {
                                        clicked = Some(cmd);
                                    }
                                });
                            }
                        });
                    }
                    let visible: Vec<Tool> = small.iter().copied().filter(|(_, cmd, _)| !hidden(cmd)).collect();
                    for row in visible.chunks(8) {
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                            for (icon, cmd, tip) in row.iter() {
                                ui.add_enabled_ui(command_known(cmd), |ui| {
                                    if icons::button(ui, *icon, 23.0, tip, false).clicked() {
                                        clicked = Some(cmd);
                                    }
                                });
                            }
                        });
                    }
                    ui.add_space(6.0);
                }
                let (sr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                ui.painter().hline(sr.x_range().shrink(6.0), sr.center().y, Stroke::new(1.0, t.border));
            }
        });
        if let Some((cmd, shown)) = tool_visible {
            if shown {
                app.ui.hidden_tools.retain(|h| h != &cmd);
            } else if !app.ui.hidden_tools.contains(&cmd) {
                app.ui.hidden_tools.push(cmd);
            }
        }
        if !restore.is_empty() {
            app.ui.hidden_tools.retain(|h| !restore.contains(h));
        }
        if let Some(g) = toggle_group {
            if let Some(i) = app.ui.collapsed_groups.iter().position(|x| *x == g) {
                app.ui.collapsed_groups.remove(i);
            } else {
                app.ui.collapsed_groups.push(g);
            }
        }
        if let Some(c) = clicked {
            app.start(c);
        }
    });
}

/// Runs this module's `ui.anacad.*` commands; `None` when `id` is not one of them.
pub fn route(app: &mut CadApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    let r = match id {
        "ui.anacad" => crate::anacad::open(app),
        "ui.anacad.toolset" => crate::docking::command(app, false, &json!({"operation":"open","panel":"anacadTools"})),
        "ui.anacad.criar.barra" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de criar a barra");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.start("anacad.barra.criar");
            }
            Ok(Value::Null)
        }
        "ui.anacad.criar.linha" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de criar a linha");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.start("anacad.linha.criar");
            }
            Ok(Value::Null)
        }
        "ui.anacad.criar.legenda" => {
            app.ui.anacad_dialogo = "criar_legenda".into();
            Ok(Value::Null)
        }
        "ui.anacad.impedancia" => {
            app.ui.anacad_dialogo = "impedancia".into();
            Ok(Value::Null)
        }
        "ui.anacad.superacao" => {
            app.ui.anacad_dialogo = "superacao".into();
            Ok(Value::Null)
        }
        "ui.anacad.editar.barra" => {
            if app.session.selection().is_empty() {
                app.set_status("Selecione uma polilinha e abra o editor");
            }
            app.ui.anacad_dialogo = "editar_barra".into();
            Ok(Value::Null)
        }
        "ui.anacad.editar.linha" => {
            if app.session.selection().is_empty() {
                app.set_status("Selecione uma polilinha e abra o editor");
            }
            app.ui.anacad_dialogo = "editar_linha".into();
            Ok(Value::Null)
        }

        "ui.anacad.banco.conectar" => {
            let Some(arquivo) = params.get("arquivo").and_then(Value::as_str) else {
                return Some(Err("informe `arquivo`".to_string()));
            };
            crate::anacad::conectar_banco(app, "anacad.banco.abrir", arquivo).map(|_| Value::Null)
        }
        "ui.anacad.equip.transformador" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "transformador".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.gerador" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "gerador".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.carga" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "carga".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.shunt" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "shunt".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.shunt_linha" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "shunt_linha".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.compensador_serie" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "compensador_serie".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.trafo_aterramento" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "trafo_aterramento".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.gerador_fonte" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "gerador_fonte".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.impedancia_mutua" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "impedancia_mutua".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.motor" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "motor".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.equip.compensador_estatico" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir equipamentos");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "compensador_estatico".into();
                app.start("anacad.equipamento.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.texto.livre" => {
            app.ui.anacad_dialogo = "texto_livre".into();
            Ok(Value::Null)
        }
        "ui.anacad.inserir.terminal" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "terminal".into();
                app.start("anacad.ponto.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.inserir.monitoracao" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "monitoracao".into();
                app.start("anacad.ponto.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.inserir.quebra" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de inserir");
            } else {
                app.session.anacad_db = app.ui.anacad_db.clone();
                app.session.anacad_documento = app.ui.anacad_documento;
                app.session.anacad_equip_tipo = "quebra".into();
                app.start("anacad.ponto.inserir");
            }
            Ok(Value::Null)
        }
        "ui.anacad.legenda.opcoes" => {
            app.ui.anacad_dialogo = "opcoes_legenda".into();
            Ok(Value::Null)
        }
        "ui.anacad.anotar.curto" => {
            if app.ui.anacad_db.trim().is_empty() {
                app.set_status("Conecte um banco antes de anotar");
            } else {
                match app.run("anacad.anotar.curto", json!({"db": app.ui.anacad_db, "documento": app.ui.anacad_documento})) {
                    Ok(v) => {
                        let n = v.get("anotacoes").and_then(Value::as_i64).unwrap_or(0);
                        app.set_status(format!("{n} anotação(ões) de curto criadas"));
                    }
                    Err(e) => app.set_status(format!("Anotar curto: {e}")),
                }
            }
            Ok(Value::Null)
        }
        "ui.anacad.exibir.normal" => {
            let msg = crate::anacad::aplicar_exibicao(app, "normal");
            app.ui.anacad_exibicao = "Desenho Normal".into();
            app.set_status(msg);
            Ok(Value::Null)
        }
        "ui.anacad.exibir.nao_desenhados" => {
            let msg = crate::anacad::aplicar_exibicao(app, "nao_desenhados");
            app.ui.anacad_exibicao = "Elementos Não Desenhados".into();
            app.set_status(msg);
            Ok(Value::Null)
        }
        "ui.anacad.exibir.mutuas" => {
            let msg = crate::anacad::aplicar_exibicao(app, "mutuas");
            app.ui.anacad_exibicao = "Linhas com Mútuas".into();
            app.set_status(msg);
            Ok(Value::Null)
        }
        "ui.anacad.exibir.esquematico" => {
            let msg = crate::anacad::aplicar_exibicao(app, "esquematico");
            app.ui.anacad_exibicao = "Esquemático".into();
            app.set_status(msg);
            Ok(Value::Null)
        }
        "ui.anacad.exibir.unifilar" => {
            let msg = crate::anacad::aplicar_exibicao(app, "unifilar");
            app.ui.anacad_exibicao = "Unifilar".into();
            app.set_status(msg);
            Ok(Value::Null)
        }
        "ui.anacad.detalhar.barra" => {
            app.ui.anacad_dialogo = "detalhar_barra".into();
            Ok(Value::Null)
        }
        "ui.anacad.renomear.terminal" => {
            app.ui.anacad_dialogo = "renomear_terminal".into();
            Ok(Value::Null)
        }
        "ui.anacad.localizar.terminal" => {
            app.ui.anacad_dialogo = "localizar_terminal".into();
            Ok(Value::Null)
        }
        "ui.anacad.renumerar.barra" => {
            app.ui.anacad_dialogo = "renumerar_barra".into();
            Ok(Value::Null)
        }
        "ui.anacad.localizar.barra" => {
            app.ui.anacad_dialogo = "localizar_barra".into();
            Ok(Value::Null)
        }
        "ui.anacad.seccionar.linha" => {
            app.ui.anacad_dialogo = "seccionar_linha".into();
            Ok(Value::Null)
        }
        "ui.anacad.separar.barra" => {
            app.ui.anacad_dialogo = "separar_barra".into();
            Ok(Value::Null)
        }
        "ui.anacad.unir.barras" => {
            app.ui.anacad_dialogo = "unir_barras".into();
            Ok(Value::Null)
        }
        "ui.anacad.banco.novo" => {
            if app.services.pick_db_save.is_some() {
                if let Some(arquivo) = app.services.pick_db_save.as_ref().and_then(|f| f("anacad.db")) {
                    match crate::anacad::conectar_banco(app, "anacad.banco.criar", &arquivo) {
                        Ok(_) => app.set_status("Banco conectado"),
                        Err(e) => app.set_status(format!("Banco: {e}")),
                    }
                }
            } else {
                app.ui.anacad_dialogo = "banco_novo".into();
            }
            Ok(Value::Null)
        }
        "ui.anacad.banco.abrir" => {
            if app.services.pick_db_open.is_some() {
                if let Some(arquivo) = app.services.pick_db_open.as_ref().and_then(|f| f()) {
                    match crate::anacad::conectar_banco(app, "anacad.banco.abrir", &arquivo) {
                        Ok(_) => app.set_status("Banco conectado"),
                        Err(e) => app.set_status(format!("Banco: {e}")),
                    }
                }
            } else {
                app.ui.anacad_dialogo = "banco_abrir".into();
            }
            Ok(Value::Null)
        }
        "ui.anacad.documento.novo" => {
            app.ui.anacad_dialogo = "documento_novo".into();
            Ok(Value::Null)
        }
        "ui.anacad.importar.lst" => {
            app.ui.anacad_dialogo = "importar_lst".into();
            Ok(Value::Null)
        }
        "ui.anacad.importar.ana" => {
            app.ui.anacad_dialogo = "importar_ana".into();
            Ok(Value::Null)
        }
        "ui.anacad.ana.exportar" => {
            app.ui.anacad_dialogo = "exportar_ana".into();
            Ok(Value::Null)
        }
        "ui.anacad.exportar" => {
            app.ui.anacad_dialogo = "exportar".into();
            Ok(Value::Null)
        }
        "ui.anacad.relatorio" => {
            app.ui.anacad_dialogo = "relatorio".into();
            Ok(Value::Null)
        }
        "ui.anacad.consultar" => {
            app.ui.anacad_dialogo = "consultar".into();
            Ok(Value::Null)
        }
        "ui.anacad.resultados" => {
            app.ui.anacad_dialogo = "resultados".into();
            Ok(Value::Null)
        }
        "ui.anacad.topologia" => {
            app.ui.anacad_dialogo = "topologia".into();
            Ok(Value::Null)
        }
        "ui.anacad.comprimentos" => {
            app.ui.anacad_dialogo = "comprimentos".into();
            Ok(Value::Null)
        }
        "ui.anacad.ybus" => {
            app.ui.anacad_dialogo = "ybus".into();
            Ok(Value::Null)
        }
        "ui.anacad.equivalente" => {
            app.ui.anacad_dialogo = "equivalente".into();
            Ok(Value::Null)
        }
        "ui.anacad.curto.simultaneo" => {
            app.ui.anacad_dialogo = "simultaneo".into();
            Ok(Value::Null)
        }
        "ui.anacad.zbus" => {
            app.ui.anacad_dialogo = "zbus".into();
            Ok(Value::Null)
        }
        "ui.anacad.niveis.curto" => {
            app.ui.anacad_dialogo = "niveis_curto".into();
            Ok(Value::Null)
        }
        "ui.anacad.curto" => {
            app.ui.anacad_dialogo = "curto".into();
            Ok(Value::Null)
        }
        "ui.anacad.atribuir.barra" => {
            app.ui.anacad_dialogo = "atribuir_barra".into();
            Ok(Value::Null)
        }
        "ui.anacad.atribuir.linha" => {
            app.ui.anacad_dialogo = "atribuir_linha".into();
            Ok(Value::Null)
        }
        "ui.anacad.mover" => {
            app.ui.anacad_dialogo = "mover".into();
            Ok(Value::Null)
        }
        "ui.anacad.apagar" => {
            app.ui.anacad_dialogo = "apagar".into();
            Ok(Value::Null)
        }
        _ => return None,
    };
    Some(r)
}
