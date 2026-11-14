//! Extension API for domain modules (AnaCAD and the ones after it).
//!
//! A module brings its own commands and the state a session keeps for it. It registers once in
//! [`builtin`]; the command catalog ([`crate::cmd::command_specs`]), the command line, the control
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
    /// The commands this module adds to the catalog.
    fn commands(&self) -> Vec<CommandSpec>;
    /// A fresh state for a new session.
    fn state(&self) -> Box<dyn Any + Send>;
}

#[cfg(feature = "anacad")]
static BUILTIN: &[&dyn Domain] = &[&crate::cmd::anacad::ANACAD];
#[cfg(not(feature = "anacad"))]
static BUILTIN: &[&dyn Domain] = &[];

/// The domains compiled into this build, in registration order.
pub fn builtin() -> &'static [&'static dyn Domain] {
    BUILTIN
}

/// Every command added by a domain module.
pub fn commands() -> Vec<CommandSpec> {
    let mut out = Vec::new();
    for domain in builtin() {
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
        for domain in builtin() {
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
