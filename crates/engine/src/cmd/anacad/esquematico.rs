//! Desenho e edicao do diagrama unifilar: barras, linhas, equipamentos, conexoes, legenda.

use super::*;

pub(super) fn run_anacad(s: &mut Session, p: &Value) -> Result<Value> {
    let mut resposta = json!({
        "versao": anacad_core::VERSAO,
        "cifrado": false,
        "registros": 0,
        "por_tipo": {},
    });

    if let Some(caminho) = str_param(p, "file") {
        let bytes = ler_arquivo(caminho)?;
        let (registros, cifrado) = anacad_core::ler_tela(&bytes);
        resposta = json!({
            "versao": anacad_core::VERSAO,
            "cifrado": cifrado,
            "registros": registros.len(),
            "por_tipo": contar_por_tipo(&registros),
        });
    }

    if let Some(caminho) = str_param(p, "import") {
        let bytes = ler_arquivo(caminho)?;
        let tela = anacad_core::ler_tela_tipada(&bytes).map_err(|e| bad("anacad", format!("{caminho}: {e}")))?;
        let nome = str_param(p, "nome")
            .map(str::to_string)
            .unwrap_or_else(|| std::path::Path::new(caminho).file_stem().and_then(|s| s.to_str()).unwrap_or("importado").to_string());
        resposta = json!({
            "versao": anacad_core::VERSAO,
            "tela": {
                "componentes": tela.componentes.len(),
                "linhas": tela.linhas.len(),
                "ligacoes": tela.ligacoes.len(),
                "legendas": tela.legendas_componente.len() + tela.legendas_ligacao.len(),
                "extensao": tela.extensao,
            },
        });
        if let Some(db) = str_param(p, "db") {
            let mut banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad", format!("{db}: {e}")))?;
            let resumo = banco.importar_screen(&tela, &nome, Some(caminho)).map_err(|e| bad("anacad", format!("{db}: {e}")))?;
            let desenho = desenhar_tela(s, &tela, &banco)?;
            resposta["banco"] = json!({
                "projeto": resumo.projeto,
                "documento": resumo.documento,
                "barras": resumo.barras,
                "componentes": resumo.componentes,
                "linhas": resumo.linhas,
                "ligacoes": resumo.ligacoes,
                "legendas": resumo.legendas,
            });
            resposta["desenho"] = desenho;
        }
    }

    if bool_or(p, "test_block", false) {
        let handle = bloco_de_teste(s)?;
        resposta["bloco"] = json!(handle);
    }

    Ok(resposta)
}

pub(super) fn run_mover(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.mover", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.mover", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.mover", "handle invalido"))?;
    let dx = f64_or(p, "dx", 0.0);
    let dy = f64_or(p, "dy", 0.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.mover", format!("{db}: {e}")))?;
    let (x, y) = banco
        .posicao_por_handle(texto)
        .map_err(|e| bad("anacad.mover", e.to_string()))?
        .ok_or_else(|| bad("anacad.mover", "handle nao encontrado no banco"))?;
    let (x, y) = (x + dx, y + dy);
    banco.atualizar_posicao_por_handle(texto, x, y).map_err(|e| bad("anacad.mover", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::Insert(ins) = &mut e.kind {
            ins.insert.x = x;
            ins.insert.y = y;
        }
    })?;
    Ok(json!({ "handle": handle.hex(), "x": x, "y": y }))
}

pub(super) fn run_apagar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.apagar", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.apagar", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.apagar", "handle invalido"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.apagar", format!("{db}: {e}")))?;
    let removidos = banco.apagar_por_handle(texto).map_err(|e| bad("anacad.apagar", e.to_string()))?;
    s.doc_mut()?.remove_entity(handle);
    Ok(json!({ "handle": texto, "removidos": removidos }))
}

pub(super) fn run_criar_legenda(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tipo = str_param(p, "tipo").unwrap_or("X").to_ascii_uppercase();
    if tipo != "X" && tipo != "Y" {
        return Err(bad("anacad.legenda.criar", "`tipo` deve ser X ou Y"));
    }
    let numero = p.get("numero").and_then(Value::as_i64).unwrap_or(0) as i32;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let canal = str_param(p, "canal");
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.criar", format!("{db}: {e}")))?;
    let id = banco.criar_legenda(documento, &tipo, numero, canal, numero, 0, x, y).map_err(|e| bad("anacad.legenda.criar", e.to_string()))?;
    let altura = s.doc()?.header.f64("TEXTSIZE", 20.0);
    let estilo = s.doc()?.header.str("TEXTSTYLE", "Standard");
    let kind = EntityKind::Text(cadcraft_doc::Text {
        insert: Vec3::new(x, y, 0.0),
        align_pt: None,
        height: altura,
        value: numero.to_string(),
        rotation: 0.0,
        width_factor: 1.0,
        oblique: 0.0,
        style: estilo,
        halign: cadcraft_doc::HAlign::Left,
        valign: cadcraft_doc::VAlign::Baseline,
    });
    let handle = s.add_entity(kind)?;
    marcar(s, handle, "ANACAD_LEGENDA", &[("id", &id.to_string()), ("tipo", &tipo), ("numero", &numero.to_string())])?;
    banco.definir_handle("Legenda", id, &handle.hex()).map_err(|e| bad("anacad.legenda.criar", e.to_string()))?;
    Ok(json!({ "legenda": id, "handle": handle.hex() }))
}

