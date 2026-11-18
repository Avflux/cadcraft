//! Comandos de desenho do unifilar: criar barra, linha e legenda; mover e apagar.

use super::*;
use super::esquematico::*;

pub(in crate::cmd::anacad) fn run_anacad(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_mover(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_apagar(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.apagar", "`db` is required"))?;
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.apagar", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.apagar", "handle invalido"))?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.apagar", format!("{db}: {e}")))?;
    let removidos = banco.apagar_por_handle(texto).map_err(|e| bad("anacad.apagar", e.to_string()))?;
    // Os textos de identificação pertencem à barra: saem junto com ela.
    remover_textos_da_barra(s, handle)?;
    // O círculo interno do donut também pertence à barra.
    remover_donut_interno(s, handle);
    s.doc_mut()?.remove_entity(handle);
    Ok(json!({ "handle": texto, "removidos": removidos }))
}

pub(in crate::cmd::anacad) fn run_criar_legenda(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_mover_legenda(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_criar_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.criar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let nome = str_param(p, "nome");
    let comprimento = f64_or(p, "comprimento", 1.0).max(0.01);
    let x2 = p.get("x2").and_then(Value::as_f64).unwrap_or(x + comprimento);
    let y2 = p.get("y2").and_then(Value::as_f64).unwrap_or(y);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.criar", format!("{db}: {e}")))?;
    // Um `.db` novo não tem documento e a UI manda `documento: 1`; resolver aqui
    // evita o `FOREIGN KEY constraint failed` da primeira barra.
    let documento = banco.garantir_documento(documento).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    // O `.db` e o documento ficam no estado da sessão: a cascata do apagar
    // (`after_command`) roda depois do comando, sem parâmetros.
    set_banco(s, db, documento);
    let codigo = match p.get("codigo").and_then(Value::as_i64) {
        Some(c) if c > 0 => c,
        _ => banco.proximo_codigo_barra(documento).map_err(|e| bad("anacad.barra.criar", e.to_string()))?,
    };
    let centro = ((x + x2) / 2.0, (y + y2) / 2.0);
    let id = banco.criar_barra(documento, codigo, nome, centro.0, centro.1).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    // Base de tensão (`VBase`) vinda da janela: é ela que escolhe a cor do
    // grupo base de tensão (`G` do `.lst`), como no ANAFAS.
    let vbase = p.get("vbase").or_else(|| p.get("tensao")).and_then(Value::as_f64);
    if let Some(vbase) = vbase {
        banco.atualizar_barra_vbase(Some(documento), codigo, vbase).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    }
    let vbase_txt = vbase.map(|v| v.to_string()).unwrap_or_default();

    // Barra = LWPOLYLINE de 2 vértices (início/fim), com a espessura do projeto
    // (o grupo Texto e Traço de Barras e Linhas do CFG).
    let lar = lar_da_barra(s);
    let lar_txt = lar.to_string();
    let poly = LwPolyline {
        vertices: vec![PolyVertex { p: Vec2::new(x, y), ..PolyVertex::default() }, PolyVertex { p: Vec2::new(x2, y2), ..PolyVertex::default() }],
        closed: false,
        const_width: lar,
        elevation: 0.0,
        plinegen: false,
    };
    let handle = s.add_entity(EntityKind::LwPolyline(poly))?;
    // A barra entra na cor do grupo base de tensão dela.
    if let Some(cor) = vbase.and_then(|tensao| cor_da_tensao(s, tensao)) {
        colorir(s, handle, cor)?;
    }
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
            ("largura", lar_txt.as_str()),
            ("vbase", &vbase_txt),
        ],
    )?;
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.criar", e.to_string()))?;
    // Textos de identificação da barra, ligados a ela pelo handle.
    let textos = sincronizar_textos_barra(s, handle, codigo, nome, Vec2::new(x, y), Vec2::new(x2, y2))?;
    Ok(json!({
        "barra": id,
        "codigo": codigo,
        "handle": handle.hex(),
        "x1": x, "y1": y, "x2": x2, "y2": y2,
        "largura": lar,
        "tipo": "LWPOLYLINE",
        "textos": textos,
    }))
}

