//! Desenho e edicao do diagrama unifilar: barras, linhas, equipamentos, conexoes, legenda.

use super::*;
use cadcraft_doc::color::Color;

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
    // Os textos de identificação pertencem à barra: saem junto com ela.
    remover_textos_da_barra(s, handle)?;
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
    if explicitos
        && let (Some(b1), Some(b2)) = (barra_de.as_ref(), barra_para.as_ref())
    {
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

/// App name do XData dos textos de identificação da barra.
pub(super) const APP_TEXTO_BARRA: &str = "ANACAD_BARRA_TEXTO";
/// App name do XData do texto com o número do circuito, desenhado junto do
/// traço da linha.
pub(super) const APP_TEXTO_LINHA: &str = "ANACAD_LINHA_TEXTO";
/// App name do XData das barras desenhadas (LWPOLYLINE).
pub(super) const APP_BARRA: &str = "ANACAD_BARRA";
/// App name do XData das linhas desenhadas (LWPOLYLINE).
pub(super) const APP_LINHA: &str = "ANACAD_LINHA";

/// Textos ligados a um elemento pelo XData (`app` = app name do texto):
/// `(campo, handle)`. O vínculo é o handle do elemento gravado no texto, com o
/// `campo` dizendo qual informação ele carrega.
pub(super) fn textos_ligados(doc: &cadcraft_doc::Drawing, app: &str, alvo: Handle) -> Vec<(String, Handle)> {
    let alvo = alvo.hex();
    doc.model
        .iter()
        .filter_map(|e| {
            if !matches!(&e.kind, EntityKind::Text(_)) {
                return None;
            }
            if !matches!(e.common.xdata.first(), Some(t) if t.code == 1001 && t.value == app) {
                return None;
            }
            let pares: Vec<&str> = e.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
            let campo = pares.chunks(2).find(|p| p.len() == 2 && p[0] == "campo").map(|p| p[1].to_string());
            let dono = pares.chunks(2).find(|p| p.len() == 2 && p[0] == "handle").map(|p| p[1]);
            if dono != Some(alvo.as_str()) {
                return None;
            }
            Some((campo?, e.handle))
        })
        .collect()
}

/// Textos de identificação ligados a uma barra: `(campo, handle)`.
///
/// O vínculo é o handle da barra gravado no XData `ANACAD_BARRA_TEXTO`
/// (`campo` = identificador | numero): o texto deixa de ser um TEXT solto e
/// passa a pertencer à barra, como a identificação que o ANAFAS desenha junto
/// ao elemento.
fn textos_da_barra(doc: &cadcraft_doc::Drawing, barra: Handle) -> Vec<(String, Handle)> {
    textos_ligados(doc, APP_TEXTO_BARRA, barra)
}

/// Posição e alinhamento do texto em relação à barra. A âncora é a geometria
/// dela, **não o ponto em que o traço começou**: numa horizontal o texto fica no
/// extremo esquerdo, alinhado à esquerda e acima da linha (pilha no normal, com
/// `y` positivo); numa vertical o TEXT não gira, então fica centrado no eixo da
/// barra e empilhado acima do extremo superior, com o identificador em cima do
/// número. Assim o lugar do texto não muda conforme o sentido em que a barra foi
/// desenhada ou editada.
fn posicao_texto_barra(campo: &str, a: Vec2, b: Vec2, altura: f64) -> (Vec3, bool) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let comprimento = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if comprimento > 1e-9 { (dx / comprimento, dy / comprimento) } else { (1.0, 0.0) };
    let (mut nx, mut ny) = (-uy, ux);
    if ny < -1e-9 {
        nx = -nx;
        ny = -ny;
    }
    // Distância de cada texto ao início: identificador em cima do número, como
    // no ANAFAS, com meia altura entre eles.
    let passo = match campo {
        "identificador" => altura * 1.8,
        _ => altura * 0.5,
    };
    if ux.abs() < uy.abs() {
        let topo = a.y.max(b.y);
        (Vec3::new(a.x, topo + passo, 0.0), true)
    } else {
        let esquerda = a.x.min(b.x);
        (Vec3::new(esquerda + nx * passo, a.y + ny * passo, 0.0), false)
    }
}

/// Cria, atualiza ou remove os textos de identificação da barra, sempre pelo
/// vínculo com o handle dela. Devolve os handles dos textos que ficaram no
/// desenho.
///
/// A base de tensão não vira texto: no ANAFAS ela é a opção
/// `mostrar_base_tensao` da janela 418, desligada por padrão.
fn sincronizar_textos_barra(
    s: &mut Session,
    barra: Handle,
    codigo: i64,
    nome: Option<&str>,
    a: Vec2,
    b: Vec2,
) -> Result<Vec<String>> {
    // Altura e cor do texto de identificação, do estilo da barra no projeto.
    let cfg = cfg_do_desenho(s.doc()?);
    let (altura, cor) = (cfg.barra_texto, cfg.barra_cor);
    let existentes = textos_da_barra(s.doc()?, barra);
    let nome = nome.map(str::trim).filter(|n| !n.is_empty());
    let campos: [(&str, Option<String>); 2] = [
        ("identificador", nome.map(str::to_string)),
        ("numero", Some(codigo.to_string())),
    ];
    let mut handles = Vec::new();
    for (campo, valor) in campos {
        let atual = existentes.iter().find(|(c, _)| c == campo).map(|(_, h)| *h);
        match (valor, atual) {
            (Some(texto), Some(handle)) => {
                let (pos, centrado) = posicao_texto_barra(campo, a, b, altura);
                s.doc_mut()?.modify_entity(handle, |e| {
                    if let EntityKind::Text(t) = &mut e.kind {
                        t.value = texto;
                        t.insert = pos;
                        t.align_pt = centrado.then_some(pos);
                        t.halign = if centrado { HAlign::Center } else { HAlign::Left };
                        t.height = altura;
                    }
                    e.common.color = cor;
                })?;
                handles.push(handle.hex());
            }
            (Some(texto), None) => {
                let (pos, centrado) = posicao_texto_barra(campo, a, b, altura);
                let kind = EntityKind::Text(Text {
                    insert: pos,
                    align_pt: centrado.then_some(pos),
                    height: altura,
                    value: texto,
                    rotation: 0.0,
                    width_factor: 1.0,
                    oblique: 0.0,
                    style: "Standard".into(),
                    halign: if centrado { HAlign::Center } else { HAlign::Left },
                    valign: VAlign::Baseline,
                });
                let handle = s.add_entity(kind)?;
                s.doc_mut()?.modify_entity(handle, |e| e.common.color = cor)?;
                marcar(
                    s,
                    handle,
                    APP_TEXTO_BARRA,
                    &[("handle", &barra.hex()), ("campo", campo), ("codigo", &codigo.to_string())],
                )?;
                handles.push(handle.hex());
            }
            (None, Some(handle)) => {
                s.doc_mut()?.remove_entity(handle);
            }
            (None, None) => {}
        }
    }
    Ok(handles)
}

fn remover_textos_da_barra(s: &mut Session, barra: Handle) -> Result<()> {
    let handles: Vec<Handle> = textos_da_barra(s.doc()?, barra).into_iter().map(|(_, h)| h).collect();
    for handle in handles {
        s.doc_mut()?.remove_entity(handle);
    }
    Ok(())
}

/// Posição do número do circuito: no meio do traço, deslocado para o lado de
/// cima (o `y` cresce para cima), com folga de meia altura de texto.
fn extremos_da_linha(e: &cadcraft_doc::Entity) -> Option<(Vec2, Vec2)> {
    let EntityKind::LwPolyline(poly) = &e.kind else { return None };
    let a = poly.vertices.first()?.p;
    let b = poly.vertices.last().unwrap_or(poly.vertices.first()?).p;
    Some((a, b))
}

fn posicao_texto_linha(a: Vec2, b: Vec2, altura: f64) -> Vec3 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let comprimento = (dx * dx + dy * dy).sqrt();
    let (mut nx, mut ny) = if comprimento > 1e-9 { (-dy / comprimento, dx / comprimento) } else { (0.0, 1.0) };
    if ny < -1e-9 {
        nx = -nx;
        ny = -ny;
    }
    // O texto assenta **por cima** do traço (`VAlign::Bottom`), centrado no
    // meio dele: a folga é só para não encostar na linha, não o afastamento de
    // meia altura que deixava o número solto longe do traço.
    let folga = altura * 0.15;
    Vec3::new((a.x + b.x) / 2.0 + nx * folga, (a.y + b.y) / 2.0 + ny * folga, 0.0)
}