pub(super) fn run_mover_legenda(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.mover", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.legenda.mover", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.legenda.mover", "handle invalido"))?;
    let dx = f64_or(p, "dx", 0.0);
    let dy = f64_or(p, "dy", 0.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.mover", format!("{db}: {e}")))?;
    let legenda = banco
        .legenda_por_handle(texto)
        .map_err(|e| bad("anacad.legenda.mover", e.to_string()))?
        .ok_or_else(|| bad("anacad.legenda.mover", "handle nao esta no banco"))?;
    let (x, y) = (legenda.x.unwrap_or(0.0) + dx, legenda.y.unwrap_or(0.0) + dy);
    banco.atualizar_legenda_posicao(texto, x, y).map_err(|e| bad("anacad.legenda.mover", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::Text(t) = &mut e.kind {
            t.insert.x = x;
            t.insert.y = y;
        }
    })?;
    Ok(json!({ "handle": handle.hex(), "x": x, "y": y }))
}

pub(super) fn run_criar_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let nome = str_param(p, "nome");
    let comprimento = f64_or(p, "comprimento", 1.0).max(0.01);
    let x2 = p.get("x2").and_then(Value::as_f64).unwrap_or(x + comprimento);
    let y2 = p.get("y2").and_then(Value::as_f64).unwrap_or(y);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.criar", format!("{db}: {e}")))?;
    let codigo = match p.get("codigo").and_then(Value::as_i64) {
        Some(c) if c > 0 => c,
        _ => banco.proximo_codigo_barra(documento).map_err(|e| bad("anacad.barra.criar", e.to_string()))?,
    };
    let centro = ((x + x2) / 2.0, (y + y2) / 2.0);
    let id = banco.criar_barra(documento, codigo, nome, centro.0, centro.1).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;

    // Barra = LWPOLYLINE de 2 vértices (início/fim), largura 0,3.
    let poly = LwPolyline {
        vertices: vec![PolyVertex { p: Vec2::new(x, y), ..PolyVertex::default() }, PolyVertex { p: Vec2::new(x2, y2), ..PolyVertex::default() }],
        closed: false,
        const_width: 0.3,
        elevation: 0.0,
        plinegen: false,
    };
    let handle = s.add_entity(EntityKind::LwPolyline(poly))?;
    marcar(
        s,
        handle,
        "ANACAD_BARRA",
        &[
            ("id", &id.to_string()),
            ("codigo", &codigo.to_string()),
            ("x1", &x.to_string()),
            ("y1", &y.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0.3"),
        ],
    )?;
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    Ok(json!({
        "barra": id,
        "codigo": codigo,
        "handle": handle.hex(),
        "x1": x, "y1": y, "x2": x2, "y2": y2,
        "largura": 0.3,
        "tipo": "LWPOLYLINE",
    }))
}

pub(super) fn run_criar_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let de = p.get("de").and_then(Value::as_i64);
    let para = p.get("para").and_then(Value::as_i64);
    let mut x1 = f64_or(p, "x1", 0.0);
    let mut y1 = f64_or(p, "y1", 0.0);
    let mut x2 = f64_or(p, "x2", 0.0);
    let mut y2 = f64_or(p, "y2", 0.0);
    let mut barra_de = None;
    let mut barra_para = None;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.criar", format!("{db}: {e}")))?;
    if let (Some(de), Some(para)) = (de, para) {
        barra_de = Some(
            banco
                .barra_por_codigo(documento, de)
                .map_err(|e| bad("anacad.linha.criar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.criar", format!("barra {de} nao existe no documento")))?,
        );
        barra_para = Some(
            banco
                .barra_por_codigo(documento, para)
                .map_err(|e| bad("anacad.linha.criar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.criar", format!("barra {para} nao existe no documento")))?,
        );
        if let (Some(b1), Some(b2)) = (barra_de.as_ref(), barra_para.as_ref()) {
            x1 = b1.x.unwrap_or(x1);
            y1 = b1.y.unwrap_or(y1);
            x2 = b2.x.unwrap_or(x2);
            y2 = b2.y.unwrap_or(y2);
        }
    }
    let id = banco
        .criar_linha(documento, barra_de.as_ref().map(|b| b.id), barra_para.as_ref().map(|b| b.id))
        .map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    let poly = LwPolyline {
        vertices: vec![PolyVertex { p: Vec2::new(x1, y1), ..PolyVertex::default() }, PolyVertex { p: Vec2::new(x2, y2), ..PolyVertex::default() }],
        closed: false,
        const_width: 0.0,
        elevation: 0.0,
        plinegen: false,
    };
    let handle = s.add_entity(EntityKind::LwPolyline(poly))?;
    let mut campos = vec![
        ("id", id.to_string()),
        ("x1", x1.to_string()),
        ("y1", y1.to_string()),
        ("x2", x2.to_string()),
        ("y2", y2.to_string()),
        ("largura", "0".to_string()),
    ];
    if let Some(de) = de {
        campos.push(("de", de.to_string()));
    }
    if let Some(para) = para {
        campos.push(("para", para.to_string()));
    }
    let refs: Vec<(&str, &str)> = campos.iter().map(|(k, v)| (*k, v.as_str())).collect();
    marcar(s, handle, "ANACAD_LINHA", &refs)?;
    banco.definir_handle("Linha", id, &handle.hex()).map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    Ok(json!({
        "linha": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": 0.0,
        "tipo": "LWPOLYLINE",
    }))
}

pub(super) fn run_editar_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.editar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.editar", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.editar", format!("{db}: {e}")))?;
    let codigo = p
        .get("codigo")
        .and_then(Value::as_i64)
        .or_else(|| dados.get("codigo").and_then(Value::as_i64))
        .ok_or_else(|| bad("anacad.barra.editar", "`codigo` is required"))?;
    let nome = dados.get("nome").and_then(Value::as_str);
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    let id = match banco.barra_por_codigo(documento, codigo).map_err(|e| bad("anacad.barra.editar", e.to_string()))? {
        Some(b) => {
            banco.atualizar_barra_nome_pos(b.id, nome, (x1 + x2) / 2.0, (y1 + y2) / 2.0).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
            b.id
        }
        None => {
            banco.criar_barra(documento, codigo, nome, (x1 + x2) / 2.0, (y1 + y2) / 2.0).map_err(|e| bad("anacad.barra.editar", e.to_string()))?
        }
    };
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
    // A base de tensão (`DBAR`) vive na coluna `VBase`: sem gravar aqui, o valor
    // digitado na janela não chega ao cálculo (`Banco::caso`).
    if let Some(vbase) = dados.get("vbase").and_then(Value::as_f64) {
        banco.atualizar_barra_vbase(Some(documento), codigo, vbase).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
    }
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.atualizar_dados_barra(&handle.hex(), &texto).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.3;
        }
    })?;
    marcar(
        s,
        handle,
        "ANACAD_BARRA",
        &[
            ("id", &id.to_string()),
            ("codigo", &codigo.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0.3"),
            ("dados", &texto),
        ],
    )?;
    Ok(json!({ "barra": id, "handle": handle.hex(), "dados": dados }))
}

