//! Extension API for domain modules in the shell: what a module adds to the window.
//!
//! A module hands the shell its UI commands, its Tool Sets tab and tool set, its Tool Bar row, the
//! content of its docked panels and the router for its `ui.*` commands. The shell keeps one
//! registration list ([`builtin`]) and no `#[cfg]` in `menus.rs`, `palettes.rs`, `chrome.rs` or
//! `docking.rs`.
//!
//! Panel *identity* still lives in [`crate::docking::Panel`] because the saved layout stores it by
//! name; only the panel's content and its title come from the module.

use serde_json::Value;

use crate::CadApp;
use crate::docking::Panel;
use crate::icons::Icon;
use crate::palettes::Tool;

/// A UI-only command: `(id, label, menu path, shortcut)`.
pub type UiCommand = (&'static str, &'static str, &'static [&'static str], Option<&'static str>);
/// A Tool Bar row: `(icon, command, label, shortcut)` pairs.
pub type ToolbarRow = &'static [(Icon, &'static str, &'static str, Option<&'static str>)];
/// A Tool Sets group: `(title, large tools, small tools)`.
pub type ToolGroup = (&'static str, &'static [Tool], &'static [Tool]);

/// What a domain module adds to the shell.
pub trait DomainUi: Send + Sync {
    /// Stable key of the module (matches its counterpart in the engine).
    fn id(&self) -> &'static str;
    /// UI-only commands, shown in menus and reachable as `ui.*`.
    fn ui_commands(&self) -> &'static [UiCommand] {
        &[]
    }
    /// Tabs this module adds to the Tool Sets palette.
    fn tabs(&self) -> &'static [&'static str] {
        &[]
    }
    /// The groups shown when `tab` is the active Tool Sets tab.
    fn toolset(&self, tab: &str) -> Option<&'static [ToolGroup]> {
        let _ = tab;
        None
    }
    /// The module's row on the Tool Bar (placed before Help).
    fn toolbar(&self) -> Option<ToolbarRow> {
        None
    }
    /// Called once per frame, before the shell draws anything: the module reads
    /// the active drawing and refreshes what its buttons depend on (the anacad
    /// takes the project database from the drawing's XData here, so switching
    /// drawings greys the tools out in the same frame).
    fn begin_frame(&self, app: &mut CadApp, ctx: &egui::Context) {
        let _ = (app, ctx);
    }
    /// Whether the module lets `id` run right now. The shell greys out the
    /// buttons and menu items it draws; a module whose project is not set up
    /// yet (anacad without a connected `.db`) keeps everything but its setup
    /// commands off.
    fn command_enabled(&self, app: &CadApp, id: &str) -> bool {
        let _ = (app, id);
        true
    }
    /// Why one of the module's commands is off, shown on the greyed button.
    fn command_disabled_reason(&self, app: &CadApp, id: &str) -> Option<&'static str> {
        let _ = (app, id);
        None
    }
    /// Draw `panel`; `false` when the panel belongs to someone else.
    fn panel(&self, panel: Panel, app: &mut CadApp, ui: &mut egui::Ui) -> bool {
        let _ = (panel, app, ui);
        false
    }
    /// Show the module's dialogs for this frame.
    fn dialogs(&self, app: &mut CadApp, ctx: &egui::Context) {
        let _ = (app, ctx);
    }
    /// Run one of the module's `ui.*` commands; `None` when the id is not its own.
    fn route(&self, app: &mut CadApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
        let _ = (app, id, params);
        None
    }
}

/// The modules compiled into this build, in registration order.
///
/// Built once, on first use: each module enters through its own feature's `#[cfg]`, so N
/// modules do not need a `static` per combination of features.
pub fn builtin() -> &'static [&'static dyn DomainUi] {
    static BUILTIN: std::sync::OnceLock<Vec<&'static dyn DomainUi>> = std::sync::OnceLock::new();
    BUILTIN.get_or_init(|| {
        let modules: Vec<Option<&'static dyn DomainUi>> = vec![
            #[cfg(feature = "anacad")]
            Some(&crate::anacad::ANACAD_UI),
            #[cfg(feature = "arquitetura")]
            Some(&crate::arquitetura::ARQUITETURA_UI),
            #[cfg(feature = "eletrico")]
            Some(&crate::eletrico::ELETRICO_UI),
            #[cfg(feature = "logico")]
            Some(&crate::logico::LOGICO_UI),
        ];
        modules.into_iter().flatten().collect()
    })
}

/// Every UI-only command a module adds.
pub fn ui_commands() -> impl Iterator<Item = UiCommand> {
    builtin().iter().flat_map(|domain| domain.ui_commands().iter().copied())
}

/// Tool Sets tabs added by the modules.
pub fn tabs() -> impl Iterator<Item = &'static str> {
    builtin().iter().flat_map(|domain| domain.tabs().iter().copied())
}

/// The tool set of the module that owns `tab`.
pub fn toolset(tab: &str) -> Option<&'static [ToolGroup]> {
    builtin().iter().find_map(|domain| domain.toolset(tab))
}

/// Tool Bar rows added by the modules.
pub fn toolbar_rows() -> impl Iterator<Item = ToolbarRow> {
    builtin().iter().filter_map(|domain| domain.toolbar())
}

/// Whether a button or menu item for `id` may run: the command must exist and
/// every module that knows it must allow it (the shell's own commands have no
/// such owner and stay on).
pub fn command_enabled(app: &CadApp, id: &str) -> bool {
    crate::palettes::command_known(id) && builtin().iter().all(|domain| domain.command_enabled(app, id))
}

/// The reason shown on a greyed button, when a module gives one.
pub fn command_disabled_reason(app: &CadApp, id: &str) -> Option<&'static str> {
    builtin().iter().find_map(|domain| domain.command_disabled_reason(app, id))
}

/// Let the module that owns `panel` draw it. An unowned panel stays empty.
pub fn panel(app: &mut CadApp, ui: &mut egui::Ui, panel: Panel) {
    for domain in builtin() {
        if domain.panel(panel, app, ui) {
            return;
        }
    }
}

/// Let each module show its dialogs for this frame.
pub fn dialogs(app: &mut CadApp, ctx: &egui::Context) {
    for domain in builtin() {
        domain.dialogs(app, ctx);
    }
}

/// Let each module refresh from the active drawing before the frame is drawn.
pub fn begin_frame(app: &mut CadApp, ctx: &egui::Context) {
    for domain in builtin() {
        domain.begin_frame(app, ctx);
    }
}

/// Run a module's `ui.*` command; `None` when no module claims the id.
pub fn route(app: &mut CadApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    builtin().iter().find_map(|domain| domain.route(app, id, params))
}
