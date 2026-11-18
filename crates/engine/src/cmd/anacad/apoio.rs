//! Funções de apoio compartilhadas pelos comandos: rede do banco, blocos dos
//! equipamentos, arquivos e marcação de XData.

use super::*;

pub(super) fn ler_polilinha(s: &Session, texto: &str) -> Result<(Handle, LwPolyline)> {
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.polilinha", "handle invalido"))?;
    let poly = {
        let doc = s.doc()?;
        let entidade = doc.entity(handle).ok_or_else(|| bad("anacad.polilinha", "entidade nao encontrada"))?;
        match &entidade.kind {
            EntityKind::LwPolyline(p) => p.clone(),
            _ => return Err(bad("anacad.polilinha", "entidade nao e LWPOLYLINE")),
        }
    };
    if poly.vertices.len() < 2 {
        return Err(bad("anacad.polilinha", "polilinha precisa de 2 vertices"));
    }
    Ok((handle, poly))
}

pub(super) fn push_linha(doc: &mut cadcraft_doc::Drawing, bloco: &mut Block, a: (f64, f64), b: (f64, f64)) {
    let h = doc.new_handle();
    bloco.entities.push(Entity::new(h, EntityKind::Line(Line { a: Vec3::new(a.0, a.1, 0.0), b: Vec3::new(b.0, b.1, 0.0) })));
}

pub(super) fn push_circulo(doc: &mut cadcraft_doc::Drawing, bloco: &mut Block, c: (f64, f64), r: f64) {
    let h = doc.new_handle();
    bloco.entities.push(Entity::new(h, EntityKind::Circle(Circle { center: Vec3::new(c.0, c.1, 0.0), radius: r })));
}

pub(super) fn bloco_personalizado(s: &Session, banco: &anacad_data::Banco, slug: &str) -> Option<String> {
    let nome = banco.ler_config(&chave_bloco_equipamento(slug)).ok().flatten()?;
    if nome.trim().is_empty() {
        return None;
    }
    let existe = s.doc().ok()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(nome.trim()));
    existe.then(|| nome.trim().to_string())
}

pub(super) fn equipamento_json(
    id: i64,
    tipo: &str,
    codigo: i64,
    handle: &str,
    x: f64,
    y: f64,
    bloco: &str,
    atributos: &[Value],
    escala: f64,
    altura_texto: f64,
) -> Value {
    json!({"equipamento": id, "tipo": tipo, "codigo": codigo, "handle": handle, "x": x, "y": y, "bloco": bloco, "atributos": atributos, "escala": escala, "altura_texto": altura_texto})
}

/// `anacad.equipamento.inserir` só começa com o tipo escolhido (o comando é
/// interativo e não tem o que inserir sem ele); sem tipo fica desabilitado.
pub(super) fn equipamento_escolhido(s: &Session) -> std::result::Result<(), String> {
    if st_get(s, |state| state.equip_tipo.is_empty()).unwrap_or(true) { Err("escolha um equipamento".into()) } else { Ok(()) }
}

/// `anacad.ponto.inserir` só começa com o tipo de ponto escolhido.
pub(super) fn ponto_escolhido(s: &Session) -> std::result::Result<(), String> {
    if st_get(s, |state| state.equip_tipo.is_empty()).unwrap_or(true) { Err("escolha o tipo de ponto".into()) } else { Ok(()) }
}

pub(super) fn terminal_da_entidade(s: &Session, handle: Handle, terminal: Option<&str>) -> Result<(String, String, f64, f64)> {
    let ent = s.doc()?.entity(handle).ok_or_else(|| bad("anacad.conexao.criar", "entidade nao encontrada"))?;
    match &ent.kind {
        EntityKind::Insert(ins) => {
            let tag = terminal.unwrap_or("T1").to_ascii_uppercase();
            let attr = ins
                .attribs
                .iter()
                .find(|a| a.tag.eq_ignore_ascii_case(&tag))
                .ok_or_else(|| bad("anacad.conexao.criar", format!("terminal {tag} nao encontrado")))?;
            Ok(("Equipamento".to_string(), tag, attr.text.insert.x, attr.text.insert.y))
        }
        EntityKind::LwPolyline(poly) => {
            let tag = terminal.unwrap_or("INICIO").to_ascii_uppercase();
            let p = if tag == "FIM" || tag == "T2" {
                poly.vertices.last().map(|v| v.p).unwrap_or(Vec2::ZERO)
            } else {
                poly.vertices.first().map(|v| v.p).unwrap_or(Vec2::ZERO)
            };
            Ok(("Linha".to_string(), tag, p.x, p.y))
        }
        _ => Err(bad("anacad.conexao.criar", "entidade nao suportada (use bloco ou polilinha)")),
    }
}

pub(super) fn distancia_ponto_linha(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let den = dx * dx + dy * dy;
    if den < 1e-18 {
        return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / den).clamp(0.0, 1.0);
    let qx = a.0 + t * dx;
    let qy = a.1 + t * dy;
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}

