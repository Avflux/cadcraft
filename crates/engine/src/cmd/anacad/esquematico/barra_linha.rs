//! Barra e linha: símbolo da barra fictícia (donut), edição, exclusão de dados e atribuição.

use super::*;

/// Medidas do donut da barra fictícia/auxiliar, na forma do comando DONUT do
/// CAD: diâmetro interno `0` (centro fechado) e diâmetro externo `2,0` — um
/// disco de raio `1,0`, desenhado como a polilinha fechada com bulge e
/// `const_width` do `donut` de `draw.rs`, dentro do bloco `ANACAD_DONUT`.
pub(in crate::cmd::anacad) const RAIO_DONUT: f64 = 1.0;
/// Nome do bloco que carrega o donut (a substituição da linha).
pub(in crate::cmd::anacad) const BLOCO_DONUT: &str = "ANACAD_DONUT";

/// Garante a definição do bloco `ANACAD_DONUT`: a geometria do comando DONUT
/// com diâmetro interno 0 e externo 2,0 (polilinha fechada de 2 vértices com
/// bulge 1 e largura 1,0, centrada na origem), como o `donut` de `draw.rs`.
pub(super) fn garantir_bloco_donut(s: &mut Session) -> Result<String> {
    let nome = BLOCO_DONUT;
    if s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(nome)) {
        return Ok(nome.to_string());
    }
    let doc = s.doc_mut()?;
    let mut bloco = Block::new(nome);
    bloco.description = "Donut da barra ficticia/auxiliar (anacad): diametro interno 0, externo 2,0".into();
    let h = doc.new_handle();
    let donut = EntityKind::LwPolyline(LwPolyline {
        vertices: vec![PolyVertex::with_bulge(Vec2::new(-RAIO_DONUT / 2.0, 0.0), 1.0), PolyVertex::with_bulge(Vec2::new(RAIO_DONUT / 2.0, 0.0), 1.0)],
        closed: true,
        const_width: RAIO_DONUT,
        elevation: 0.0,
        plinegen: false,
    });
    bloco.entities.push(Entity::new(h, donut));
    doc.blocks.insert(nome.to_string(), Arc::new(bloco));
    Ok(nome.to_string())
}

/// Barra fictícia ou auxiliar: no ANAFAS ela não é desenhada como linha —
/// vira um donut, que é como o transformador se liga a ela no desenho.
pub(super) fn barra_ficticia(dados: &Value) -> bool {
    matches!(dados.get("tipo").and_then(Value::as_str), Some("Fictícia de transformador") | Some("Auxiliar"))
}

/// Geometria da barra no desenho: o traço (LWPOLYLINE) ou, no donut, os
/// extremos originais guardados no XData (a rosca fica no centro deles).
pub(super) fn ler_geometria_barra(s: &Session, texto: &str) -> Result<(Handle, f64, f64, f64, f64)> {
    let handle = Handle::parse_hex(texto).ok_or_else(|| bad("anacad.barra.editar", "handle invalido"))?;
    let doc = s.doc()?;
    let entidade = doc.entity(handle).ok_or_else(|| bad("anacad.barra.editar", "entidade nao encontrada"))?;
    match &entidade.kind {
        EntityKind::LwPolyline(p) => {
            let a = p.vertices.first().ok_or_else(|| bad("anacad.barra.editar", "polilinha precisa de 2 vertices"))?;
            let b = p.vertices.last().unwrap_or(a);
            Ok((handle, a.p.x, a.p.y, b.p.x, b.p.y))
        }
        EntityKind::Circle(c) => {
            // Donut antigo (CIRCLE 8/4): os extremos da linha original estão
            // no XData, para a troca de volta para Normal restaurar o traço.
            let campo = |nome: &str| campo_xdata(entidade, nome).and_then(|v| v.parse::<f64>().ok());
            let (x1, y1, x2, y2) = match (campo("x1"), campo("y1"), campo("x2"), campo("y2")) {
                (Some(x1), Some(y1), Some(x2), Some(y2)) => (x1, y1, x2, y2),
                _ => (c.center.x, c.center.y, c.center.x, c.center.y),
            };
            Ok((handle, x1, y1, x2, y2))
        }
        EntityKind::Insert(ins) => {
            // Donut em bloco (`ANACAD_DONUT`): os extremos da linha original
            // estão no XData, como no CIRCLE antigo.
            let campo = |nome: &str| campo_xdata(entidade, nome).and_then(|v| v.parse::<f64>().ok());
            let (x1, y1, x2, y2) = match (campo("x1"), campo("y1"), campo("x2"), campo("y2")) {
                (Some(x1), Some(y1), Some(x2), Some(y2)) => (x1, y1, x2, y2),
                _ => (ins.insert.x, ins.insert.y, ins.insert.x, ins.insert.y),
            };
            Ok((handle, x1, y1, x2, y2))
        }
        _ => Err(bad("anacad.barra.editar", "entidade nao e barra")),
    }
}

