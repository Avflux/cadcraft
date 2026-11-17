//! Ferramentas de topologia da barra, terminais, ponto, texto livre e opções de legenda.

use super::*;

pub(in crate::cmd::anacad) fn run_barra_componentes(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.componentes", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.componentes", "`codigo` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.componentes", format!("{db}: {e}")))?;
    let componentes: Vec<Value> = banco
        .componentes_da_barra(documento, codigo)
        .map_err(|e| bad("anacad.barra.componentes", e.to_string()))?
        .into_iter()
        .map(|(id, nome)| json!({"id": id, "nome": nome}))
        .collect();
    Ok(json!({"componentes": componentes}))
}

pub(in crate::cmd::anacad) fn run_barra_separar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.separar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let atual = p.get("atual").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.separar", "`atual` is required"))?;
    let nova = p.get("nova").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.separar", "`nova` is required"))?;
    let nome = str_param(p, "nome").unwrap_or("Nova barra");
    let componentes: Vec<i64> =
        p.get("componentes").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    let jumper = bool_or(p, "jumper", false);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.separar", format!("{db}: {e}")))?;
    let id = banco.separar_barra(documento, atual, nova, nome, &componentes, jumper).map_err(|e| bad("anacad.barra.separar", e.to_string()))?;
    Ok(json!({"barra": id, "codigo": nova, "componentes": componentes.len(), "jumper": jumper}))
}

pub(in crate::cmd::anacad) fn run_barra_unir(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.unir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let final_codigo = p.get("final").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.unir", "`final` is required"))?;
    let eliminada = p.get("eliminada").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.unir", "`eliminada` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.unir", format!("{db}: {e}")))?;
    let removidos = banco.unir_barras(documento, final_codigo, eliminada).map_err(|e| bad("anacad.barra.unir", e.to_string()))?;
    Ok(json!({"final": final_codigo, "eliminada": eliminada, "removidos": removidos}))
}

pub(in crate::cmd::anacad) fn run_linha_seccionar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.seccionar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let de = p.get("de").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.seccionar", "`de` is required"))?;
    let para = p.get("para").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.seccionar", "`para` is required"))?;
    let nova = p.get("nova").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.seccionar", "`nova` is required"))?;
    let nome = str_param(p, "nome").unwrap_or("Barra de seccionamento");
    let percentual = f64_or(p, "percentual", 50.0);
    let reatores = bool_or(p, "reatores", false);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.seccionar", format!("{db}: {e}")))?;
    let barra = banco
        .seccionar_linha(&anacad_data::SeccionarLinha {
            documento,
            de_codigo: de,
            para_codigo: para,
            nova_codigo: nova,
            nome: nome.to_string(),
            percentual,
            eliminar_reatores: reatores,
        })
        .map_err(|e| bad("anacad.linha.seccionar", e.to_string()))?;
    Ok(json!({"barra": barra, "codigo": nova, "de": de, "para": para, "percentual": percentual}))
}

pub(in crate::cmd::anacad) fn run_barra_renumerar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.renumerar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let atual = p.get("atual").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.renumerar", "`atual` is required"))?;
    let nova = p.get("nova").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.renumerar", "`nova` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.renumerar", format!("{db}: {e}")))?;
    banco.renumerar_barra(documento, atual, nova).map_err(|e| bad("anacad.barra.renumerar", e.to_string()))?;
    Ok(json!({"atual": atual, "nova": nova}))
}

pub(in crate::cmd::anacad) fn run_barra_localizar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.localizar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.localizar", "`codigo` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.localizar", format!("{db}: {e}")))?;
    let barra = banco
        .barra_por_codigo(documento, codigo)
        .map_err(|e| bad("anacad.barra.localizar", e.to_string()))?
        .ok_or_else(|| bad("anacad.barra.localizar", format!("barra {codigo} nao existe")))?;
    Ok(json!({"codigo": codigo, "x": barra.x, "y": barra.y, "handle": barra.handle}))
}

pub(in crate::cmd::anacad) fn run_terminal_listar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.terminal.listar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.terminal.listar", format!("{db}: {e}")))?;
    let terminais: Vec<Value> = banco
        .terminais(documento)
        .map_err(|e| bad("anacad.terminal.listar", e.to_string()))?
        .into_iter()
        .map(|t| json!({"id": t.id, "nome": t.nome, "x": t.x, "y": t.y, "handle": t.handle}))
        .collect();
    Ok(json!({"terminais": terminais}))
}

pub(in crate::cmd::anacad) fn run_terminal_renomear(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.terminal.renomear", "`db` is required"))?;
    let id = p.get("id").and_then(Value::as_i64).ok_or_else(|| bad("anacad.terminal.renomear", "`id` is required"))?;
    let nome = str_param(p, "nome").ok_or_else(|| bad("anacad.terminal.renomear", "`nome` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.terminal.renomear", format!("{db}: {e}")))?;
    let n = banco.renomear_terminal(id, nome).map_err(|e| bad("anacad.terminal.renomear", e.to_string()))?;
    Ok(json!({"id": id, "nome": nome, "linhas": n}))
}

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

pub(in crate::cmd::anacad) fn run_barra_detalhar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.detalhar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.detalhar", "`codigo` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.detalhar", format!("{db}: {e}")))?;
    let barra = banco
        .barra_por_codigo(documento, codigo)
        .map_err(|e| bad("anacad.barra.detalhar", e.to_string()))?
        .ok_or_else(|| bad("anacad.barra.detalhar", format!("barra {codigo} nao existe")))?;
    let componentes: Vec<Value> = banco
        .componentes_da_barra(documento, codigo)
        .map_err(|e| bad("anacad.barra.detalhar", e.to_string()))?
        .into_iter()
        .map(|(id, nome)| json!({"id": id, "nome": nome}))
        .collect();
    let circuitos: Vec<Value> = banco
        .circuitos_da_barra(documento, codigo)
        .map_err(|e| bad("anacad.barra.detalhar", e.to_string()))?
        .into_iter()
        .map(|c| json!({"de": c.de, "para": c.para, "tipo": c.tipo, "R1": c.r1, "X1": c.x1}))
        .collect();
    let mut vizinhos: Vec<i64> = Vec::new();
    for c in &circuitos {
        for k in ["de", "para"] {
            if let Some(v) = c.get(k).and_then(Value::as_i64)
                && v != codigo
                && !vizinhos.contains(&v)
            {
                vizinhos.push(v);
            }
        }
    }
    Ok(
        json!({"barra": {"codigo": codigo, "nome": barra.nome, "x": barra.x, "y": barra.y}, "componentes": componentes, "circuitos": circuitos, "vizinhos": vizinhos}),
    )
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

pub(in crate::cmd::anacad) fn run_legenda_config_ler(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.config.ler", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.config.ler", format!("{db}: {e}")))?;
    let dados = banco
        .ler_config("legenda_opcoes")
        .map_err(|e| bad("anacad.legenda.config.ler", e.to_string()))?
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .unwrap_or_else(|| json!({}));
    Ok(json!({"dados": dados}))
}

pub(in crate::cmd::anacad) fn run_legenda_config_gravar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.config.gravar", "`db` is required"))?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.config.gravar", format!("{db}: {e}")))?;
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.gravar_config("legenda_opcoes", &texto).map_err(|e| bad("anacad.legenda.config.gravar", e.to_string()))?;
    Ok(json!({"ok": true}))
}
