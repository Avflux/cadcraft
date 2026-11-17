//! Gerenciador de Dados: árvore de tabelas e grade do banco.

use super::*;

/// Árvore do **Gerenciador de Dados** no ANAFAS (menu Dados): `Rede CA` com os
/// elementos da rede. A chave é o item lido do banco (a lista
/// `anacad_data::ITENS_GERENCIADOR`) e o rótulo é o do ANAFAS — os dados do
/// caso, não as tabelas do `.db`.
pub(super) const GERENCIADOR_REDE: &[(&str, &str, crate::icons::Icon)] = &[
    ("Barra", "Barra", crate::icons::Icon::AnacadBarra),
    ("Linha", "Linha de Transmissão", crate::icons::Icon::AnacadLinha),
    ("Transformador", "Transformador", crate::icons::Icon::AnacadTransformador),
    ("CapacitorSerie", "Capacitor Série", crate::icons::Icon::AnacadCompensadorSerie),
    ("Gerador", "Gerador", crate::icons::Icon::AnacadGerador),
    ("GeradorEolico", "Gerador Eólico", crate::icons::Icon::AnacadGerador),
    ("ShuntBarra", "Shunt de Barra", crate::icons::Icon::AnacadShunt),
    ("ShuntLinha", "Shunt de Linha", crate::icons::Icon::AnacadShuntLinha),
    ("TrafoAterramento", "Transformador de Aterramento", crate::icons::Icon::AnacadTrafoAterramento),
    ("CompensadorEstatico", "Compensador Estático", crate::icons::Icon::AnacadCompensadorEstatico),
    ("Mutua", "Impedância Mútua", crate::icons::Icon::AnacadMutua),
    ("Carga", "Carga", crate::icons::Icon::AnacadCarga),
];

/// Raiz da árvore (`Rede CA`), que no ANAFAS também é selecionável (a grade
/// mostra a quantidade de elementos), e `Área`, que fica ao lado dela.
pub(super) const GERENCIADOR_RAIZ: (&str, &str, crate::icons::Icon) = ("Rede", "Quantidade", crate::icons::Icon::AnacadRede);

pub(super) const GERENCIADOR_FORA: &[(&str, &str, crate::icons::Icon)] = &[("Area", "Área", crate::icons::Icon::Layers)];

/// Carrega o item escolhido no Gerenciador de Dados: os campos da janela
/// `Dados de …` do elemento e até 500 registros do documento ativo. Quem lê o
/// banco é o motor (`anacad.dados.tabela`).
pub(super) fn carregar_tabela(app: &mut CadApp, estado: &mut Estado) {
    let pedido = json!({
        "db": estado.db,
        "documento": estado.documento,
        "item": estado.gd_tabela,
        "limite": 500,
    });
    estado.gd_colunas.clear();
    estado.gd_linhas.clear();
    estado.gd_erro.clear();
    match app.run("anacad.dados.tabela", pedido) {
        Ok(v) => {
            estado.gd_colunas = v
                .get("colunas")
                .and_then(Value::as_array)
                .map(|colunas| colunas.iter().filter_map(|c| c.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            estado.gd_linhas = v
                .get("linhas")
                .and_then(Value::as_array)
                .map(|linhas| {
                    linhas
                        .iter()
                        .map(|linha| {
                            linha
                                .as_array()
                                .map(|celulas| celulas.iter().map(|c| c.as_str().unwrap_or_default().to_string()).collect())
                                .unwrap_or_default()
                        })
                        .collect()
                })
                .unwrap_or_default();
        }
        Err(e) => estado.gd_erro = format!("erro: {e}"),
    }
}

// Gerenciador de Dados (333/336): a árvore de tabelas do ANAFAS
// vira a coluna de ícones da esquerda e a grade dos registros
// fica à direita. Os registros vêm do `.db` conectado, nunca do
// desenho.
/// Janela `gerenciador` do diálogo AnaCAD.
pub(super) fn janela_gerenciador(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool) {
    let chave = format!("{}|{}|{}", estado.db, estado.documento, estado.gd_tabela);
    if estado.gd_em != chave {
        carregar_tabela(app, estado);
        estado.gd_em = chave;
    }
    let mut escolhida: Option<&'static str> = None;
    let mut recarregar = false;
    // Só a coluna dos ícones rola; a grade ocupa o resto.
    let altura_icones = (ui.available_height() - 44.0).max(120.0);
    ui.horizontal_top(|ui| {
        egui::ScrollArea::vertical().id_salt("anacad.gerenciador.tabelas").max_height(altura_icones).auto_shrink([true, false]).show(ui, |ui| {
            // Folga à direita dos ícones: a barra de rolagem
            // fica fora do alcance dos cliques na árvore.
            ui.set_min_width(64.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                // Raiz da árvore do ANAFAS, com os elementos
                // da rede; depois vêm os itens de fora.
                let (item, _, icon) = GERENCIADOR_RAIZ;
                if crate::icons::button(ui, icon, 30.0, "Rede CA", estado.gd_tabela == item).clicked() {
                    escolhida = Some(item);
                }
                ui.indent("anacad.gerenciador.rede", |ui| {
                    for (item, rotulo, icon) in GERENCIADOR_REDE {
                        let sel = estado.gd_tabela == *item;
                        if crate::icons::button(ui, *icon, 30.0, rotulo, sel).clicked() {
                            escolhida = Some(*item);
                        }
                    }
                });
                for (item, rotulo, icon) in GERENCIADOR_FORA {
                    let sel = estado.gd_tabela == *item;
                    if crate::icons::button(ui, *icon, 30.0, rotulo, sel).clicked() {
                        escolhida = Some(*item);
                    }
                }
            });
        });
        ui.separator();
        ui.vertical(|ui| {
            if let Some(nome) = escolhida
                && estado.gd_tabela != nome
            {
                estado.gd_tabela = nome.to_string();
            }
            let raiz = GERENCIADOR_RAIZ;
            let rotulo = GERENCIADOR_REDE
                .iter()
                .chain(GERENCIADOR_FORA.iter())
                .chain(std::iter::once(&(raiz.0, "Rede CA", raiz.2)))
                .find(|(item, _, _)| *item == estado.gd_tabela)
                .map_or("", |(_, rotulo, _)| *rotulo);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(rotulo).strong());
                ui.weak(format!("{} registros", estado.gd_linhas.len()));
                if ui.button("Atualizar").clicked() {
                    recarregar = true;
                }
            });
            ui.separator();
            let altura_grade = (ui.available_height() - 40.0).max(120.0);
            egui::ScrollArea::both().id_salt("anacad.gerenciador.grade").max_height(altura_grade).auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("anacad.gerenciador.linhas").striped(true).min_col_width(48.0).show(ui, |ui| {
                    for coluna in &estado.gd_colunas {
                        ui.label(egui::RichText::new(coluna).strong());
                    }
                    ui.end_row();
                    for linha in &estado.gd_linhas {
                        for valor in linha {
                            ui.label(valor);
                        }
                        ui.end_row();
                    }
                });
                if estado.gd_linhas.is_empty() && estado.gd_erro.is_empty() {
                    ui.add_space(8.0);
                    ui.weak(format!("Nenhum registro em {rotulo}."));
                }
            });
            if !estado.gd_erro.is_empty() {
                ui.label(&estado.gd_erro);
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Fechar").clicked() {
                    *fechar = true;
                }
            });
        });
    });
    if recarregar {
        estado.gd_em.clear();
    }
}
