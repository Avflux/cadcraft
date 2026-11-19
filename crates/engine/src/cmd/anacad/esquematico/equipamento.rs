//! Equipamentos: bloco, inserção, edição e ligação ao alvo (barra ou linha).

use super::*;

pub(in crate::cmd::anacad) fn run_equipamento_bloco(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_equipamento_inserir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.equipamento.inserir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let tipo_txt = str_param(p, "tipo").ok_or_else(|| bad("anacad.equipamento.inserir", "`tipo` is required"))?;
    let (slug, nome_tipo) = tipo_equipamento(tipo_txt).ok_or_else(|| bad("anacad.equipamento.inserir", format!("tipo desconhecido: {tipo_txt}")))?;
    let x = f64_or(p, "x", 0.0);
    let y = f64_or(p, "y", 0.0);
    let rotacao_deg = p.get("rotacao").and_then(Value::as_f64).unwrap_or(0.0);
    let rotacao = if rotacao_deg.is_finite() { rotacao_deg.to_radians() } else { 0.0 };
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.inserir", format!("{db}: {e}")))?;
    let codigo = match p.get("codigo").and_then(Value::as_i64) {
        Some(c) if c > 0 => c,
        _ => banco.proximo_codigo_equipamento(documento, nome_tipo).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?,
    };
    let id = banco.criar_equipamento(documento, nome_tipo, codigo, nome_tipo, x, y).map_err(|e| bad("anacad.equipamento.inserir", e.to_string()))?;
    let bloco = if let Some(custom) = bloco_personalizado(s, &banco, slug) {
        custom
    } else if slug == "transformador" {
        bloco_do_transformador_orientado(s, &json!({}), rotacao_deg)?
    } else {
        bloco_equipamento(s, slug)?
    };
    let mut valores = serde_json::Map::new();
    valores.insert("CODIGO".into(), json!(codigo.to_string()));
    valores.insert("NOME".into(), json!(nome_tipo));
    valores.insert("TENSAO".into(), json!(""));
    valores.insert("DEFASAMENTO".into(), json!(""));
    valores.insert("T1".into(), json!("DE"));
    valores.insert("T2".into(), json!("PARA"));
    // Escala do bloco: a do pedido, ou a configurada no projeto (CFG), com
    // 1 = tamanho natural do bloco quando o desenho ainda não tem a chave.
    let escala = p
        .get("escala")
        .and_then(Value::as_f64)
        .filter(|v| *v > 0.0)
        .unwrap_or_else(|| s.doc().map(|doc| cfg_do_desenho(doc).equip_escala).unwrap_or(1.0));
    // Altura do texto dos atributos (CODIGO, NOME, TENSAO, T1, T2): a do
    // pedido (`altura_texto`), ou a configurada no projeto (CFG), em
    // unidades do desenho — independente da escala do bloco, que também
    // multiplicaria a altura da definição. Sem a chave, o desenho vale 2,0.
    let altura_texto = p
        .get("altura_texto")
        .and_then(Value::as_f64)
        .filter(|v| *v > 0.0)
        .unwrap_or_else(|| s.doc().map(|doc| cfg_do_desenho(doc).equip_texto).unwrap_or(2.0));
    let handle = super::blocks::insert(s, &bloco, Vec2::new(x, y), escala, rotacao, &valores)?;
    // A altura do texto vale para todos os atributos da referência, inclusive
    // os multilinha (a altura do `mtext` deles).
    s.doc_mut()?.modify_entity(handle, |e| {
        if let EntityKind::Insert(ins) = &mut e.kind {
            for attrib in &mut ins.attribs {
                attrib.text.height = altura_texto;
                if let Some(ml) = &mut attrib.multiline {
                    ml.mtext.height = altura_texto;
                }
            }
        }
    })?;
    if slug == "transformador" {
        posicionar_rotulos_transformador(s, handle, Vec2::new(x, y), escala, rotacao_deg)?;
    }
    // O componente entra na cor do grupo base de tensão da barra em que foi
    // inserido: as entidades do bloco ficam `ByBlock` e a cor vai na referência,
    // como o campo `Cor` do `C` no `.lst` (cada instância com a sua). Blocos
    // personalizados do usuário ficam como estão.
    if bloco.starts_with(PREFIXO_BLOCO_EQUIPAMENTO) {
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
    // A inserção interativa de dois terminais adia as ligações até escolher
    // ambas as barras, evitando gravar um circuito incompleto ao cancelar.
    let ligacao = if p.get("auto_conectar").and_then(Value::as_bool) == Some(false) {
        None
    } else {
        match ligar_equipamento(s, &banco, documento, handle, Vec2::new(x, y), &mut dados) {
            Ok(ligacao) => ligacao,
            Err(e) => {
                s.echo(format!("ligação automática não saiu: {e}"));
                None
            }
        }
    };
    let mut saida = equipamento_json(id, nome_tipo, codigo, &handle.hex(), x, y, &bloco, &atributos, escala, altura_texto);
    if let Some(obj) = saida.as_object_mut() {
        obj.insert("rotacao".into(), json!(rotacao_deg));
    }
    if let Some(info) = ligacao {
        run_equipamento_editar(s, &json!({ "db": db, "documento": documento, "handle": handle.hex(), "dados": dados }))?;
        if let (Some(destino), Some(fonte)) = (saida.as_object_mut(), info.as_object()) {
            for (k, v) in fonte {
                destino.insert(k.clone(), v.clone());
            }
        }
    }
    s.set_selection(vec![handle]);
    Ok(saida)
}

/// Põe nome e defasamento do transformador pela regra do domínio
/// (`anacad_app::rotulos_do_transformador`), com a altura atual dos atributos:
/// em pé, afastados do símbolo e um do outro, em qualquer ângulo.
pub(super) fn posicionar_rotulos_transformador(s: &mut Session, handle: Handle, insercao: Vec2, escala: f64, angulo_graus: f64) -> Result<()> {
    s.doc_mut()?.modify_entity(handle, |e| {
        let EntityKind::Insert(ins) = &mut e.kind else { return };
        let altura = ins.attribs.iter().filter(|a| a.tag == "NOME" || a.tag == "DEFASAMENTO").map(|a| a.text.height).fold(0.0, f64::max);
        for (tag, _, (x, y), alinhamento) in anacad_app::rotulos_do_transformador(angulo_graus, escala, altura) {
            let Some(atributo) = ins.attribs.iter_mut().find(|a| a.tag == tag) else { continue };
            let alvo = Vec3::new(insercao.x + x, insercao.y + y, atributo.text.insert.z);
            let centrado = alinhamento == anacad_app::Alinhamento::CentroBase;
            atributo.text.insert = alvo;
            atributo.text.align_pt = centrado.then_some(alvo);
            atributo.text.halign = if centrado { HAlign::Center } else { HAlign::Left };
            atributo.text.valign = VAlign::Baseline;
            atributo.text.rotation = 0.0;
        }
    })?;
    Ok(())
}

/// Botão Remover das janelas de dados dos equipamentos (`Dados de …`):
/// exclui os dados elétricos (ANAFAS 33057), mantendo bloco, nome e posição.
pub(in crate::cmd::anacad) fn run_equipamento_dados_excluir(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_equipamento_editar(s: &mut Session, p: &Value) -> Result<Value> {
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
        let defasamento_val = dados.get("defasamento").and_then(Value::as_f64).map(|v| v.to_string());
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
                    "DEFASAMENTO" => {
                        if let Some(v) = &defasamento_val {
                            a.text.value = v.clone();
                        }
                    }
                    _ => {}
                }
            }
        })?;
    }
    // O símbolo do transformador mostra a ligação de cada enrolamento: a
    // referência passa ao bloco do `dados` gravado. Bloco personalizado do
    // usuário fica como está.
    let bloco_atual = s.doc()?.entity(handle).and_then(|e| match &e.kind {
        EntityKind::Insert(ins) => Some((ins.block.clone(), ins.rotation)),
        _ => None,
    });
    if let Some((atual, rotacao)) = bloco_atual.filter(|(bloco, _)| anacad_app::e_bloco_transformador(bloco)) {
        let novo = bloco_do_transformador_orientado(s, &dados, rotacao.to_degrees())?;
        if novo != atual {
            bloco_byblock(s.doc_mut()?, &novo);
            s.doc_mut()?.modify_entity(handle, |e| {
                if let EntityKind::Insert(ins) = &mut e.kind {
                    ins.block = novo.clone();
                }
            })?;
        }
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
pub(super) fn terminal_do_equipamento(s: &Session, handle: Handle) -> Result<(&'static str, Vec2)> {
    terminal_do_equipamento_em(s, handle, "T1")
}

fn terminal_do_equipamento_em(s: &Session, handle: Handle, tag: &'static str) -> Result<(&'static str, Vec2)> {
    let ent = s.doc()?.entity(handle).ok_or_else(|| bad("anacad.equipamento.inserir", "equipamento nao encontrado"))?;
    let EntityKind::Insert(ins) = &ent.kind else {
        return Err(bad("anacad.equipamento.inserir", "o equipamento nao e um bloco"));
    };
    let posicao = ins
        .attribs
        .iter()
        .find(|a| a.tag.eq_ignore_ascii_case(tag))
        .map(|a| Vec2::new(a.text.insert.x, a.text.insert.y))
        .or_else(|| if tag == "T1" { Some(Vec2::new(ins.insert.x, ins.insert.y)) } else { None })
        .ok_or_else(|| bad("anacad.equipamento.inserir", format!("terminal {tag} nao encontrado no bloco")))?;
    Ok((tag, posicao))
}

fn tem_dois_circuitos(tipo: &str) -> bool {
    tipo_equipamento(tipo).is_some_and(|(slug, _)| matches!(slug, "transformador" | "compensador_serie"))
}

fn prompt_barra_terminal(tipo: &str, terminal_tag: &str) -> &'static str {
    let transformador = tipo_equipamento(tipo).is_some_and(|(slug, _)| slug == "transformador");
    match (transformador, terminal_tag.eq_ignore_ascii_case("T2")) {
        (true, false) => "Clique na barra do primário do transformador",
        (true, true) => "Clique na barra do secundário do transformador",
        (false, false) => "Clique na barra do terminal 1",
        (false, true) => "Clique na barra do terminal 2",
    }
}