pub(super) fn run_editar_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.editar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.linha.editar", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let de = p.get("de").and_then(Value::as_i64).or_else(|| dados.get("de").and_then(Value::as_i64));
    let para = p.get("para").and_then(Value::as_i64).or_else(|| dados.get("para").and_then(Value::as_i64));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.editar", format!("{db}: {e}")))?;
    let barra_de = match de {
        Some(c) => Some(
            banco
                .barra_por_codigo(documento, c)
                .map_err(|e| bad("anacad.linha.editar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.editar", format!("barra {c} nao existe")))?,
        ),
        None => None,
    };
    let barra_para = match para {
        Some(c) => Some(
            banco
                .barra_por_codigo(documento, c)
                .map_err(|e| bad("anacad.linha.editar", e.to_string()))?
                .ok_or_else(|| bad("anacad.linha.editar", format!("barra {c} nao existe")))?,
        ),
        None => None,
    };
    let id = banco
        .linha_id_por_handle(&handle.hex())
        .map_err(|e| bad("anacad.linha.editar", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.editar", "linha nao encontrada pelo handle"))?;
    banco
        .atualizar_barras_linha(&handle.hex(), barra_de.as_ref().map(|b| b.id), barra_para.as_ref().map(|b| b.id))
        .map_err(|e| bad("anacad.linha.editar", e.to_string()))?;
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.atualizar_dados_linha(&handle.hex(), &texto).map_err(|e| bad("anacad.linha.editar", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.0;
        }
    })?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    marcar(
        s,
        handle,
        "ANACAD_LINHA",
        &[
            ("id", &id.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0"),
            ("dados", &texto),
        ],
    )?;
    Ok(json!({ "linha": id, "handle": handle.hex(), "dados": dados }))
}

/// "Excluir Dados Elétricos" da barra (ANAFAS 33057): zera `VBase`/`Dados` no
/// banco e o `Dados` no XData, mantendo o desenho e a topologia. É o botão
/// Remover da janela Dados de Barra (149).
pub(super) fn run_barra_dados_excluir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.dados.excluir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.dados.excluir", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.dados.excluir", format!("{db}: {e}")))?;
    let codigo = match p.get("codigo").and_then(Value::as_i64) {
        Some(c) => c,
        None => banco
            .barras(documento, 100_000)
            .map_err(|e| bad("anacad.barra.dados.excluir", e.to_string()))?
            .into_iter()
            .find(|b| b.handle.as_deref() == Some(handle_txt))
            .and_then(|b| b.codigo)
            .ok_or_else(|| bad("anacad.barra.dados.excluir", "barra nao encontrada pelo handle"))?,
    };
    let removidos = banco.limpar_dados_barra(documento, codigo).map_err(|e| bad("anacad.barra.dados.excluir", e.to_string()))?;
    let id = banco.barra_por_codigo(documento, codigo).map_err(|e| bad("anacad.barra.dados.excluir", e.to_string()))?.map(|b| b.id).unwrap_or(0);
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    marcar(
        s,
        handle,
        "ANACAD_BARRA",
        &[
            ("id", &id.to_string()),
            ("codigo", &codigo.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0.3"),
            ("dados", "{}"),
        ],
    )?;
    Ok(json!({ "barra": id, "handle": handle.hex(), "removidos": removidos }))
}

/// "Excluir Dados Elétricos" da linha (ANAFAS 33057): zera o `Dados` no banco
/// e no XData, mantendo desenho, nome e as barras De/Para. Botão Remover da
/// janela Dados de Linha (164).
pub(super) fn run_linha_dados_excluir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.dados.excluir", "`db` is required"))?;
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.linha.dados.excluir", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.dados.excluir", format!("{db}: {e}")))?;
    let id = banco
        .linha_id_por_handle(&handle.hex())
        .map_err(|e| bad("anacad.linha.dados.excluir", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.dados.excluir", "linha nao encontrada pelo handle"))?;
    let removidos = banco.limpar_dados_linha(&handle.hex()).map_err(|e| bad("anacad.linha.dados.excluir", e.to_string()))?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    marcar(
        s,
        handle,
        "ANACAD_LINHA",
        &[
            ("id", &id.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", "0"),
            ("dados", "{}"),
        ],
    )?;
    Ok(json!({ "linha": id, "handle": handle.hex(), "removidos": removidos }))
}

pub(super) fn run_atribuir_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.atribuir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.atribuir", "`handle` is required"))?;
    let codigo = p.get("codigo").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.atribuir", "`codigo` is required"))?;
    let nome = str_param(p, "nome");
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.3;
        }
    })?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.atribuir", format!("{db}: {e}")))?;
    let id = banco.criar_barra(documento, codigo, nome, (x1 + x2) / 2.0, (y1 + y2) / 2.0).map_err(|e| bad("anacad.barra.atribuir", e.to_string()))?;
    marcar(
        s,
        handle,
        "ANACAD_BARRA",
        &[
            ("id", &id.to_string()),
            ("codigo", &codigo.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", &poly.const_width.to_string()),
        ],
    )?;
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.atribuir", e.to_string()))?;
    Ok(json!({
        "barra": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": poly.const_width,
    }))
}

pub(super) fn run_atribuir_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.atribuir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.linha.atribuir", "`handle` is required"))?;
    let de = p.get("de").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.atribuir", "`de` is required"))?;
    let para = p.get("para").and_then(Value::as_i64).ok_or_else(|| bad("anacad.linha.atribuir", "`para` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = 0.0;
        }
    })?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.linha.atribuir", format!("{db}: {e}")))?;
    let barra_de = banco
        .barra_por_codigo(documento, de)
        .map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.atribuir", format!("barra {de} nao existe")))?;
    let barra_para = banco
        .barra_por_codigo(documento, para)
        .map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?
        .ok_or_else(|| bad("anacad.linha.atribuir", format!("barra {para} nao existe")))?;
    let id = banco.criar_linha(documento, Some(barra_de.id), Some(barra_para.id)).map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?;
    marcar(
        s,
        handle,
        "ANACAD_LINHA",
        &[
            ("id", &id.to_string()),
            ("de", &de.to_string()),
            ("para", &para.to_string()),
            ("x1", &x1.to_string()),
            ("y1", &y1.to_string()),
            ("x2", &x2.to_string()),
            ("y2", &y2.to_string()),
            ("largura", &poly.const_width.to_string()),
        ],
    )?;
    banco.definir_handle("Linha", id, &handle.hex()).map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?;
    Ok(json!({
        "linha": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": poly.const_width,
    }))
}

#[derive(Default)]
pub(super) struct BarraM {
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
    fn preview(&self, _s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        if let Some(a) = self.pts.first() {
            let mut pl = LwPolyline {
                vertices: vec![PolyVertex { p: *a, ..PolyVertex::default() }, PolyVertex { p: cursor, ..PolyVertex::default() }],
                closed: false,
                const_width: 0.3,
                elevation: 0.0,
                plinegen: false,
            };
            pl.const_width = 0.3;
            return vec![EntityKind::LwPolyline(pl)];
        }
        Vec::new()
    }
}

#[derive(Default)]
pub(super) struct LinhaM {
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
    fn preview(&self, _s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        if let Some(a) = self.pts.first() {
            let pl = LwPolyline {
                vertices: vec![PolyVertex { p: *a, ..PolyVertex::default() }, PolyVertex { p: cursor, ..PolyVertex::default() }],
                closed: false,
                const_width: 0.0,
                elevation: 0.0,
                plinegen: false,
            };
            return vec![EntityKind::LwPolyline(pl)];
        }
        Vec::new()
    }
}

