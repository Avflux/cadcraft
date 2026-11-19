//! Blocos dos equipamentos (`ANACAD_EQ_*`): o desenho vem pronto do
//! `anacad-app` (`simbolo_equipamento`, `simbolo_transformador`) como
//! primitivas neutras, e aqui vira a definição de bloco do desenho.

use super::*;
use anacad_app::{Alinhamento, Primitiva, Simbolo};

/// Bloco do equipamento `slug`, criado no desenho na primeira inserção.
pub(super) fn bloco_equipamento(s: &mut Session, slug: &str) -> Result<String> {
    garantir_bloco(s, &anacad_app::nome_bloco_equipamento(slug), || anacad_app::simbolo_equipamento(slug))
}

/// Bloco que o transformador deve usar para o `dados` dele: a variante da
/// ligação de cada enrolamento.
/// Variante do bloco que mantém as marcas de ligação na orientação do desenho.
pub(super) fn bloco_do_transformador_orientado(s: &mut Session, dados: &Value, angulo_graus: f64) -> Result<String> {
    let (primario, secundario) = anacad_app::ligacoes_do_transformador(dados);
    let simbolo = anacad_app::simbolo_transformador_orientado(&primario, &secundario, angulo_graus);
    let nome = simbolo.nome.clone();
    garantir_bloco(s, &nome, || simbolo)
}

/// Cria o bloco `nome` com o símbolo de `simbolo` se o desenho ainda não o tem;
/// a definição existente é substituída pela geometria atual, para corrigir
/// também blocos criados antes da orientação horizontal.
fn garantir_bloco(s: &mut Session, nome: &str, simbolo: impl FnOnce() -> Simbolo) -> Result<String> {
    let atualizar_horizontal = nome.eq_ignore_ascii_case("ANACAD_EQ_transformador")
        || nome.starts_with("ANACAD_EQ_transformador_")
        || nome.eq_ignore_ascii_case("ANACAD_EQ_compensador_serie");
    if s.doc()?.blocks.keys().any(|k| k.eq_ignore_ascii_case(nome)) && !atualizar_horizontal {
        return Ok(nome.to_string());
    }
    let simbolo = simbolo();
    let doc = s.doc_mut()?;
    let mut bloco = Block::new(&simbolo.nome);
    bloco.description = simbolo.descricao;
    for primitiva in &simbolo.primitivas {
        let h = doc.new_handle();
        bloco.entities.push(Entity::new(h, entidade(primitiva)));
    }
    doc.blocks.insert(simbolo.nome.clone(), Arc::new(bloco));
    Ok(simbolo.nome)
}

fn v3((x, y): (f64, f64)) -> Vec3 {
    Vec3::new(x, y, 0.0)
}

fn vertice(p: (f64, f64)) -> PolyVertex {
    PolyVertex { p: Vec2::new(p.0, p.1), ..PolyVertex::default() }
}

fn polilinha(vertices: Vec<PolyVertex>, closed: bool, const_width: f64) -> EntityKind {
    EntityKind::LwPolyline(LwPolyline { vertices, closed, const_width, elevation: 0.0, plinegen: false })
}

fn texto(em: (f64, f64), valor: &str, altura: f64, alinhamento: Alinhamento) -> Text {
    let (halign, valign) = match alinhamento {
        Alinhamento::Base => (HAlign::Left, VAlign::Baseline),
        Alinhamento::CentroTopo => (HAlign::Center, VAlign::Top),
        Alinhamento::Centro => (HAlign::Center, VAlign::Middle),
        Alinhamento::CentroBase => (HAlign::Center, VAlign::Baseline),
    };
    Text {
        insert: v3(em),
        align_pt: (alinhamento != Alinhamento::Base).then(|| v3(em)),
        height: altura,
        value: valor.to_string(),
        rotation: 0.0,
        width_factor: 1.0,
        oblique: 0.0,
        style: "Standard".to_string(),
        halign,
        valign,
    }
}

/// A entidade do desenho de uma primitiva do símbolo.
fn entidade(primitiva: &Primitiva) -> EntityKind {
    match primitiva {
        Primitiva::Traco { pontos, fechado } => polilinha(pontos.iter().copied().map(vertice).collect(), *fechado, 0.0),
        Primitiva::Circulo { centro, raio } => EntityKind::Circle(Circle { center: v3(*centro), radius: *raio }),
        Primitiva::Arco { centro, raio, de, ate } => {
            EntityKind::Arc(cadcraft_doc::Arc { center: v3(*centro), radius: *raio, start: de.to_radians(), end: ate.to_radians() })
        }
        Primitiva::Elipse { centro, a, razao, de, ate } => {
            EntityKind::Ellipse(cadcraft_doc::Ellipse { center: v3(*centro), major: Vec3::new(*a, 0.0, 0.0), ratio: *razao, start: *de, end: *ate })
        }
        // O SOLID do DXF percorre os cantos em zigue-zague (1, 2, 4, 3).
        Primitiva::Solido { cantos } => {
            let c = |i: usize| cantos.get(i).or(cantos.last()).copied().map(v3).unwrap_or_default();
            let corners = if cantos.len() >= 4 { [c(0), c(1), c(3), c(2)] } else { [c(0), c(1), c(2), c(2)] };
            EntityKind::Solid(cadcraft_doc::Solid { corners })
        }
        Primitiva::PontoCheio { centro: (x, y), diametro } => {
            let r = diametro / 2.0;
            let vertices = vec![PolyVertex { bulge: 1.0, ..vertice((*x, y + r)) }, PolyVertex { bulge: 1.0, ..vertice((*x, y - r)) }];
            polilinha(vertices, true, *diametro)
        }
        Primitiva::Texto { em, valor, altura, alinhamento } => EntityKind::Text(texto(*em, valor, *altura, *alinhamento)),
        Primitiva::Atributo { tag, prompt, padrao, em, alinhamento, invisivel } => EntityKind::AttDef(Attrib {
            tag: tag.clone(),
            // A altura vale até a inserção, que aplica a do CFG.
            text: texto(*em, padrao, 2.0, *alinhamento),
            invisible: *invisivel,
            constant: false,
            prompt: prompt.clone(),
            props: Default::default(),
            multiline: None,
            modes: Default::default(),
        }),
    }
}

#[cfg(test)]
#[path = "simbolos_tests.rs"]
mod tests;
