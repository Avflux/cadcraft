//! Utilitários de conversão (cores, tensões, JSON) compartilhados pelas janelas.

use super::*;

/// Cor vinda do `anacad.cfg.ler`: `"3"` (ACI) ou `"255,128,0"` (true color).
pub(super) fn cor_do_json(v: &Value, campo: &str) -> cadcraft_color::Color {
    let Some(c) = v.get(campo) else { return cadcraft_color::Color::Index(dominio::COR_PADRAO_ACI) };
    let texto = c.as_str().map(str::to_string).unwrap_or_else(|| c.to_string());
    cor_do_texto(&texto)
}

/// Número vindo do `anacad.cfg.ler` (altura do texto ou espessura do traço).
pub(super) fn f64_do_json(v: &Value, campo: &str, padrao: f64) -> f64 {
    v.get(campo).and_then(Value::as_f64).unwrap_or(padrao)
}

/// Cor a partir do texto guardado no desenho, no formato do `ANACAD_CFG`.
pub(super) fn cor_do_texto(texto: &str) -> cadcraft_color::Color {
    use cadcraft_color::Color;
    match Color::parse(texto) {
        Some(cor @ (Color::Index(_) | Color::True(_))) => cor,
        _ => Color::Index(dominio::COR_PADRAO_ACI),
    }
}

/// Texto da cor para o `anacad.cfg.gravar` (`"3"` ou `"255,128,0"`).
pub(super) fn texto_cor(cor: cadcraft_color::Color) -> String {
    use cadcraft_color::Color;
    match cor {
        Color::Index(i) => i.to_string(),
        Color::True(rgb) => format!("{},{},{}", rgb.0, rgb.1, rgb.2),
        _ => dominio::COR_PADRAO_ACI.to_string(),
    }
}

/// Botão de cor com o seletor do CAD (paleta ACI e true color), o equivalente
/// do `CColorDialog` do ANAFAS em &Ferramentas > Definir &Cores.
pub(super) fn botao_cor(ui: &mut Ui, id: &str, cor: &mut cadcraft_color::Color) {
    if let Some(nova) = crate::layers::color_button(ui, ui.id().with(id), *cor, false, 170.0) {
        *cor = nova;
    }
}

/// Bloco de um elemento (barra ou linha) do grupo **Texto e Traço de Barras e
/// Linhas**: tamanho e cor do texto de identificação e espessura do traço.
pub(super) fn estilo_elemento(ui: &mut Ui, rotulo: &str, id: &str, texto: &mut f64, cor: &mut cadcraft_color::Color, lar: &mut f64) {
    ui.label(egui::RichText::new(rotulo).strong());
    ui.horizontal(|ui| {
        ui.label("Tamanho do texto:");
        ui.add(egui::DragValue::new(texto).range(dominio::ALTURA_TEXTO.min..=dominio::ALTURA_TEXTO.max).speed(0.1));
        ui.label("Cor do texto:");
        botao_cor(ui, id, cor);
    });
    ui.horizontal(|ui| {
        ui.label("Espessura da linha:");
        ui.add(egui::DragValue::new(lar).range(dominio::LARGURA_BARRA.min..=dominio::LARGURA_BARRA.max).speed(0.05));
    });
}

/// Tensão como o ANAFAS escreve na tela (vírgula decimal).
pub(super) fn texto_tensao(tensao: f64) -> String {
    format!("{tensao}").replace('.', ",")
}

/// Grupos base de tensão do `anacad.cfg.ler`:
/// `[{ "tensao": 500.0, "cor": "255,0,0" }, …]` (tensão e cor, como o `G` do
/// `.lst`; a espessura do traço é a do estilo, uma só).
pub(super) fn grupos_do_json(v: &Value) -> Vec<(f64, cadcraft_color::Color)> {
    let grupos: Vec<(f64, cadcraft_color::Color)> = v
        .get("grupos")
        .and_then(Value::as_array)
        .map(|lista| {
            lista
                .iter()
                .filter_map(|item| {
                    let tensao = item.get("tensao").and_then(Value::as_f64)?;
                    let texto = item.get("cor").and_then(Value::as_str).unwrap_or("7");
                    Some((tensao, cor_do_texto(texto)))
                })
                .collect()
        })
        .unwrap_or_default();
    if grupos.is_empty() { grupos_padrao_ui() } else { grupos }
}

/// Grupos base de tensão padrão do projeto (`dominio::grupos_padrao`).
pub(super) fn grupos_padrao_ui() -> Vec<(f64, cadcraft_color::Color)> {
    dominio::grupos_padrao()
        .iter()
        .map(|grupo| (grupo.tensao, cadcraft_color::Color::True(cadcraft_color::Rgb(grupo.rgb.0, grupo.rgb.1, grupo.rgb.2))))
        .collect()
}

/// Janela dos diálogos do domínio, com tamanho inicial proporcional ao
/// diálogo do ANAFAS (1 DLU ≈ 1,5 px na horizontal e ≈ 1,6 px na vertical).
pub(super) fn com_altura<'a>(janela: egui::Window<'a>, altura: f32) -> egui::Window<'a> {
    if altura > 0.0 { janela.default_height(altura) } else { janela }
}

/// Tipo (`Transformador`, `Gerador`…) gravado no XData `ANACAD_EQ`.
pub(super) fn tipo_do_equipamento(app: &CadApp, handle: &str) -> String {
    dados_entidade(app, handle).and_then(|d| d.get("tipo").and_then(Value::as_str).map(str::to_string)).unwrap_or_default()
}

pub(super) fn resumir(v: &Value) -> String {
    let mut s = String::new();
    if let Some(docs) = v.get("documentos").and_then(Value::as_array) {
        s.push_str("Documentos:\n");
        for d in docs {
            let id = d.get("id").and_then(Value::as_i64).unwrap_or(0);
            let nome = d.get("nome").and_then(Value::as_str).unwrap_or("");
            s.push_str(&format!("  {id} - {nome}\n"));
        }
    }
    if let Some(c) = v.get("contagens") {
        s.push_str("Contagens: ");
        for chave in ["barras", "componentes", "linhas", "ligacoes", "legendas"] {
            if let Some(n) = c.get(chave).and_then(Value::as_i64) {
                s.push_str(&format!("{chave}={n} "));
            }
        }
        s.push('\n');
    }
    if let Some(comps) = v.get("componentes").and_then(Value::as_array) {
        s.push_str(&format!("Componentes ({} listados):\n", comps.len()));
        for c in comps.iter().take(12) {
            let id = c.get("id").and_then(Value::as_i64).unwrap_or(0);
            let tipo = c.get("tipo").and_then(Value::as_i64).unwrap_or(0);
            let x = c.get("x").and_then(Value::as_f64).unwrap_or(0.0);
            let y = c.get("y").and_then(Value::as_f64).unwrap_or(0.0);
            let h = c.get("handle").and_then(Value::as_str).unwrap_or("-");
            s.push_str(&format!("  #{id} tipo {tipo} em ({x:.0},{y:.0}) handle {h}\n"));
        }
    }
    if s.is_empty() {
        s.push_str("sem dados");
    }
    s
}