/// Ponta da barra onde fica a identificação — e onde o donut entra, para
/// ficar junto do texto: no eixo horizontal é o extremo de menor `x`
/// (alinhado à esquerda) e no vertical é o de cima, a mesma escolha de
/// `posicao_texto_barra`, então independe do sentido em que a barra foi
/// desenhada (o "ponto inicial" do traço é essa ponta).
pub(in crate::cmd::anacad) fn ponta_do_texto(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    if dx.abs() < dy.abs() {
        if a.1 >= b.1 { a } else { b }
    } else if a.0 <= b.0 {
        a
    } else {
        b
    }
}

/// Troca a geometria da barra entre o traço e o donut, mantendo o mesmo
/// handle (textos, conexões e banco continuam apontando para ela): o donut
/// é o INSERT do bloco `ANACAD_DONUT` (diâmetros interno 0 / externo 2,0,
/// centro fechado) posicionado em `centro` — a ponta da identificação, junto
/// do texto —, substituindo a entidade no mesmo handle. Donuts antigos
/// (CIRCLE 8/4 com círculo interno no campo `donut` do XData) são migrados:
/// o interno sai junto e o CIRCLE vira o INSERT. Ao voltar para Normal, o
/// traço é restaurado. Devolve sempre `None` (o bloco não tem interno).
pub(in crate::cmd::anacad) fn trocar_simbolo_barra(
    s: &mut Session,
    handle: Handle,
    ficticia: bool,
    centro: (f64, f64),
    a: (f64, f64),
    b: (f64, f64),
    largura: f64,
) -> Result<Option<String>> {
    let kind_atual = s.doc().ok().and_then(|doc| doc.entity(handle)).map(|e| e.kind.clone());
    let eh_donut = matches!(&kind_atual, Some(EntityKind::Circle(_)))
        || matches!(&kind_atual, Some(EntityKind::Insert(ins)) if ins.block.eq_ignore_ascii_case(BLOCO_DONUT));
    if ficticia {
        let bloco = garantir_bloco_donut(s)?;
        // Migração: o círculo interno do donut 8/4 antigo sai junto.
        remover_donut_interno(s, handle);
        s.doc_mut()?.modify_entity(handle, |e| {
            e.kind = EntityKind::Insert(Insert {
                block: bloco.clone(),
                insert: Vec3::new(centro.0, centro.1, 0.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                rotation: 0.0,
                attribs: Vec::new(),
                cols: 1,
                rows: 1,
                col_spacing: 0.0,
                row_spacing: 0.0,
            });
        })?;
        return Ok(None);
    }
    if eh_donut {
        remover_donut_interno(s, handle);
        s.doc_mut()?.modify_entity(handle, |e| {
            e.kind = EntityKind::LwPolyline(LwPolyline {
                vertices: vec![
                    PolyVertex { p: Vec2::new(a.0, a.1), ..PolyVertex::default() },
                    PolyVertex { p: Vec2::new(b.0, b.1), ..PolyVertex::default() },
                ],
                closed: false,
                const_width: largura,
                elevation: 0.0,
                plinegen: false,
            });
        })?;
    }
    Ok(None)
}

/// Remove o círculo interno do donut 8/4 antigo da barra, se houver (o donut
/// em bloco não tem interno; histórico para migrar desenhos antigos).
pub(in crate::cmd::anacad) fn remover_donut_interno(s: &mut Session, handle: Handle) {
    let texto = s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, "donut")).map(str::to_string);
    if let Some(texto) = texto
        && let Some(interno) = Handle::parse_hex(&texto)
        && let Ok(doc) = s.doc_mut()
    {
        doc.remove_entity(interno);
    }
}