#[derive(Default)]
pub(super) struct EditorM {
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

pub(super) fn run_equipamento_bloco(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.bloco", "`db` is required"))?;
    let tipo_txt = str_param(p, "tipo").ok_or_else(|| bad("anacad.equipamento.bloco", "`tipo` is required"))?;
    let (slug, _nome_tipo) = tipo_equipamento(tipo_txt).ok_or_else(|| bad("anacad.equipamento.bloco", format!("tipo desconhecido: {tipo_txt}")))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.bloco", format!("{db}: {e}")))?;
    let bloco = str_param(p, "bloco").unwrap_or("").trim().to_string();
    if !bloco.is_empty() {
        let existe = s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(&bloco));
        if !existe {
            return Err(bad("anacad.equipamento.bloco", format!("bloco nao encontrado no desenho: {bloco}")));
        }
    }
    banco.gravar_config(&chave_bloco_equipamento(slug), &bloco).map_err(|e| bad("anacad.equipamento.bloco", e.to_string()))?;
    Ok(json!({"tipo": tipo_txt, "slug": slug, "bloco": bloco}))
}

pub(super) fn run_equipamento_inserir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.inserir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tipo_txt = str_param(p, "tipo").ok_or_else(|| bad("anacad.equipamento.inserir", "`tipo` is required"))?;
    let (slug, nome_tipo) = tipo_equipamento(tipo_txt).ok_or_else(|| bad("anacad.equipamento.inserir", format!("tipo desconhecido: {tipo_txt}")))?;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.inserir", format!("{db}: {e}")))?;
    let codigo = match p.get("codigo").and_then(Value::as_i64) {
        Some(c) if c > 0 => c,
        _ => banco.proximo_codigo_equipamento(documento, nome_tipo).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?,
    };
    let id = banco.criar_equipamento(documento, nome_tipo, codigo, nome_tipo, x, y).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?;
    let bloco = if let Some(custom) = bloco_personalizado(s, &banco, slug) { custom } else { bloco_equipamento(s, slug)? };
    let mut valores = serde_json::Map::new();
    valores.insert("CODIGO".into(), json!(codigo.to_string()));
    valores.insert("NOME".into(), json!(nome_tipo));
    valores.insert("TENSAO".into(), json!(""));
    valores.insert("T1".into(), json!("DE"));
    valores.insert("T2".into(), json!("PARA"));
    let handle = super::blocks::insert(s, &bloco, Vec2::new(x, y), 1.0, 0.0, &valores)?;
    let atributos: Vec<Value> = s
        .doc()?
        .entity(handle)
        .and_then(|e| match &e.kind {
            EntityKind::Insert(ins) => Some(ins.attribs.iter().map(|a| json!({"tag": a.tag, "valor": a.text.value})).collect()),
            _ => None,
        })
        .unwrap_or_default();
    banco.definir_handle("Equipamento", id, &handle.hex()).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?;
    marcar(
        s,
        handle,
        "ANACAD_EQ",
        &[("id", &id.to_string()), ("tipo", nome_tipo), ("codigo", &codigo.to_string()), ("x", &x.to_string()), ("y", &y.to_string())],
    )?;
    st(s, |state| state.equip_handle = Some(handle.hex()));
    s.set_selection(vec![handle]);
    Ok(equipamento_json(id, nome_tipo, codigo, &handle.hex(), x, y, &bloco, &atributos))
}

/// Botão Remover das janelas de dados dos equipamentos (`Dados de …`):
/// exclui os dados elétricos (ANAFAS 33057), mantendo bloco, nome e posição.
pub(super) fn run_equipamento_dados_excluir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.dados.excluir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.equipamento.dados.excluir", "`handle` is required"))?;
    let handle = Handle::parse_hex(handle_txt).ok_or_else(|| bad("anacad.equipamento.dados.excluir", "handle invalido"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.dados.excluir", format!("{db}: {e}")))?;
    let equip = banco
        .equipamentos(documento, 100_000)
        .map_err(|e| bad("anacad.equipamento.dados.excluir", e.to_string()))?
        .into_iter()
        .find(|e| e.handle.as_deref() == Some(handle_txt))
        .ok_or_else(|| bad("anacad.equipamento.dados.excluir", "equipamento nao encontrado pelo handle"))?;
    let removidos = banco.limpar_dados_equipamento(&handle.hex()).map_err(|e| bad("anacad.equipamento.dados.excluir", e.to_string()))?;
    let pos = s.doc()?.entity(handle).and_then(|e| match &e.kind {
        EntityKind::Insert(ins) => Some((ins.insert.x, ins.insert.y)),
        _ => None,
    });
    let (x, y) = pos.unwrap_or((0.0, 0.0));
    marcar(
        s,
        handle,
        "ANACAD_EQ",
        &[
            ("id", &equip.id.to_string()),
            ("tipo", equip.tipo.as_str()),
            ("codigo", &equip.codigo.to_string()),
            ("x", &x.to_string()),
            ("y", &y.to_string()),
        ],
    )?;
    Ok(json!({ "equipamento": equip.id, "handle": handle.hex(), "removidos": removidos }))
}

pub(super) fn run_equipamento_editar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.editar", "`db` is required"))?;
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.equipamento.editar", "`handle` is required"))?;
    let handle = Handle::parse_hex(handle_txt).ok_or_else(|| bad("anacad.equipamento.editar", "handle invalido"))?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.editar", format!("{db}: {e}")))?;
    let nome = dados.get("nome").and_then(Value::as_str);
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.atualizar_dados_equipamento(&handle.hex(), &texto).map_err(|e| bad("anacad.equipamento.editar", e.to_string()))?;
    {
        let codigo_val = dados.get("codigo").and_then(Value::as_i64).map(|v| v.to_string());
        let nome_val = dados.get("nome").and_then(Value::as_str).map(str::to_string);
        let de_val = dados.get("de").or_else(|| dados.get("barra")).and_then(Value::as_i64).map(|v| v.to_string());
        let para_val = dados.get("para").and_then(Value::as_i64).map(|v| v.to_string());
        s.doc_mut()?.modify_entity(handle, |e| {
            let EntityKind::Insert(ins) = &mut e.kind else { return };
            for a in &mut ins.attribs {
                match a.tag.as_str() {
                    "CODIGO" => {
                        if let Some(v) = &codigo_val {
                            a.text.value = v.clone()
                        }
                    }
                    "NOME" => {
                        if let Some(v) = &nome_val {
                            a.text.value = v.clone()
                        }
                    }
                    "T1" => {
                        if let Some(v) = &de_val {
                            a.text.value = v.clone()
                        }
                    }
                    "T2" => {
                        if let Some(v) = &para_val {
                            a.text.value = v.clone()
                        }
                    }
                    _ => {}
                }
            }
        })?;
    }
    let pos = s.doc()?.entity(handle).and_then(|e| match &e.kind {
        EntityKind::Insert(ins) => Some((ins.insert.x, ins.insert.y)),
        _ => None,
    });
    if let Some((x, y)) = pos {
        banco.atualizar_equipamento_nome_pos(&handle.hex(), nome, x, y).map_err(|e| bad("anacad.equipamento.editar", e.to_string()))?;
    }
    let tipo = s
        .doc()?
        .entity(handle)
        .and_then(|e| {
            e.common
                .xdata
                .iter()
                .filter(|t| t.code == 1000)
                .map(|t| t.value.as_str())
                .collect::<Vec<_>>()
                .windows(2)
                .find(|w| w[0] == "tipo")
                .map(|w| w[1].to_string())
        })
        .unwrap_or_default();
    let codigo = s.doc()?.entity(handle).and_then(|e| {
        e.common
            .xdata
            .iter()
            .filter(|t| t.code == 1000)
            .map(|t| t.value.as_str())
            .collect::<Vec<_>>()
            .windows(2)
            .find(|w| w[0] == "codigo")
            .and_then(|w| w[1].parse::<i64>().ok())
    });
    let id = s
        .doc()?
        .entity(handle)
        .and_then(|e| {
            e.common
                .xdata
                .iter()
                .filter(|t| t.code == 1000)
                .map(|t| t.value.as_str())
                .collect::<Vec<_>>()
                .windows(2)
                .find(|w| w[0] == "id")
                .and_then(|w| w[1].parse::<i64>().ok())
        })
        .unwrap_or(0);
    marcar(s, handle, "ANACAD_EQ", &[("id", &id.to_string()), ("tipo", &tipo), ("codigo", &codigo.unwrap_or(0).to_string()), ("dados", &texto)])?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    banco.sincronizar_equipamento(documento, &tipo, &dados).map_err(|e| bad("anacad.equipamento.editar", e.to_string()))?;
    Ok(json!({"equipamento": id, "tipo": tipo, "handle": handle.hex(), "dados": dados}))
}