/// Cria, atualiza ou remove o texto com o número do circuito junto do traço da
/// linha, sempre pelo vínculo com o handle dela (`ANACAD_LINHA_TEXTO`): o
/// número acompanha a linha como a identificação acompanha a barra. Sem
/// circuito (dados elétricos removidos), o texto sai do desenho.
fn sincronizar_texto_linha(
    s: &mut Session,
    linha: Handle,
    circuito: Option<i64>,
    a: Vec2,
    b: Vec2,
) -> Result<Vec<String>> {
    let cfg = cfg_do_desenho(s.doc()?);
    let (altura, cor) = (cfg.linha_texto, cfg.linha_cor);
    let atual = textos_ligados(s.doc()?, APP_TEXTO_LINHA, linha)
        .into_iter()
        .find(|(campo, _)| campo == "circuito")
        .map(|(_, handle)| handle);
    let pos = posicao_texto_linha(a, b, altura);
    match (circuito, atual) {
        (Some(numero), Some(handle)) => {
            s.doc_mut()?.modify_entity(handle, |e| {
                if let EntityKind::Text(t) = &mut e.kind {
                    t.value = numero.to_string();
                    t.insert = pos;
                    t.align_pt = Some(pos);
                    t.halign = HAlign::Center;
                    t.valign = VAlign::Bottom;
                    t.height = altura;
                }
                e.common.color = cor;
            })?;
            Ok(vec![handle.hex()])
        }
        (Some(numero), None) => {
            let kind = EntityKind::Text(Text {
                insert: pos,
                align_pt: Some(pos),
                height: altura,
                value: numero.to_string(),
                rotation: 0.0,
                width_factor: 1.0,
                oblique: 0.0,
                style: "Standard".into(),
                halign: HAlign::Center,
                valign: VAlign::Bottom,
            });
            let handle = s.add_entity(kind)?;
            s.doc_mut()?.modify_entity(handle, |e| e.common.color = cor)?;
            marcar(
                s,
                handle,
                APP_TEXTO_LINHA,
                &[("handle", &linha.hex()), ("campo", "circuito"), ("numero", &numero.to_string())],
            )?;
            Ok(vec![handle.hex()])
        }
        (None, Some(handle)) => {
            s.doc_mut()?.remove_entity(handle);
            Ok(Vec::new())
        }
        (None, None) => Ok(Vec::new()),
    }
}

/// Handles dos textos de identificação de uma barra (`barra = Some`) ou de
/// todas as barras do desenho (`None`).
fn handles_textos(doc: &cadcraft_doc::Drawing, app: &str, alvo: Option<Handle>) -> Vec<Handle> {
    let alvo = alvo.map(|b| b.hex());
    doc.model
        .iter()
        .filter_map(|e| {
            if !matches!(&e.kind, EntityKind::Text(_)) {
                return None;
            }
            if !matches!(e.common.xdata.first(), Some(t) if t.code == 1001 && t.value == app) {
                return None;
            }
            if let Some(alvo) = alvo.as_deref() {
                let pares: Vec<&str> = e.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
                let dono = pares.chunks(2).find(|p| p.len() == 2 && p[0] == "handle").map(|p| p[1]);
                if dono != Some(alvo) {
                    return None;
                }
            }
            Some(e.handle)
        })
        .collect()
}

/// Liga ou desliga os textos de identificação de **todas** as barras
/// (`tipo = "barra"`, o default) ou o número do circuito de **todas** as linhas
/// (`tipo = "linha"`). Sem `visivel`, inverte o estado atual; `consultar: true`
/// só devolve o estado. É o switch da janela CFG, com o mesmo `Visible` por
/// atributo que o Eletron usa (`clsAtributos.OcultaTextosHandle`).
pub(super) fn run_textos_barra(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "anacad.barra.textos";
    let tipo = str_param(p, "tipo").map(str::trim).filter(|t| !t.is_empty()).unwrap_or("barra");
    let app = match tipo {
        "barra" => APP_TEXTO_BARRA,
        "linha" => APP_TEXTO_LINHA,
        outro => return Err(bad(CMD, format!("tipo invalido: {outro}"))),
    };
    let handles = handles_textos(s.doc()?, app, None);
    let todos_visiveis = {
        let doc = s.doc()?;
        handles.iter().all(|h| doc.model.get(*h).is_none_or(|e| e.common.visible))
    };
    if p.get("consultar").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(json!({ "tipo": tipo, "visivel": todos_visiveis, "textos": handles.len() }));
    }
    let visivel = p.get("visivel").and_then(Value::as_bool).unwrap_or(!todos_visiveis);
    for handle in &handles {
        s.doc_mut()?.modify_entity(*handle, |e| e.common.visible = visivel)?;
    }
    Ok(json!({
        "tipo": tipo,
        "visivel": visivel,
        "textos": handles.len(),
    }))
}

pub(super) fn run_editar_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.editar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.editar", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let lar = lar_da_barra(s);
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
        // A barra passa a ter a cor do grupo base de tensão dela.
        if let Some(cor) = cor_da_tensao(s, vbase) {
            colorir(s, handle, cor)?;
        }
    }
    let texto = serde_json::to_string(&dados).unwrap_or_else(|_| "{}".to_string());
    banco.atualizar_dados_barra(&handle.hex(), &texto).map_err(|e| bad("anacad.barra.editar", e.to_string()))?;
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = lar;
        }
    })?;
    // `marcar` regrava o XData inteiro: sem o `vbase` aqui, editar a janela
    // apagava a base de tensão do desenho (o banco ficava com ela) e o
    // recadastro do Sincronizar a perdia.
    let vbase_txt = dados
        .get("vbase")
        .and_then(Value::as_f64)
        .map(|v| v.to_string())
        .or_else(|| s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, "vbase")).map(str::to_string))
        .unwrap_or_default();
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
            ("largura", lar.to_string().as_str()),
            ("vbase", vbase_txt.as_str()),
            ("dados", &texto),
        ],
    )?;
    // A identificação no desenho acompanha a alteração: nome (identificador),
    // número e base de tensão, reposicionados com a barra.
    sincronizar_textos_barra(s, handle, codigo, nome, Vec2::new(x1, y1), Vec2::new(x2, y2))?;
    Ok(json!({ "barra": id, "handle": handle.hex(), "dados": dados }))
}

pub(super) fn run_editar_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.editar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.linha.editar", "`handle` is required"))?;
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let lar = lar_da_linha(s);
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
            p.const_width = lar;
        }
    })?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    // `marcar` regrava o XData inteiro: as barras De/Para e a base de tensão
    // que a janela não mexeu continuam no desenho, senão o recadastro do
    // Sincronizar recria a linha solta (sem `DeBarra`/`ParaBarra`).
    let anterior = |campo: &str| -> Option<String> {
        s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, campo)).map(str::to_string)
    };
    let de_txt = barra_de.as_ref().and_then(|b| b.codigo).map(|c| c.to_string()).or_else(|| anterior("de"));
    let para_txt = barra_para.as_ref().and_then(|b| b.codigo).map(|c| c.to_string()).or_else(|| anterior("para"));
    let vbase_txt = barra_de
        .as_ref()
        .and_then(|b| b.vbase)
        .map(|v| v.to_string())
        .or_else(|| anterior("vbase"))
        .unwrap_or_default();
    let mut campos = vec![
        ("id", id.to_string()),
        ("x1", x1.to_string()),
        ("y1", y1.to_string()),
        ("x2", x2.to_string()),
        ("y2", y2.to_string()),
        ("largura", lar.to_string()),
        ("vbase", vbase_txt),
        ("dados", texto),
    ];
    if let Some(de) = de_txt {
        campos.push(("de", de));
    }
    if let Some(para) = para_txt {
        campos.push(("para", para));
    }
    let refs: Vec<(&str, &str)> = campos.iter().map(|(k, v)| (*k, v.as_str())).collect();
    marcar(s, handle, "ANACAD_LINHA", &refs)?;
    // O número do circuito impresso junto da linha acompanha o que a janela
    // Dados de Linha (164) gravou.
    sincronizar_texto_linha(s, handle, dados.get("numeroCircuito").and_then(Value::as_i64), Vec2::new(x1, y1), Vec2::new(x2, y2))?;
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
    let lar = lar_da_barra(s);
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
            ("largura", lar.to_string().as_str()),
            ("vbase", ""),
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
    let lar = lar_da_linha(s);
    // O banco mantém `DeBarra`/`ParaBarra` ao excluir os dados elétricos; o
    // XData tem de manter também, senão o recadastro perde o vínculo.
    let anterior = |campo: &str| -> Option<String> {
        s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, campo)).map(str::to_string)
    };
    let mut campos = vec![
        ("id", id.to_string()),
        ("x1", x1.to_string()),
        ("y1", y1.to_string()),
        ("x2", x2.to_string()),
        ("y2", y2.to_string()),
        ("largura", lar.to_string()),
        ("vbase", anterior("vbase").unwrap_or_default()),
        ("dados", "{}".to_string()),
    ];
    if let Some(de) = anterior("de") {
        campos.push(("de", de));
    }
    if let Some(para) = anterior("para") {
        campos.push(("para", para));
    }
    let refs: Vec<(&str, &str)> = campos.iter().map(|(k, v)| (*k, v.as_str())).collect();
    marcar(s, handle, "ANACAD_LINHA", &refs)?;
    // Sem dados elétricos não há número de circuito: o texto sai do desenho.
    sincronizar_texto_linha(s, handle, None, Vec2::new(x1, y1), Vec2::new(x2, y2))?;
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
    let lar = lar_da_barra(s);
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = lar;
        }
    })?;
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.atribuir", format!("{db}: {e}")))?;
    let id = banco.criar_barra(documento, codigo, nome, (x1 + x2) / 2.0, (y1 + y2) / 2.0).map_err(|e| bad("anacad.barra.atribuir", e.to_string()))?;
    // A base de tensão que veio do `.lst`/`.ANA` fica no desenho também.
    let vbase_txt = banco
        .barra_por_codigo(documento, codigo)
        .ok()
        .flatten()
        .and_then(|b| b.vbase)
        .map(|v| v.to_string())
        .unwrap_or_default();
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
            ("largura", lar.to_string().as_str()),
            ("vbase", vbase_txt.as_str()),
        ],
    )?;
    banco.definir_handle("Barra", id, &handle.hex()).map_err(|e| bad("anacad.barra.atribuir", e.to_string()))?;
    sincronizar_textos_barra(s, handle, codigo, nome, Vec2::new(x1, y1), Vec2::new(x2, y2))?;
    Ok(json!({
        "barra": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": lar,
    }))
}

