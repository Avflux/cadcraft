//! Extension API for domain modules in the shell: what a module adds to the window.
//!
//! A module hands the shell its UI commands, its Tool Sets tab and tool set, its Tool Bar row, the
//! content of its docked panels and the router for its `ui.*` commands. The shell keeps one
//! registration list ([`registrados`]) and no `#[cfg]` in `menus.rs`, `palettes.rs`, `chrome.rs` or
//! `docking.rs`.
//!
//! Panel *identity* still lives in [`crate::docking::Panel`] because the saved layout stores it by
//! name; only the panel's content and its title come from the module.

use std::any::Any;
use std::collections::BTreeMap;

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
    /// A fresh screen state for a new `CadApp`: the module's own bag slot,
    /// seeded by the shell's [`UiStates::new`]. A module that keeps no screen
    /// state returns the unit state.
    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(())
    }
    /// UI-only commands, shown in menus and reachable as `ui.*`.
    fn ui_commands(&self) -> &'static [UiCommand] {
        &[]
    }
    /// Short names that open a UI command (the original command names, such as
    /// `DISP` opening `ui.ele.dispositivos`): `(alias, ui command id)`.
    fn ui_aliases(&self) -> &'static [(&'static str, &'static str)] {
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
    /// The module's rows on the Tool Bar, each drawn as its own group. A
    /// module with many tools splits them into groups (one per ribbon panel,
    /// for instance) so that a group that does not fit goes to the "…" menu on
    /// its own instead of hiding every other tool. Empty means [`Self::toolbar`].
    fn toolbars(&self) -> &'static [ToolbarRow] {
        &[]
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
    /// The docking panels this module owns: while the plugin is inactive they
    /// stay closed (the shell hides them every frame).
    fn panels(&self) -> &'static [Panel] {
        &[]
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

/// The plugin registry: every UI module the host app registered at start-up.
///
/// Mirrors the engine's registry ([`cadcraft_engine::domain::registrar`]): the app
/// registers each plugin's UI module once, before the first session, and the shell
/// never names a plugin itself.
static REGISTRO: std::sync::OnceLock<std::sync::RwLock<Vec<&'static dyn DomainUi>>> = std::sync::OnceLock::new();

fn registro() -> &'static std::sync::RwLock<Vec<&'static dyn DomainUi>> {
    REGISTRO.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

/// Register a UI domain module for this process. Idempotent by [`DomainUi::id`].
pub fn registrar(domain: &'static dyn DomainUi) -> bool {
    let mut list = match registro().write() {
        Ok(list) => list,
        Err(poisoned) => poisoned.into_inner(),
    };
    if list.iter().any(|d| d.id() == domain.id()) {
        return false;
    }
    list.push(domain);
    true
}

/// Every registered UI module, in registration order.
pub fn registrados() -> Vec<&'static dyn DomainUi> {
    match registro().read() {
        Ok(list) => list.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Backwards-compatible alias: the registered UI modules, in registration order.
pub fn builtin() -> Vec<&'static dyn DomainUi> {
    registrados()
}

/// The registered modules whose plugin is active in this session: only they
/// expose commands, tabs, toolbars, panels and dialogs.
fn ativos(app: &CadApp) -> Vec<&'static dyn DomainUi> {
    registrados().into_iter().filter(|domain| app.session.plugin_ativo(domain.id())).collect()
}

/// Every UI-only command an active module adds.
pub fn ui_commands(app: &CadApp) -> Vec<UiCommand> {
    ativos(app).iter().flat_map(|domain| domain.ui_commands().iter().copied()).collect()
}

/// Tool Sets tabs added by the active modules.
pub fn tabs(app: &CadApp) -> Vec<&'static str> {
    ativos(app).iter().flat_map(|domain| domain.tabs().iter().copied()).collect()
}

/// The tool set of the active module that owns `tab`.
pub fn toolset(app: &CadApp, tab: &str) -> Option<&'static [ToolGroup]> {
    ativos(app).iter().find_map(|domain| domain.toolset(tab))
}

/// The module that owns the Tool Sets right now and the tab it opens: the last
/// module that became active and adds tabs. `None` when no plugin is active.
pub fn aba_do_plugin_ativo(app: &CadApp) -> Option<(&'static str, &'static str)> {
    ativos(app).iter().rev().find_map(|domain| domain.tabs().first().map(|tab| (domain.id(), *tab)))
}

/// Whether `tab` is one of the tabs some registered module adds (a plugin tab,
/// as opposed to the shell's own "Drafting" and "Modeling").
pub fn aba_de_plugin(tab: &str) -> bool {
    registrados().iter().any(|domain| domain.tabs().contains(&tab))
}