pub(super) struct EquipM {
    pub(super) tipo: String,
}

impl Interactive for EquipM {
    fn name(&self) -> &'static str {
        "ANACAD_EQ"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        Prompt::new("Ponto de inserção do equipamento", Accept::POINT)
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        let (db, documento) = db_e_documento(s);
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de inserir".into()));
        }
        run_equipamento_inserir(s, &json!({"db": db, "documento": documento, "tipo": self.tipo, "x": p.x, "y": p.y}))?;
        Ok(Step::Done)
    }
    fn preview(&self, _s: &Session, _cursor: Vec2) -> Vec<EntityKind> {
        Vec::new()
    }
}

pub(super) fn run_barra_componentes(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) fn run_barra_separar(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) fn run_barra_unir(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.unir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let final_codigo = p.get("final").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.unir", "`final` is required"))?;
    let eliminada = p.get("eliminada").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.unir", "`eliminada` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.unir", format!("{db}: {e}")))?;
    let removidos = banco.unir_barras(documento, final_codigo, eliminada).map_err(|e| bad("anacad.barra.unir", e.to_string()))?;
    Ok(json!({"final": final_codigo, "eliminada": eliminada, "removidos": removidos}))
}

pub(super) fn run_linha_seccionar(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) fn run_barra_renumerar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.renumerar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let atual = p.get("atual").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.renumerar", "`atual` is required"))?;
    let nova = p.get("nova").and_then(Value::as_i64).ok_or_else(|| bad("anacad.barra.renumerar", "`nova` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.renumerar", format!("{db}: {e}")))?;
    banco.renumerar_barra(documento, atual, nova).map_err(|e| bad("anacad.barra.renumerar", e.to_string()))?;
    Ok(json!({"atual": atual, "nova": nova}))
}

pub(super) fn run_barra_localizar(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) fn run_terminal_listar(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) fn run_terminal_renomear(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.terminal.renomear", "`db` is required"))?;
    let id = p.get("id").and_then(Value::as_i64).ok_or_else(|| bad("anacad.terminal.renomear", "`id` is required"))?;
    let nome = str_param(p, "nome").ok_or_else(|| bad("anacad.terminal.renomear", "`nome` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.terminal.renomear", format!("{db}: {e}")))?;
    let n = banco.renomear_terminal(id, nome).map_err(|e| bad("anacad.terminal.renomear", e.to_string()))?;
    Ok(json!({"id": id, "nome": nome, "linhas": n}))
}

pub(super) fn run_ponto_inserir(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) struct PontoM {
    pub(super) tipo: String,
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

pub(super) fn run_barra_detalhar(_s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) fn run_texto_inserir(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(super) struct TextoM {
    pub(super) texto: String,
    pub(super) altura: f64,
    pub(super) cor: u8,
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

pub(super) fn run_legenda_config_ler(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.config.ler", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.config.ler", format!("{db}: {e}")))?;
    let dados = banco
        .ler_config("legenda_opcoes")
        .map_err(|e| bad("anacad.legenda.config.ler", e.to_string()))?
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .unwrap_or_else(|| json!({}));
    Ok(json!({"dados": dados}))
}

pub(super) fn run_legenda_config_gravar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.legenda.config.gravar", "`db` is required"))?;
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.legenda.config.gravar", format!("{db}: {e}")))?;
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.gravar_config("legenda_opcoes", &texto).map_err(|e| bad("anacad.legenda.config.gravar", e.to_string()))?;
    Ok(json!({"ok": true}))
}

pub(super) fn run_conexao_criar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.conexao.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let de_txt = str_param(p, "de_handle").ok_or_else(|| bad("anacad.conexao.criar", "`de_handle` is required"))?;
    let para_txt = str_param(p, "para_handle").ok_or_else(|| bad("anacad.conexao.criar", "`para_handle` is required"))?;
    let de_handle = Handle::parse_hex(de_txt).ok_or_else(|| bad("anacad.conexao.criar", "handle De invalido"))?;
    let para_handle = Handle::parse_hex(para_txt).ok_or_else(|| bad("anacad.conexao.criar", "handle Para invalido"))?;
    let de_terminal = str_param(p, "de_terminal");
    let para_terminal = str_param(p, "para_terminal");
    let (de_tipo, de_term, de_x, de_y) = terminal_da_entidade(s, de_handle, de_terminal)?;
    let (para_tipo, para_term, para_x, para_y) = terminal_da_entidade(s, para_handle, para_terminal)?;
    let desenhar = p.get("desenhar").and_then(Value::as_bool).unwrap_or(true);
    let handle = if desenhar {
        let poly = LwPolyline {
            vertices: vec![
                PolyVertex { p: Vec2::new(de_x, de_y), ..PolyVertex::default() },
                PolyVertex { p: Vec2::new(para_x, para_y), ..PolyVertex::default() },
            ],
            closed: false,
            const_width: 0.0,
            elevation: 0.0,
            plinegen: false,
        };
        Some(s.add_entity(EntityKind::LwPolyline(poly))?)
    } else {
        None
    };
    let potencial = p.get("potencial").and_then(Value::as_i64);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.conexao.criar", format!("{db}: {e}")))?;
    let id = banco
        .inserir_conexao(&anacad_data::ConexaoNova {
            documento: Some(documento),
            de_tipo,
            de_handle: de_handle.hex(),
            de_terminal: de_term.clone(),
            para_tipo,
            para_handle: para_handle.hex(),
            para_terminal: para_term.clone(),
            potencial,
            handle: handle.map(|h| h.hex()),
        })
        .map_err(|e| bad("anacad.conexao.criar", e.to_string()))?;
    if let Some(handle) = handle {
        let potencial_txt = potencial.map(|v| v.to_string()).unwrap_or_default();
        marcar(
            s,
            handle,
            "ANACAD_CONEXAO",
            &[("de", de_txt), ("para", para_txt), ("de_terminal", &de_term), ("para_terminal", &para_term), ("potencial", &potencial_txt)],
        )?;
    }
    Ok(json!({"conexao": id, "handle": handle.map(|h| h.hex())}))
}