pub(super) fn endpoints_da_linha(s: &Session, handle: Handle) -> Option<((f64, f64), (f64, f64))> {
    let ent = s.doc().ok()?.entity(handle)?;
    let EntityKind::LwPolyline(poly) = &ent.kind else { return None };
    let a = poly.vertices.first()?.p;
    let b = poly.vertices.last()?.p;
    Some(((a.x, a.y), (b.x, b.y)))
}

pub(super) fn codigo_da_barra(s: &Session, handle: Handle) -> Option<i64> {
    let ent = s.doc().ok()?.entity(handle)?;
    let tags: Vec<&str> = ent.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
    if !ent.common.xdata.iter().any(|t| t.code == 1001 && t.value == "ANACAD_BARRA") {
        return None;
    }
    tags.windows(2).find(|w| w[0] == "codigo").and_then(|w| w[1].parse::<i64>().ok())
}

/// Cria (uma vez) o bloco `ANACAD_TESTE` com uma linha, insere e marca o INSERT
/// com XData do app `ANACAD`; devolve o handle do INSERT (hex).
pub(super) fn bloco_de_teste(s: &mut Session) -> Result<String> {
    let existe = s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(BLOCO_TESTE));
    if !existe {
        let doc = s.doc_mut()?;
        let mut bloco = Block::new(BLOCO_TESTE);
        bloco.description = "Bloco de teste do anacad (M1)".into();
        let h = doc.new_handle();
        let linha = Entity::new(h, EntityKind::Line(Line { a: Vec3::new(0.0, 0.0, 0.0), b: Vec3::new(10.0, 0.0, 0.0) }));
        bloco.entities.push(linha);
        doc.blocks.insert(BLOCO_TESTE.to_string(), Arc::new(bloco));
    }

    let handle = super::blocks::insert(s, BLOCO_TESTE, Vec2::new(0.0, 0.0), 1.0, 0.0, &serde_json::Map::new())?;
    marcar(s, handle, "ANACAD", &[("versao", "1"), ("bloco", BLOCO_TESTE)])?;
    Ok(handle.hex())
}

/// Desenha os componentes como INSERTs de símbolos e as ligações como linhas;
/// registra os handles nas tabelas do banco.
pub(super) fn desenhar_tela(s: &mut Session, tela: &anacad_core::Screen, banco: &anacad_data::Banco) -> Result<Value> {
    let mut simbolos = 0usize;
    let mut componentes = 0usize;
    for c in &tela.componentes {
        let bloco = garantir_bloco_simbolo(s, c.tipo)?;
        let handle = super::blocks::insert(s, &bloco, Vec2::new(c.px, c.py), 1.0, 0.0, &serde_json::Map::new())?;
        marcar(s, handle, "ANACAD_COMP", &[("id", &c.numero.to_string()), ("tipo", &c.tipo.to_string()), ("barra", &c.refs[0].to_string())])?;
        let _ = banco.definir_handle("Componente", c.numero as i64, &handle.hex());
        componentes += 1;
    }
    simbolos += tela.componentes.iter().map(|c| c.tipo).collect::<std::collections::BTreeSet<_>>().len();

    let mut trechos = 0usize;
    for u in &tela.ligacoes {
        let mut primeiro: Option<Handle> = None;
        for par in u.quebras.windows(2) {
            let (Some(a), Some(b)) = (par.first(), par.get(1)) else { continue };
            let kind = EntityKind::Line(Line { a: Vec3::new(a.0, a.1, 0.0), b: Vec3::new(b.0, b.1, 0.0) });
            let handle = s.add_entity(kind)?;
            marcar(s, handle, "ANACAD_LIG", &[("id", &u.numero.to_string())])?;
            if primeiro.is_none() {
                primeiro = Some(handle);
            }
            trechos += 1;
        }
        if let Some(handle) = primeiro {
            let _ = banco.definir_handle("Ligacao", u.numero as i64, &handle.hex());
        }
    }

    Ok(json!({
        "simbolos": simbolos,
        "componentes": componentes,
        "trechos_ligacao": trechos,
    }))
}

pub(super) fn garantir_bloco_simbolo(s: &mut Session, tipo: i32) -> Result<String> {
    let nome = anacad_app::nome_bloco_simbolo(tipo);
    let existe = s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(&nome));
    if !existe {
        let doc = s.doc_mut()?;
        let mut bloco = Block::new(&nome);
        bloco.description = format!("Simbolo do tipo {tipo} (anacad)");
        let circulo = doc.new_handle();
        bloco.entities.push(Entity::new(circulo, EntityKind::Circle(Circle { center: Vec3::ZERO, radius: 16.0 })));
        let horizontal = doc.new_handle();
        bloco.entities.push(Entity::new(horizontal, EntityKind::Line(Line { a: Vec3::new(-24.0, 0.0, 0.0), b: Vec3::new(24.0, 0.0, 0.0) })));
        let vertical = doc.new_handle();
        bloco.entities.push(Entity::new(vertical, EntityKind::Line(Line { a: Vec3::new(0.0, -24.0, 0.0), b: Vec3::new(0.0, 24.0, 0.0) })));
        doc.blocks.insert(nome.clone(), Arc::new(bloco));
    }
    Ok(nome)
}

pub(super) fn marcar(s: &mut Session, handle: Handle, app: &str, campos: &[(&str, &str)]) -> Result<()> {
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
