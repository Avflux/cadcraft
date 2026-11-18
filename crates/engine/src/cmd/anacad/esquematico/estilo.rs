//! Estilo do desenho: espessura, cor por grupo de tensão e consulta de XData.

use super::*;

/// Espessura do traço da barra configurada no projeto (0,3 sem configuração).
pub(super) fn lar_da_barra(s: &Session) -> f64 {
    s.doc().map(|doc| cfg_do_desenho(doc).barra_lar).unwrap_or(0.3)
}

/// Espessura do traço da linha configurada no projeto (0 = fio).
pub(super) fn lar_da_linha(s: &Session) -> f64 {
    s.doc().map(|doc| cfg_do_desenho(doc).linha_lar).unwrap_or(0.0)
}

/// Cor do grupo base de tensão de um elemento (a barra, a linha e o componente
/// entram na cor do nível de tensão deles, como no ANAFAS).
pub(super) fn cor_da_tensao(s: &Session, tensao: f64) -> Option<Color> {
    let cfg = s.doc().map(cfg_do_desenho).unwrap_or_default();
    cor_do_grupo(&cfg, tensao)
}

/// Cor configurada para uma tensão base (igualdade com folga de 0,5%).
pub(super) fn cor_do_grupo(cfg: &CfgProjeto, tensao: f64) -> Option<Color> {
    cfg.grupos.iter().find(|grupo| mesma_tensao(grupo.tensao, tensao)).map(|grupo| grupo.cor)
}

/// Pinta a entidade recém-criada com a cor do grupo.
pub(super) fn colorir(s: &mut Session, handle: Handle, cor: Color) -> Result<()> {
    s.doc_mut()?.modify_entity(handle, |e| e.common.color = cor)?;
    Ok(())
}

/// Deixa as entidades do bloco do tipo com a cor da referência (`ByBlock`),
/// para cada instância ter a cor do grupo de tensão da barra dela.
pub(super) fn bloco_byblock(doc: &mut cadcraft_doc::Drawing, nome: &str) {
    let Some(bloco) = doc.blocks.get_mut(nome) else { return };
    let bloco = std::sync::Arc::make_mut(bloco);
    for handle in bloco.entities.handles() {
        bloco.entities.modify(handle, |e| e.common.color = Color::ByBlock);
    }
}

/// Grupo base de tensão gravado no `ANACAD_CFG` (formato em
/// `anacad_app::grupo_do_texto`).
pub(super) fn grupo_do_texto(texto: &str) -> Option<GrupoCor> {
    let (tensao, cor) = anacad_app::grupo_do_texto(texto)?;
    Some(GrupoCor { tensao, cor: cor_do_texto(cor) })
}

/// Texto do grupo no `ANACAD_CFG`.
pub(super) fn texto_grupo(grupo: &GrupoCor) -> String {
    anacad_app::texto_grupo(grupo.tensao, &texto_cor(grupo.cor))
}

/// O desenho tem cores de grupos gravadas no `ANACAD_CFG`?
pub(super) fn tem_grupos_no_desenho(doc: &cadcraft_doc::Drawing) -> bool {
    entidades_cfg(doc).iter().any(|handle| {
        doc.entity(*handle).is_some_and(|e| {
            let pares: Vec<&str> = e.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
            pares.chunks(2).any(|par| par.len() == 2 && par[0] == "grupo")
        })
    })
}

/// App name do XData de um elemento (`None` para entidade sem XData).
pub(in crate::cmd::anacad) fn app_xdata(e: &cadcraft_doc::Entity) -> Option<&str> {
    e.common.xdata.first().filter(|t| t.code == 1001).map(|t| t.value.as_str())
}

/// Valor de um campo do XData do elemento.
pub(in crate::cmd::anacad) fn campo_xdata<'a>(e: &'a cadcraft_doc::Entity, nome: &str) -> Option<&'a str> {
    let pares: Vec<&str> = e.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
    pares.chunks(2).find(|par| par.len() == 2 && par[0] == nome).map(|par| par[1])
}

/// Tensão base (`VBase`) gravada na barra do desenho: o par `vbase` ou o `dados`
/// da janela Dados de Barra.
pub(super) fn tensao_da_barra(e: &cadcraft_doc::Entity) -> Option<f64> {
    if let Some(vbase) = campo_xdata(e, "vbase").and_then(|v| v.parse::<f64>().ok()) {
        return Some(vbase);
    }
    let dados: Value = serde_json::from_str(campo_xdata(e, "dados")?).ok()?;
    dados.get("vbase").or_else(|| dados.get("tensao")).and_then(Value::as_f64)
}

