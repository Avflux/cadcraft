//! `PLUGIN` — manage the session's active plugins.
//!
//! The casca never names a plugin: [`crate::domain::registrados`] provides the
//! list, the active set lives on [`crate::Session::plugins_ativos`] and is
//! persisted on the current drawing's `CADCRAFT_PLUGINS` header variable.
//! JSON: `{"acao": "listar"}` → `{"plugins": [{"id", "nome", "ativo"}]}`;
//! `{"acao": "ativar"|"desativar", "id": "anacad"}` → `{"id", "ativo"}`.
//!
//! The short form is the plugin's own name with a leading slash: `/anacad`
//! activates it and `/` alone lists them ([`barra`]), from the command line,
//! a script, the control channel or MCP (`Session::start`/`Session::execute`).

use serde_json::{Value, json};

use super::{CommandSpec, Result, Session, bad, str_param};
use crate::domain;
use crate::prompt::{Accept, Input, Interactive, Prompt, Step};

/// Where the interactive machine is.
#[derive(Default, PartialEq)]
enum Etapa {
    #[default]
    Opcao,
    Nome {
        acao: String,
        rotulo: &'static str,
    },
}

/// Interactive `PLUGIN`: pick an action, then the plugin id.
struct PluginM {
    etapa: Etapa,
}

/// The registered plugins, one per line, with each one's state: what `PLUGIN` and `/` show.
pub(crate) fn lista(s: &Session) -> String {
    let linhas: Vec<String> = domain::registrados()
        .iter()
        .map(|d| {
            let estado = if s.plugin_ativo(d.id()) { "ativo" } else { "inativo" };
            format!("  {} ({}) — {}", d.id(), d.nome(), estado)
        })
        .collect();
    linhas.join("\n")
}

impl Interactive for PluginM {
    fn name(&self) -> &'static str {
        "plugin"
    }
    fn prompt(&self, s: &Session) -> Prompt {
        match &self.etapa {
            Etapa::Opcao => {
                Prompt::new(format!("Plugins:\n{}\nAtivar, Desativar ou Listar", lista(s)), Accept::TEXT).kw(&["Ativar", "Desativar", "Listar"])
            }
            Etapa::Nome { rotulo, .. } => Prompt::new(format!("{rotulo} plugin (nome)"), Accept::TEXT),
        }
    }

    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let text = match i {
            Input::Cancel => return Ok(Step::Cancel),
            Input::Enter => String::new(),
            Input::Text(t) | Input::Keyword(t) => t.trim().to_string(),
            _ => return Ok(Step::Continue),
        };
        match std::mem::take(&mut self.etapa) {
            Etapa::Opcao => match text.to_ascii_lowercase().as_str() {
                "listar" => {
                    s.echo(format!("Plugins:\n{}", lista(s)));
                    Ok(Step::Done)
                }
                "ativar" | "desativar" => {
                    let rotulo = if text.eq_ignore_ascii_case("ativar") { "Ativar" } else { "Desativar" };
                    self.etapa = Etapa::Nome { acao: text.to_ascii_lowercase(), rotulo };
                    Ok(Step::Continue)
                }
                "" => Ok(Step::Done),
                _ => {
                    s.echo("Use Ativar, Desativar ou Listar.");
                    Ok(Step::Continue)
                }
            },
            Etapa::Nome { acao, .. } => {
                match aplicar(s, &acao, text.trim()) {
                    Ok(v) => s.echo(mensagem(&v)),
                    Err(e) => s.echo(e.to_string()),
                }
                Ok(Step::Done)
            }
        }
    }
}

fn listar(s: &Session) -> Value {
    json!({
        "plugins": domain::registrados().iter().map(|d| json!({
            "id": d.id(),
            "nome": d.nome(),
            "ativo": s.plugin_ativo(d.id()),
        })).collect::<Vec<_>>()
    })
}