pub(super) fn run_atribuir_linha(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.linha.atribuir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.linha.atribuir", "`handle` is required"))?;
    let mut de = p.get("de").and_then(Value::as_i64).filter(|codigo| *codigo != 0);
    let mut para = p.get("para").and_then(Value::as_i64).filter(|codigo| *codigo != 0);
    let (handle, poly) = ler_polilinha(s, handle_txt)?;
    let (x1, y1) = (poly.vertices[0].p.x, poly.vertices[0].p.y);
    let ultimo = poly.vertices.last().copied().unwrap_or(poly.vertices[0]);
    let (x2, y2) = (ultimo.p.x, ultimo.p.y);
    // As barras saem das extremidades da polilinha, como no `criar`.
    if de.is_none() || para.is_none() {
        let doc = s.doc()?;
        de = de.or_else(|| codigo_da_barra_no_ponto(doc, x1, y1));
        para = para.or_else(|| codigo_da_barra_no_ponto(doc, x2, y2));
    }
    let de = de.ok_or_else(|| bad("anacad.linha.atribuir", "nenhuma barra na extremidade De da polilinha"))?;
    let para = para.ok_or_else(|| bad("anacad.linha.atribuir", "nenhuma barra na extremidade Para da polilinha"))?;
    let lar = lar_da_linha(s);
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::LwPolyline(p) = &mut e.kind {
            p.const_width = lar;
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
            ("largura", lar.to_string().as_str()),
        ],
    )?;
    banco.definir_handle("Linha", id, &handle.hex()).map_err(|e| bad("anacad.linha.atribuir", e.to_string()))?;
    sincronizar_texto_linha(s, handle, Some(1), Vec2::new(x1, y1), Vec2::new(x2, y2))?;
    Ok(json!({
        "linha": id,
        "handle": handle.hex(),
        "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        "largura": lar,
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

/// Mão da janela **Equipamentos**: pega o equipamento no desenho e deixa o
/// handle para a UI abrir a janela `Dados de …` dele (o mesmo `equip_handle`
/// que a inserção deixa).
#[derive(Default)]
pub(super) struct EquipSelM {
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
pub(super) struct ArtefatoSelM {
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
pub(super) fn run_artefato_selecionar(_s: &mut Session, _p: &Value) -> Result<Value> {
    Err(bad("anacad.artefato.selecionar", "selecione o artefato no desenho"))
}

/// `anacad.equipamento.selecionar` fora do modo interativo não tem o que fazer:
/// quem escolhe o equipamento é o `EquipSelM` (a UI chama `app.start`).
pub(super) fn run_equipamento_selecionar(_s: &mut Session, _p: &Value) -> Result<Value> {
    Err(bad("anacad.equipamento.selecionar", "selecione o equipamento no desenho"))
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
    // O componente entra na cor do grupo base de tensão da barra em que foi
    // inserido: as entidades do bloco ficam `ByBlock` e a cor vai na referência,
    // como o campo `Cor` do `C` no `.lst` (cada instância com a sua). Blocos
    // personalizados do usuário ficam como estão.
    if bloco.starts_with("ANACAD_EQ_") {
        if let Ok(doc) = s.doc_mut() {
            bloco_byblock(doc, &bloco);
        }
        let tensao = s.doc().ok().and_then(|doc| tensao_no_ponto(doc, x, y, TOLERANCIA_BARRA));
        if let Some(cor) = tensao.and_then(|tensao| cor_da_tensao(s, tensao)) {
            colorir(s, handle, cor)?;
        }
    }
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
    // A derivação nasce junto com o bloco, como no ANAFAS: o equipamento
    // entra ligado à barra ou à linha mais perto, com o vínculo já no
    // `dados` dele. Inserido longe de uma barra ou linha, fica sem ligação.
    let mut dados = json!({});
    let ligacao = match ligar_equipamento(s, &banco, documento, handle, Vec2::new(x, y), &mut dados) {
        Ok(ligacao) => ligacao,
        Err(e) => {
            // A ligação não pode custar o bloco: o `EquipM` pede a barra ou a
            // linha no segundo clique.
            s.echo(format!("ligação automática não saiu: {e}"));
            None
        }
    };
    let mut saida = equipamento_json(id, nome_tipo, codigo, &handle.hex(), x, y, &bloco, &atributos);
    if let Some(info) = ligacao {
        run_equipamento_editar(
            s,
            &json!({ "db": db, "documento": documento, "handle": handle.hex(), "dados": dados }),
        )?;
        if let (Some(destino), Some(fonte)) = (saida.as_object_mut(), info.as_object()) {
            for (k, v) in fonte {
                destino.insert(k.clone(), v.clone());
            }
        }
    }
    s.set_selection(vec![handle]);
    Ok(saida)
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

/// Terminal do equipamento onde o traço fecha: a posição do atributo `T1` (o
/// mesmo casamento por posição do Eletron) e, sem ele, o ponto de inserção.
fn terminal_do_equipamento(s: &Session, handle: Handle) -> Result<(&'static str, Vec2)> {
    let ent = s.doc()?.entity(handle).ok_or_else(|| bad("anacad.equipamento.inserir", "equipamento nao encontrado"))?;
    let EntityKind::Insert(ins) = &ent.kind else {
        return Err(bad("anacad.equipamento.inserir", "o equipamento nao e um bloco"));
    };
    Ok(match ins.attribs.iter().find(|a| a.tag.eq_ignore_ascii_case("T1")) {
        Some(a) => ("T1", Vec2::new(a.text.insert.x, a.text.insert.y)),
        None => ("T1", Vec2::new(ins.insert.x, ins.insert.y)),
    })
}

/// Barra ou linha mais perto de um ponto: é ela que recebe a derivação do
/// equipamento, como o `verificaDispConex` do Eletron, que olha o que está sob
/// a extremidade da conexão. `tolerancia` é a folga em unidades do desenho.
fn alvo_perto(s: &Session, ponto: Vec2, tolerancia: f64) -> Option<Handle> {
    let doc = s.doc().ok()?;
    doc.model
        .iter()
        .filter(|e| matches!(app_xdata(e), Some(APP_BARRA) | Some(APP_LINHA)))
        .filter_map(|e| {
            let EntityKind::LwPolyline(poly) = &e.kind else { return None };
            let mut distancia = f64::INFINITY;
            let mut anterior: Option<Vec2> = None;
            for v in &poly.vertices {
                if let Some(a) = anterior {
                    let d = distancia_ao_segmento((ponto.x, ponto.y), (a.x, a.y), (v.p.x, v.p.y));
                    if d < distancia {
                        distancia = d;
                    }
                }
                anterior = Some(v.p);
            }
            (distancia <= tolerancia).then_some((distancia, e.handle))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, handle)| handle)
}

/// Ponto da barra ou da linha onde o traço encosta: a projeção do terminal do
/// equipamento no traço, então a derivação sai do meio da barra e não só da
/// ponta. O terminal (`INICIO`/`FIM`, os nomes de `terminal_da_entidade`) é o
/// da ponta mais perto.
fn ponto_do_alvo(s: &Session, alvo: Handle, perto: Vec2) -> Option<(&'static str, Vec2)> {
    let ent = s.doc().ok()?.entity(alvo)?;
    let EntityKind::LwPolyline(poly) = &ent.kind else { return None };
    let mut ponto = poly.vertices.first()?.p;
    let mut distancia = f64::INFINITY;
    let mut anterior: Option<Vec2> = None;
    for v in &poly.vertices {
        if let Some(a) = anterior {
            let (dx, dy) = (v.p.x - a.x, v.p.y - a.y);
            let comprimento = dx * dx + dy * dy;
            let t = if comprimento > 0.0 {
                (((perto.x - a.x) * dx + (perto.y - a.y) * dy) / comprimento).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let p = Vec2::new(a.x + t * dx, a.y + t * dy);
            let d = (p.x - perto.x).powi(2) + (p.y - perto.y).powi(2);
            if d < distancia {
                distancia = d;
                ponto = p;
            }
        }
        anterior = Some(v.p);
    }
    let a = poly.vertices.first()?.p;
    let b = poly.vertices.last()?.p;
    let da = (a.x - perto.x).powi(2) + (a.y - perto.y).powi(2);
    let db = (b.x - perto.x).powi(2) + (b.y - perto.y).powi(2);
    Some(if db < da { ("FIM", ponto) } else { ("INICIO", ponto) })
}
/// Vínculo semântico do `C` do `.lst`: o código da barra, ou o `de`/`para` da
/// linha com o número do circuito (a posição dela entre as linhas do mesmo par,
/// como os `L` do `.lst`).
fn vinculo_do_alvo(s: &Session, banco: &anacad_data::Banco, documento: i64, alvo_txt: &str) -> Result<(&'static str, Value)> {
    let alvo = Handle::parse_hex(alvo_txt).ok_or_else(|| bad("anacad.equipamento.inserir", "handle do alvo invalido"))?;
    let doc = s.doc()?;
    let ent = doc.entity(alvo).ok_or_else(|| bad("anacad.equipamento.inserir", "barra ou linha nao encontrada"))?;
    Ok(match app_xdata(ent) {
        Some(APP_BARRA) => {
            let codigo = campo_xdata(ent, "codigo").and_then(|v| v.parse::<i64>().ok()).ok_or_else(|| {
                bad("anacad.equipamento.inserir", "a barra escolhida nao tem codigo no XData")
            })?;
            ("barra", json!({ "barra": codigo }))
        }
        Some(APP_LINHA) => {
            let de = campo_xdata(ent, "de").and_then(|v| v.parse::<i64>().ok()).ok_or_else(|| {
                bad("anacad.equipamento.inserir", "a linha escolhida nao tem a barra De no XData")
            })?;
            let para = campo_xdata(ent, "para").and_then(|v| v.parse::<i64>().ok()).ok_or_else(|| {
                bad("anacad.equipamento.inserir", "a linha escolhida nao tem a barra Para no XData")
            })?;
            let circuito = circuito_da_linha(banco, documento, alvo_txt, de, para)?;
            ("linha", json!({ "de": de, "para": para, "numeroCircuito": circuito }))
        }
        _ => return Err(bad("anacad.equipamento.inserir", "selecione uma barra ou uma linha")),
    })
}

/// Tensão base do grupo do alvo: na barra, a dela; na linha, a da barra `de`
/// — a mesma que dá a cor do traço da linha.
fn tensao_do_alvo(s: &Session, alvo: Handle) -> Option<f64> {
    let doc = s.doc().ok()?;
    let ent = doc.entity(alvo)?;
    match app_xdata(ent) {
        Some(APP_BARRA) => tensao_da_barra(ent),
        Some(APP_LINHA) => {
            let de = campo_xdata(ent, "de")?.parse::<i64>().ok()?;
            doc.model
                .iter()
                .filter(|e| app_xdata(e) == Some(APP_BARRA))
                .find(|e| campo_xdata(e, "codigo").and_then(|v| v.parse::<i64>().ok()) == Some(de))
                .and_then(|e| tensao_da_barra(e))
        }
        _ => None,
    }
}

/// A derivação do ANAFAS nasce junto com o bloco: com o equipamento inserido, o
/// traço sai do `T1` dele até a barra ou a linha mais perto e o vínculo entra no
/// `dados` do equipamento. Sem barra nem linha sob o bloco, o `EquipM` pede o
/// segundo clique em vez de deixar o equipamento solto.
fn ligar_equipamento(
    s: &mut Session,
    banco: &anacad_data::Banco,
    documento: i64,
    handle: Handle,
    clique: Vec2,
    dados: &mut Value,
) -> Result<Option<Value>> {
    let (_, terminal) = terminal_do_equipamento(s, handle)?;
    // O ponto do clique acha a barra ou a linha em que o bloco foi solto; o
    // terminal `T1` é a reserva para quando o bloco fica ao lado do traço.
    let alvo = alvo_perto(s, clique, TOLERANCIA_BARRA).or_else(|| alvo_perto(s, terminal, TOLERANCIA_BARRA));
    let Some(alvo) = alvo else { return Ok(None) };
    let info = ligar_ao_alvo(s, banco, documento, handle, alvo, dados)?;
    Ok(Some(info))
}

/// Grava a derivação na barra ou na linha `alvo` — o par `C` + `U` do `.lst`:
/// a polilinha `ANACAD_CONEXAO` do ponto do alvo ao `T1` do equipamento, com a
/// linha em `ConexaoTerminal`, e o vínculo no `dados` (`barra`, ou
/// `de`/`para`/`numeroCircuito`).
fn ligar_ao_alvo(
    s: &mut Session,
    banco: &anacad_data::Banco,
    documento: i64,
    handle: Handle,
    alvo: Handle,
    dados: &mut Value,
) -> Result<Value> {
    let (para_terminal, terminal) = terminal_do_equipamento(s, handle)?;
    let alvo_txt = alvo.hex();
    let equip_txt = handle.hex();
    let (tipo_alvo, vinculo) = vinculo_do_alvo(s, banco, documento, &alvo_txt)?;
    if let (Some(destino), Some(fonte)) = (dados.as_object_mut(), vinculo.as_object()) {
        for (k, v) in fonte {
            destino.insert(k.clone(), v.clone());
        }
    }
    let (de_terminal, ponta) = ponto_do_alvo(s, alvo, terminal)
        .ok_or_else(|| bad("anacad.equipamento.inserir", "a barra ou a linha perto nao e um traco"))?;
    let traco = s.add_entity(EntityKind::LwPolyline(LwPolyline {
        vertices: vec![
            PolyVertex { p: ponta, ..PolyVertex::default() },
            PolyVertex { p: terminal, ..PolyVertex::default() },
        ],
        closed: false,
        const_width: 0.0,
        elevation: 0.0,
        plinegen: false,
    }))?;
    marcar(
        s,
        traco,
        "ANACAD_CONEXAO",
        &[("de", &alvo_txt), ("para", &equip_txt), ("de_terminal", de_terminal), ("para_terminal", para_terminal), ("potencial", "")],
    )?;
    // A derivação e o equipamento usam a cor do grupo de tensão do alvo,
    // assim como os traços da barra ou linha em que estão ligados.
    if let Some(cor) = tensao_do_alvo(s, alvo).and_then(|tensao| cor_da_tensao(s, tensao)) {
        colorir(s, handle, cor)?;
        colorir(s, traco, cor)?;
    }
    let de_tipo = if tipo_alvo == "barra" { "Barra" } else { "Linha" };
    let id = banco
        .inserir_conexao(&anacad_data::ConexaoNova {
            documento: Some(documento),
            de_tipo: de_tipo.to_string(),
            de_handle: alvo_txt.clone(),
            de_terminal: de_terminal.to_string(),
            para_tipo: "Equipamento".to_string(),
            para_handle: equip_txt,
            para_terminal: para_terminal.to_string(),
            potencial: None,
            handle: Some(traco.hex()),
        })
        .map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?;
    Ok(json!({
        "alvo": alvo_txt,
        "tipo_alvo": tipo_alvo,
        "vinculo": vinculo,
        "conexao": { "conexao": id, "handle": traco.hex() },
    }))
}
/// Número do circuito da linha entre `de` e `para`: a posição dela na ordem das
/// linhas do mesmo par, como os `L` do `.lst` contam os circuitos paralelos.
fn circuito_da_linha(banco: &anacad_data::Banco, documento: i64, handle: &str, de: i64, para: i64) -> Result<i64> {
    let linhas = banco.linhas(documento, 100_000).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?;
    let mut n = 0i64;
    for linha in linhas.iter().filter(|l| l.de == Some(de) && l.para == Some(para)) {
        n += 1;
        if linha.handle.as_deref().is_some_and(|h| h.eq_ignore_ascii_case(handle)) {
            return Ok(n);
        }
    }
    Ok(n.max(1))
}

pub(super) struct EquipM {
    pub(super) tipo: String,
    /// Equipamento recém-inserido esperando a barra ou a linha do segundo
    /// clique: (handle, terminal `T1` de onde sai o traço).
    pendente: Option<(Handle, Vec2)>,
}

impl EquipM {
    pub(super) fn novo(tipo: String) -> Self {
        EquipM { tipo, pendente: None }
    }
}

impl Interactive for EquipM {
    fn name(&self) -> &'static str {
        "ANACAD_EQ"
    }
    fn begin(&mut self, _s: &mut Session) -> Result<Step> {
        self.pendente = None;
        Ok(Step::Continue)
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if self.pendente.is_some() {
            Prompt::new("Clique na barra ou na linha do equipamento", Accept::POINT)
        } else {
            Prompt::new("Ponto de inserção do equipamento", Accept::POINT)
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        let Input::Point(p) = i else { return Ok(Step::Continue) };
        if let Some((handle, terminal)) = self.pendente {
            let alvo = alvo_perto(s, p, TOLERANCIA_BARRA).or_else(|| alvo_perto(s, terminal, TOLERANCIA_BARRA));
            let Some(alvo) = alvo else {
                return Err(crate::EngineError::Other("clique na barra ou na linha do equipamento".into()));
            };
            conectar_pendente(s, handle, alvo)?;
            // Ligação feita: abre a janela `Dados de …` do equipamento, como o
            // ANAFAS faz ao inserir o componente.
            st(s, |state| state.equip_handle = Some(handle.hex()));
            s.set_selection(vec![handle]);
            return Ok(Step::Done);
        }
        let (db, documento) = db_e_documento(s);
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de inserir".into()));
        }
        let saida = run_equipamento_inserir(s, &json!({ "db": db, "documento": documento, "tipo": self.tipo, "x": p.x, "y": p.y }))?;
        // Barra ou linha sob o bloco: a derivação já saiu com ele.
        if saida.get("alvo").is_some() {
            // A derivação saiu junto com o bloco: abre a janela dele.
            if let Some(handle) = saida.get("handle").and_then(Value::as_str) {
                st(s, |state| state.equip_handle = Some(handle.to_string()));
            }
            return Ok(Step::Done);
        }
        // Sem alvo, o traço começa no `T1` e o segundo clique escolhe a barra
        // ou a linha — o caminho do Eletron, só na ordem inversa.
        let Some(handle) = saida.get("handle").and_then(Value::as_str).and_then(Handle::parse_hex) else {
            return Ok(Step::Done);
        };
        let (_, terminal) = terminal_do_equipamento(s, handle)?;
        self.pendente = Some((handle, terminal));
        s.echo("clique na barra ou na linha para ligar o equipamento".to_string());
        Ok(Step::Continue)
    }
    fn preview(&self, _s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        let Some((_, terminal)) = self.pendente else { return Vec::new() };
        vec![EntityKind::LwPolyline(LwPolyline {
            vertices: vec![
                PolyVertex { p: terminal, ..PolyVertex::default() },
                PolyVertex { p: cursor, ..PolyVertex::default() },
            ],
            closed: false,
            const_width: 0.0,
            elevation: 0.0,
            plinegen: false,
        })]
    }
}

/// Liga um equipamento já inserido à barra ou à linha do segundo clique: o
/// mesmo par do `.lst` (polilinha `ANACAD_CONEXAO` + `ConexaoTerminal`) com o
/// vínculo gravado no `dados` dele.
fn conectar_pendente(s: &mut Session, handle: Handle, alvo: Handle) -> Result<()> {
    let (db, documento) = db_e_documento(s);
    let banco = anacad_data::Banco::abrir(&db).map_err(|e| bad("anacad.equipamento.inserir", format!("{db}: {e}")))?;
    let mut dados = json!({});
    ligar_ao_alvo(s, &banco, documento, handle, alvo, &mut dados)?;
    run_equipamento_editar(s, &json!({ "db": db, "documento": documento, "handle": handle.hex(), "dados": dados }))?;
    Ok(())
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

pub(super) fn run_banco_criar(s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.criar", "`arquivo` is required"))?;
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.criar", format!("{arquivo}: {e}")))?;
    // O banco aberto vale para os comandos seguintes (e para a cascata do apagar).
    set_banco(s, arquivo, 1);
    resumo_banco(&banco, arquivo)
}

pub(super) fn run_banco_abrir(s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.banco.abrir", "`arquivo` is required"))?;
    if !std::path::Path::new(arquivo).exists() {
        return Err(bad("anacad.banco.abrir", format!("banco nao encontrado: {arquivo}")));
    }
    let banco = anacad_data::Banco::abrir(arquivo).map_err(|e| bad("anacad.banco.abrir", format!("{arquivo}: {e}")))?;
    set_banco(s, arquivo, banco.documentos().map(|d| d.first().map(|d| d.id).unwrap_or(1)).unwrap_or(1));
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

// ---------- configuração do projeto ----------
// Mesmo modelo do Eletron: a configuração do projeto anda com o desenho. Lá o
// `DirProjeto` (o `.db`) e o nome do caderno (`Conf.Local`) ficam no XData do
// DWG (`GravarXDataConfiguracao2`) e o add-in relê na abertura
// (`LerXDataConfiguracao2`), então o usuário não escolhe o banco a cada
// sessão. O cadcraft guarda XData por entidade, então a configuração mora num
// POINT oculto na origem (`ANACAD_CFG`), que viaja no DXF como o resto.

/// App name do XData que guarda a configuração do projeto no desenho.
const APP_CFG: &str = "ANACAD_CFG";

/// Configuração do projeto: o banco usado, o nome do caderno e o estilo de
/// desenho das barras e das linhas.
///
/// O estilo é o que se precisa para desenhar a identificação do diagrama —
/// tamanho do texto, cor do texto e espessura do traço —, um conjunto para a
/// barra e outro para a linha. Tamanho e espessura são em unidades do desenho;
/// a cor pode ser um índice ACI ou um true color RGB, como no `CColorDialog`
/// do ANAFAS.
///
/// `grupos` são os **grupos base de tensão** do projeto (o `G` do `.lst`:
/// `ten, lar, R, G, B`): tensão em kV e cor — a espessura `lar` não é editada
/// aqui, porque o traço da barra e o da linha têm uma espessura só, no estilo
/// do desenho. No ANAFAS a cor de cada elemento vem do grupo da tensão base
/// dele — 500 kV vermelho `255,0,0`, 345 kV azul `0,128,255`, 138 kV e 13,8 kV
/// laranja `255,165,0` no desenho de exemplo —, não do enrolamento do trafo nem
/// do tipo do elemento. Aqui os grupos são editados no CFG, com os valores do
/// exemplo como padrão.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CfgProjeto {
    pub banco: String,
    pub caderno: String,
    pub documento: i64,
    /// Barra: altura e cor do texto de identificação e espessura do traço.
    pub barra_texto: f64,
    pub barra_cor: Color,
    pub barra_lar: f64,
    /// Linha: o mesmo para a identificação e o traço dela.
    pub linha_texto: f64,
    pub linha_cor: Color,
    pub linha_lar: f64,
    pub grupos: Vec<GrupoCor>,
}

/// Grupo base de tensão: a tensão em kV e a cor com que os elementos daquele
/// nível entram no desenho. A espessura (`lar`, o deslizante Espessura da
/// janela 210) não é do grupo: o traço da barra e o da linha têm uma espessura
/// só, no grupo **Texto e Traço de Barras e Linhas**.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GrupoCor {
    pub tensao: f64,
    pub cor: Color,
}

/// Grupos base de tensão do desenho de exemplo do ANAFAS, os seis do cabeçalho
/// do `.lst` (`G ten lar R G B`): 500 kV vermelho, 345 kV azul, 138 kV,
/// 13,8 kV e 4,16 kV laranja e 0 kV preto. A lista não é fixa — cada projeto
/// tem os seus grupos, com tensão e cor livres (janela 210 — Dados de Grupo
/// Base de Tensão).
pub(crate) fn grupos_padrao() -> Vec<GrupoCor> {
    let laranja = Color::True(cadcraft_doc::color::Rgb(255, 165, 0));
    vec![
        GrupoCor { tensao: 500.0, cor: Color::True(cadcraft_doc::color::Rgb(255, 0, 0)) },
        GrupoCor { tensao: 345.0, cor: Color::True(cadcraft_doc::color::Rgb(0, 128, 255)) },
        GrupoCor { tensao: 138.0, cor: laranja },
        GrupoCor { tensao: 13.8, cor: laranja },
        GrupoCor { tensao: 4.16, cor: laranja },
        GrupoCor { tensao: 0.0, cor: Color::True(cadcraft_doc::color::Rgb(0, 0, 0)) },
    ]
}

impl Default for CfgProjeto {
    fn default() -> Self {
        Self {
            banco: String::new(),
            caderno: String::new(),
            documento: 1,
            // 2,5 de altura e 7 = branco/preto, a cor de quem não escolheu
            // outra; a barra nasce com 0,3 de espessura e a linha com 0 (fio).
            barra_texto: 2.5,
            barra_cor: Color::Index(7),
            barra_lar: 0.3,
            linha_texto: 2.5,
            linha_cor: Color::Index(7),
            linha_lar: 0.0,
            // Os três RGBs do desenho de exemplo do ANAFAS (grupos base de
            // 500 kV, 13,8 kV e 345 kV).
            grupos: grupos_padrao(),
        }
    }
}

/// Entidades que carregam a configuração do projeto (normalmente uma só).
fn entidades_cfg(doc: &cadcraft_doc::Drawing) -> Vec<Handle> {
    doc.model
        .iter()
        .filter_map(|e| match e.common.xdata.first() {
            Some(t) if t.code == 1001 && t.value == APP_CFG => Some(e.handle),
            _ => None,
        })
        .collect()
}

/// Cor lida do `ANACAD_CFG`: índice ACI ("3") ou true color ("255,128,0",
/// "#ff8000", "red"); o que não for cor volta ao branco.
fn cor_do_texto(valor: &str) -> Color {
    match Color::parse(valor) {
        Some(cor @ (Color::Index(_) | Color::True(_))) => cor,
        _ => Color::Index(7),
    }
}

/// Texto gravado no `ANACAD_CFG` para a cor: o índice ACI ("3", grupo 62 do
/// DXF) ou o RGB ("255,128,0", grupo 420).
fn texto_cor(cor: Color) -> String {
    match cor {
        Color::Index(i) => i.to_string(),
        Color::True(rgb) => format!("{},{},{}", rgb.0, rgb.1, rgb.2),
        _ => "7".to_string(),
    }
}

/// Cor vinda de um parâmetro JSON: número (ACI) ou texto em qualquer formato
/// que o desenho aceita.
fn cor_do_valor(v: &Value) -> Option<Color> {
    if let Some(n) = v.as_u64() {
        return Some(cor_do_texto(&n.to_string()));
    }
    v.as_str().map(cor_do_texto)
}

/// Altura de texto lida do `ANACAD_CFG`, no intervalo aceito.
fn altura_do_texto(valor: &str) -> f64 {
    valor.parse::<f64>().unwrap_or(2.5).clamp(0.1, 100.0)
}

/// Espessura de traço lida do `ANACAD_CFG`, no intervalo aceito.
fn lar_do_texto(valor: &str) -> f64 {
    valor.parse::<f64>().unwrap_or(0.0).clamp(0.0, 10.0)
}

/// Configuração gravada no desenho; sem XData, os padrões.
pub(super) fn cfg_do_desenho(doc: &cadcraft_doc::Drawing) -> CfgProjeto {
    let mut cfg = CfgProjeto::default();
    let mut grupos_lidos = false;
    for handle in entidades_cfg(doc) {
        let Some(entity) = doc.entity(handle) else { continue };
        let pares: Vec<&str> = entity.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
        for par in pares.chunks(2) {
            if par.len() != 2 {
                continue;
            }
            match par[0] {
                "banco" => cfg.banco = par[1].to_string(),
                "caderno" => cfg.caderno = par[1].to_string(),
                "documento" => cfg.documento = par[1].parse().unwrap_or(1).max(1),
                "barra_texto" => cfg.barra_texto = altura_do_texto(par[1]),
                "barra_cor" => cfg.barra_cor = cor_do_texto(par[1]),
                "barra_lar" => cfg.barra_lar = lar_do_texto(par[1]),
                "linha_texto" => cfg.linha_texto = altura_do_texto(par[1]),
                "linha_cor" => cfg.linha_cor = cor_do_texto(par[1]),
                "linha_lar" => cfg.linha_lar = lar_do_texto(par[1]),
                // Desenho da versão 4: as três cores da identificação valem
                // para o texto da barra e o da linha.
                "cor_identificador" | "cor_numero" | "cor_outros" => {
                    let cor = cor_do_texto(par[1]);
                    cfg.barra_cor = cor;
                    cfg.linha_cor = cor;
                }
                "grupo" => {
                    // Os pares `grupo` do desenho substituem a lista padrão.
                    if !grupos_lidos {
                        cfg.grupos.clear();
                        grupos_lidos = true;
                    }
                    if let Some(grupo) = grupo_do_texto(par[1]) {
                        cfg.grupos.push(grupo);
                    }
                }
                _ => {}
            }
        }
    }
    cfg
}

/// Grava a configuração no desenho, criando o marcador (POINT oculto na
/// origem) quando ele ainda não existe.
pub(super) fn gravar_cfg(s: &mut Session, cfg: &CfgProjeto) -> Result<()> {
    let handle = match entidades_cfg(s.doc()?).into_iter().next() {
        Some(handle) => handle,
        None => {
            let handle = s.add_entity(EntityKind::Point(Point { p: Vec3::ZERO, angle: 0.0 }))?;
            s.doc_mut()?.modify_entity(handle, |e| e.common.visible = false)?;
            handle
        }
    };
    let (barra_cor, linha_cor) = (texto_cor(cfg.barra_cor), texto_cor(cfg.linha_cor));
    let (barra_texto, barra_lar) = (cfg.barra_texto.to_string(), cfg.barra_lar.to_string());
    let (linha_texto, linha_lar) = (cfg.linha_texto.to_string(), cfg.linha_lar.to_string());
    let documento = cfg.documento.to_string();
    let grupos: Vec<String> = cfg.grupos.iter().map(texto_grupo).collect();
    let mut campos: Vec<(&str, &str)> = vec![
        ("versao", "6"),
        ("banco", &cfg.banco),
        ("caderno", &cfg.caderno),
        ("documento", &documento),
        ("barra_texto", &barra_texto),
        ("barra_cor", &barra_cor),
        ("barra_lar", &barra_lar),
        ("linha_texto", &linha_texto),
        ("linha_cor", &linha_cor),
        ("linha_lar", &linha_lar),
    ];
    campos.extend(grupos.iter().map(|grupo| ("grupo", grupo.as_str())));
    marcar(s, handle, APP_CFG, &campos)
}

/// `anacad.cfg.ler`: a configuração gravada no desenho, já aplicada à sessão
/// (banco, documento e cores), para o próximo comando usá-la.
pub(super) fn run_cfg_ler(s: &mut Session, _p: &Value) -> Result<Value> {
    let mut cfg = cfg_do_desenho(s.doc()?);
    // Sem grupos no desenho, valem os do projeto: o `Grupos` do documento, o
    // mesmo lugar em que o `G` do `.lst` fica no ANAFAS.
    if !tem_grupos_no_desenho(s.doc()?)
        && let Some(grupos) = grupos_do_documento(&cfg)
        && !grupos.is_empty()
    {
        cfg.grupos = grupos;
    }
    guardar_cfg_na_sessao(s, &cfg);
    Ok(resumo_cfg(&cfg))
}

/// `anacad.cfg.gravar`: grava no desenho os campos recebidos (os ausentes
/// ficam como estão) e recolore a identificação das barras já desenhada.
pub(super) fn run_cfg_gravar(s: &mut Session, p: &Value) -> Result<Value> {
    let mut cfg = cfg_do_desenho(s.doc()?);
    if let Some(v) = p.get("banco").and_then(Value::as_str) {
        cfg.banco = v.trim().to_string();
    }
    if let Some(v) = p.get("caderno").and_then(Value::as_str) {
        cfg.caderno = v.trim().to_string();
    }
    if let Some(v) = p.get("documento").and_then(Value::as_i64) {
        cfg.documento = v.max(1);
    }
    if let Some(v) = p.get("barra_texto").and_then(Value::as_f64) {
        cfg.barra_texto = v.clamp(0.1, 100.0);
    }
    if let Some(cor) = p.get("barra_cor").and_then(cor_do_valor) {
        cfg.barra_cor = cor;
    }
    if let Some(v) = p.get("barra_lar").and_then(Value::as_f64) {
        cfg.barra_lar = v.clamp(0.0, 10.0);
    }
    if let Some(v) = p.get("linha_texto").and_then(Value::as_f64) {
        cfg.linha_texto = v.clamp(0.1, 100.0);
    }
    if let Some(cor) = p.get("linha_cor").and_then(cor_do_valor) {
        cfg.linha_cor = cor;
    }
    if let Some(v) = p.get("linha_lar").and_then(Value::as_f64) {
        cfg.linha_lar = v.clamp(0.0, 10.0);
    }
    // Cores dos grupos base de tensão: repintam o que já está desenhado com a
    // cor do nível de tensão de cada elemento, como no ANAFAS.
    let mut grupos: Option<Vec<GrupoCor>> = None;
    if let Some(lista) = p.get("grupos").and_then(Value::as_array) {
        grupos = Some(
            lista
                .iter()
                .filter_map(|item| {
                    let tensao = item.get("tensao").and_then(Value::as_f64)?;
                    let cor = item.get("cor").and_then(cor_do_valor)?;
                    Some(GrupoCor { tensao, cor })
                })
                .collect(),
        );
    }
    // Sem `grupos` no pedido, o desenho herda os do projeto na primeira vez: o
    // `Grupos` do documento, como o `G` do `.lst`.
    if grupos.is_none()
        && !tem_grupos_no_desenho(s.doc()?)
        && let Some(do_banco) = grupos_do_documento(&cfg)
        && !do_banco.is_empty()
    {
        cfg.grupos = do_banco;
    }
    let mudou_grupos = grupos.is_some();
    if let Some(lista) = grupos {
        cfg.grupos = lista;
    }
    gravar_cfg(s, &cfg)?;
    guardar_cfg_na_sessao(s, &cfg);
    if mudou_grupos {
        gravar_grupos_no_documento(&cfg);
        repintar_por_tensao(s, &cfg)?;
    }
    // O estilo vale também para o que já está desenhado, sem esperar a
    // próxima edição da barra ou da linha.
    aplicar_estilo(s, &cfg)?;
    Ok(resumo_cfg(&cfg))
}

/// Aplica ao desenho o estilo da configuração: altura e cor dos TEXT de
/// identificação da barra e espessura do traço das barras e das linhas (com o
/// campo `largura` do XData acompanhando a espessura).
fn aplicar_estilo(s: &mut Session, cfg: &CfgProjeto) -> Result<()> {
    let alvos: Vec<(Handle, &'static str)> = s
        .doc()?
        .model
        .iter()
        .filter_map(|e| {
            let app = match e.common.xdata.first() {
                Some(t) if t.code == 1001 => t.value.as_str(),
                _ => return None,
            };
            match app {
                APP_TEXTO_BARRA => Some((e.handle, APP_TEXTO_BARRA)),
                APP_TEXTO_LINHA => Some((e.handle, APP_TEXTO_LINHA)),
                APP_BARRA => Some((e.handle, APP_BARRA)),
                APP_LINHA => Some((e.handle, APP_LINHA)),
                _ => None,
            }
        })
        .collect();
    // Posição nova de cada texto de circuito, achada no dono dele (o `handle`
    // do XData é o da linha): sem isso um desenho antigo continuaria com o
    // número afastado do traço, só com a altura nova.
    let mut posicoes: Vec<(Handle, Vec3)> = Vec::new();
    for e in s.doc()?.model.iter() {
        if app_xdata(e) != Some(APP_TEXTO_LINHA) {
            continue;
        }
        let Some(dono) = campo_xdata(e, "handle").and_then(Handle::parse_hex) else { continue };
        let Some((a, b)) = s.doc()?.entity(dono).and_then(|linha| extremos_da_linha(linha)) else { continue };
        posicoes.push((e.handle, posicao_texto_linha(a, b, cfg.linha_texto)));
    }
    for (handle, app) in alvos {
        let (texto, cor, lar) = match app {
            APP_LINHA | APP_TEXTO_LINHA => (cfg.linha_texto, cfg.linha_cor, cfg.linha_lar),
            _ => (cfg.barra_texto, cfg.barra_cor, cfg.barra_lar),
        };
        let mut largura = None;
        let posicao = posicoes.iter().find(|(dono, _)| *dono == handle).map(|(_, pos)| *pos);
        s.doc_mut()?.modify_entity(handle, |e| {
            match &mut e.kind {
                EntityKind::Text(t) => {
                    t.height = texto;
                    // O texto do circuito é reassentado na altura nova: no meio
                    // da linha, centrado e por cima do traço.
                    if let Some(pos) = posicao {
                        t.insert = pos;
                        t.align_pt = Some(pos);
                        t.halign = HAlign::Center;
                        t.valign = VAlign::Bottom;
                    }
                    e.common.color = cor;
                }
                EntityKind::LwPolyline(p) => {
                    p.const_width = lar;
                    largura = Some(lar.to_string());
                }
                _ => {}
            }
            if let Some(valor) = &largura {
                let tags = &mut e.common.xdata;
                for i in 0..tags.len().saturating_sub(1) {
                    if tags[i].code == 1000 && tags[i].value == "largura" && tags[i + 1].code == 1000 {
                        tags[i + 1].value = valor.clone();
                        break;
                    }
                }
            }
        })?;
    }
    Ok(())
}

fn guardar_cfg_na_sessao(s: &mut Session, cfg: &CfgProjeto) {
    st(s, |state| {
        state.db = cfg.banco.clone();
        state.documento = cfg.documento;
        state.grupos = cfg.grupos.clone();
    });
}

fn resumo_cfg(cfg: &CfgProjeto) -> Value {
    json!({
        "banco": cfg.banco,
        "caderno": cfg.caderno,
        "documento": cfg.documento,
        "barra_texto": cfg.barra_texto,
        "barra_cor": texto_cor(cfg.barra_cor),
        "barra_lar": cfg.barra_lar,
        "linha_texto": cfg.linha_texto,
        "linha_cor": texto_cor(cfg.linha_cor),
        "linha_lar": cfg.linha_lar,
        "grupos": cfg.grupos.iter().map(|grupo| json!({"tensao": grupo.tensao, "cor": texto_cor(grupo.cor)})).collect::<Vec<_>>(),
    })
}

/// Espessura do traço da barra configurada no projeto (0,3 sem configuração).
fn lar_da_barra(s: &Session) -> f64 {
    s.doc().map(|doc| cfg_do_desenho(doc).barra_lar).unwrap_or(0.3)
}

/// Espessura do traço da linha configurada no projeto (0 = fio).
fn lar_da_linha(s: &Session) -> f64 {
    s.doc().map(|doc| cfg_do_desenho(doc).linha_lar).unwrap_or(0.0)
}

/// Cor do grupo base de tensão de um elemento (a barra, a linha e o componente
/// entram na cor do nível de tensão deles, como no ANAFAS).
fn cor_da_tensao(s: &Session, tensao: f64) -> Option<Color> {
    let cfg = s.doc().map(cfg_do_desenho).unwrap_or_default();
    cor_do_grupo(&cfg, tensao)
}

/// Cor configurada para uma tensão base (igualdade com folga de 0,5%).
fn cor_do_grupo(cfg: &CfgProjeto, tensao: f64) -> Option<Color> {
    cfg.grupos
        .iter()
        .find(|grupo| (grupo.tensao - tensao).abs() <= grupo.tensao.abs().max(1.0) * 0.005)
        .map(|grupo| grupo.cor)
}

/// Pinta a entidade recém-criada com a cor do grupo.
fn colorir(s: &mut Session, handle: Handle, cor: Color) -> Result<()> {
    s.doc_mut()?.modify_entity(handle, |e| e.common.color = cor)?;
    Ok(())
}

/// Deixa as entidades do bloco do tipo com a cor da referência (`ByBlock`),
/// para cada instância ter a cor do grupo de tensão da barra dela.
fn bloco_byblock(doc: &mut cadcraft_doc::Drawing, nome: &str) {
    let Some(bloco) = doc.blocks.get_mut(nome) else { return };
    let bloco = std::sync::Arc::make_mut(bloco);
    for handle in bloco.entities.handles() {
        bloco.entities.modify(handle, |e| e.common.color = Color::ByBlock);
    }
}

/// Grupo base de tensão gravado no `ANACAD_CFG` como `"500|255,0,0"` (tensão e
/// cor). O formato das versões 3 a 5 (`"500|2|255,0,0"`, com a espessura entre
/// a tensão e a cor) continua sendo lido, com a espessura ignorada.
fn grupo_do_texto(texto: &str) -> Option<GrupoCor> {
    let mut partes = texto.split('|');
    let tensao = partes.next()?.trim().parse().ok()?;
    let segunda = partes.next()?;
    let cor = partes.next().unwrap_or(segunda);
    Some(GrupoCor { tensao, cor: cor_do_texto(cor.trim()) })
}

/// Texto do grupo no `ANACAD_CFG`: `"500|255,0,0"`.
fn texto_grupo(grupo: &GrupoCor) -> String {
    format!("{}|{}", grupo.tensao, texto_cor(grupo.cor))
}

/// O desenho tem cores de grupos gravadas no `ANACAD_CFG`?
fn tem_grupos_no_desenho(doc: &cadcraft_doc::Drawing) -> bool {
    entidades_cfg(doc).iter().any(|handle| {
        doc.entity(*handle).is_some_and(|e| {
            let pares: Vec<&str> = e.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
            pares.chunks(2).any(|par| par.len() == 2 && par[0] == "grupo")
        })
    })
}

/// App name do XData de um elemento (`None` para entidade sem XData).
pub(super) fn app_xdata(e: &cadcraft_doc::Entity) -> Option<&str> {
    e.common.xdata.first().filter(|t| t.code == 1001).map(|t| t.value.as_str())
}

/// Valor de um campo do XData do elemento.
pub(super) fn campo_xdata<'a>(e: &'a cadcraft_doc::Entity, nome: &str) -> Option<&'a str> {
    let pares: Vec<&str> = e.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
    pares.chunks(2).find(|par| par.len() == 2 && par[0] == nome).map(|par| par[1])
}

/// Tensão base (`VBase`) gravada na barra do desenho: o par `vbase` ou o `dados`
/// da janela Dados de Barra.
fn tensao_da_barra(e: &cadcraft_doc::Entity) -> Option<f64> {
    if let Some(vbase) = campo_xdata(e, "vbase").and_then(|v| v.parse::<f64>().ok()) {
        return Some(vbase);
    }
    let dados: Value = serde_json::from_str(campo_xdata(e, "dados")?).ok()?;
    dados.get("vbase").or_else(|| dados.get("tensao")).and_then(Value::as_f64)
}

/// Barras do desenho com tensão base conhecida.
fn barras_com_tensao(doc: &cadcraft_doc::Drawing) -> Vec<(Handle, f64)> {
    doc.model
        .iter()
        .filter(|e| app_xdata(e) == Some("ANACAD_BARRA"))
        .filter_map(|e| tensao_da_barra(e).map(|tensao| (e.handle, tensao)))
        .collect()
}

/// Folga (unidades do desenho) para o componente herdar a barra em que foi
/// inserido.
const TOLERANCIA_BARRA: f64 = 5.0;

/// Distância de um ponto ao segmento `a`–`b`.
fn distancia_ao_segmento(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let comprimento = dx * dx + dy * dy;
    let t = if comprimento > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / comprimento).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (x, y) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - x).powi(2) + (p.1 - y).powi(2)).sqrt()
}

/// Tensão base da barra mais próxima de um ponto: o componente herda a cor do
/// grupo da barra em que foi inserido, como o campo `Cor` do `C` no `.lst`.
fn tensao_no_ponto(doc: &cadcraft_doc::Drawing, x: f64, y: f64, tolerancia: f64) -> Option<f64> {
    doc.model
        .iter()
        .filter(|e| app_xdata(e) == Some("ANACAD_BARRA"))
        .filter_map(|e| {
            let campo = |nome: &str| campo_xdata(e, nome).and_then(|v| v.parse::<f64>().ok());
            let a = (campo("x1")?, campo("y1")?);
            let b = (campo("x2")?, campo("y2")?);
            Some((distancia_ao_segmento((x, y), a, b), tensao_da_barra(e)?))
        })
        .filter(|(distancia, _)| *distancia <= tolerancia)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, tensao)| tensao)
}

