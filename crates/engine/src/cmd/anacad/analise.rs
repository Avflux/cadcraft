//! Calculos sobre a rede do banco: curto-circuito, fluxo, Ybus/Zbarra, niveis, superacao.

use super::*;

pub(super) fn run_validar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.validar", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.validar", format!("{db}: {e}")))?;
    let documentos = banco.documentos().map_err(|e| bad("anacad.validar", e.to_string()))?;
    let mut componentes = 0usize;
    let mut componentes_ok = 0usize;
    let mut ligacoes = 0usize;
    let mut ligacoes_ok = 0usize;
    for documento in &documentos {
        for c in banco.componentes(documento.id, 10_000).map_err(|e| bad("anacad.validar", e.to_string()))? {
            let Some(texto) = c.handle else { continue };
            componentes += 1;
            if let Some(h) = Handle::parse_hex(&texto)
                && s.doc()?.entity(h).is_some()
            {
                componentes_ok += 1;
            }
        }
        for u in banco.ligacoes(documento.id, 10_000).map_err(|e| bad("anacad.validar", e.to_string()))? {
            let Some(texto) = u.handle else { continue };
            ligacoes += 1;
            if let Some(h) = Handle::parse_hex(&texto)
                && s.doc()?.entity(h).is_some()
            {
                ligacoes_ok += 1;
            }
        }
    }
    Ok(json!({
        "documentos": documentos.len(),
        "componentes": componentes,
        "componentes_ok": componentes_ok,
        "componentes_sem_entidade": componentes - componentes_ok,
        "ligacoes": ligacoes,
        "ligacoes_ok": ligacoes_ok,
        "ligacoes_sem_entidade": ligacoes - ligacoes_ok,
    }))
}

pub(super) fn run_anotar_curto(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.anotar.curto", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let limite = p.get("limite").and_then(Value::as_i64).unwrap_or(50);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.anotar.curto", format!("{db}: {e}")))?;
    let resultados = banco.resultados(documento, limite).map_err(|e| bad("anacad.anotar.curto", e.to_string()))?;
    let mut anotacoes = 0;
    for r in resultados {
        if r.tipo != "curto_circuito" {
            continue;
        }
        let Some(saida) = r.saida.as_deref().and_then(|t| serde_json::from_str::<Value>(t).ok()) else { continue };
        let Some(codigo) = saida.get("barra").and_then(Value::as_i64) else { continue };
        let Some(i_ka) = saida.get("i_ka").and_then(Value::as_f64) else { continue };
        let Ok(Some(barra)) = banco.barra_por_codigo(documento, codigo) else { continue };
        let (x, y) = (barra.x.unwrap_or(0.0), barra.y.unwrap_or(0.0) + 3.0);
        let texto = format!("Icc {i_ka:.2} kA");
        let entidade = EntityKind::Text(Text {
            insert: Vec3::new(x, y, 0.0),
            align_pt: None,
            height: 2.0,
            value: texto,
            rotation: 0.0,
            width_factor: 1.0,
            oblique: 0.0,
            style: "Standard".into(),
            halign: HAlign::Left,
            valign: VAlign::Baseline,
        });
        let handle = s.add_entity(entidade)?;
        marcar(s, handle, "ANACAD_ANOTACAO", &[("resultado", &r.id.to_string()), ("barra", &codigo.to_string())])?;
        anotacoes += 1;
    }
    Ok(json!({"anotacoes": anotacoes}))
}

pub(super) fn run_teste_bloco(s: &mut Session, p: &Value) -> Result<Value> {
    let nome = str_param(p, "nome").unwrap_or("ANACAD_TESTE_BLOCO").trim().to_string();
    if nome.is_empty() {
        return Err(bad("anacad.teste.bloco", "nome vazio"));
    }
    if s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(&nome)) {
        return Err(bad("anacad.teste.bloco", format!("bloco ja existe: {nome}")));
    }
    let mut estado = p.get("semente").and_then(Value::as_u64).unwrap_or(0x5eed_1234);
    let mut proximo = || {
        estado = estado.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((estado >> 33) as f64) / ((1u64 << 31) as f64)
    };
    let entidades = 6usize;
    let doc = s.doc_mut()?;
    let mut bloco = Block::new(&nome);
    bloco.description = "Bloco aleatorio de teste (anacad)".to_string();
    for i in 0..entidades {
        let x = (proximo() - 0.5) * 40.0;
        let y = (proximo() - 0.5) * 40.0;
        let dx = (proximo() - 0.5) * 20.0;
        let dy = (proximo() - 0.5) * 20.0;
        if i % 2 == 0 {
            push_linha(doc, &mut bloco, (x, y), (x + dx, y + dy));
        } else {
            push_circulo(doc, &mut bloco, (x, y), 2.0 + proximo() * 4.0);
        }
    }
    doc.blocks.insert(nome.clone(), Arc::new(bloco));
    let handle = if p.get("inserir").and_then(Value::as_bool).unwrap_or(false) {
        Some(super::blocks::insert(s, &nome, Vec2::new(0.0, 0.0), 1.0, 0.0, &serde_json::Map::new())?)
    } else {
        None
    };
    Ok(json!({
        "bloco": nome,
        "entidades": entidades,
        "handle": handle.map(|h| h.hex()),
    }))
}
