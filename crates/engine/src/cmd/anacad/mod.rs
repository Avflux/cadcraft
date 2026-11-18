//! Comandos do dominio anacad (diagnostico, importacao, edicao e analise).
//!
//! O modulo AnaCAD vive inteiro nesta pasta, atras da feature opcional
//! `anacad`; `ana` cuida dos arquivos `.ANA`/`.LST`, `analise` dos calculos e
//! `esquematico` das entidades e maquinas interativas.
//!
//! A superficie do cadcraft usada aqui e curta e nomeada de proposito: a sessao
//! (`Session`), as maquinas interativas (`Interactive`, `Step`, `Prompt`,
//! `Input`, `Accept`), `super::blocks::insert`, `super::machines::{
//! SelOutcome, SelectPhase}` e os auxiliares de parametro importados abaixo.
//! Manter essa lista explicita (sem `use super::*`) e o que permite extrair o
//! modulo para `anacad/crates/*` sem depender do interior do motor.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use cadcraft_doc::{Attrib, Block, Circle, Entity, EntityKind, HAlign, Handle, Insert, Line, LwPolyline, Point, RawTag, Text, VAlign};
use cadcraft_geom::{PolyVertex, Vec2, Vec3};
use serde_json::{Value, json};

use super::machines::{SelOutcome, SelectPhase};
use super::{CommandSpec, always, bad, blocks, bool_or, f64_or, str_param};
use crate::domain::Domain;
use crate::{Accept, Input, Interactive, Prompt, Result, Session, Step};

// Regras do domínio (tipos de equipamento, rede, arquivos): ficam no
// `anacad-app`; aqui só o que o desenho precisa delas.
/// O domínio do anacad, para as telas do módulo (que só dependem do motor).
pub use anacad_app as dominio;
use anacad_app::{
    ALTURA_ATRIBUTO, ALTURA_TEXTO, COR_PADRAO_ACI, ESCALA_BLOCO, LARGURA_BARRA, LARGURA_LINHA, PREFIXO_BLOCO_EQUIPAMENTO, VERSAO_CFG, barra_ficticia,
    chave_bloco_equipamento, circuito_da_linha, contar_por_tipo, ler_arquivo, mesma_tensao, resumo_banco, tipo_equipamento, tipo_ponto,
};

/// Comando que só usa o banco: a regra inteira está no `anacad-app` e o CAD só
/// o registra.
macro_rules! app {
    ($f:path) => {
        |_: &mut Session, p: &Value| -> Result<Value> { Ok($f(p)?) }
    };
}

impl From<anacad_app::Erro> for crate::EngineError {
    fn from(e: anacad_app::Erro) -> Self {
        bad(&e.comando, e.mensagem)
    }
}

mod analise;
mod apoio;
mod esquematico;
mod simbolos;
mod sincronizar;
mod specs;

use analise::*;
use apoio::*;
use esquematico::*;
use simbolos::*;
use sincronizar::*;
use specs::*;

/// What the module keeps for the current session: the connected `.db`, the active document and
/// the editors' drafts. Before the domain API these were eight `anacad_*` fields of `Session`.
pub(crate) struct State {
    pub db: String,
    pub documento: i64,
    /// Elementos por **grupo base de tensão** (tensão em kV e cor), como o `G`
    /// do `.lst` do ANAFAS: a barra, a linha e o componente entram na cor do
    /// nível de tensão deles (a espessura do traço é a do estilo, uma só).
    pub grupos: Vec<GrupoCor>,
    pub edit_handle: Option<String>,
    pub equip_tipo: String,
    pub equip_handle: Option<String>,
    /// Handle que o seletor **Configurações** deixou: barra, linha ou
    /// equipamento, resolvido do lado da UI.
    pub artefato_handle: Option<String>,
    pub texto: String,
    pub texto_altura: f64,
    pub texto_cor: u8,
}

impl Default for State {
    fn default() -> Self {
        Self {
            db: String::new(),
            documento: 1,
            grupos: grupos_padrao(),
            edit_handle: None,
            equip_tipo: String::new(),
            equip_handle: None,
            artefato_handle: None,
            texto: String::new(),
            texto_altura: 2.5,
            texto_cor: 7,
        }
    }
}

/// The module as the engine sees it: its commands and the state a session keeps for it.
pub struct AnacadModule;