/// Código da barra desenhada sobre um ponto: é assim que a linha pega o **De**
/// e o **Para** das extremidades do traço. É o mesmo caminho do
/// `verificaDispConex` do Eletron, que olha o que está sob a extremidade da
/// conexão e grava o vínculo (potencial/handle) no XData. A geometria da
/// polilinha manda; o XData `x1..y2` serve de reserva.
fn codigo_da_barra_no_ponto(doc: &cadcraft_doc::Drawing, x: f64, y: f64) -> Option<i64> {
    doc.model
        .iter()
        .filter(|e| app_xdata(e) == Some(APP_BARRA))
        .filter_map(|e| {
            let (a, b) = match &e.kind {
                EntityKind::LwPolyline(poly) => {
                    let a = poly.vertices.first()?.p;
                    let b = poly.vertices.last()?.p;
                    ((a.x, a.y), (b.x, b.y))
                }
                _ => {
                    let campo = |nome: &str| campo_xdata(e, nome).and_then(|v| v.parse::<f64>().ok());
                    ((campo("x1")?, campo("y1")?), (campo("x2")?, campo("y2")?))
                }
            };
            let codigo = campo_xdata(e, "codigo")?.parse::<i64>().ok()?;
            Some((distancia_ao_segmento((x, y), a, b), codigo))
        })
        .filter(|(distancia, _)| *distancia <= TOLERANCIA_BARRA)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, codigo)| codigo)
}

