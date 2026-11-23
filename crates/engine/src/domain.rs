//! Extension API for domain modules (AnaCAD and the ones after it).
//!
//! A module brings its own commands and the state a session keeps for it. It registers once, at start-up, with [`registrar`]; the command catalog ([`crate::cmd::command_specs`]), the command line, the control
//! channel and MCP reach those commands through [`commands`]. Neither `cmd/mod.rs` nor [`crate::Session`]
//! needs a `#[cfg]` per module: the module's own code is the only place that knows its domain.
//!
//! The state is a type-erased bag ([`DomainStates`]): a module reads it back with
//! [`DomainStates::with`] instead of adding a field to the session.

use std::any::Any;
use std::collections::BTreeMap;

use crate::cmd::CommandSpec;

/// A domain module: the commands it adds and the state a session keeps for it.
pub trait Domain: Send + Sync {
    /// Stable key of the module; [`DomainStates::with`] uses it to find the state.
    fn id(&self) -> &'static str;
    /// Display name for `PLUGIN LISTAR` (defaults to [`Domain::id`]).
    fn nome(&self) -> &'static str {
        self.id()
    }
    /// Short names accepted by the `/alias` form and by `PLUGIN ATIVAR` (e.g.
    /// `arq` for `arquitetura`). The canonical id is always accepted too.
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }
    /// The commands this module adds to the catalog.
    fn commands(&self) -> Vec<CommandSpec>;
    /// A fresh state for a new session.
    fn state(&self) -> Box<dyn Any + Send>;

    /// Runs after every successful command, with the drawing the command started from and the
    /// document it ran in: the moment a module reacts to what the user just changed.
    ///
    /// The anacad uses it to take the lines of a bar that was erased along with it — the same place
    /// the Eletron's `onObjectEraseded` runs its cascade. `uid` is the document the command started
    /// in (`None` when unknown): a pass must not read a document switch as "everything was
    /// erased".
    fn after_command(&self, _s: &mut crate::Session, _before: Option<&std::sync::Arc<cadcraft_doc::Drawing>>, _uid: Option<u64>) {}
}

/// The plugin registry: every domain module the host app registered at start-up.
///
/// A plugin is registered once, before the first session, by the app that embeds it
/// ([`registrar`]); the engine itself never names a plugin, so a build without any
/// plugin compiles and runs the bare CAD. Registry order is registration order.
static REGISTRO: std::sync::OnceLock<std::sync::RwLock<Vec<&'static dyn Domain>>> = std::sync::OnceLock::new();

fn registro() -> &'static std::sync::RwLock<Vec<&'static dyn Domain>> {
    REGISTRO.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

/// Register a domain module for this process. Idempotent by [`Domain::id`]:
/// registering the same id twice keeps the first entry and returns `false`.
pub fn registrar(domain: &'static dyn Domain) -> bool {
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

/// The registered module whose id or [`Domain::aliases`] matches `id`
/// (case-insensitive), if any.
pub fn por_alias(id: &str) -> Option<&'static dyn Domain> {
    registrados().into_iter().find(|d| d.id().eq_ignore_ascii_case(id) || d.aliases().iter().any(|a| a.eq_ignore_ascii_case(id)))
}

/// Every registered domain module, in registration order.
pub fn registrados() -> Vec<&'static dyn Domain> {
    match registro().read() {
        Ok(list) => list.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Backwards-compatible alias: the registered modules, in registration order.
pub fn builtin() -> Vec<&'static dyn Domain> {
    registrados()
}

/// The plugin that owns the command `id`, derived from the registered modules'
/// own [`Domain::commands`] (ids vary per plugin — `anacad.*`, `arq.*`, … — so
/// no prefix is ever guessed). `None` for the shell's own commands.
pub fn dono_do_comando(id: &str) -> Option<&'static str> {
    registrados().iter().find(|domain| domain.commands().iter().any(|c| c.id == id)).map(|domain| domain.id())
}

