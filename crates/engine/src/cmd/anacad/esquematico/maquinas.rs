//! Máquinas interativas de seleção (barra, linha, editor, equipamento, artefato).

use super::*;

#[derive(Default)]
pub(in crate::cmd::anacad) struct BarraM {
    pts: Vec<Vec2>,
}

impl Interactive for BarraM {
    fn name(&self) -> &'static str {
        "ANACAD_BARRA"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if self.pts.is_empty() {
            Prompt::new("Primeiro ponto da barra", Accept::POINT)
        } else {
            Prompt::new("Segundo ponto da barra", Accept::POINT).base(self.pts[0])
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if self.pts.is_empty() {
            self.pts.push(p);
            s.last_point = p;
            return Ok(Step::Continue);
        }
        let a = self.pts[0];
        if a.near(p, 1e-12) {
            return Err(crate::EngineError::Other("os dois pontos da barra coincidem".into()));
        }
        let (db, documento) = db_e_documento(s);
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de criar a barra".into()));
        }
        let params = json!({
            "db": db, "documento": documento,
            "x": a.x, "y": a.y, "x2": p.x, "y2": p.y,
        });
        let criado = run_criar_barra(s, &params)?;
        if let Some(handle) = criado.get("handle").and_then(Value::as_str) {
            st(s, |state| state.edit_handle = Some(handle.to_string()));
            if let Some(h) = Handle::parse_hex(handle) {
                s.set_selection(vec![h]);
            }
        }
        Ok(Step::Done)
    }
    fn preview(&self, s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        if let Some(a) = self.pts.first() {
            let pl = LwPolyline {
                vertices: vec![PolyVertex { p: *a, ..PolyVertex::default() }, PolyVertex { p: cursor, ..PolyVertex::default() }],
                closed: false,
                const_width: lar_da_barra(s),
                elevation: 0.0,
                plinegen: false,
            };
            return vec![EntityKind::LwPolyline(pl)];
        }
        Vec::new()
    }
}

#[derive(Default)]
pub(in crate::cmd::anacad) struct LinhaM {
    pts: Vec<Vec2>,
}

impl Interactive for LinhaM {
    fn name(&self) -> &'static str {
        "ANACAD_LINHA"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if self.pts.is_empty() {
            Prompt::new("Primeiro ponto da linha", Accept::POINT)
        } else {
            Prompt::new("Segundo ponto da linha", Accept::POINT).base(self.pts[0])
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if self.pts.is_empty() {
            self.pts.push(p);
            s.last_point = p;
            return Ok(Step::Continue);
        }
        let a = self.pts[0];
        if a.near(p, 1e-12) {
            return Err(crate::EngineError::Other("os dois pontos da linha coincidem".into()));
        }
        let (db, documento) = db_e_documento(s);
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de criar a linha".into()));
        }
        let params = json!({
            "db": db, "documento": documento,
            "x1": a.x, "y1": a.y, "x2": p.x, "y2": p.y,
        });
        let criado = run_criar_linha(s, &params)?;
        if let Some(handle) = criado.get("handle").and_then(Value::as_str) {
            st(s, |state| state.edit_handle = Some(handle.to_string()));
            if let Some(h) = Handle::parse_hex(handle) {
                s.set_selection(vec![h]);
            }
        }
        Ok(Step::Done)
    }
    fn preview(&self, s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        if let Some(a) = self.pts.first() {
            let pl = LwPolyline {
                vertices: vec![PolyVertex { p: *a, ..PolyVertex::default() }, PolyVertex { p: cursor, ..PolyVertex::default() }],
                closed: false,
                const_width: lar_da_linha(s),
                elevation: 0.0,
                plinegen: false,
            };
            return vec![EntityKind::LwPolyline(pl)];
        }
        Vec::new()
    }
}

#[derive(Default)]
pub(in crate::cmd::anacad) struct EditorM {
    sel: SelectPhase,
}

impl Interactive for EditorM {
    fn name(&self) -> &'static str {
        "ANACAD_EDITAR"
    }
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        self.sel = SelectPhase::begin(s);
        self.sel.single = true;
        Ok(Step::Continue)
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Selecione a polilinha", Accept::SELECT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        match self.sel.feed(s, &i)? {
            SelOutcome::More => Ok(Step::Continue),
            SelOutcome::Empty => Ok(Step::Done),
            SelOutcome::Done(hs) => {
                let Some(handle) = hs.first().copied() else { return Ok(Step::Done) };
                let ok = matches!(s.doc()?.entity(handle).map(|e| &e.kind), Some(EntityKind::LwPolyline(_)));
                if !ok {
                    return Err(crate::EngineError::Other("selecione uma LWPOLYLINE".into()));
                }
                st(s, |state| state.edit_handle = Some(handle.hex()));
                s.set_selection(vec![handle]);
                Ok(Step::Done)
            }
        }
    }
}