/// Grupos base de tensão do projeto (tensão e cor): o `Grupos` do documento, o
/// mesmo lugar do `G` do `.lst` no ANAFAS.
fn grupos_do_documento(cfg: &CfgProjeto) -> Option<Vec<GrupoCor>> {
    if cfg.banco.trim().is_empty() {
        return None;
    }
    let banco = anacad_data::Banco::abrir(&cfg.banco).ok()?;
    let tela = banco.exportar_screen(cfg.documento).ok()?;
    Some(
        tela.grupos
            .iter()
            .map(|grupo| GrupoCor {
                tensao: grupo.tensao,
                cor: Color::True(cadcraft_doc::color::Rgb(grupo.r, grupo.g, grupo.b)),
            })
            .collect(),
    )
}

/// Grava os grupos base de tensão (tensão e cor) no documento do banco: o
/// exportador `.lst` passa a emitir o `G` com as mesmas tensões e cores do
/// desenho. A espessura do `G` não é editada no CFG (o traço tem uma espessura
/// só): fica a que o documento já tinha para a tensão, 2 num grupo novo.
fn gravar_grupos_no_documento(cfg: &CfgProjeto) {
    if cfg.banco.trim().is_empty() {
        return;
    }
    let Ok(banco) = anacad_data::Banco::abrir(&cfg.banco) else { return };
    let Ok(mut tela) = banco.exportar_screen(cfg.documento) else { return };
    let antes: Vec<(f64, i32)> = tela.grupos.iter().map(|grupo| (grupo.tensao, grupo.lar)).collect();
    tela.grupos = cfg
        .grupos
        .iter()
        .map(|grupo| {
            let rgb = rgb_da_cor(grupo.cor);
            let lar = antes
                .iter()
                .find(|(tensao, _)| (tensao - grupo.tensao).abs() <= grupo.tensao.abs().max(1.0) * 0.005)
                .map_or(2, |(_, lar)| *lar);
            anacad_core::screen::Grupo { tensao: grupo.tensao, lar, r: rgb.0, g: rgb.1, b: rgb.2 }
        })
        .collect();
    let _ = banco.atualizar_tela_documento(cfg.documento, &tela);
}