/// Which registered plugins the drawing carries, by the XData app names it stores
/// (`id` uppercased, e.g. `ANACAD_CFG`): the fallback activation probe when the
/// drawing has no `CADCRAFT_PLUGINS` header variable yet.
pub fn plugins_do_desenho(doc: &cadcraft_doc::Drawing) -> Vec<&'static str> {
    let prefixos: Vec<(&'static str, String)> = registrados().iter().map(|domain| (domain.id(), domain.id().to_ascii_uppercase())).collect();
    if prefixos.is_empty() {
        return Vec::new();
    }
    let mut achados: Vec<&'static str> = Vec::new();
    let mut visitar = |entities: &mut dyn Iterator<Item = &std::sync::Arc<cadcraft_doc::Entity>>| {
        for entity in entities {
            for tag in &entity.common.xdata {
                if tag.code == 1001 {
                    for (id, prefixo) in &prefixos {
                        if tag.value.to_ascii_uppercase().starts_with(prefixo) && !achados.contains(id) {
                            achados.push(*id);
                        }
                    }
                }
            }
        }
    };
    visitar(&mut doc.model.iter());
    for layout in &doc.layouts {
        visitar(&mut layout.entities.iter());
    }
    for bloco in doc.blocks.values() {
        visitar(&mut bloco.entities.iter());
    }
    achados
}

/// Every command added by a domain module.
pub fn commands() -> Vec<CommandSpec> {
    let mut out = Vec::new();
    for domain in registrados() {
        out.extend(domain.commands());
    }
    out
}

/// The per-session state of the domain modules, keyed by [`Domain::id`].
#[derive(Default)]
pub struct DomainStates {
    states: BTreeMap<&'static str, Box<dyn Any + Send>>,
}

impl DomainStates {
    /// A bag seeded with each module's fresh state.
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

    /// Run `f` with the state of type `T` kept under `id`.
    ///
    /// A missing slot, or one holding another type, is replaced by `T::default()` first, so a
    /// module can use its state without the session having to know about it.
    pub fn with<T: Default + Send + 'static, R>(&mut self, id: &'static str, f: impl FnOnce(&mut T) -> R) -> R {
        if !self.states.get(id).is_some_and(|state| state.is::<T>()) {
            self.states.insert(id, Box::new(T::default()));
        }
        match self.state_mut::<T>(id) {
            Some(state) => f(state),
            // Only reachable if the slot were swapped underneath; a scratch value keeps the
            // contract (never panic) without touching the bag.
            None => f(&mut T::default()),
        }
    }

    fn state_mut<T: Send + 'static>(&mut self, id: &'static str) -> Option<&mut T> {
        self.states.get_mut(id).and_then(|state| state.downcast_mut::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn noop(_: &mut crate::Session, _: &Value) -> crate::Result<Value> {
        Ok(Value::Null)
    }

    struct Doubled;
    impl Domain for Doubled {
        fn id(&self) -> &'static str {
            "test.doubled"
        }
        fn commands(&self) -> Vec<CommandSpec> {
            vec![CommandSpec::new("anacad.test.noop", "test", noop)]
        }
        fn state(&self) -> Box<dyn Any + Send> {
            Box::new(3u32)
        }
    }

    /// A module's state survives round trips and a wrong-type slot is replaced, never panics.
    #[test]
    fn domain_state_is_type_checked() {
        let mut states = DomainStates::default();
        let id = "test.doubled";

        assert_eq!(states.get::<u32>(id), None);
        assert_eq!(states.with::<u32, _>(id, |n| *n), 0);

        states.with::<u32, _>(id, |n| *n = 7);
        assert_eq!(states.get::<u32>(id), Some(&7));

        // Same slot, another type: replaced by that type's default.
        assert_eq!(states.with::<String, _>(id, |s| s.clone()), String::new());
        assert_eq!(states.with::<u32, _>(id, |n| *n), 0);

        // The state a module declares is seeded as it is.
        assert_eq!(Doubled.state().downcast_ref::<u32>(), Some(&3));
    }
}