pub(super) fn run_conexao_validar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.conexao.validar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tol = f64_or(p, "tolerancia", 0.5);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.conexao.validar", format!("{db}: {e}")))?;
    let conexoes = banco.conexoes(documento).map_err(|e| bad("anacad.conexao.validar", e.to_string()))?;
    let mut saida = Vec::new();
    let mut validas = 0usize;
    for c in &conexoes {
        let mut erro = None;
        let mut d_de = None;
        let mut d_para = None;
        let de_pos = c
            .de_handle
            .as_deref()
            .and_then(Handle::parse_hex)
            .and_then(|h| terminal_da_entidade(s, h, c.de_terminal.as_deref()).ok())
            .map(|(_, _, x, y)| (x, y));
        let para_pos = c
            .para_handle
            .as_deref()
            .and_then(Handle::parse_hex)
            .and_then(|h| terminal_da_entidade(s, h, c.para_terminal.as_deref()).ok())
            .map(|(_, _, x, y)| (x, y));
        if de_pos.is_none() || para_pos.is_none() {
            erro = Some("terminal nao encontrado".to_string());
        }
        if let (Some(linha), Some(dp), Some(pp)) =
            (c.handle.as_deref().and_then(Handle::parse_hex).and_then(|h| endpoints_da_linha(s, h)), de_pos, para_pos)
        {
            let a = distancia_ponto_linha(dp, linha.0, linha.1);
            let b = distancia_ponto_linha(pp, linha.0, linha.1);
            d_de = Some(a);
            d_para = Some(b);
            if erro.is_none() && (a > tol || b > tol) {
                erro = Some(format!("ponta fora da tolerancia ({a:.3}, {b:.3})"));
            }
        }
        let ok = erro.is_none();
        if ok {
            validas += 1;
        }
        saida.push(json!({
            "id": c.id,
            "ok": ok,
            "erro": erro,
            "dist_de": d_de,
            "dist_para": d_para,
        }));
    }
    Ok(json!({"conexoes": saida, "validas": validas, "total": conexoes.len()}))
}

pub(super) fn run_conexao_topologia(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.conexao.topologia", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tol = f64_or(p, "tolerancia", 0.5);
    let apenas_validas = p.get("apenas_validas").and_then(Value::as_bool).unwrap_or(true);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.conexao.topologia", format!("{db}: {e}")))?;
    let conexoes = banco.conexoes(documento).map_err(|e| bad("anacad.conexao.topologia", e.to_string()))?;
    let equipamentos = banco.equipamentos(documento, 100_000).map_err(|e| bad("anacad.conexao.topologia", e.to_string()))?;
    let mut pai: BTreeMap<String, String> = BTreeMap::new();
    for e in &equipamentos {
        if let Some(h) = &e.handle {
            pai.entry(h.clone()).or_insert_with(|| h.clone());
        }
    }
    fn find(pai: &mut BTreeMap<String, String>, chave: &str) -> String {
        let atual = pai.get(chave).cloned().unwrap_or_else(|| chave.to_string());
        if atual == chave {
            return atual;
        }
        let raiz = find(pai, &atual);
        pai.insert(chave.to_string(), raiz.clone());
        raiz
    }
    fn unir(pai: &mut BTreeMap<String, String>, a: &str, b: &str) {
        let (ra, rb) = (find(pai, a), find(pai, b));
        if ra != rb {
            pai.insert(ra, rb);
        }
    }
    let mut arestas = Vec::new();
    let mut ignoradas = 0usize;
    for c in &conexoes {
        let de = c.de_handle.clone().unwrap_or_default();
        let para = c.para_handle.clone().unwrap_or_default();
        if de.is_empty() || para.is_empty() {
            ignoradas += 1;
            continue;
        }
        let mut ok = true;
        if apenas_validas {
            let de_ok = Handle::parse_hex(&de).and_then(|h| terminal_da_entidade(s, h, c.de_terminal.as_deref()).ok()).is_some();
            let para_ok = Handle::parse_hex(&para).and_then(|h| terminal_da_entidade(s, h, c.para_terminal.as_deref()).ok()).is_some();
            ok = de_ok && para_ok;
            if ok && let Some(linha) = c.handle.as_deref().and_then(Handle::parse_hex).and_then(|h| endpoints_da_linha(s, h)) {
                let dp = Handle::parse_hex(&de).and_then(|h| terminal_da_entidade(s, h, c.de_terminal.as_deref()).ok()).map(|(_, _, x, y)| (x, y));
                let pp =
                    Handle::parse_hex(&para).and_then(|h| terminal_da_entidade(s, h, c.para_terminal.as_deref()).ok()).map(|(_, _, x, y)| (x, y));
                if let (Some(dp), Some(pp)) = (dp, pp) {
                    ok = distancia_ponto_linha(dp, linha.0, linha.1) <= tol && distancia_ponto_linha(pp, linha.0, linha.1) <= tol;
                }
            }
        }
        if !ok {
            ignoradas += 1;
            continue;
        }
        pai.entry(de.clone()).or_insert_with(|| de.clone());
        pai.entry(para.clone()).or_insert_with(|| para.clone());
        unir(&mut pai, &de, &para);
        arestas.push(json!({"id": c.id, "de": de, "para": para}));
    }
    let mut grupos: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let chaves: Vec<String> = pai.keys().cloned().collect();
    for chave in &chaves {
        let raiz = find(&mut pai, chave);
        grupos.entry(raiz).or_default().push(chave.clone());
    }
    let componentes: Vec<Value> = grupos
        .values()
        .map(|membros| {
            let mut ordenados = membros.clone();
            ordenados.sort();
            json!({"nos": ordenados, "tamanho": ordenados.len()})
        })
        .collect();
    Ok(json!({
        "conexoes": conexoes.len(),
        "equipamentos": equipamentos.len(),
        "arestas": arestas,
        "nos": chaves.len(),
        "componentes": componentes,
        "ignoradas": ignoradas,
    }))
}

