use super::*;
use crate::Session;
use serde_json::json;

/// Sessão com o domínio de teste registrado e ativo, como o `apps/cadcraft`
/// faz em produção com os plugins reais (registrar no arranque). O registro
/// em si acontece em [`registrar_teste`], que a sessão já chama no teste
/// (em [`Session::empty`]); aqui só garantimos que ele correu.
fn ajuda() -> Session {
    super::registrar_teste();
    // `Session::new()` ativa tudo o que está registrado em teste.
    Session::new()
}

fn listar(s: &mut Session) -> Vec<(String, bool)> {
    let v = s.execute("plugin", &json!({"acao": "listar"})).unwrap();
    v["plugins"].as_array().unwrap().iter().map(|p| (p["id"].as_str().unwrap().to_string(), p["ativo"].as_bool().unwrap())).collect()
}

#[test]
fn listar_ativar_desativar_e_recusa() {
    let mut s = ajuda();
    let plugins = listar(&mut s);
    assert!(!plugins.is_empty());
    assert!(plugins.iter().all(|(_, ativo)| *ativo));

    s.execute("plugin", &json!({"acao": "desativar", "id": "foo"})).unwrap();
    assert!(!s.plugin_ativo("foo"));
    let err = s.execute("foo.consultar", &json!({})).unwrap_err().to_string();
    assert!(err.contains("não está ativo"), "unexpected error: {err}");
    // The interactive entry point refuses too.
    assert!(s.start("foo.consultar").is_err());

    s.execute("plugin", &json!({"acao": "ativar", "id": "foo"})).unwrap();
    assert!(s.plugin_ativo("foo"));
    assert!(s.execute("plugin", &json!({"acao": "ativar", "id": "nope"})).is_err());
    assert!(s.execute("plugin", &json!({"acao": "x"})).is_err());
}

#[test]
fn persiste_no_desenho_e_sonda_xdata() {
    // The header variable carries the active set to the next session.
    let mut s = ajuda();
    s.execute("plugin", &json!({"acao": "desativar", "id": "foo"})).unwrap();
    let doc = s.doc().unwrap().clone();
    let mut s2 = Session::empty();
    s2.open_drawing(doc.clone(), "t", None);
    assert!(!s2.plugin_ativo("foo"));
    s2.execute("plugin", &json!({"acao": "ativar", "id": "foo"})).unwrap();
    assert!(s2.plugin_ativo("foo"));

    // Without the header, a drawing carrying the plugin's XData activates it.
    let s3 = Session::new();
    let mut doc = s3.doc().unwrap().clone();
    {
        let handle = cadcraft_doc::Handle(0xC0FFEE);
        let e = cadcraft_doc::Entity::new(handle, cadcraft_doc::EntityKind::Point(cadcraft_doc::Point { p: cadcraft_geom::Vec3::ZERO, angle: 0.0 }));
        doc.model.push(e);
        doc.model.modify(handle, |e| {
            e.common.xdata.push(cadcraft_doc::RawTag { code: 1001, value: "FOO_CFG".to_string() });
        });
    }
    let mut s4 = Session::empty();
    s4.open_drawing(doc.clone(), "t", None);
    assert!(s4.plugin_ativo("foo"));

    // Sem o cabecalho e sem o XData do plugin, o desenho nao o liga.
    let mut s5 = Session::empty();
    s5.open_drawing(cadcraft_doc::Drawing::default(), "t", None);
    assert!(!s5.plugin_ativo("foo"));
}

#[test]
fn alias_curto_resolve_para_o_comando_do_plugin() {
    let mut s = ajuda();
    assert_eq!(crate::cmd::resolve_alias_ativa(&s, "fc"), "foo.consultar");
    assert_eq!(crate::cmd::resolve_alias_ativa(&s, "FOO.CONSULTAR"), "foo.consultar");

    // Alias de um plugin inativo nao vira comando de outro (nem resolve).
    s.execute("plugin", &json!({"acao": "desativar", "id": "foo"})).unwrap();
    assert_eq!(crate::cmd::resolve_alias_ativa(&s, "fc"), "fc");
}

#[test]
fn alias_ativa_o_plugin_canonico() {
    let mut s = ajuda();
    s.execute("plugin", &json!({"acao": "desativar", "id": "foo"})).unwrap();

    // /f resolves to the canonical id and persists it, not the alias.
    let v = s.execute("/f", &json!({})).unwrap();
    assert_eq!(v["id"], "foo");
    assert_eq!(v["ativo"].as_bool(), Some(true));
    assert!(s.plugin_ativo("foo"));
    assert_eq!(s.doc().unwrap().header.get("CADCRAFT_PLUGINS").and_then(|v| v.as_str()), Some("foo"));

    s.execute("plugin", &json!({"acao": "desativar", "id": "foo"})).unwrap();
    s.execute("plugin", &json!({"acao": "ativar", "id": "f"})).unwrap();
    assert!(s.plugin_ativo("foo"));
}

/// /id is the plugin own call: the short form of PLUGIN ATIVAR id.
#[test]
fn barra_ativa_lista_e_recusa() {
    let mut s = ajuda();
    s.execute("plugin", &json!({"acao": "desativar", "id": "foo"})).unwrap();

    // /foo activates it and the drawing carries the plugin from then on.
    let v = s.execute("/foo", &json!({})).unwrap();
    assert_eq!(v["id"], "foo");
    assert_eq!(v["ativo"].as_bool(), Some(true));
    assert!(s.plugin_ativo("foo"));
    assert_eq!(s.doc().unwrap().header.get("CADCRAFT_PLUGINS").and_then(|v| v.as_str()), Some("foo"));

    // The command line reaches the same path, as typed (and so do scripts, the
    // control channel and MCP, which all go through start/execute).
    s.execute("plugin", &json!({"acao": "desativar", "id": "foo"})).unwrap();
    s.cmdline("/foo").unwrap();
    assert!(s.plugin_ativo("foo"));

    // "/" alone lists the registered plugins, like PLUGIN LISTAR.
    let v = s.execute("/", &json!({})).unwrap();
    assert!(v["plugins"].as_array().is_some_and(|list| list.iter().any(|p| p["id"] == "foo")));

    // An unknown id is refused with the PLUGIN hint, never silently ignored.
    let err = s.start("/nope").unwrap_err().to_string();
    assert!(err.contains("desconhecido"), "unexpected error: {err}");
}

#[test]
fn interativo_desativa_e_lista() {
    let mut s = ajuda();
    s.start("plugin").unwrap();
    // Each interactive answer leaves the machine prompting for the next one
    // until the plugin id is given.
    s.input(Input::Text("desativar".into())).unwrap();
    assert!(s.current_prompt().is_some());
    s.input(Input::Text("foo".into())).unwrap();
    assert!(!s.plugin_ativo("foo"));
}