/// Mão da janela **Equipamentos**: pega o equipamento no desenho e deixa o
/// handle para a UI abrir a janela `Dados de …` dele (o mesmo `equip_handle`
/// que a inserção deixa).
#[derive(Default)]
pub(in crate::cmd::anacad) struct EquipSelM {
    sel: SelectPhase,
}

impl Interactive for EquipSelM {
    fn name(&self) -> &'static str {
        "ANACAD_EQ_SEL"
    }
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        self.sel = SelectPhase::begin(s);
        self.sel.single = true;
        Ok(Step::Continue)
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Selecione o equipamento", Accept::SELECT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        match self.sel.feed(s, &i)? {
            SelOutcome::More => Ok(Step::Continue),
            SelOutcome::Empty => Ok(Step::Done),
            SelOutcome::Done(hs) => {
                let Some(handle) = hs.first().copied() else { return Ok(Step::Done) };
                let equipamento = s.doc()?.entity(handle).is_some_and(|e| app_xdata(e) == Some("ANACAD_EQ"));
                if !equipamento {
                    return Err(crate::EngineError::Other("selecione um equipamento".into()));
                }
                st(s, |state| state.equip_handle = Some(handle.hex()));
                s.set_selection(vec![handle]);
                Ok(Step::Done)
            }
        }
    }
}

/// Seletor **Configurações** do tool set: pega no desenho **qualquer** artefato
/// do domínio — barra, linha ou equipamento — e deixa o handle para a UI abrir
/// a janela de dados dele, como as mãos das janelas Barras, Linhas e
/// Equipamentos fazem, cada uma, para o seu tipo.
#[derive(Default)]
pub(in crate::cmd::anacad) struct ArtefatoSelM {
    sel: SelectPhase,
}

impl Interactive for ArtefatoSelM {
    fn name(&self) -> &'static str {
        "ANACAD_ARTEFATO_SEL"
    }
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        self.sel = SelectPhase::begin(s);
        self.sel.single = true;
        Ok(Step::Continue)
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Selecione a barra, a linha ou o equipamento", Accept::SELECT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        match self.sel.feed(s, &i)? {
            SelOutcome::More => Ok(Step::Continue),
            SelOutcome::Empty => Ok(Step::Done),
            SelOutcome::Done(hs) => {
                let Some(handle) = hs.first().copied() else { return Ok(Step::Done) };
                let artefato = {
                    let Some(entidade) = s.doc()?.entity(handle) else { return Ok(Step::Done) };
                    let marca = app_xdata(entidade).unwrap_or_default();
                    let bloco = match &entidade.kind {
                        EntityKind::Insert(ins) => ins.block.as_str(),
                        _ => "",
                    };
                    matches!(marca, "ANACAD_BARRA" | "ANACAD_LINHA" | "ANACAD_EQ")
                        || bloco.starts_with("ANACAD_EQ_")
                        || bloco.starts_with("ANACAD_SIMB_")
                };
                if !artefato {
                    return Err(crate::EngineError::Other("selecione uma barra, uma linha ou um equipamento".into()));
                }
                st(s, |state| state.artefato_handle = Some(handle.hex()));
                s.set_selection(vec![handle]);
                Ok(Step::Done)
            }
        }
    }
}

/// `anacad.artefato.selecionar` fora do modo interativo não tem o que fazer:
/// quem escolhe o artefato é o `ArtefatoSelM` (a UI chama `app.start`).
pub(in crate::cmd::anacad) fn run_artefato_selecionar(_s: &mut Session, _p: &Value) -> Result<Value> {
    Err(bad("anacad.artefato.selecionar", "selecione o artefato no desenho"))
}

/// `anacad.equipamento.selecionar` fora do modo interativo não tem o que fazer:
/// quem escolhe o equipamento é o `EquipSelM` (a UI chama `app.start`).
pub(in crate::cmd::anacad) fn run_equipamento_selecionar(_s: &mut Session, _p: &Value) -> Result<Value> {
    Err(bad("anacad.equipamento.selecionar", "selecione o equipamento no desenho"))
}