/// Tool Bar rows added by the active modules; each row is drawn as a group.
pub fn toolbar_rows(app: &CadApp) -> Vec<ToolbarRow> {
    ativos(app)
        .iter()
        .flat_map(|domain| {
            let rows = domain.toolbars();
            if rows.is_empty() { domain.toolbar().into_iter().collect::<Vec<_>>() } else { rows.to_vec() }
        })
        .collect()
}

/// Whether a button or menu item for `id` may run: the command must exist and
/// every module that knows it must allow it (the shell's own commands have no
/// such owner and stay on).
pub fn command_enabled(app: &CadApp, id: &str) -> bool {
    crate::palettes::command_known(id) && ativos(app).iter().all(|domain| domain.command_enabled(app, id))
}

/// The reason shown on a greyed button, when a module gives one.
pub fn command_disabled_reason(app: &CadApp, id: &str) -> Option<&'static str> {
    ativos(app).iter().find_map(|domain| domain.command_disabled_reason(app, id))
}

/// Let the module that owns `panel` draw it. An unowned panel stays empty.
pub fn panel(app: &mut CadApp, ui: &mut egui::Ui, panel: Panel) {
    for domain in ativos(app) {
        if domain.panel(panel, app, ui) {
            return;
        }
    }
}

/// Let each module show its dialogs for this frame.
pub fn dialogs(app: &mut CadApp, ctx: &egui::Context) {
    for domain in ativos(app) {
        domain.dialogs(app, ctx);
    }
}

/// Let each module refresh from the active drawing before the frame is drawn.
pub fn begin_frame(app: &mut CadApp, ctx: &egui::Context) {
    for domain in ativos(app) {
        domain.begin_frame(app, ctx);
    }
}

/// Run a module's `ui.*` command; `None` when no active module claims the id.
pub fn route(app: &mut CadApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    let id = ativos(app)
        .iter()
        .flat_map(|domain| domain.ui_aliases())
        .find(|(alias, _)| alias.eq_ignore_ascii_case(id))
        .map_or(id, |(_, destino)| *destino);
    ativos(app).iter().find_map(|domain| domain.route(app, id, params))
}

/// The per-app screen state of the UI modules, keyed by [`DomainUi::id`].
///
/// Mirrors the engine's [`cadcraft_engine::domain::DomainStates`]: the shell
/// owns the bag, each module owns its slot's type, and a missing or
/// wrong-typed slot is replaced by that type's default instead of panicking.
#[derive(Default)]
pub struct UiStates {
    states: BTreeMap<&'static str, Box<dyn Any + Send>>,
}

impl UiStates {
    /// A bag seeded with each registered module's fresh state.
    pub fn new() -> Self {
        let mut states: BTreeMap<&'static str, Box<dyn Any + Send>> = BTreeMap::new();
        for domain in registrados() {
            states.insert(domain.id(), domain.state());
        }
        Self { states }
    }

    /// The state kept under `id`, when the slot holds a `T`.
    pub fn get<T: 'static>(&self, id: &'static str) -> Option<&T> {
        self.states.get(id).and_then(|state| state.downcast_ref::<T>())
    }

    /// The state of type `T` kept under `id`, replacing a missing or
    /// wrong-typed slot with `T::default()` first.
    pub fn state_mut<T: Default + Send + 'static>(&mut self, id: &'static str) -> &mut T {
        if !self.states.get(id).is_some_and(|state| state.is::<T>()) {
            self.states.insert(id, Box::new(T::default()));
        }
        if let Some(state) = self.states.get_mut(id)
            && let Some(t) = state.downcast_mut::<T>()
        {
            return t;
        }
        // Only reachable if the slot were swapped underneath; a leaked default
        // keeps the contract (never panic) without touching the bag again.
        Box::leak(Box::new(T::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Contador;
    impl DomainUi for Contador {
        fn id(&self) -> &'static str {
            "test.contador"
        }
        fn state(&self) -> Box<dyn Any + Send> {
            Box::new(3u32)
        }
    }

    #[test]
    fn ui_state_is_type_checked() {
        let mut states = UiStates::default();
        assert_eq!(states.get::<u32>("test.contador"), None);
        assert_eq!(*states.state_mut::<u32>("test.contador"), 0);
        *states.state_mut::<u32>("test.contador") = 7;
        assert_eq!(states.get::<u32>("test.contador"), Some(&7));
        // Same slot, another type: replaced by that type's default.
        assert_eq!(states.state_mut::<String>("test.contador").clone(), String::new());
        assert_eq!(*states.state_mut::<u32>("test.contador"), 0);
        assert_eq!(Contador.state().downcast_ref::<u32>(), Some(&3));
    }
}