fn aplicar(s: &mut Session, acao: &str, id: &str) -> Result<Value> {
    match acao {
        "ativar" | "desativar" => {
            let pedido = id.trim();
            let Some(d) = domain::por_alias(pedido) else {
                return Err(bad("plugin", format!("plugin desconhecido \"{}\"; use PLUGIN LISTAR", pedido.to_lowercase())));
            };
            let ativo = acao == "ativar";
            s.set_plugin_ativo(d.id(), ativo);
            Ok(json!({"id": d.id(), "ativo": ativo}))
        }
        _ => Err(bad("plugin", format!("ação desconhecida \"{acao}\"; use listar, ativar ou desativar"))),
    }
}

/// What the command line echoes after a plugin is activated or deactivated.
pub(crate) fn mensagem(v: &Value) -> String {
    format!("plugin \"{}\" {}.", v["id"].as_str().unwrap_or_default(), if v["ativo"].as_bool().unwrap_or(false) { "ativado" } else { "desativado" })
}

/// The short form of the plugin call: `/id` activates a plugin and `/` alone lists them.
///
/// `None` when `texto` is not a plugin call, so the normal command lookup goes on. The command
/// line, a script, the control channel and MCP all reach it through
/// [`crate::Session::start`] and [`crate::Session::execute`], so there is one activation path.
pub(crate) fn barra(s: &mut Session, texto: &str) -> Option<Result<Value>> {
    let id = texto.trim().strip_prefix('/')?;
    let id = id.trim();
    if id.is_empty() {
        return Some(Ok(listar(s)));
    }
    // A clearer message than `bad("plugin", …)`: the user typed a command, not PLUGIN
    // parameters, so the answer names the plugin and where the list is.
    if domain::por_alias(id).is_none() {
        return Some(Err(crate::EngineError::Other(format!("plugin desconhecido \"{id}\"; use PLUGIN LISTAR ou / para a lista"))));
    }
    Some(aplicar(s, "ativar", id))
}

fn run_plugin(s: &mut Session, p: &Value) -> Result<Value> {
    match str_param(p, "acao").unwrap_or("listar") {
        "listar" => Ok(listar(s)),
        acao => {
            let id = str_param(p, "id").ok_or_else(|| bad("plugin", "`id` é obrigatório"))?;
            aplicar(s, acao, id)
        }
    }
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("plugin", "Plugins", run_plugin)
            .params("{acao: listar|ativar|desativar, id: plugin id (ativar/desativar)}")
            .menu(&["Tools", "Plugins..."])
            .enabled(|_| Ok(()))
            .noundo()
            .interactive(|_| Ok(Box::new(PluginM { etapa: Etapa::default() }))),
    ]
}

#[cfg(test)]
mod tests;

/// O plugin falso dos testes de `PLUGIN`: nenhum plugin real mora na casca
/// (os domínios vivem nos próprios repositórios), então os testes registram
/// um domínio mínimo, como o `apps/cadcraft` faz em produção com os reais.
#[cfg(test)]
struct Teste;

#[cfg(test)]
fn consultar_teste(_: &mut Session, _p: &Value) -> Result<Value> {
    Ok(json!({ "ok": true }))
}

#[cfg(test)]
impl domain::Domain for Teste {
    fn id(&self) -> &'static str {
        "foo"
    }
    fn nome(&self) -> &'static str {
        "Foo (teste)"
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["f"]
    }
    fn commands(&self) -> Vec<CommandSpec> {
        vec![CommandSpec::new("foo.consultar", "teste", consultar_teste).alias(&["fc"]).enabled(super::always)]
    }
    fn state(&self) -> Box<dyn std::any::Any + Send> {
        Box::new(())
    }
}

#[cfg(test)]
static FOO: Teste = Teste;

/// Registra o plugin falso (idempotente, como `registrar` já é por `id`).
#[cfg(test)]
pub(crate) fn registrar_teste() {
    domain::registrar(&FOO);
}