pub(in crate::cmd::anacad) fn run_editar_barra(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.editar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.editar", "`handle` is required"))?;
    let (handle, x1, y1, x2, y2) = ler_geometria_barra(s, handle_txt)?;
    let lar = lar_da_barra(s);
    let dados = p.get("dados").cloned().unwrap_or_else(|| json!({}));
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.barra.editar", format!("{db}: {e}")))?;
    let codigo = p
        .get("codigo")
        .and_then(Value::as_i64)
        .or_else(|| dados.get("codigo").and_then(Value::as_i64))
        .ok_or_else(|| bad("anacad.barra.editar", "`codigo` is required"))?;
    let nome = dados.get("nome").and_then(Value::as_str);
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
    // Barra fictícia ou auxiliar vira donut no desenho, no lugar da linha;
    // voltar para Normal desfaz a troca (o handle e o XData ficam os mesmos).
    // O donut entra na ponta da identificação (o ponto inicial da barra),
    // junto do texto, e não no meio do traço.
    let centro = ponta_do_texto((x1, y1), (x2, y2));
    let donut = trocar_simbolo_barra(s, handle, barra_ficticia(&dados), centro, (x1, y1), (x2, y2), lar)?;
    // `marcar` regrava o XData inteiro: sem o `vbase` aqui, editar a janela
    // apagava a base de tensão do desenho (o banco ficava com ela) e o
    // recadastro do Sincronizar a perdia.
    let vbase_txt = dados
        .get("vbase")
        .and_then(Value::as_f64)
        .map(|v| v.to_string())
        .or_else(|| s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, "vbase")).map(str::to_string))
        .unwrap_or_default();
    let id_txt = id.to_string();
    let codigo_txt = codigo.to_string();
    let x1_txt = x1.to_string();
    let y1_txt = y1.to_string();
    let x2_txt = x2.to_string();
    let y2_txt = y2.to_string();
    let lar_txt = lar.to_string();
    let mut campos: Vec<(&str, &str)> = vec![
        ("id", id_txt.as_str()),
        ("codigo", codigo_txt.as_str()),
        ("x1", x1_txt.as_str()),
        ("y1", y1_txt.as_str()),
        ("x2", x2_txt.as_str()),
        ("y2", y2_txt.as_str()),
        ("largura", lar_txt.as_str()),
        ("vbase", vbase_txt.as_str()),
        ("dados", texto.as_str()),
    ];
    if let Some(donut) = &donut {
        campos.push(("donut", donut.as_str()));
    }
    marcar(s, handle, "ANACAD_BARRA", &campos)?;
    // A identificação no desenho acompanha a alteração: nome (identificador),
    // número e base de tensão, reposicionados com a barra.
    sincronizar_textos_barra(s, handle, codigo, nome, Vec2::new(x1, y1), Vec2::new(x2, y2))?;
    Ok(json!({ "barra": id, "handle": handle.hex(), "dados": dados, "donut": donut.is_some() }))
}

pub(in crate::cmd::anacad) fn run_editar_linha(s: &mut Session, p: &Value) -> Result<Value> {
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
    let anterior =
        |campo: &str| -> Option<String> { s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, campo)).map(str::to_string) };
    let de_txt = barra_de.as_ref().and_then(|b| b.codigo).map(|c| c.to_string()).or_else(|| anterior("de"));
    let para_txt = barra_para.as_ref().and_then(|b| b.codigo).map(|c| c.to_string()).or_else(|| anterior("para"));
    let vbase_txt = barra_de.as_ref().and_then(|b| b.vbase).map(|v| v.to_string()).or_else(|| anterior("vbase")).unwrap_or_default();
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
pub(in crate::cmd::anacad) fn run_barra_dados_excluir(s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.barra.dados.excluir", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let handle_txt = str_param(p, "handle").ok_or_else(|| bad("anacad.barra.dados.excluir", "`handle` is required"))?;
    // A barra pode estar como donut (CIRCLE antigo ou INSERT do bloco): a
    // geometria de referência são os extremos no XData.
    let (handle, x1, y1, x2, y2) = ler_geometria_barra(s, handle_txt)?;
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
pub(in crate::cmd::anacad) fn run_linha_dados_excluir(s: &mut Session, p: &Value) -> Result<Value> {
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
    let anterior =
        |campo: &str| -> Option<String> { s.doc().ok().and_then(|doc| doc.entity(handle)).and_then(|e| campo_xdata(e, campo)).map(str::to_string) };
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

pub(in crate::cmd::anacad) fn run_atribuir_barra(s: &mut Session, p: &Value) -> Result<Value> {
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
    let vbase_txt = banco.barra_por_codigo(documento, codigo).ok().flatten().and_then(|b| b.vbase).map(|v| v.to_string()).unwrap_or_default();
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

pub(in crate::cmd::anacad) fn run_atribuir_linha(s: &mut Session, p: &Value) -> Result<Value> {
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