impl Domain for AnacadModule {
    fn id(&self) -> &'static str {
        "anacad"
    }
    fn commands(&self) -> Vec<CommandSpec> {
        specs()
    }
    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(State::default())
    }
    /// O que se apaga leva junto o que dependia dele na hora, como a derivação do Eletron sai
    /// junto da conexão apagada (`clsConexao.ApagarDerivacoesLinhas` em `onObjectEraseded`):
    /// a barra leva as linhas dela, a linha leva o texto do circuito e o registro do banco,
    /// o equipamento e a conexão (circuito de ligação) levam um ao outro.
    fn after_command(&self, s: &mut Session, before: Option<&Arc<cadcraft_doc::Drawing>>, uid: Option<u64>) {
        let antes = before.map(|desenho| &**desenho);
        // A inversa primeiro: apagar o circuito de ligação apaga o equipamento.
        // Barra, linha e equipamento rodam depois para não disparar esta.
        sincronizar::cascata_conexoes_apagadas(s, antes, uid);
        sincronizar::cascata_barras_apagadas(s, antes, uid);
        sincronizar::cascata_linhas_apagadas(s, antes, uid);
        sincronizar::cascata_equipamentos_apagados(s, antes, uid);
    }
}

/// The module registered in [`crate::domain::builtin`].
pub static ANACAD: AnacadModule = AnacadModule;

/// Run `f` with this session's module state.
pub(crate) fn st<R>(s: &mut Session, f: impl FnOnce(&mut State) -> R) -> R {
    s.domains.with::<State, R>("anacad", f)
}

/// Read the module state of a session that is only borrowed (a command's `enabled` check).
pub(crate) fn st_get<R>(s: &Session, f: impl FnOnce(&State) -> R) -> Option<R> {
    s.domains.get::<State>("anacad").map(f)
}

/// The connected database and the active document, copied out for a command's parameters.
pub(crate) fn db_e_documento(s: &mut Session) -> (String, i64) {
    st(s, |state| (state.db.clone(), state.documento))
}

// ---------- bridge for the UI ----------
// The UI keeps its own copy of the context (its dialogs own the fields) and hands it over before
// starting a command that reads it: the interactive `anacad.*` commands take `db`/`documento` from
// the session, not from their parameters.

/// Point this session at the connected `.db` and the active document.
pub fn set_banco(s: &mut Session, db: &str, documento: i64) {
    st(s, |state| {
        state.db.clear();
        state.db.push_str(db);
        state.documento = documento;
    });
}

/// O banco que o **desenho** aponta no XData (`ANACAD_CFG.banco`) — o
/// `DirProjeto` que o Eletron relê do DWG em `LerXDataConfiguracao2`. Vazio
/// quando o desenho ainda não foi configurado; é o que a UI consulta ao trocar
/// de aba para decidir se as ferramentas ficam habilitadas.
pub fn banco_do_desenho(s: &Session) -> String {
    s.doc().map(|doc| esquematico::cfg_do_desenho(doc).banco.trim().to_string()).unwrap_or_default()
}

/// The equipment type the next `anacad.equipamento.inserir` inserts.
pub fn set_equip_tipo(s: &mut Session, tipo: &str) {
    st(s, |state| {
        state.equip_tipo.clear();
        state.equip_tipo.push_str(tipo);
    });
}

/// The free text the next `anacad.texto.inserir` places.
pub fn set_texto(s: &mut Session, texto: String, altura: f64, cor: u8) {
    st(s, |state| {
        state.texto = texto;
        state.texto_altura = altura;
        state.texto_cor = cor;
    });
}

/// Take the handle an editor asked the UI to open (a bar, a line or an equipment).
pub fn take_edit_handle(s: &mut Session) -> Option<String> {
    st(s, |state| state.edit_handle.take())
}

/// Take the handle of the equipment just inserted or opened.
pub fn take_equip_handle(s: &mut Session) -> Option<String> {
    st(s, |state| state.equip_handle.take())
}

/// Take the handle the **Configurações** selector picked (a bar, a line or an equipment).
pub fn take_artefato_handle(s: &mut Session) -> Option<String> {
    st(s, |state| state.artefato_handle.take())
}

const BLOCO_TESTE: &str = "ANACAD_TESTE";
