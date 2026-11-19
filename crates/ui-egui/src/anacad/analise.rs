//! Janelas de análise (curto-circuito, equivalente, Zbarra, níveis, impedância, superação).

use super::*;

/// Janela `equivalente` do diálogo AnaCAD.
pub(super) fn janela_equivalente(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.label("Barras a manter (códigos separados por vírgula):");
    ui.add(egui::TextEdit::singleline(&mut estado.eqv_barras).desired_width(420.0));
    ui.checkbox(&mut estado.eqv_validar, "validar contra a rede original");
    if ui.button("Calcular equivalente").clicked() {
        let barras: Vec<i64> =
            estado.eqv_barras.split(|c: char| c == ',' || c == ';' || c.is_whitespace()).filter_map(|t| t.trim().parse::<i64>().ok()).collect();
        match app.run("anacad.equivalente", json!({"db": estado.db, "documento": estado.documento, "barras": barras, "validar": estado.eqv_validar}))
        {
            Ok(v) => {
                estado.eqv_resumo = format!(
                    "barras: {:?}\npares não nulos: {}\nsimétrica: {}\nmaior diagonal: {:.4}",
                    v.get("barras").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>()).unwrap_or_default(),
                    v.get("pares_nao_nulos").and_then(Value::as_i64).unwrap_or(0),
                    v.get("simetrica").and_then(Value::as_bool).unwrap_or(false),
                    v.get("maior_diagonal").and_then(Value::as_f64).unwrap_or(0.0),
                );
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
    ui.separator();
    ui.monospace(&estado.eqv_resumo);
    if !estado.mensagem.is_empty() {
        ui.label(&estado.mensagem);
    }
}

/// Janela `simultaneo` do diálogo AnaCAD.
pub(super) fn janela_simultaneo(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.label("Barras com falta trifásica simultânea (códigos separados por vírgula):");
    ui.add(egui::TextEdit::singleline(&mut estado.sim_barras).desired_width(420.0));
    if ui.button("Calcular faltas simultâneas").clicked() {
        let barras: Vec<i64> =
            estado.sim_barras.split(|c: char| c == ',' || c == ';' || c.is_whitespace()).filter_map(|t| t.trim().parse::<i64>().ok()).collect();
        match app.run("anacad.curto.simultaneo", json!({"db": estado.db, "documento": estado.documento, "barras": barras})) {
            Ok(v) => {
                let mut texto = String::new();
                if let Some(lista) = v.get("correntes").and_then(Value::as_array) {
                    for c in lista {
                        let barra = c.get("barra").and_then(Value::as_i64).unwrap_or(0);
                        let ika = c.get("i_ka").and_then(Value::as_f64).unwrap_or(0.0);
                        let ang = c.get("angulo_graus").and_then(Value::as_f64).unwrap_or(0.0);
                        texto.push_str(&format!("barra {barra}: {ika:.3} kA · {ang:.1}°\n"));
                    }
                }
                estado.sim_resumo = texto;
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
    ui.separator();
    ui.monospace(&estado.sim_resumo);
    if !estado.mensagem.is_empty() {
        ui.label(&estado.mensagem);
    }
}

/// Janela `zbus` do diálogo AnaCAD.
pub(super) fn janela_zbus(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.label("Zbarra (diagonal) a partir da Ybus do documento.");
    ui.separator();
    if ui.button("Calcular Zbarra").clicked() {
        match app.run("anacad.zbus", json!({"db": estado.db, "documento": estado.documento})) {
            Ok(v) => {
                let mut texto = String::new();
                if let Some(lista) = v.get("diagonal").and_then(Value::as_array) {
                    for z in lista {
                        let barra = z.get("barra").and_then(Value::as_i64).unwrap_or(0);
                        let modulo = z.get("modulo").and_then(Value::as_f64).unwrap_or(0.0);
                        let re = z.get("z_re").and_then(Value::as_f64).unwrap_or(0.0);
                        let im = z.get("z_im").and_then(Value::as_f64).unwrap_or(0.0);
                        texto.push_str(&format!("barra {barra}: |Z| {modulo:.5} ({re:.5} + j{im:.5})\n"));
                    }
                }
                estado.zbus_resumo = texto;
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
    ui.separator();
    ui.monospace(&estado.zbus_resumo);
    if !estado.mensagem.is_empty() {
        ui.label(&estado.mensagem);
    }
}

/// Janela `niveis_curto` do diálogo AnaCAD.
pub(super) fn janela_niveis_curto(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.label("Calcula I3F/I1F/I2F e X/R por barra a partir do banco.");
    ui.separator();
    if ui.button("Calcular").clicked() {
        match app.run("anacad.niveis.curto", json!({"db": estado.db, "documento": estado.documento})) {
            Ok(v) => {
                let mut texto = String::new();
                if let Some(lista) = v.get("niveis").and_then(Value::as_array) {
                    for n in lista {
                        let barra = n.get("barra").and_then(Value::as_i64).unwrap_or(0);
                        let i3 = n.get("i3f_ka").and_then(Value::as_f64).unwrap_or(0.0);
                        let i1 = n.get("i1f_ka").and_then(Value::as_f64).unwrap_or(0.0);
                        let i2 = n.get("i2f_ka").and_then(Value::as_f64).unwrap_or(0.0);
                        let xr = n.get("xr").and_then(Value::as_f64).unwrap_or(0.0);
                        texto.push_str(&format!("barra {barra}: I3F {i3:.2} kA · I1F {i1:.2} kA · I2F {i2:.2} kA · X/R {xr:.2}\n"));
                    }
                }
                estado.niv_resumo = texto;
            }
            Err(e) => estado.mensagem = format!("erro: {e}"),
        }
    }
    ui.separator();
    ui.monospace(&estado.niv_resumo);
    if !estado.mensagem.is_empty() {
        ui.label(&estado.mensagem);
    }
}

/// Janela `impedancia` do diálogo AnaCAD.
pub(super) fn janela_impedancia(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool) {
    let chave = format!("imp|{}|{}", estado.db, estado.documento);
    if estado.ed_listas_em != chave {
        carregar_barras_opcoes(app, estado);
        estado.ed_listas_em = chave;
    }
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra 1:");
            campo_numero(ui, "anacad.imp.barra1", &mut estado.linha_de, &estado.barras_opcoes);
            combo_nome(ui, "anacad.imp.nome1", &mut estado.linha_de, &estado.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Barra 2:");
            campo_numero(ui, "anacad.imp.barra2", &mut estado.linha_para, &estado.barras_opcoes);
            combo_nome(ui, "anacad.imp.nome2", &mut estado.linha_para, &estado.barras_opcoes);
        });
    });
    ui.group(|ui| {
        ui.label(egui::RichText::new("Opções").strong());
        ui.horizontal(|ui| {
            ui.label("Unidade:");
            egui::ComboBox::from_id_salt("anacad.imp.unidade").width(120.0).selected_text(&estado.imp_unidade).show_ui(ui, |ui| {
                for u in ["Polar", "Retangular"] {
                    ui.selectable_value(&mut estado.imp_unidade, u.to_string(), u);
                }
            });
        });
        ui.add_enabled(false, egui::Checkbox::new(&mut estado.imp_um_gerador, "Desconsiderar todos os geradores do sistema exceto um"))
            .on_disabled_hover_text("equivalente com um único gerador ainda não implementado");
    });
    ui.horizontal(|ui| {
        if ui.button("Executar").clicked() {
            let cmd = json!({
                "db": estado.db,
                "documento": estado.documento,
                "de": estado.linha_de,
                "para": estado.linha_para,
                "unidade": if estado.imp_unidade == "Retangular" { "retangular" } else { "polar" },
            });
            estado.mensagem = match app.run("anacad.impedancia.barras", cmd) {
                Ok(v) => {
                    let re = v.get("z_re").and_then(Value::as_f64).unwrap_or(0.0);
                    let im = v.get("z_im").and_then(Value::as_f64).unwrap_or(0.0);
                    if estado.imp_unidade == "Retangular" {
                        format!("Z = {re:.5} {im:+.5}j pu")
                    } else {
                        format!(
                            "|Z| = {:.5} pu · ângulo = {:.2}°",
                            v.get("modulo").and_then(Value::as_f64).unwrap_or(0.0),
                            v.get("angulo_graus").and_then(Value::as_f64).unwrap_or(0.0),
                        )
                    }
                }
                Err(e) => format!("erro: {e}"),
            };
        }
        if ui.button("Cancelar").clicked() {
            *fechar = true;
        }
    });
}

/// Janela `superacao` do diálogo AnaCAD.
pub(super) fn janela_superacao(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado, fechar: &mut bool) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Opções").strong());
        ui.horizontal(|ui| {
            ui.label("Fazer análise detalhada em barras com Percentual de Superação maior que:");
            ui.add(egui::DragValue::new(&mut estado.sup_limite).speed(1.0).range(0.0..=1000.0));
            ui.label("%");
        });
        ui.checkbox(&mut estado.sup_flags[0], "Imprimir tabelas relacionando percentual de superação e constante de tempo (X/R)");
        ui.checkbox(&mut estado.sup_flags[1], "Imprimir tabelas com barras nas condições Alerta (>90%) e Superado (>100%)");
        ui.checkbox(&mut estado.sup_flags[2], "Imprimir relação de barras que terão análise detalhada");
        ui.checkbox(&mut estado.sup_flags[3], "Imprimir apenas circuitos nas condições Alerta (>90%) e Superado (>100%)");
        ui.checkbox(&mut estado.sup_flags[4], "Imprimir valores com vírgula ao invés de ponto decimal");
        ui.add_enabled(false, egui::Checkbox::new(&mut estado.sup_flags[5], "Exibir resultados no diagrama unifilar"))
            .on_disabled_hover_text("anotação no diagrama depende do estudo por circuito");
    });
    ui.group(|ui| {
        ui.label(egui::RichText::new("Barras Analisadas").strong());
        egui::ScrollArea::vertical().max_height(90.0).auto_shrink([false, false]).show(ui, |ui| {
            if estado.sup_barras.is_empty() {
                ui.label("(vazio = todas as barras)");
            }
            for (i, barra) in estado.sup_barras.iter().enumerate() {
                if ui.selectable_label(estado.sup_sel == Some(i), format!("{barra}")).clicked() {
                    estado.sup_sel = if estado.sup_sel == Some(i) { None } else { Some(i) };
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Selecionar...").clicked() {
                estado.sup_painel = if estado.sup_painel == 1 { 0 } else { 1 };
            }
            if ui.button("Especificar...").clicked() {
                estado.sup_painel = if estado.sup_painel == 2 { 0 } else { 2 };
            }
            if ui.add_enabled(estado.sup_sel.is_some(), egui::Button::new("Remover")).clicked() {
                let alvo = estado.sup_sel.take().unwrap_or(usize::MAX);
                if alvo < estado.sup_barras.len() {
                    estado.sup_barras.remove(alvo);
                }
            }
            if ui.button("Importar").clicked() {
                estado.mensagem = match std::fs::read_to_string(&estado.sup_arquivo) {
                    Ok(texto) => {
                        let codigos: Vec<i64> =
                            texto.split(|c: char| c == ',' || c == ';' || c.is_whitespace()).filter_map(|t| t.parse::<i64>().ok()).collect();
                        let mut novas = 0;
                        for codigo in codigos {
                            if !estado.sup_barras.contains(&codigo) {
                                estado.sup_barras.push(codigo);
                                novas += 1;
                            }
                        }
                        format!("{novas} barras importadas de {}", estado.sup_arquivo)
                    }
                    Err(e) => format!("erro ao ler {}: {e}", estado.sup_arquivo),
                };
            }
            if ui.button("Exportar").clicked() {
                let mut texto = String::new();
                for codigo in &estado.sup_barras {
                    texto.push_str(&format!("{codigo}\n"));
                }
                estado.mensagem = match std::fs::write(&estado.sup_arquivo, texto) {
                    Ok(()) => format!("{} barras exportadas para {}", estado.sup_barras.len(), estado.sup_arquivo),
                    Err(e) => format!("erro ao gravar {}: {e}", estado.sup_arquivo),
                };
            }
        });
        ui.horizontal(|ui| {
            ui.label("arquivo:");
            ui.add(egui::TextEdit::singleline(&mut estado.sup_arquivo).desired_width(220.0));
        });
        if estado.sup_painel == 1 {
            egui::ScrollArea::vertical().max_height(80.0).auto_shrink([false, false]).show(ui, |ui| {
                for (codigo, nome) in &estado.barras_opcoes {
                    if ui.selectable_label(false, format!("{codigo} — {nome}")).clicked() && !estado.sup_barras.contains(codigo) {
                        estado.sup_barras.push(*codigo);
                    }
                }
            });
        }
        if estado.sup_painel == 2 {
            ui.horizontal(|ui| {
                ui.label("código da barra:");
                ui.add(egui::DragValue::new(&mut estado.sup_nova).speed(1.0));
                if ui.button("Adicionar").clicked() && !estado.sup_barras.contains(&estado.sup_nova) {
                    estado.sup_barras.push(estado.sup_nova);
                }
            });
        }
    });
    ui.horizontal(|ui| {
        if ui.button("Executar").clicked() {
            let mut cmd = json!({
                "db": estado.db,
                "documento": estado.documento,
            });
            if !estado.sup_barras.is_empty() {
                cmd["barras"] = json!(estado.sup_barras);
            }
            estado.sup_lista.clear();
            estado.sup_resumo = match app.run("anacad.superacao", cmd) {
                Ok(v) => {
                    let barras = v.get("barras").and_then(Value::as_array).cloned().unwrap_or_default();
                    let virgula = estado.sup_flags[4];
                    let so_alerta = estado.sup_flags[1];
                    let mostrar_analise = estado.sup_flags[2];
                    let tabela = estado.sup_flags[0];
                    let fmt = |x: f64| if virgula { format!("{x:.1}").replace('.', ",") } else { format!("{x:.1}") };
                    if tabela {
                        estado.sup_lista.push("Barra   %Sup.   X/R   Situação".to_string());
                    }
                    for b in &barras {
                        let percentual = b.get("margem").and_then(Value::as_f64).unwrap_or(0.0) * 100.0;
                        let alerta = percentual > 90.0;
                        let superado = percentual > 100.0;
                        if so_alerta && !alerta {
                            continue;
                        }
                        if mostrar_analise && percentual <= estado.sup_limite {
                            continue;
                        }
                        let barra = b.get("barra").and_then(Value::as_i64).unwrap_or(0);
                        let i3f = b.get("i3f_ka").and_then(Value::as_f64).unwrap_or(0.0);
                        let cap = b.get("capacidade_ka").and_then(Value::as_f64).unwrap_or(0.0);
                        let xr = b.get("xr").and_then(Value::as_f64).unwrap_or(0.0);
                        let situacao = if superado {
                            "SUPERADO"
                        } else if alerta {
                            "ALERTA"
                        } else {
                            "ok"
                        };
                        if tabela {
                            estado.sup_lista.push(format!("{barra:>5}  {:>7}  {:>6}  {situacao}", fmt(percentual), fmt(xr)));
                        } else {
                            estado.sup_lista.push(format!(
                                "Barra {barra}: {}% ({}) · I3F {} kA × cap. {} kA · X/R {}",
                                fmt(percentual),
                                situacao,
                                fmt(i3f),
                                fmt(cap),
                                fmt(xr)
                            ));
                        }
                    }
                    if estado.sup_flags[3] {
                        let circuitos = v.get("circuitos").and_then(Value::as_array).cloned().unwrap_or_default();
                        let mut linhas = 0;
                        for c in &circuitos {
                            let supera_circuito = c.get("supera").and_then(Value::as_bool).unwrap_or(false);
                            if so_alerta && !supera_circuito {
                                continue;
                            }
                            let de = c.get("de").and_then(Value::as_i64).unwrap_or(0);
                            let para = c.get("para").and_then(Value::as_i64).unwrap_or(0);
                            let i_de = c.get("i_de_ka").and_then(Value::as_f64).unwrap_or(0.0);
                            let p_de = c.get("percentual_de").and_then(Value::as_f64);
                            let p_para = c.get("percentual_para").and_then(Value::as_f64);
                            let pct = |p: Option<f64>| p.map(fmt).unwrap_or_else(|| "-".to_string());
                            estado.sup_lista.push(format!(
                                "Circuito {de}-{para}: I {} kA · de {}% · para {}%{}",
                                fmt(i_de),
                                pct(p_de),
                                pct(p_para),
                                if supera_circuito { " · SUPERADO" } else { "" }
                            ));
                            linhas += 1;
                        }
                        if linhas == 0 {
                            estado.sup_lista.push("(sem circuitos com capacidade)".to_string());
                        }
                    }
                    let superam = v.get("superam").and_then(Value::as_i64).unwrap_or(0);
                    let sem = v.get("sem_capacidade").and_then(Value::as_i64).unwrap_or(0);
                    let id = v.get("resultado").and_then(Value::as_i64).unwrap_or(0);
                    format!("{superam} superam · {sem} sem capacidade · resultado {id}")
                }
                Err(e) => format!("erro: {e}"),
            };
            if estado.sup_barras.is_empty() {
                estado.sup_barras = estado
                    .sup_lista
                    .iter()
                    .filter_map(|linha| linha.strip_prefix("Barra ").and_then(|t| t.split(':').next()).and_then(|n| n.parse().ok()))
                    .collect();
            }
        }
        if ui.button("Cancelar").clicked() {
            *fechar = true;
        }
    });
}

/// Janela `curto` do diálogo AnaCAD.
pub(super) fn janela_curto(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
    ui.horizontal(|ui| {
        ui.label("barra");
        ui.add(egui::DragValue::new(&mut estado.curto_barra).speed(1.0));
        ui.label("referência");
        ui.add(egui::DragValue::new(&mut estado.curto_ref).speed(1.0));
        ui.label("tipo");
        egui::ComboBox::from_id_salt("anacad.curto.tipo").selected_text(&estado.curto_tipo).show_ui(ui, |ui| {
            for tipo in ["trifasica", "bifasica", "monofasica"] {
                ui.selectable_value(&mut estado.curto_tipo, tipo.to_string(), tipo);
            }
        });
    });
    ui.horizontal(|ui| {
        ui.label("Zf (pu)");
        ui.add(egui::TextEdit::singleline(&mut estado.curto_zf).desired_width(80.0));
    });
    ui.checkbox(&mut estado.curto_contrib, "mostrar contribuições de ramo");
    ui.checkbox(&mut estado.curto_tensoes, "mostrar tensões pós-falta");
    ui.horizontal(|ui| {
        ui.label("abrir ramos (de,para; ...)");
        ui.add(egui::TextEdit::singleline(&mut estado.curto_abrir).desired_width(260.0));
    });
    if ui.button("Calcular curto").clicked() {
        let abrir: Vec<[i64; 2]> = estado
            .curto_abrir
            .split(';')
            .filter_map(|par| {
                let mut it = par.split(',');
                Some([it.next()?.trim().parse().ok()?, it.next()?.trim().parse().ok()?])
            })
            .collect();
        estado.mensagem = match app.run(
            "anacad.curto",
            json!({
                "db": estado.db, "documento": estado.documento,
                "barra": estado.curto_barra, "referencia": estado.curto_ref,
                "tipo": estado.curto_tipo, "abrir": abrir,
            "zf": estado.curto_zf.trim().parse::<f64>().unwrap_or(0.0),
            "contribuicoes": estado.curto_contrib,
            "tensoes": estado.curto_tensoes,
            }),
        ) {
            Ok(v) => {
                let mut texto = format!(
                    "Icc {:.3} kA · ângulo {:.1}°",
                    v.get("i_ka").and_then(Value::as_f64).unwrap_or(0.0),
                    v.get("angulo_graus").and_then(Value::as_f64).unwrap_or(0.0),
                );
                if let Some(lista) = v.get("tensoes").and_then(Value::as_array)
                    && let Some(menor) = lista.iter().min_by(|a, b| {
                        a.get("modulo").and_then(Value::as_f64).unwrap_or(9.0).total_cmp(&b.get("modulo").and_then(Value::as_f64).unwrap_or(9.0))
                    })
                {
                    texto.push_str(&format!(
                        "\nmenor tensão: barra {} = {:.3} pu",
                        menor.get("barra").and_then(Value::as_i64).unwrap_or(0),
                        menor.get("modulo").and_then(Value::as_f64).unwrap_or(0.0),
                    ));
                }
                texto
            }
            Err(e) => format!("erro: {e}"),
        };
    }
}