pub(super) fn run_equipamento_mover(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.mover", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.equipamento.mover", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.equipamento.mover", "handle invalido"))?;
    let dx = f64_or(p, "dx", 0.0);
    let dy = f64_or(p, "dy", 0.0);
    let (x, y) = {
        let ent = s.doc()?.entity(handle).ok_or_else(|| bad("anacad.equipamento.mover", "entidade nao encontrada"))?;
        let EntityKind::Insert(ins) = &ent.kind else { return Err(bad("anacad.equipamento.mover", "nao e um bloco")) };
        (ins.insert.x + dx, ins.insert.y + dy)
    };
    s.doc_mut()?.modify_entity(handle, |e| {
        let EntityKind::Insert(ins) = &mut e.kind else { return };
        ins.insert.x += dx;
        ins.insert.y += dy;
        for a in &mut ins.attribs {
            a.text.insert.x += dx;
            a.text.insert.y += dy;
            if let Some(ap) = &mut a.text.align_pt {
                ap.x += dx;
                ap.y += dy;
            }
        }
    })?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.mover", format!("{db}: {e}")))?;
    banco.atualizar_equipamento_nome_pos(texto, None, x, y).map_err(|e| bad("anacad.equipamento.mover", e.to_string()))?;
    Ok(json!({"handle": handle.hex(), "x": x, "y": y}))
}

pub(super) fn run_equipamento_transformar(s: &mut Session, p: &Value) -> Result<Value> {
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.equipamento.transformar", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.equipamento.transformar", "handle invalido"))?;
    let (old_rot, old_scale, insert) = {
        let ent = s.doc()?.entity(handle).ok_or_else(|| bad("anacad.equipamento.transformar", "entidade nao encontrada"))?;
        let EntityKind::Insert(ins) = &ent.kind else { return Err(bad("anacad.equipamento.transformar", "nao e um bloco")) };
        (ins.rotation, ins.scale.x, (ins.insert.x, ins.insert.y))
    };
    let nova_rot = p.get("rotacao").and_then(Value::as_f64).map(f64::to_radians).unwrap_or(old_rot);
    let nova_escala = p.get("escala").and_then(Value::as_f64).filter(|v| *v > 0.0).unwrap_or(old_scale);
    let delta = nova_rot - old_rot;
    let razao = nova_escala / old_scale;
    s.doc_mut()?.modify_entity(handle, |e| {
        let EntityKind::Insert(ins) = &mut e.kind else { return };
        ins.rotation = nova_rot;
        ins.scale.x = nova_escala;
        ins.scale.y = nova_escala;
        let (cx, cy) = (ins.insert.x, ins.insert.y);
        let ajustar = |p: &mut Vec3| {
            let (mut x, mut y) = (p.x - cx, p.y - cy);
            let (s, c) = delta.sin_cos();
            let (rx, ry) = (x * c - y * s, x * s + y * c);
            x = rx * razao;
            y = ry * razao;
            p.x = cx + x;
            p.y = cy + y;
        };
        for a in &mut ins.attribs {
            ajustar(&mut a.text.insert);
            if let Some(ap) = &mut a.text.align_pt {
                ajustar(ap);
            }
            a.text.rotation += delta;
            a.text.height *= razao;
        }
    })?;
    Ok(json!({"handle": handle.hex(), "rotacao": nova_rot.to_degrees(), "escala": nova_escala, "insert": insert}))
}

pub(super) fn run_equipamento_sincronizar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.sincronizar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.sincronizar", format!("{db}: {e}")))?;
    let equipamentos = banco.equipamentos(documento, 100_000).map_err(|e| bad("anacad.equipamento.sincronizar", e.to_string()))?;
    let mut atualizados = 0usize;
    let mut ausentes = 0usize;
    for e in &equipamentos {
        let Some(handle_txt) = &e.handle else {
            ausentes += 1;
            continue;
        };
        let Some(handle) = Handle::parse_hex(handle_txt) else {
            ausentes += 1;
            continue;
        };
        let pos = s.doc()?.entity(handle).and_then(|ent| match &ent.kind {
            EntityKind::Insert(ins) => Some((ins.insert.x, ins.insert.y)),
            _ => None,
        });
        match pos {
            Some((x, y)) => {
                banco.atualizar_equipamento_nome_pos(handle_txt, None, x, y).map_err(|err| bad("anacad.equipamento.sincronizar", err.to_string()))?;
                atualizados += 1;
            }
            None => ausentes += 1,
        }
    }
    Ok(json!({"atualizados": atualizados, "ausentes": ausentes, "total": equipamentos.len()}))
}

pub(super) fn run_conexao_comparar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.conexao.comparar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.conexao.comparar", format!("{db}: {e}")))?;
    let equipamentos = banco.equipamentos(documento, 100_000).map_err(|e| bad("anacad.conexao.comparar", e.to_string()))?;
    let conexoes = banco.conexoes(documento).map_err(|e| bad("anacad.conexao.comparar", e.to_string()))?;

    // Banco: equipamentos que compartilham o mesmo código de barra (de/para/barra).
    let mut por_barra: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    let mut handles: BTreeSet<String> = BTreeSet::new();
    for e in &equipamentos {
        let Some(h) = &e.handle else { continue };
        handles.insert(h.clone());
        let Some(dados) = &e.dados else { continue };
        let Ok(d) = serde_json::from_str::<Value>(dados) else { continue };
        for chave in ["de", "para", "barra"] {
            if let Some(codigo) = d.get(chave).and_then(Value::as_i64) {
                por_barra.entry(codigo).or_default().push(h.clone());
            }
        }
    }
    let mut pares_banco: BTreeSet<(String, String)> = BTreeSet::new();
    for lista in por_barra.values() {
        let mut ordenada = lista.clone();
        ordenada.sort();
        ordenada.dedup();
        for i in 0..ordenada.len() {
            for j in (i + 1)..ordenada.len() {
                pares_banco.insert((ordenada[i].clone(), ordenada[j].clone()));
            }
        }
    }

    // CAD: componentes pelas conexões.
    let mut pai: BTreeMap<String, String> = BTreeMap::new();
    for h in &handles {
        pai.insert(h.clone(), h.clone());
    }
    fn find(pai: &mut BTreeMap<String, String>, chave: &str) -> String {
        let atual = pai.get(chave).cloned().unwrap_or_else(|| chave.to_string());
        if atual == chave {
            return atual;
        }
        let raiz = find(pai, &atual);
        pai.insert(chave.to_string(), raiz.clone());
        raiz
    }
    for c in &conexoes {
        let (Some(de), Some(para)) = (c.de_handle.clone(), c.para_handle.clone()) else { continue };
        if !pai.contains_key(&de) || !pai.contains_key(&para) {
            continue;
        }
        let (ra, rb) = (find(&mut pai, &de), find(&mut pai, &para));
        if ra != rb {
            pai.insert(ra, rb);
        }
    }
    let mut grupos: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for h in handles.iter() {
        let raiz = find(&mut pai, h);
        grupos.entry(raiz).or_default().push(h.clone());
    }
    let mut pares_cad: BTreeSet<(String, String)> = BTreeSet::new();
    for lista in grupos.values() {
        let mut ordenada = lista.clone();
        ordenada.sort();
        for i in 0..ordenada.len() {
            for j in (i + 1)..ordenada.len() {
                pares_cad.insert((ordenada[i].clone(), ordenada[j].clone()));
            }
        }
    }
    let faltando: Vec<Value> = pares_banco.difference(&pares_cad).map(|(a, b)| json!({"de": a, "para": b})).collect();
    let extras: Vec<Value> = pares_cad.difference(&pares_banco).map(|(a, b)| json!({"de": a, "para": b})).collect();
    Ok(json!({
        "cad": pares_cad.len(),
        "banco": pares_banco.len(),
        "faltando_em_cad": faltando,
        "extras_cad": extras,
        "equipamentos": equipamentos.len(),
    }))
}

