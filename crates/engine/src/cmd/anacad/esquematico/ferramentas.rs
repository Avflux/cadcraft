//! Ferramentas de topologia da barra, terminais, ponto, texto livre e opções de legenda.

use super::*;

pub(in crate::cmd::anacad) fn run_ponto_inserir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ponto.inserir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tipo_txt = p.get("tipo").and_then(Value::as_str).unwrap_or("terminal");
    let (tipo, nome_tipo) = tipo_ponto(tipo_txt).ok_or_else(|| bad("anacad.ponto.inserir", "tipo desconhecido"))?;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let nome = str_param(p, "nome").unwrap_or(nome_tipo);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ponto.inserir", format!("{db}: {e}")))?;
    let id = banco.criar_componente(documento, tipo, None, nome, x, y).map_err(|e| bad("anacad.ponto.inserir", e.to_string()))?;
    let bloco = garantir_bloco_simbolo(s, tipo as i32)?;
    let handle = super::blocks::insert(s, &bloco, Vec2::new(x, y), 1.0, 0.0, &serde_json::Map::new())?;
    banco.definir_handle("Componente", id, &handle.hex()).map_err(|e| bad("anacad.ponto.inserir", e.to_string()))?;
    marcar(s, handle, "ANACAD_COMP", &[("id", &id.to_string()), ("tipo", &tipo.to_string()), ("codigo", "0"), ("nome", nome)])?;
    s.set_selection(vec![handle]);
    Ok(json!({"componente": id, "tipo": tipo, "handle": handle.hex(), "nome": nome}))
}

pub(in crate::cmd::anacad) struct PontoM {
    pub(in crate::cmd::anacad) tipo: String,
}

impl Interactive for PontoM {
    fn name(&self) -> &'static str {
        "ANACAD_PONTO"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Ponto de inserção", Accept::POINT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        let (db, documento) = db_e_documento(s);
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de inserir".into()));
        }
        run_ponto_inserir(s, &json!({"db": db, "documento": documento, "tipo": self.tipo, "x": p.x, "y": p.y}))?;
        Ok(Step::Done)
    }
}

pub(in crate::cmd::anacad) fn run_texto_inserir(s: &mut Session, p: &Value) -> Result<Value> {
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let texto = str_param(p, "texto").ok_or_else(|| bad("anacad.texto.inserir", "`texto` is required"))?;
    let altura = f64_or(p, "altura", 2.5);
    let cor = p.get("cor").and_then(Value::as_u64).unwrap_or(7) as u8;
    let entidade = EntityKind::Text(Text {
        insert: Vec3::new(x, y, 0.0),
        align_pt: None,
        height: altura,
        value: texto.to_string(),
        rotation: 0.0,
        width_factor: 1.0,
        oblique: 0.0,
        style: "Standard".into(),
        halign: HAlign::Left,
        valign: VAlign::Baseline,
    });
    let handle = s.add_entity(entidade)?;
    s.doc_mut()?.modify_entity(handle, |e| e.common.color = cadcraft_doc::color::Color::Index(cor))?;
    marcar(s, handle, "ANACAD_TEXTO", &[("texto", texto), ("altura", &altura.to_string()), ("cor", &cor.to_string())])?;
    Ok(json!({"handle": handle.hex(), "texto": texto}))
}

pub(in crate::cmd::anacad) struct TextoM {
    pub(in crate::cmd::anacad) texto: String,
    pub(in crate::cmd::anacad) altura: f64,
    pub(in crate::cmd::anacad) cor: u8,
}

impl Interactive for TextoM {
    fn name(&self) -> &'static str {
        "ANACAD_TEXTO"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Ponto do texto", Accept::POINT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if self.texto.trim().is_empty() {
            return Err(crate::EngineError::Other("informe o texto".into()));
        }
        run_texto_inserir(s, &json!({"x": p.x, "y": p.y, "texto": self.texto, "altura": self.altura, "cor": self.cor}))?;
        Ok(Step::Done)
    }
}