fn equipamento_tem_dois_circuitos(s: &Session, handle: Handle) -> bool {
    s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, "tipo")).is_some_and(tem_dois_circuitos)
}

fn alvo_perto_de_equipamento(s: &Session, tipo: &str, ponto: Vec2) -> Option<Handle> {
    if tem_dois_circuitos(tipo) {
        return alvo_perto_tipo(s, ponto, TOLERANCIA_BARRA, true);
    }
    alvo_perto(s, ponto, TOLERANCIA_BARRA)
}

/// Barra ou linha mais perto de um ponto: é ela que recebe a derivação do
/// equipamento, como o `verificaDispConex` do Eletron, que olha o que está sob
/// a extremidade da conexão. `tolerancia` é a folga em unidades do desenho.
pub(super) fn alvo_perto(s: &Session, ponto: Vec2, tolerancia: f64) -> Option<Handle> {
    alvo_perto_tipo(s, ponto, tolerancia, false)
}

fn alvo_perto_tipo(s: &Session, ponto: Vec2, tolerancia: f64, somente_barra: bool) -> Option<Handle> {
    let doc = s.doc().ok()?;
    doc.model
        .iter()
        .filter(|e| {
            if somente_barra {
                app_xdata(e) == Some(APP_BARRA)
                    && campo_xdata(e, "codigo").and_then(|codigo| codigo.parse::<i64>().ok()).is_some_and(|codigo| codigo > 0)
            } else {
                matches!(app_xdata(e), Some(APP_BARRA) | Some(APP_LINHA))
            }
        })
        .filter_map(|e| {
            let distancia = match &e.kind {
                // Donut (barra fictícia/auxiliar): o alvo é a rosca.
                EntityKind::Circle(c) => {
                    let d = ((ponto.x - c.center.x).powi(2) + (ponto.y - c.center.y).powi(2)).sqrt();
                    (d - c.radius).abs()
                }
                EntityKind::Insert(ins) if ins.block.eq_ignore_ascii_case(BLOCO_DONUT) => {
                    let d = ((ponto.x - ins.insert.x).powi(2) + (ponto.y - ins.insert.y).powi(2)).sqrt();
                    (d - RAIO_DONUT).abs()
                }
                EntityKind::LwPolyline(poly) => {
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
                    distancia
                }
                _ => return None,
            };
            (distancia <= tolerancia).then_some((distancia, e.handle))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, handle)| handle)
}