/// Barras do desenho com tensão base conhecida.
pub(super) fn barras_com_tensao(doc: &cadcraft_doc::Drawing) -> Vec<(Handle, f64)> {
    doc.model.iter().filter(|e| app_xdata(e) == Some("ANACAD_BARRA")).filter_map(|e| tensao_da_barra(e).map(|tensao| (e.handle, tensao))).collect()
}

/// Folga (unidades do desenho) para o componente herdar a barra em que foi
/// inserido.
pub(super) const TOLERANCIA_BARRA: f64 = 5.0;

/// Distância de um ponto ao segmento `a`–`b`.
pub(super) fn distancia_ao_segmento(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let comprimento = dx * dx + dy * dy;
    let t = if comprimento > 0.0 { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / comprimento).clamp(0.0, 1.0) } else { 0.0 };
    let (x, y) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - x).powi(2) + (p.1 - y).powi(2)).sqrt()
}

/// Tensão base da barra mais próxima de um ponto: o componente herda a cor do
/// grupo da barra em que foi inserido, como o campo `Cor` do `C` no `.lst`.
pub(super) fn tensao_no_ponto(doc: &cadcraft_doc::Drawing, x: f64, y: f64, tolerancia: f64) -> Option<f64> {
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
pub(super) fn codigo_da_barra_no_ponto(doc: &cadcraft_doc::Drawing, x: f64, y: f64) -> Option<i64> {
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

/// Grupos base de tensão do documento do projeto, na cor do desenho.
pub(super) fn grupos_do_documento(cfg: &CfgProjeto) -> Option<Vec<GrupoCor>> {
    Some(anacad_app::grupos_do_documento(&cfg.banco, cfg.documento)?.iter().map(grupo_cor).collect())
}

/// Grava os grupos base de tensão do CFG no documento do banco (o `G` do `.lst`).
pub(super) fn gravar_grupos_no_documento(cfg: &CfgProjeto) {
    let grupos: Vec<anacad_app::GrupoTensao> =
        cfg.grupos.iter().map(|grupo| anacad_app::GrupoTensao { tensao: grupo.tensao, rgb: rgb_da_cor(grupo.cor) }).collect();
    anacad_app::gravar_grupos_no_documento(&cfg.banco, cfg.documento, &grupos);
}

/// RGB de uma cor do desenho (índice ACI pela paleta do CAD, true color direto).
pub(super) fn rgb_da_cor(cor: Color) -> (u8, u8, u8) {
    let rgb = cor.resolve(Color::Index(7), Color::Index(7));
    (rgb.0, rgb.1, rgb.2)
}

/// Repinta o que já está desenhado com a cor do grupo de tensão de cada
/// elemento: a barra pela `VBase`, a linha pela barra de origem e o componente
/// pela barra em que foi inserido.
pub(super) fn repintar_por_tensao(s: &mut Session, cfg: &CfgProjeto) -> Result<()> {
    let banco = if cfg.banco.trim().is_empty() { None } else { anacad_data::Banco::abrir(&cfg.banco).ok() };
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
                    let tensao = campo_xdata(e, "vbase").and_then(|v| v.parse::<f64>().ok()).or_else(|| {
                        de.and_then(|de| banco.as_ref().and_then(|b| b.barra_por_codigo(documento, de).ok().flatten())).and_then(|barra| barra.vbase)
                    });
                    if let Some(tensao) = tensao {
                        alvos.push((e.handle, tensao));
                    }
                }
                Some("ANACAD_EQ") => {
                    let (Some(x), Some(y)) =
                        (campo_xdata(e, "x").and_then(|v| v.parse::<f64>().ok()), campo_xdata(e, "y").and_then(|v| v.parse::<f64>().ok()))
                    else {
                        continue;
                    };
                    let Some(tensao) = tensao_no_ponto(doc, x, y, TOLERANCIA_BARRA) else { continue };
                    // O bloco da própria referência: o transformador usa a
                    // variante da ligação dos enrolamentos.
                    match &e.kind {
                        EntityKind::Insert(ins) if ins.block.starts_with(PREFIXO_BLOCO_EQUIPAMENTO) => {
                            blocos.push((ins.block.clone(), e.handle, tensao))
                        }
                        _ => alvos.push((e.handle, tensao)),
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