pub(super) fn run_equipamento_sincronizar_conexoes(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.sincronizar_conexoes", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.sincronizar_conexoes", format!("{db}: {e}")))?;
    let equipamentos = banco.equipamentos(documento, 100_000).map_err(|e| bad("anacad.equipamento.sincronizar_conexoes", e.to_string()))?;
    let conexoes = banco.conexoes(documento).map_err(|e| bad("anacad.equipamento.sincronizar_conexoes", e.to_string()))?;
    let mut dados_por_handle: BTreeMap<String, Value> = BTreeMap::new();
    let mut tipos: BTreeMap<String, String> = BTreeMap::new();
    for e in &equipamentos {
        if let Some(h) = &e.handle {
            tipos.insert(h.clone(), e.tipo.clone());
            let dados = e.dados.as_deref().and_then(|t| serde_json::from_str::<Value>(t).ok()).unwrap_or_else(|| json!({}));
            dados_por_handle.insert(h.clone(), dados);
        }
    }
    let mut alterados: BTreeSet<String> = BTreeSet::new();
    let mut ignoradas = 0usize;
    for c in &conexoes {
        let pares = [
            (c.de_handle.clone(), c.de_terminal.clone(), c.para_handle.clone()),
            (c.para_handle.clone(), c.para_terminal.clone(), c.de_handle.clone()),
        ];
        for (eq_handle, eq_terminal, bar_handle) in pares {
            let (Some(eq_handle), Some(bar_handle)) = (eq_handle, bar_handle) else { continue };
            if !dados_por_handle.contains_key(&eq_handle) {
                continue;
            }
            let Some(bar_code) = Handle::parse_hex(&bar_handle).and_then(|h| codigo_da_barra(s, h)) else { continue };
            let Some(terminal) = eq_terminal else {
                ignoradas += 1;
                continue;
            };
            let Some(dados) = dados_por_handle.get_mut(&eq_handle) else { continue };
            if !dados.is_object() {
                *dados = json!({});
            }
            let chave = if terminal.eq_ignore_ascii_case("T2") { "para" } else { "de" };
            dados[chave] = json!(bar_code);
            alterados.insert(eq_handle);
        }
    }
    let mut atualizados = 0usize;
    for handle in alterados {
        let Some(dados) = dados_por_handle.get(&handle) else { continue };
        let Some(tipo) = tipos.get(&handle) else { continue };
        let texto = dados.to_string();
        banco.atualizar_dados_equipamento(&handle, &texto).map_err(|e| bad("anacad.equipamento.sincronizar_conexoes", e.to_string()))?;
        banco.sincronizar_equipamento(documento, tipo, dados).map_err(|e| bad("anacad.equipamento.sincronizar_conexoes", e.to_string()))?;
        atualizados += 1;
    }
    Ok(json!({"atualizados": atualizados, "ignoradas": ignoradas}))
}

pub(super) fn run_banco_criar(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.criar", "`arquivo` is required"))?;
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.criar", format!("{arquivo}: {e}")))?;
    resumo_banco(&banco, arquivo)
}

pub(super) fn run_banco_abrir(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.abrir", "`arquivo` is required"))?;
    if !std::path::Path::new(arquivo).exists() {
        return Err(bad("anacad.banco.abrir", format!("banco nao encontrado: {arquivo}")));
    }
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.abrir", format!("{arquivo}: {e}")))?;
    resumo_banco(&banco, arquivo)
}

pub(super) fn run_documento_listar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.documento.listar", "`db` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.documento.listar", format!("{db}: {e}")))?;
    let documentos: Vec<Value> = banco
        .documentos()
        .map_err(|e| bad("anacad.documento.listar", e.to_string()))?
        .into_iter()
        .map(|d| json!({"id": d.id, "nome": d.nome, "pagina": d.pagina}))
        .collect();
    Ok(json!({ "documentos": documentos }))
}

pub(super) fn run_documento_criar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.documento.criar", "`db` is required"))?;
    let nome = str_param(p, "nome").ok_or_else(|| bad("anacad.documento.criar", "`nome` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.documento.criar", format!("{db}: {e}")))?;
    let projeto = match p.get("projeto").and_then(Value::as_i64) {
        Some(id) => id,
        None => match banco.projetos().map_err(|e| bad("anacad.documento.criar", e.to_string()))?.first() {
            Some((id, _)) => *id,
            None => banco.criar_projeto("AnaCAD").map_err(|e| bad("anacad.documento.criar", e.to_string()))?,
        },
    };
    let pagina = str_param(p, "pagina");
    let documento = banco.inserir_documento(projeto, nome, None, pagina, 0).map_err(|e| bad("anacad.documento.criar", e.to_string()))?;
    let documentos = banco.documentos().map_err(|e| bad("anacad.documento.criar", e.to_string()))?.len();
    Ok(json!({ "documento": documento, "projeto": projeto, "nome": nome, "documentos": documentos }))
}

pub(super) fn run_exportar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.exportar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let saida = str_param(p, "saida").ok_or_else(|| bad("anacad.exportar", "`saida` is required"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.exportar", format!("{db}: {e}")))?;
    let tela = banco.exportar_screen(documento).map_err(|e| bad("anacad.exportar", e.to_string()))?;
    let texto = tela.escrever_texto();
    let registros = anacad_core::ler_registros(texto.as_bytes()).len();
    escrever_arquivo(saida, &anacad_core::cifrar(texto.as_bytes()))?;
    Ok(json!({ "saida": saida, "registros": registros, "mensagem": "tela exportada" }))
}