/// Ponto da barra ou da linha onde o traço encosta: a projeção do terminal do
/// equipamento no traço, então a derivação sai do meio da barra e não só da
/// ponta. O terminal (`INICIO`/`FIM`, os nomes de `terminal_da_entidade`) é o
/// da ponta mais perto.
pub(super) fn ponto_do_alvo(s: &Session, alvo: Handle, perto: Vec2) -> Option<(&'static str, Vec2)> {
    let ent = s.doc().ok()?.entity(alvo)?;
    if let EntityKind::Circle(c) = &ent.kind {
        // Donut: a derivação encosta na rosca, no ponto mais perto do terminal.
        let (dx, dy) = (perto.x - c.center.x, perto.y - c.center.y);
        let d = (dx * dx + dy * dy).sqrt();
        let p = if d > 1e-9 {
            Vec2::new(c.center.x + dx / d * c.radius, c.center.y + dy / d * c.radius)
        } else {
            Vec2::new(c.center.x + c.radius, c.center.y)
        };
        return Some(("INICIO", p));
    }
    if let EntityKind::Insert(ins) = &ent.kind {
        if !ins.block.eq_ignore_ascii_case(BLOCO_DONUT) {
            return None;
        }
        // Donut em bloco: a derivação encosta na rosca (raio 1,0), no ponto
        // mais perto do terminal.
        let (dx, dy) = (perto.x - ins.insert.x, perto.y - ins.insert.y);
        let d = (dx * dx + dy * dy).sqrt();
        let p = if d > 1e-9 {
            Vec2::new(ins.insert.x + dx / d * RAIO_DONUT, ins.insert.y + dy / d * RAIO_DONUT)
        } else {
            Vec2::new(ins.insert.x + RAIO_DONUT, ins.insert.y)
        };
        return Some(("INICIO", p));
    }
    let EntityKind::LwPolyline(poly) = &ent.kind else { return None };
    let mut ponto = poly.vertices.first()?.p;
    let mut distancia = f64::INFINITY;
    let mut anterior: Option<Vec2> = None;
    for v in &poly.vertices {
        if let Some(a) = anterior {
            let (dx, dy) = (v.p.x - a.x, v.p.y - a.y);
            let comprimento = dx * dx + dy * dy;
            let t = if comprimento > 0.0 { (((perto.x - a.x) * dx + (perto.y - a.y) * dy) / comprimento).clamp(0.0, 1.0) } else { 0.0 };
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
pub(super) fn vinculo_do_alvo(s: &Session, banco: &anacad_data::Banco, documento: i64, alvo_txt: &str) -> Result<(&'static str, Value)> {
    let alvo = Handle::parse_hex(alvo_txt).ok_or_else(|| bad("anacad.equipamento.inserir", "handle do alvo invalido"))?;
    let doc = s.doc()?;
    let ent = doc.entity(alvo).ok_or_else(|| bad("anacad.equipamento.inserir", "barra ou linha nao encontrada"))?;
    Ok(match app_xdata(ent) {
        Some(APP_BARRA) => {
            let codigo = campo_xdata(ent, "codigo")
                .and_then(|v| v.parse::<i64>().ok())
                .ok_or_else(|| bad("anacad.equipamento.inserir", "a barra escolhida nao tem codigo no XData"))?;
            ("barra", json!({ "barra": codigo }))
        }
        Some(APP_LINHA) => {
            let de = campo_xdata(ent, "de")
                .and_then(|v| v.parse::<i64>().ok())
                .ok_or_else(|| bad("anacad.equipamento.inserir", "a linha escolhida nao tem a barra De no XData"))?;
            let para = campo_xdata(ent, "para")
                .and_then(|v| v.parse::<i64>().ok())
                .ok_or_else(|| bad("anacad.equipamento.inserir", "a linha escolhida nao tem a barra Para no XData"))?;
            let circuito = circuito_da_linha(banco, documento, alvo_txt, de, para)?;
            ("linha", json!({ "de": de, "para": para, "numeroCircuito": circuito }))
        }
        _ => return Err(bad("anacad.equipamento.inserir", "selecione uma barra ou uma linha")),
    })
}

/// Tensão base do grupo do alvo: na barra, a dela; na linha, a da barra `de`
/// — a mesma que dá a cor do traço da linha.
pub(super) fn tensao_do_alvo(s: &Session, alvo: Handle) -> Option<f64> {
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
pub(super) fn ligar_equipamento(
    s: &mut Session,
    banco: &anacad_data::Banco,
    documento: i64,
    handle: Handle,
    clique: Vec2,
    dados: &mut Value,
) -> Result<Option<Value>> {
    let (_, terminal) = terminal_do_equipamento(s, handle)?;
    // O ponto do clique acha o alvo em que o bloco foi solto; o terminal `T1`
    // é a reserva para quando o bloco fica ao lado do traço. Equipamentos com
    // dois circuitos precisam de barras, pois cada terminal preenche De/Para.
    let dois_circuitos = equipamento_tem_dois_circuitos(s, handle);
    let alvo = if dois_circuitos {
        alvo_perto_tipo(s, terminal, TOLERANCIA_BARRA, true)
    } else {
        alvo_perto(s, clique, TOLERANCIA_BARRA).or_else(|| alvo_perto(s, terminal, TOLERANCIA_BARRA))
    };
    let Some(alvo) = alvo else { return Ok(None) };
    let info = ligar_ao_alvo(s, banco, documento, handle, alvo, dados)?;
    Ok(Some(info))
}

/// Grava a derivação na barra ou na linha `alvo` — o par `C` + `U` do `.lst`:
/// a polilinha `ANACAD_CONEXAO` do ponto do alvo ao `T1` do equipamento, com a
/// linha em `ConexaoTerminal`, e o vínculo no `dados` (`barra`, ou
/// `de`/`para`/`numeroCircuito`).
pub(super) fn ligar_ao_alvo(
    s: &mut Session,
    banco: &anacad_data::Banco,
    documento: i64,
    handle: Handle,
    alvo: Handle,
    dados: &mut Value,
) -> Result<Value> {
    ligar_ao_terminal(s, banco, documento, handle, alvo, "T1", dados)
}

fn ligar_ao_terminal(
    s: &mut Session,
    banco: &anacad_data::Banco,
    documento: i64,
    handle: Handle,
    alvo: Handle,
    terminal_tag: &'static str,
    dados: &mut Value,
) -> Result<Value> {
    let (para_terminal, terminal) = terminal_do_equipamento_em(s, handle, terminal_tag)?;
    let alvo_txt = alvo.hex();
    let equip_txt = handle.hex();
    let (tipo_alvo, vinculo) = vinculo_do_alvo(s, banco, documento, &alvo_txt)?;
    if equipamento_tem_dois_circuitos(s, handle) {
        if tipo_alvo != "barra" {
            return Err(bad("anacad.equipamento.inserir", format!("o terminal {terminal_tag} exige uma barra de conexão")));
        }
        if !dados.is_object() {
            *dados = json!({});
        }
        let chave = if terminal_tag.eq_ignore_ascii_case("T2") { "para" } else { "de" };
        dados[chave] = vinculo.get("barra").cloned().ok_or_else(|| bad("anacad.equipamento.inserir", "a barra escolhida nao tem codigo"))?;
    } else if let (Some(destino), Some(fonte)) = (dados.as_object_mut(), vinculo.as_object()) {
        for (k, v) in fonte {
            destino.insert(k.clone(), v.clone());
        }
    }
    let (de_terminal, ponta) =
        ponto_do_alvo(s, alvo, terminal).ok_or_else(|| bad("anacad.equipamento.inserir", "a barra ou a linha perto nao e um traco"))?;
    let traco = s.add_entity(EntityKind::LwPolyline(LwPolyline {
        vertices: vec![PolyVertex { p: ponta, ..PolyVertex::default() }, PolyVertex { p: terminal, ..PolyVertex::default() }],
        closed: false,
        const_width: 0.0,
        elevation: 0.0,
        plinegen: false,
    }))?;
    marcar(
        s,
        traco,
        APP_CONEXAO,
        &[("de", &alvo_txt), ("para", &equip_txt), ("de_terminal", de_terminal), ("para_terminal", para_terminal), ("potencial", "")],
    )?;
    // A derivação e o equipamento usam a cor do grupo de tensão do alvo,
    // assim como os traços da barra ou linha em que estão ligados. O bloco
    // mantém a cor da primeira conexão quando o T2 usa outro grupo de tensão.
    if let Some(cor) = tensao_do_alvo(s, alvo).and_then(|tensao| cor_da_tensao(s, tensao)) {
        if terminal_tag.eq_ignore_ascii_case("T1") {
            colorir(s, handle, cor)?;
        }
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

/// Equipamento esperando o segundo clique, **antes** de entrar no desenho:
/// o ponto de inserção do primeiro clique e o terminal `T1` calculado dele.
/// A inserção (bloco + registro no banco) só acontece no segundo clique,
/// com a ligação — então cancelar com ESC no meio não deixa o bloco solto
/// no desenho nem o registro no banco.
pub(super) enum Pendente {
    /// Só o ponto foi clicado; nada foi inserido ainda.
    Ponto { insercao: Vec2, terminal: Vec2, rotacao: f64 },
    /// As duas barras foram escolhidas; o bloco e as conexões ainda não foram
    /// gravados, então cancelar no segundo terminal não deixa dados órfãos.
    DoisTerminais { insercao: Vec2, rotacao: f64, alvo_t1: Handle, terminal_t2: Vec2 },
    /// Reserva para o caso (não esperado) de a inserção não achar o alvo que
    /// o primeiro clique já tinha visto: o bloco entrou, falta a ligação.
    Inserido { handle: Handle, terminal: Vec2, terminal_tag: &'static str },
}

pub(in crate::cmd::anacad) struct EquipM {
    pub(in crate::cmd::anacad) tipo: String,
    rotacao: Option<f64>,
    pendente: Option<Pendente>,
}

impl EquipM {
    pub(in crate::cmd::anacad) fn novo(tipo: String) -> Self {
        EquipM { tipo, rotacao: None, pendente: None }
    }
}

/// Onde o terminal `T1` do bloco vai cair com a inserção em `insercao` — a
/// mesma posição de que `terminal_do_equipamento` lê depois de inserir (o
/// `attdef` `T1` da definição do bloco, na escala do CFG), calculada antes:
/// é o que decide se o primeiro clique já tem barra ou linha para ligar.
pub(super) fn terminal_previsto(s: &mut Session, tipo: &str, insercao: Vec2, rotacao: f64) -> Result<Vec2> {
    terminal_previsto_tag(s, tipo, insercao, rotacao, "T1")
}

fn terminal_previsto_tag(s: &mut Session, tipo: &str, insercao: Vec2, rotacao: f64, tag: &str) -> Result<Vec2> {
    let (slug, _) = tipo_equipamento(tipo).ok_or_else(|| bad("anacad.equipamento.inserir", format!("tipo desconhecido: {tipo}")))?;
    let (db, _) = db_e_documento(s);
    let banco = anacad_data::Banco::abrir(&db).map_err(|e| bad("anacad.equipamento.inserir", format!("{db}: {e}")))?;
    let bloco = if let Some(custom) = bloco_personalizado(s, &banco, slug) { custom } else { bloco_equipamento(s, slug)? };
    let escala = s.doc().map(|doc| cfg_do_desenho(doc).equip_escala).unwrap_or(1.0);
    let deslocamento = s
        .doc()?
        .blocks
        .get(&bloco)
        .and_then(|b| {
            b.entities.iter().find_map(|e| match &e.kind {
                EntityKind::AttDef(attdef) if attdef.tag.eq_ignore_ascii_case(tag) => Some(Vec2::new(attdef.text.insert.x, attdef.text.insert.y)),
                _ => None,
            })
        })
        .or_else(|| (tag == "T1").then_some(Vec2::default()))
        .ok_or_else(|| bad("anacad.equipamento.inserir", format!("terminal {tag} nao encontrado no bloco")))?;
    Ok(insercao + deslocamento.rotate(rotacao) * escala)
}

impl Interactive for EquipM {
    fn name(&self) -> &'static str {
        "ANACAD_EQ"
    }
    fn begin(&mut self, _s: &mut Session) -> Result<Step> {
        self.rotacao = None;
        self.pendente = None;
        Ok(Step::Continue)
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        if self.rotacao.is_none() {
            Prompt::new("Ângulo de rotação do bloco", Accept::NUMBER).default("0")
        } else {
            match self.pendente.as_ref() {
                Some(Pendente::DoisTerminais { .. }) => Prompt::new(prompt_barra_terminal(&self.tipo, "T2"), Accept::POINT),
                Some(Pendente::Inserido { terminal_tag, .. }) if tem_dois_circuitos(&self.tipo) => {
                    Prompt::new(prompt_barra_terminal(&self.tipo, terminal_tag), Accept::POINT)
                }
                Some(Pendente::Ponto { .. }) if tem_dois_circuitos(&self.tipo) => Prompt::new(prompt_barra_terminal(&self.tipo, "T1"), Accept::POINT),
                Some(_) => Prompt::new("Clique na barra ou na linha do equipamento", Accept::POINT),
                None => Prompt::new("Ponto de inserção do equipamento", Accept::POINT),
            }
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        if self.rotacao.is_none() {
            let rot = match i {
                Input::Text(t) => {
                    let t = t.trim();
                    s.angle_settings()
                        .rotation(t)
                        .or_else(|| t.parse::<f64>().ok().map(f64::to_radians))
                        .filter(|v| v.is_finite())
                        .ok_or_else(|| crate::EngineError::Other("Ângulo inválido.".into()))?
                }
                Input::Enter => 0.0,
                _ => return Ok(Step::Continue),
            };
            self.rotacao = Some(rot);
            return Ok(Step::Continue);
        }

        let Input::Point(p) = i else { return Ok(Step::Continue) };
        let (db, documento) = db_e_documento(s);
        if db.is_empty() {
            return Err(crate::EngineError::Other("conecte um banco antes de inserir".into()));
        }
        let rotacao = self.rotacao.unwrap_or(0.0);
        // Seleções dos terminais pendentes são resolvidas antes de inserir os
        // equipamentos com dois circuitos.
        if let Some(pendente) = self.pendente.take() {
            return match pendente {
                Pendente::Ponto { insercao, terminal, rotacao: rot_insercao } if tem_dois_circuitos(&self.tipo) => {
                    let Some(alvo_t1) = alvo_perto_de_equipamento(s, &self.tipo, p).or_else(|| alvo_perto_de_equipamento(s, &self.tipo, terminal))
                    else {
                        self.pendente = Some(Pendente::Ponto { insercao, terminal, rotacao: rot_insercao });
                        return Err(crate::EngineError::Other("clique na barra do terminal 1".into()));
                    };
                    let terminal_t2 = terminal_previsto_tag(s, &self.tipo, insercao, rot_insercao, "T2")?;
                    self.pendente = Some(Pendente::DoisTerminais { insercao, rotacao: rot_insercao, alvo_t1, terminal_t2 });
                    s.echo(prompt_barra_terminal(&self.tipo, "T2").to_lowercase());
                    Ok(Step::Continue)
                }
                Pendente::DoisTerminais { insercao, rotacao, alvo_t1, terminal_t2 } => {
                    let Some(alvo_t2) = alvo_perto_de_equipamento(s, &self.tipo, p).or_else(|| alvo_perto_de_equipamento(s, &self.tipo, terminal_t2))
                    else {
                        self.pendente = Some(Pendente::DoisTerminais { insercao, rotacao, alvo_t1, terminal_t2 });
                        return Err(crate::EngineError::Other("clique na barra do terminal 2".into()));
                    };
                    let handle = inserir_equipamento_com_duas_barras(s, &db, documento, &self.tipo, insercao, rotacao, alvo_t1, alvo_t2)?;
                    st(s, |state| state.equip_handle = Some(handle.hex()));
                    s.set_selection(vec![handle]);
                    Ok(Step::Done)
                }
                Pendente::Ponto { insercao, terminal, rotacao: rot_insercao } => {
                    let Some(alvo) = alvo_perto(s, p, TOLERANCIA_BARRA).or_else(|| alvo_perto(s, terminal, TOLERANCIA_BARRA)) else {
                        self.pendente = Some(Pendente::Ponto { insercao, terminal, rotacao: rot_insercao });
                        return Err(crate::EngineError::Other("clique na barra ou na linha do equipamento".into()));
                    };
                    let saida = run_equipamento_inserir(
                        s,
                        &json!({
                            "db": db,
                            "documento": documento,
                            "tipo": self.tipo,
                            "x": insercao.x,
                            "y": insercao.y,
                            "rotacao": rot_insercao.to_degrees()
                        }),
                    )?;
                    // A barra ou a linha apareceu sob o bloco entre os dois
                    // cliques e a derivação já saiu com ele: nada a ligar.
                    if let Some(handle) = saida.get("handle").and_then(Value::as_str) {
                        st(s, |state| state.equip_handle = Some(handle.to_string()));
                    }
                    let Some(handle) = saida.get("handle").and_then(Value::as_str).and_then(Handle::parse_hex) else {
                        return Ok(Step::Done);
                    };
                    if saida.get("alvo").is_none() {
                        conectar_pendente(s, handle, alvo, "T1")?;
                    }
                    st(s, |state| state.equip_handle = Some(handle.hex()));
                    s.set_selection(vec![handle]);
                    Ok(Step::Done)
                }
                Pendente::Inserido { handle, terminal, terminal_tag } => {
                    let Some(alvo) = alvo_perto(s, p, TOLERANCIA_BARRA).or_else(|| alvo_perto(s, terminal, TOLERANCIA_BARRA)) else {
                        self.pendente = Some(Pendente::Inserido { handle, terminal, terminal_tag });
                        return Err(crate::EngineError::Other("clique na barra ou na linha do equipamento".into()));
                    };
                    conectar_pendente(s, handle, alvo, terminal_tag)?;
                    st(s, |state| state.equip_handle = Some(handle.hex()));
                    s.set_selection(vec![handle]);
                    Ok(Step::Done)
                }
            };
        }
        // Para equipamentos simples, a barra/linha sob o ponto ainda permite
        // ligar no primeiro clique. Nos de dois terminais, só ficam guardadas
        // as escolhas até o segundo alvo ser definido.
        let terminal = terminal_previsto(s, &self.tipo, p, rotacao)?;
        let tem_alvo = if tem_dois_circuitos(&self.tipo) {
            alvo_perto_de_equipamento(s, &self.tipo, terminal).is_some()
        } else {
            alvo_perto(s, p, TOLERANCIA_BARRA).is_some() || alvo_perto(s, terminal, TOLERANCIA_BARRA).is_some()
        };
        if !tem_alvo {
            self.pendente = Some(Pendente::Ponto { insercao: p, terminal, rotacao });
            let prompt = if tem_dois_circuitos(&self.tipo) {
                prompt_barra_terminal(&self.tipo, "T1")
            } else {
                "clique na barra ou na linha para ligar o equipamento"
            };
            s.echo(prompt.to_lowercase());
            return Ok(Step::Continue);
        }
        if tem_dois_circuitos(&self.tipo) {
            let Some(alvo_t1) = alvo_perto_de_equipamento(s, &self.tipo, terminal) else {
                return Err(crate::EngineError::Other("barra do terminal 1 nao encontrada".into()));
            };
            let terminal_t2 = terminal_previsto_tag(s, &self.tipo, p, rotacao, "T2")?;
            self.pendente = Some(Pendente::DoisTerminais { insercao: p, rotacao, alvo_t1, terminal_t2 });
            s.echo(prompt_barra_terminal(&self.tipo, "T2").to_lowercase());
            return Ok(Step::Continue);
        }
        let saida = run_equipamento_inserir(
            s,
            &json!({
                "db": db,
                "documento": documento,
                "tipo": self.tipo,
                "x": p.x,
                "y": p.y,
                "rotacao": rotacao.to_degrees()
            }),
        )?;
        // Barra ou linha sob o bloco: a derivação já saiu com ele.
        if saida.get("alvo").is_some() {
            if let Some(handle) = saida.get("handle").and_then(Value::as_str) {
                st(s, |state| state.equip_handle = Some(handle.to_string()));
            }
            return Ok(Step::Done);
        }
        // Não esperado: a inserção não achou o alvo que o primeiro clique
        // viu. O bloco já entrou; o segundo clique só completa a ligação.
        let Some(handle) = saida.get("handle").and_then(Value::as_str).and_then(Handle::parse_hex) else {
            return Ok(Step::Done);
        };
        let (terminal_tag, terminal) = terminal_do_equipamento(s, handle)?;
        self.pendente = Some(Pendente::Inserido { handle, terminal, terminal_tag });
        s.echo("clique na barra ou na linha para ligar o equipamento".to_string());
        Ok(Step::Continue)
    }
    fn preview(&self, _s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        let Some(terminal) = self.pendente.as_ref().map(|pendente| match pendente {
            Pendente::Ponto { terminal, .. } | Pendente::Inserido { terminal, .. } => *terminal,
            Pendente::DoisTerminais { terminal_t2, .. } => *terminal_t2,
        }) else {
            return Vec::new();
        };
        vec![EntityKind::LwPolyline(LwPolyline {
            vertices: vec![PolyVertex { p: terminal, ..PolyVertex::default() }, PolyVertex { p: cursor, ..PolyVertex::default() }],
            closed: false,
            const_width: 0.0,
            elevation: 0.0,
            plinegen: false,
        })]
    }
}

fn inserir_equipamento_com_duas_barras(
    s: &mut Session,
    db: &str,
    documento: i64,
    tipo: &str,
    insercao: Vec2,
    rotacao: f64,
    alvo_t1: Handle,
    alvo_t2: Handle,
) -> Result<Handle> {
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.equipamento.inserir", format!("{db}: {e}")))?;
    let terminal_t1 = terminal_previsto_tag(s, tipo, insercao, rotacao, "T1")?;
    let terminal_t2 = terminal_previsto_tag(s, tipo, insercao, rotacao, "T2")?;
    for (alvo, terminal) in [(alvo_t1, terminal_t1), (alvo_t2, terminal_t2)] {
        let (tipo_alvo, _) = vinculo_do_alvo(s, &banco, documento, &alvo.hex())?;
        if tipo_alvo != "barra" || ponto_do_alvo(s, alvo, terminal).is_none() {
            return Err(bad("anacad.equipamento.inserir", "os dois terminais precisam estar ligados a barras validas"));
        }
    }
    let saida = run_equipamento_inserir(
        s,
        &json!({
            "db": db,
            "documento": documento,
            "tipo": tipo,
            "x": insercao.x,
            "y": insercao.y,
            "rotacao": rotacao.to_degrees(),
            "auto_conectar": false
        }),
    )?;
    let handle = saida
        .get("handle")
        .and_then(Value::as_str)
        .and_then(Handle::parse_hex)
        .ok_or_else(|| bad("anacad.equipamento.inserir", "handle do equipamento inserido nao foi retornado"))?;
    conectar_pendente(s, handle, alvo_t1, "T1")?;
    conectar_pendente(s, handle, alvo_t2, "T2")?;
    Ok(handle)
}

/// Liga um equipamento já inserido à barra ou à linha escolhida, preservando
/// os demais vínculos existentes no `dados` dele.
pub(super) fn conectar_pendente(s: &mut Session, handle: Handle, alvo: Handle, terminal_tag: &'static str) -> Result<()> {
    let (db, documento) = db_e_documento(s);
    let banco = anacad_data::Banco::abrir(&db).map_err(|e| bad("anacad.equipamento.inserir", format!("{db}: {e}")))?;
    let mut dados = s
        .doc()?
        .entity(handle)
        .and_then(|e| campo_xdata(e, "dados"))
        .and_then(|texto| serde_json::from_str::<Value>(texto).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    ligar_ao_terminal(s, &banco, documento, handle, alvo, terminal_tag, &mut dados)?;
    run_equipamento_editar(s, &json!({ "db": db, "documento": documento, "handle": handle.hex(), "dados": dados }))?;
    Ok(())
}

#[cfg(test)]
#[path = "equipamento_tests.rs"]
mod tests;
