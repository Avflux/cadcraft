//! Conexões: criar, validar, topologia, comparar e sincronizar equipamentos.

use super::*;

pub(in crate::cmd::anacad) fn run_conexao_criar(s: &mut Session, p: &Value) -> Result<Value> {
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
            APP_CONEXAO,
            &[("de", de_txt), ("para", para_txt), ("de_terminal", &de_term), ("para_terminal", &para_term), ("potencial", &potencial_txt)],
        )?;
    }
    Ok(json!({"conexao": id, "handle": handle.map(|h| h.hex())}))
}

pub(in crate::cmd::anacad) fn run_conexao_validar(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_conexao_topologia(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_equipamento_mover(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_equipamento_transformar(s: &mut Session, p: &Value) -> Result<Value> {
    let texto = str_param(p, "handle").ok_or_else(|| bad("anacad.equipamento.transformar", "`handle` is required"))?;
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.equipamento.transformar", "handle invalido"))?;
    let (old_rot, old_scale, insert, is_transformador) = {
        let ent = s.doc()?.entity(handle).ok_or_else(|| bad("anacad.equipamento.transformar", "entidade nao encontrada"))?;
        let EntityKind::Insert(ins) = &ent.kind else { return Err(bad("anacad.equipamento.transformar", "nao e um bloco")) };
        (ins.rotation, ins.scale.x, (ins.insert.x, ins.insert.y), anacad_app::e_bloco_transformador(&ins.block))
    };
    let rotacao_pedida = p.get("rotacao").and_then(Value::as_f64).map(f64::to_radians).unwrap_or(old_rot);
    // A referência, os dois enrolamentos e seus terminais giram juntos; os
    // rótulos do transformador são reposicionados para permanecerem legíveis.
    let nova_rot = rotacao_pedida;
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
    if is_transformador {
        let dados = s
            .doc()?
            .entity(handle)
            .and_then(|entidade| campo_xdata(entidade, "dados"))
            .and_then(|texto| serde_json::from_str::<Value>(texto).ok())
            .unwrap_or_else(|| json!({}));
        let novo = bloco_do_transformador_orientado(s, &dados, nova_rot.to_degrees())?;
        let atual = s.doc()?.entity(handle).and_then(|entidade| match &entidade.kind {
            EntityKind::Insert(ins) => Some(ins.block.clone()),
            _ => None,
        });
        if atual.as_deref() != Some(&novo) {
            bloco_byblock(s.doc_mut()?, &novo);
            s.doc_mut()?.modify_entity(handle, |entidade| {
                if let EntityKind::Insert(ins) = &mut entidade.kind {
                    ins.block = novo.clone();
                }
            })?;
        }
    }
    if is_transformador {
        posicionar_rotulos_transformador(s, handle, Vec2::new(insert.0, insert.1), nova_escala, nova_rot.to_degrees())?;
    }
    Ok(json!({"handle": handle.hex(), "rotacao": nova_rot.to_degrees(), "escala": nova_escala, "insert": insert}))
}

pub(in crate::cmd::anacad) fn run_equipamento_sincronizar(s: &mut Session, p: &Value) -> Result<Value> {
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

pub(in crate::cmd::anacad) fn run_equipamento_sincronizar_conexoes(s: &mut Session, p: &Value) -> Result<Value> {
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
