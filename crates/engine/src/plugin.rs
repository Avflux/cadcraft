//! The public extension contract: what a plugin (a domain module living outside
//! this crate) may use of the engine.
//!
//! Everything a plugin needs is re-exported here, so a plugin crate depends only
//! on `cadcraft_engine::plugin` (plus `cadcraft_doc`/`cadcraft_geom` for the
//! drawing types) and never on the engine's internals (`pub(crate)` items).
//! Changing this module is changing the plugins' API.
//!
//! Registration is separate: the embedding app registers each plugin once at
//! start-up with [`registrar`], before the first session.

/// Insercao de bloco no desenho: `blocks::insert`/`blocks::insert_with`.
pub mod blocks {
    pub use crate::cmd::blocks::{insert, insert_with};
}
pub use crate::cmd::blocks::{insert, insert_with};
pub use crate::cmd::machines::{SelOutcome, SelectPhase};
pub use crate::cmd::{
    CommandSpec, always, bad, bool_or, f64_or, f64_req, has_doc, has_selection, point_param, point_req, points_param, size_param, str_param,
};
pub use crate::domain::{Domain, DomainStates, registrados, registrar};
pub use crate::prompt::{Accept, Input, Interactive, Prompt, Step};
pub use crate::{EngineError, Result, Session};

use cadcraft_doc::{Entity, Handle, RawTag};

/// Read a field from an entity's XData: the value that follows a `1000` tag
/// whose text equals `nome`, under any application name.
pub fn campo_xdata<'a>(e: &'a Entity, nome: &str) -> Option<&'a str> {
    let tags = &e.common.xdata;
    tags.windows(2).find_map(|pair| match (&pair[0].code, &pair[0].value, &pair[1].code, &pair[1].value) {
        (1000, chave, 1000, valor) if chave == nome => Some(valor.as_str()),
        _ => None,
    })
}

/// Write XData on an entity: an application tag (`1001`) followed by paired
/// `1000` key/value tags, replacing whatever XData the entity had.
pub fn marcar(s: &mut Session, handle: Handle, app: &str, campos: &[(&str, &str)]) -> Result<()> {
    let mut tags = vec![RawTag { code: 1001, value: app.to_string() }];
    for (chave, valor) in campos {
        tags.push(RawTag { code: 1000, value: (*chave).to_string() });
        tags.push(RawTag { code: 1000, value: (*valor).to_string() });
    }
    s.doc_mut()?.modify_entity(handle, |e| {
        e.common.xdata = tags;
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_doc::EntityKind;
    use serde_json::json;

    #[test]
    fn campo_xdata_reads_paired_tags() {
        let mut s = Session::new();
        let h = s.add_entity(EntityKind::Point(cadcraft_doc::Point { p: cadcraft_geom::Vec3::ZERO, angle: 0.0 })).ok();
        let h = match h {
            Some(h) => h,
            None => return,
        };
        marcar(&mut s, h, "TESTE", &[("id", "42"), ("nome", "barra")]).ok();
        let e = s.doc().ok().and_then(|d| d.entity(h));
        let e = match e {
            Some(e) => e,
            None => return,
        };
        assert_eq!(campo_xdata(e, "id"), Some("42"));
        assert_eq!(campo_xdata(e, "nome"), Some("barra"));
        assert_eq!(campo_xdata(e, "ausente"), None);
        // Param helpers stay total on hostile input.
        let p = json!({"n": "x", "b": true});
        assert_eq!(f64_or(&p, "n", 7.0), 7.0);
        assert!(bool_or(&p, "b", false));
        assert!(matches!(bad("cmd", "msg"), EngineError::BadParams { .. }));
    }
}
