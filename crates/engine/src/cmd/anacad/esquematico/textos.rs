//! Textos de identificação ligados à barra e à linha (XData `ANACAD_*_TEXTO`).

use super::*;

/// App name do XData dos textos de identificação da barra.
pub(in crate::cmd::anacad) const APP_TEXTO_BARRA: &str = "ANACAD_BARRA_TEXTO";
/// App name do XData do texto com o número do circuito, desenhado junto do
/// traço da linha.
pub(in crate::cmd::anacad) const APP_TEXTO_LINHA: &str = "ANACAD_LINHA_TEXTO";
/// App name do XData das barras desenhadas (LWPOLYLINE).
pub(in crate::cmd::anacad) const APP_BARRA: &str = "ANACAD_BARRA";
/// App name do XData das linhas desenhadas (LWPOLYLINE).
pub(in crate::cmd::anacad) const APP_LINHA: &str = "ANACAD_LINHA";
/// App name do XData das conexões terminais (derivações entre equipamento e barra/linha).
pub(in crate::cmd::anacad) const APP_CONEXAO: &str = "ANACAD_CONEXAO";

/// Textos ligados a um elemento pelo XData (`app` = app name do texto):
/// `(campo, handle)`. O vínculo é o handle do elemento gravado no texto, com o
/// `campo` dizendo qual informação ele carrega.
pub(in crate::cmd::anacad) fn textos_ligados(doc: &cadcraft_doc::Drawing, app: &str, alvo: Handle) -> Vec<(String, Handle)> {
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
pub(super) fn textos_da_barra(doc: &cadcraft_doc::Drawing, barra: Handle) -> Vec<(String, Handle)> {
    textos_ligados(doc, APP_TEXTO_BARRA, barra)
}

/// Posição e alinhamento do texto em relação à barra. A âncora é a geometria
/// dela, **não o ponto em que o traço começou**: numa horizontal o texto fica no
/// extremo esquerdo, alinhado à esquerda e acima da linha (pilha no normal, com
/// `y` positivo); numa vertical o TEXT não gira, então fica centrado no eixo da
/// barra e empilhado acima do extremo superior, com o identificador em cima do
/// número. Assim o lugar do texto não muda conforme o sentido em que a barra foi
/// desenhada ou editada.
pub(super) fn posicao_texto_barra(campo: &str, a: Vec2, b: Vec2, altura: f64) -> (Vec3, bool) {
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
pub(in crate::cmd::anacad) fn sincronizar_textos_barra(s: &mut Session, barra: Handle, codigo: i64, nome: Option<&str>, a: Vec2, b: Vec2) -> Result<Vec<String>> {
    // Altura e cor do texto de identificação, do estilo da barra no projeto.
    let cfg = cfg_do_desenho(s.doc()?);
    let (altura, cor) = (cfg.barra_texto, cfg.barra_cor);
    let existentes = textos_da_barra(s.doc()?, barra);
    let nome = nome.map(str::trim).filter(|n| !n.is_empty());
    let campos: [(&str, Option<String>); 2] = [("identificador", nome.map(str::to_string)), ("numero", Some(codigo.to_string()))];
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
                marcar(s, handle, APP_TEXTO_BARRA, &[("handle", &barra.hex()), ("campo", campo), ("codigo", &codigo.to_string())])?;
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

pub(in crate::cmd::anacad) fn remover_textos_da_barra(s: &mut Session, barra: Handle) -> Result<()> {
    let handles: Vec<Handle> = textos_da_barra(s.doc()?, barra).into_iter().map(|(_, h)| h).collect();
    for handle in handles {
        s.doc_mut()?.remove_entity(handle);
    }
    Ok(())
}

/// Posição do número do circuito: no meio do traço, deslocado para o lado de
/// cima (o `y` cresce para cima), com folga de meia altura de texto.
pub(super) fn extremos_da_linha(e: &cadcraft_doc::Entity) -> Option<(Vec2, Vec2)> {
    let EntityKind::LwPolyline(poly) = &e.kind else { return None };
    let a = poly.vertices.first()?.p;
    let b = poly.vertices.last().unwrap_or(poly.vertices.first()?).p;
    Some((a, b))
}

pub(super) fn posicao_texto_linha(a: Vec2, b: Vec2, altura: f64) -> Vec3 {
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
pub(in crate::cmd::anacad) fn sincronizar_texto_linha(s: &mut Session, linha: Handle, circuito: Option<i64>, a: Vec2, b: Vec2) -> Result<Vec<String>> {
    let cfg = cfg_do_desenho(s.doc()?);
    let (altura, cor) = (cfg.linha_texto, cfg.linha_cor);
    let atual = textos_ligados(s.doc()?, APP_TEXTO_LINHA, linha).into_iter().find(|(campo, _)| campo == "circuito").map(|(_, handle)| handle);
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
            marcar(s, handle, APP_TEXTO_LINHA, &[("handle", &linha.hex()), ("campo", "circuito"), ("numero", &numero.to_string())])?;
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
pub(super) fn handles_textos(doc: &cadcraft_doc::Drawing, app: &str, alvo: Option<Handle>) -> Vec<Handle> {
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
pub(in crate::cmd::anacad) fn run_textos_barra(s: &mut Session, p: &Value) -> Result<Value> {
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