pub(in crate::cmd::anacad) fn run_criar_linha(s: &mut Session, p: &Value) -> Result<Value> {
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
    let documento = banco.garantir_documento(documento).map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    set_banco(s, db, documento);
    // De/Para: os códigos vêm do comando (janela, `0` = vazio) ou das barras
    // desenhadas sob as extremidades do traço — no Eletron o `verificaDispConex`
    // faz o mesmo: olha o que está no ponto da extremidade da conexão e guarda
    // o vínculo (potencial/handle) no XData.
    let mut de = de.filter(|codigo| *codigo != 0);
    let mut para = para.filter(|codigo| *codigo != 0);
    let explicitos = de.is_some() && para.is_some();
    if !explicitos {
        let doc = s.doc()?;
        de = de.or_else(|| codigo_da_barra_no_ponto(doc, x1, y1));
        para = para.or_else(|| codigo_da_barra_no_ponto(doc, x2, y2));
    }
    if let Some(codigo) = de {
        match banco.barra_por_codigo(documento, codigo).map_err(|e| bad("anacad.linha.criar", e.to_string()))? {
            Some(barra) => barra_de = Some(barra),
            // Código digitado na janela precisa existir; o que veio do desenho
            // pode ser de outro documento/banco e aí a linha fica sem ele.
            None if explicitos => return Err(bad("anacad.linha.criar", format!("barra {codigo} nao existe no documento"))),
            None => de = None,
        }
    }
    if let Some(codigo) = para {
        match banco.barra_por_codigo(documento, codigo).map_err(|e| bad("anacad.linha.criar", e.to_string()))? {
            Some(barra) => barra_para = Some(barra),
            None if explicitos => return Err(bad("anacad.linha.criar", format!("barra {codigo} nao existe no documento"))),
            None => para = None,
        }
    }
    // Com os códigos no comando o traço é puxado para a posição da barra no
    // banco; vindo do desenho, os pontos escolhidos são os que valem.
    if explicitos && let (Some(b1), Some(b2)) = (barra_de.as_ref(), barra_para.as_ref()) {
        x1 = b1.x.unwrap_or(x1);
        y1 = b1.y.unwrap_or(y1);
        x2 = b2.x.unwrap_or(x2);
        y2 = b2.y.unwrap_or(y2);
    }
    let id = banco
        .criar_linha(documento, barra_de.as_ref().map(|b| b.id), barra_para.as_ref().map(|b| b.id))
        .map_err(|e| bad("anacad.linha.criar", e.to_string()))?;
    let lar = lar_da_linha(s);
    let poly = LwPolyline {
        vertices: vec![PolyVertex { p: Vec2::new(x1, y1), ..PolyVertex::default() }, PolyVertex { p: Vec2::new(x2, y2), ..PolyVertex::default() }],
        closed: false,
        const_width: lar,
        elevation: 0.0,
        plinegen: false,
    };
    let handle = s.add_entity(EntityKind::LwPolyline(poly))?;
    // A linha entra na cor do grupo base de tensão da barra de origem.
    let vbase = barra_de.as_ref().and_then(|barra| barra.vbase);
    if let Some(cor) = vbase.and_then(|tensao| cor_da_tensao(s, tensao)) {
        colorir(s, handle, cor)?;
    }
    let mut campos = vec![
        ("id", id.to_string()),
        ("x1", x1.to_string()),
        ("y1", y1.to_string()),
        ("x2", x2.to_string()),
        ("y2", y2.to_string()),
        ("largura", lar.to_string()),
        ("vbase", vbase.map(|v| v.to_string()).unwrap_or_default()),
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
    // O circuito nasce com 1 (o `Nsg` que `Banco::criar_linha` grava) e é
    // impresso junto do traço; a janela Dados de Linha (164) atualiza o número
    // no Alterar.
    let textos = sincronizar_texto_linha(s, handle, Some(1), Vec2::new(x1, y1), Vec2::new(x2, y2))?;
    Ok(json!({
        "linha": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": lar,
        "tipo": "LWPOLYLINE",
        "de": de,
        "para": para,
        "textos": textos,
    }))
}