/// RGB de uma cor do desenho (índice ACI pela paleta do CAD, true color direto).
fn rgb_da_cor(cor: Color) -> (u8, u8, u8) {
    let rgb = cor.resolve(Color::Index(7), Color::Index(7));
    (rgb.0, rgb.1, rgb.2)
}

/// Repinta o que já está desenhado com a cor do grupo de tensão de cada
/// elemento: a barra pela `VBase`, a linha pela barra de origem e o componente
/// pela barra em que foi inserido.
fn repintar_por_tensao(s: &mut Session, cfg: &CfgProjeto) -> Result<()> {
    let banco = if cfg.banco.trim().is_empty() {
        None
    } else {
        anacad_data::Banco::abrir(&cfg.banco).ok()
    };
    let documento = cfg.documento;
    let mut alvos: Vec<(Handle, f64)> = Vec::new();
    let mut blocos: Vec<(String, Handle, f64)> = Vec::new();
    {
        let doc = s.doc()?;
        alvos.extend(barras_com_tensao(doc));
        for e in doc.model.iter() {
            match app_xdata(e) {
                Some("ANACAD_LINHA") => {
                    let de = campo_xdata(e, "de").and_then(|v| v.parse::<i64>().ok());
                    let tensao = campo_xdata(e, "vbase")
                        .and_then(|v| v.parse::<f64>().ok())
                        .or_else(|| {
                            de.and_then(|de| banco.as_ref().and_then(|b| b.barra_por_codigo(documento, de).ok().flatten()))
                                .and_then(|barra| barra.vbase)
                        });
                    if let Some(tensao) = tensao {
                        alvos.push((e.handle, tensao));
                    }
                }
                Some("ANACAD_EQ") => {
                    let (Some(x), Some(y)) = (
                        campo_xdata(e, "x").and_then(|v| v.parse::<f64>().ok()),
                        campo_xdata(e, "y").and_then(|v| v.parse::<f64>().ok()),
                    ) else {
                        continue;
                    };
                    let Some(tensao) = tensao_no_ponto(doc, x, y, TOLERANCIA_BARRA) else { continue };
                    match campo_xdata(e, "tipo").and_then(tipo_equipamento) {
                        Some((slug, _)) => blocos.push((format!("ANACAD_EQ_{slug}"), e.handle, tensao)),
                        None => alvos.push((e.handle, tensao)),
                    }
                }
                _ => {}
            }
        }
    }
    for (handle, tensao) in alvos {
        if let Some(cor) = cor_do_grupo(cfg, tensao) {
            s.doc_mut()?.modify_entity(handle, |e| e.common.color = cor)?;
        }
    }
    let mut feitos: Vec<String> = Vec::new();
    for (bloco, handle, tensao) in blocos {
        let Some(cor) = cor_do_grupo(cfg, tensao) else { continue };
        s.doc_mut()?.modify_entity(handle, |e| e.common.color = cor)?;
        if !feitos.contains(&bloco) {
            bloco_byblock(s.doc_mut()?, &bloco);
            feitos.push(bloco);
        }
    }
    Ok(())
}
