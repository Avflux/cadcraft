//! Janelas `Dados de …` dos equipamentos sem janela própria (carga, shunts, motor, mútua, compensadores, trafo de aterramento, fonte de corrente).

use super::*;

/// Carrega handle/estado comum das janelas `Dados de …` de equipamento.
pub(super) fn abrir_dados_equipamento(app: &mut CadApp, estado: &mut Estado, chave_tipo: &str) -> String {
    let handle = if !estado.comp_handle.trim().is_empty() {
        estado.comp_handle.clone()
    } else {
        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
    };
    estado.comp_handle = handle.clone();
    let chave = format!("{chave_tipo}|{}|{}|{}", estado.db, estado.documento, handle);
    if estado.ed_listas_em != chave {
        if estado.ed_handle != handle {
            if let Some(dados) = dados_entidade(app, &handle) {
                preencher_equipamento(&mut *estado, &dados);
            }
            estado.ed_handle = handle.clone();
        }
        carregar_barras_opcoes(app, &mut *estado);
        estado.ed_listas_em = chave;
    }
    handle
}

/// Título (caption do ANAFAS) de cada janela `Dados de …` de equipamento.
pub(super) fn caption_equipamento(tipo: &str) -> &'static str {
    match tipo {
        "Carga" => "Dados de Carga",
        "Shunt de Barra" => "Dados de Shunt de Barra",
        "Shunt de Linha" => "Dados de Shunt de Linha",
        "Motor de Indução" => "Dados de Motor de Indução",
        "Impedância Mútua" => "Dados de Impedância Mútua",
        "Compensador Série" => "Dados de Compensador Série",
        "Trafo de Aterramento" => "Dados de Transformador de Aterramento",
        "Gerador Fonte de Corrente" => "Dados de Gerador Tipo Fonte de Corrente",
        "Compensador Estático" => "Dados de Compensador Estático",
        _ => "Dados do Equipamento",
    }
}

/// Rodapé comum: Inserir · Alterar · Remover · Fechar (Remover = 33057).
pub(super) fn rodape_equipamento(
    app: &mut CadApp,
    ui: &mut Ui,
    estado: &mut Estado,
    handle: &str,
    rotulo: &str,
    dados: &Value,
    fechar: &mut bool,
    notificacao: &mut Option<String>,
) {
    ui.separator();
    ui.horizontal(|ui| {
        let existe = estado.eq_existe;
        let salvar = |app: &mut CadApp, estado: &mut Estado| {
            let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle, "dados": dados});
            estado.mensagem = match app.run("anacad.equipamento.editar", cmd) {
                Ok(_) => String::new(),
                Err(e) => format!("erro: {e}"),
            };
        };
        if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
            salvar(app, estado);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some(format!("dados do {rotulo} inseridos"));
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
            salvar(app, estado);
            if !estado.mensagem.starts_with("erro") {
                *notificacao = Some(format!("{rotulo} atualizado"));
                *fechar = true;
            }
        }
        if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
            let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle});
            match app.run("anacad.equipamento.dados.excluir", cmd) {
                Ok(_) => {
                    estado.eq_existe = false;
                    estado.mensagem = format!("dados elétricos do {rotulo} removidos");
                }
                Err(e) => estado.mensagem = format!("erro: {e}"),
            }
        }
        if ui.button("Fechar").clicked() {
            *fechar = true;
        }
    });
    ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
}

/// Grupo "Capacidades de Interrupção Simétrica (kA)": 1: 2: 3:.
pub(super) fn grupo_capacidades3(ui: &mut Ui, capacidades: &mut [f64; 3]) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
        ui.horizontal(|ui| {
            for (i, v) in capacidades.iter_mut().enumerate() {
                ui.label(format!("{}:", i + 1));
                ui.add(egui::DragValue::new(v).speed(0.1));
            }
        });
    });
}

/// Rádios do "Tipo de Conexão" sem os campos Rn/Xn (159).
pub(super) fn grupo_radios_conexao(ui: &mut Ui, conexao: &mut String) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Tipo de Conexão").strong());
        for c in ["Delta", "Estrela não aterrado", "Estrela aterrado"] {
            ui.radio_value(conexao, c.to_string(), c);
        }
    });
}

pub(super) fn grupos_carga(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.carga.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.carga.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            campo_grupo(ui, &mut e.eq_grupo, &e.grupos_opcoes);
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(150.0));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Carga Ativa (P MW):");
            ui.add(egui::DragValue::new(&mut e.eq_p).speed(1.0));
            ui.label("Carga Reativa (Q Mvar):");
            ui.add(egui::DragValue::new(&mut e.eq_q).speed(1.0));
        });
    });
    grupo_radios_conexao(ui, &mut e.eq_conexao);
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Número de Unidades:");
            ui.add(egui::DragValue::new(&mut e.eq_unidades).range(1..=9999));
            ui.label("Número de Unidades em Operação:");
            ui.add(egui::DragValue::new(&mut e.eq_unid_oper).range(0..=9999));
        });
    });
    grupo_capacidades3(ui, &mut e.eq_cap_de);
}

pub(super) fn grupos_shunt_barra(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.shb.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.shb.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            campo_grupo(ui, &mut e.eq_grupo, &e.grupos_opcoes);
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(130.0));
        });
        ui.horizontal(|ui| {
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
            ui.checkbox(&mut e.eq_status, "Ligado");
            ui.checkbox(&mut e.eq_circ_eq, "Circuito Equivalente");
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Potência Nominal (Q Mvar):");
            ui.add(egui::DragValue::new(&mut e.eq_pot).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Reatância (X %):");
            ui.add(egui::DragValue::new(&mut e.eq_x1).speed(0.1));
            ui.label("Resistência Seq. Zero (R0 %):");
            ui.add(egui::DragValue::new(&mut e.eq_r0).speed(0.1));
        });
    });
    grupo_conexao(ui, "Tipo de Conexão", &mut e.eq_conexao, &mut e.eq_rn, &mut e.eq_xn);
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Número de Unidades:");
            ui.add(egui::DragValue::new(&mut e.eq_unidades).range(1..=9999));
            ui.label("Número de Unidades em Operação:");
            ui.add(egui::DragValue::new(&mut e.eq_unid_oper).range(0..=9999));
        });
    });
    grupo_capacidades3(ui, &mut e.eq_cap_de);
}

pub(super) fn grupos_shunt_linha(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Barra De:");
            campo_conexao(ui, "anacad.shl.de", &mut e.eq_de, &e.barras_opcoes);
            ui.label("Nome De:");
            combo_conexao(ui, "anacad.shl.de.nome", &mut e.eq_de, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Barra Para:");
            campo_conexao(ui, "anacad.shl.para", &mut e.eq_para, &e.barras_opcoes);
            ui.label("Nome Para:");
            combo_conexao(ui, "anacad.shl.para.nome", &mut e.eq_para, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Número do Circuito:");
            ui.add(egui::DragValue::new(&mut e.eq_circuito).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            campo_grupo(ui, &mut e.eq_grupo, &e.grupos_opcoes);
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(110.0));
        });
        ui.horizontal(|ui| {
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Lado:").strong());
            ui.horizontal(|ui| {
                ui.radio_value(&mut e.eq_lado, "De".to_string(), "De");
                ui.radio_value(&mut e.eq_lado, "Para".to_string(), "Para");
            });
        });
    });
    ui.horizontal(|ui| {
        ui.label("Potência Reativa (Mvar):");
        ui.add(egui::DragValue::new(&mut e.eq_pot).speed(0.1));
    });
    ui.group(|ui| {
        for c in ["Delta", "Estrela não aterrado", "Estrela aterrado"] {
            ui.radio_value(&mut e.eq_conexao, c.to_string(), c);
        }
        ui.horizontal(|ui| {
            ui.label("Resistência de Aterramento (%):");
            ui.add(egui::DragValue::new(&mut e.eq_rn).speed(0.1));
        });
        ui.horizontal(|ui| {
            ui.label("Reatância de Aterramento (%):");
            ui.add(egui::DragValue::new(&mut e.eq_xn).speed(0.1));
        });
        ui.checkbox(&mut e.eq_bypass, "Bypass (Aterramento)");
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Número de Unidades:");
            ui.add(egui::DragValue::new(&mut e.eq_unidades).range(1..=9999));
            ui.label("Número de Unidades em Operação:");
            ui.add(egui::DragValue::new(&mut e.eq_unid_oper).range(0..=9999));
        });
    });
}

pub(super) fn grupos_motor(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.mot.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.mot.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            campo_grupo(ui, &mut e.eq_grupo, &e.grupos_opcoes);
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
    });
    ui.group(|ui| {
        ui.label(egui::RichText::new("Estator").strong());
        ui.horizontal(|ui| {
            ui.label("Resistência (R %):");
            ui.add(egui::DragValue::new(&mut e.eq_r1).speed(0.1));
            ui.label("Reatância (X %):");
            ui.add(egui::DragValue::new(&mut e.eq_x1).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.label(egui::RichText::new("Rotor").strong());
        ui.horizontal(|ui| {
            ui.label("Resistência (R %):");
            ui.add(egui::DragValue::new(&mut e.eq_r0).speed(0.1));
            ui.label("Reatância (X %):");
            ui.add(egui::DragValue::new(&mut e.eq_x0).speed(0.1));
        });
    });
    ui.horizontal_top(|ui| {
        ui.group(|ui| {
            ui.label("Reatância de Magnetização (Xm %):");
            ui.add(egui::DragValue::new(&mut e.eq_xm).speed(0.1));
        });
        ui.group(|ui| {
            ui.label("Base de Potência (HP):");
            ui.add(egui::DragValue::new(&mut e.eq_base_hp).speed(0.1));
        });
    });
    grupo_conexao(ui, "Tipo de Conexão", &mut e.eq_conexao, &mut e.eq_rn, &mut e.eq_xn);
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Número de Unidades:");
            ui.add(egui::DragValue::new(&mut e.eq_unidades).range(1..=9999));
            ui.label("Número de Unidades em Operação:");
            ui.add(egui::DragValue::new(&mut e.eq_unid_oper).range(0..=9999));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Capacidade de Interrupção do disjuntor (kA):");
            ui.add(egui::DragValue::new(&mut e.eq_capacidade).speed(0.1));
        });
    });
}

pub(super) fn grupos_mutua(ui: &mut Ui, e: &mut Estado) {
    for trecho in 1..=2 {
        let (de, para, nc, ini, fim) = if trecho == 1 {
            (&mut e.eq_mut_de1, &mut e.eq_mut_para1, &mut e.eq_mut_nc1, &mut e.eq_mut_ini1, &mut e.eq_mut_fim1)
        } else {
            (&mut e.eq_mut_de2, &mut e.eq_mut_para2, &mut e.eq_mut_nc2, &mut e.eq_mut_ini2, &mut e.eq_mut_fim2)
        };
        ui.group(|ui| {
            ui.label(egui::RichText::new(format!("Trecho {trecho}")).strong());
            ui.horizontal(|ui| {
                ui.label("Barra De:");
                campo_conexao(ui, &format!("anacad.mut{trecho}.de"), de, &e.barras_opcoes);
                ui.label("Barra Para:");
                campo_conexao(ui, &format!("anacad.mut{trecho}.para"), para, &e.barras_opcoes);
            });
            ui.horizontal(|ui| {
                ui.label("Número do Circuito:");
                ui.add(egui::DragValue::new(nc).range(1..=9999));
                ui.label("Início (%):");
                ui.add(egui::DragValue::new(ini).speed(0.1));
                ui.label("Fim (%):");
                ui.add(egui::DragValue::new(fim).speed(0.1));
            });
        });
    }
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Resistência (Rm %):");
            ui.add(egui::DragValue::new(&mut e.eq_mut_rm).speed(0.1));
            ui.label("Reatância (Xm %):");
            ui.add(egui::DragValue::new(&mut e.eq_mut_xm).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
    });
}

pub(super) fn grupos_comp_serie(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra De:");
            campo_conexao(ui, "anacad.cs.de", &mut e.eq_de, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.cs.de.nome", &mut e.eq_de, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Barra Para:");
            campo_conexao(ui, "anacad.cs.para", &mut e.eq_para, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.cs.para.nome", &mut e.eq_para, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Número do Circuito:");
            ui.add(egui::DragValue::new(&mut e.eq_circuito).speed(1.0));
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(120.0));
        });
        ui.horizontal(|ui| {
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Estado Operativo").strong());
            ui.horizontal(|ui| {
                for c in ["Ligado", "Desligado", "Bypass"] {
                    ui.radio_value(&mut e.eq_estado_oper, c.to_string(), c);
                }
            });
        });
    });
    ui.horizontal_top(|ui| {
        ui.group(|ui| {
            ui.label("Reatância (X %):");
            ui.add(egui::DragValue::new(&mut e.eq_x1).speed(0.1));
        });
        ui.group(|ui| {
            ui.label("Capacidade (MVA):");
            ui.add(egui::DragValue::new(&mut e.eq_capacidade).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.label(egui::RichText::new("Dados de Proteção de Capacitor Série").strong());
        ui.checkbox(&mut e.eq_prot_ligado, "Ligado");
        ui.group(|ui| {
            ui.label(egui::RichText::new("Varistor").strong());
            ui.horizontal(|ui| {
                ui.label("Corrente de proteção (Arms / fase):");
                ui.add(egui::DragValue::new(&mut e.eq_var_corrente).speed(0.1));
            });
        });
        ui.group(|ui| {
            ui.label(egui::RichText::new("Gap").strong());
            ui.horizontal(|ui| {
                ui.label("Corrente máxima (Arms / fase):");
                ui.add(egui::DragValue::new(&mut e.eq_gap_corrente).speed(0.1));
                ui.label("Tipo de Disparo:");
                ui.add(egui::TextEdit::singleline(&mut e.eq_gap_tipo).desired_width(70.0));
            });
            ui.horizontal(|ui| {
                ui.label("Energia máxima (MJ / fase):");
                ui.add(egui::DragValue::new(&mut e.eq_gap_energia).speed(0.1));
                ui.label("Potência máxima (MW / fase):");
                ui.add(egui::DragValue::new(&mut e.eq_gap_potencia).speed(0.1));
            });
        });
    });
    ui.group(|ui| {
        ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
        // De e Para em linhas separadas: os seis campos não cabem numa linha
        // só e o "Para" ficava cortado.
        ui.horizontal(|ui| {
            ui.label("De");
            for (i, v) in e.eq_cap_de.iter_mut().enumerate() {
                ui.label(format!("{}:", i + 1));
                ui.add(egui::DragValue::new(v).speed(0.1));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Para");
            for (i, v) in e.eq_cap_para.iter_mut().enumerate() {
                ui.label(format!("{}:", i + 1));
                ui.add(egui::DragValue::new(v).speed(0.1));
            }
        });
    });
}

pub(super) fn grupos_trafo_aterramento(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.ta.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.ta.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            campo_grupo(ui, &mut e.eq_grupo, &e.grupos_opcoes);
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(100.0));
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Capacidade (MVA):");
            ui.add(egui::DragValue::new(&mut e.eq_capacidade).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Resistência (R %):");
            ui.add(egui::DragValue::new(&mut e.eq_r1).speed(0.1));
            ui.label("Reatância (X %):");
            ui.add(egui::DragValue::new(&mut e.eq_x1).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Resistência de Aterramento (Rn %):");
            ui.add(egui::DragValue::new(&mut e.eq_rn).speed(0.1));
            ui.label("Reatância de Aterramento (Xn %):");
            ui.add(egui::DragValue::new(&mut e.eq_xn).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Número de Unidades:");
            ui.add(egui::DragValue::new(&mut e.eq_unidades).range(1..=9999));
            ui.label("Número de Unidades em Operação:");
            ui.add(egui::DragValue::new(&mut e.eq_unid_oper).range(0..=9999));
        });
    });
    grupo_capacidades3(ui, &mut e.eq_cap_de);
}

pub(super) fn grupos_gerador_fonte(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.gf.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.gf.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            campo_grupo(ui, &mut e.eq_grupo, &e.grupos_opcoes);
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(130.0));
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Potência Nominal (MVA):");
            ui.add(egui::DragValue::new(&mut e.eq_pot).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Corrente Máxima de Seq. Positiva (Imáx Arms):");
            ui.add(egui::DragValue::new(&mut e.eq_imax).speed(0.1));
            ui.label("Fator de Potência de Operação (FP_pré):");
            ui.add(egui::DragValue::new(&mut e.eq_fp_pre).speed(0.01));
        });
        ui.horizontal(|ui| {
            ui.label("Tensão Mínima para Injetar (Vmin pu):");
            ui.add(egui::DragValue::new(&mut e.eq_vmin).speed(0.01));
            ui.label("Potência Ativa Pré-Falta (Pinic MW):");
            ui.add(egui::DragValue::new(&mut e.eq_pinic).speed(0.1));
        });
        ui.horizontal(|ui| {
            ui.label("Tensão Máxima para Injetar (Vmax pu):");
            ui.add(egui::DragValue::new(&mut e.eq_vmax).speed(0.01));
        });
        ui.horizontal(|ui| {
            ui.label("Fator de Potência de Curto (FP_CC):");
            ui.add(egui::DragValue::new(&mut e.eq_fp_cc).speed(0.01));
        });
        ui.group(|ui| {
            ui.checkbox(&mut e.eq_curva, "Usar curva (deltaQ) x (Vpos)");
            ui.horizontal(|ui| {
                ui.label("VP1:");
                ui.add(egui::DragValue::new(&mut e.eq_vp1).speed(0.01));
                ui.label("VP2:");
                ui.add(egui::DragValue::new(&mut e.eq_vp2).speed(0.01));
            });
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Número de Unidades:");
            ui.add(egui::DragValue::new(&mut e.eq_unidades).range(1..=9999));
            ui.label("Número de Unidades em Operação:");
            ui.add(egui::DragValue::new(&mut e.eq_unid_oper).range(0..=9999));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Capacidade de Interrupção do Disjuntor (kA):");
            ui.add(egui::DragValue::new(&mut e.eq_capacidade).speed(0.1));
        });
    });
}

pub(super) fn grupos_comp_estatico(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.ce.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.ce.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            campo_grupo(ui, &mut e.eq_grupo, &e.grupos_opcoes);
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(130.0));
        });
        ui.horizontal(|ui| {
            ui.label("Área:");
            campo_area(ui, &mut e.eq_area, &e.areas_opcoes);
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Potência Nominal (Q Mvar):");
            ui.add(egui::DragValue::new(&mut e.eq_pot).speed(0.1));
        });
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Reatância (X %):");
            ui.add(egui::DragValue::new(&mut e.eq_x1).speed(0.1));
            ui.label("Resistência Seq. Zero (R0 %):");
            ui.add(egui::DragValue::new(&mut e.eq_r0).speed(0.1));
        });
    });
    grupo_conexao(ui, "Tipo de Conexão", &mut e.eq_conexao, &mut e.eq_rn, &mut e.eq_xn);
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Número de Unidades:");
            ui.add(egui::DragValue::new(&mut e.eq_unidades).range(1..=9999));
            ui.label("Número de Unidades em Operação:");
            ui.add(egui::DragValue::new(&mut e.eq_unid_oper).range(0..=9999));
        });
    });
    grupo_capacidades3(ui, &mut e.eq_cap_de);
}

/// JSON de dados elétricos de cada equipamento (chaves gravadas no `.db`/XData).
pub(super) fn dados_equipamento(estado: &Estado) -> Value {
    let tipo = estado.eq_tipo.as_str();
    let comuns = json!({
        "codigo": estado.eq_codigo,
        "nome": estado.eq_nome,
        "tipo": tipo,
        "status": estado.eq_status,
        "grupo": estado.eq_grupo,
        "area": estado.eq_area,
        "unidades": estado.eq_unidades,
        "unidadesOperacao": estado.eq_unid_oper,
    });
    let mut dados = match tipo {
        "Carga" => json!({
            "barra": estado.eq_barra, "P": estado.eq_p, "Q": estado.eq_q,
            "conexao": estado.eq_conexao, "capacidadesDe": estado.eq_cap_de.to_vec(),
        }),
        "Shunt de Barra" => json!({
            "barra": estado.eq_barra, "potencia": estado.eq_pot, "X": estado.eq_x1, "R0": estado.eq_r0,
            "conexao": estado.eq_conexao, "Rn": estado.eq_rn, "Xn": estado.eq_xn,
            "circuitoEquivalente": estado.eq_circ_eq, "capacidadesDe": estado.eq_cap_de.to_vec(),
        }),
        "Shunt de Linha" => json!({
            "de": estado.eq_de, "para": estado.eq_para, "numeroCircuito": estado.eq_circuito,
            "potencia": estado.eq_pot, "conexao": estado.eq_conexao, "Rn": estado.eq_rn, "Xn": estado.eq_xn,
            "lado": estado.eq_lado, "bypass": estado.eq_bypass,
        }),
        "Motor de Indução" => json!({
            "barra": estado.eq_barra, "R1": estado.eq_r1, "X1": estado.eq_x1,
            "R0": estado.eq_r0, "X0": estado.eq_x0, "Xm": estado.eq_xm, "baseHP": estado.eq_base_hp,
            "conexao": estado.eq_conexao, "Rn": estado.eq_rn, "Xn": estado.eq_xn,
            "capacidade": estado.eq_capacidade,
        }),
        "Impedância Mútua" => json!({
            "mutDe1": estado.eq_mut_de1, "mutPara1": estado.eq_mut_para1, "mutNC1": estado.eq_mut_nc1,
            "mutIni1": estado.eq_mut_ini1, "mutFim1": estado.eq_mut_fim1,
            "mutDe2": estado.eq_mut_de2, "mutPara2": estado.eq_mut_para2, "mutNC2": estado.eq_mut_nc2,
            "mutIni2": estado.eq_mut_ini2, "mutFim2": estado.eq_mut_fim2,
            "Rm": estado.eq_mut_rm, "mutXm": estado.eq_mut_xm,
        }),
        "Compensador Série" => json!({
            "de": estado.eq_de, "para": estado.eq_para, "numeroCircuito": estado.eq_circuito,
            "estadoOperativo": estado.eq_estado_oper, "X": estado.eq_x1, "capacidade": estado.eq_capacidade,
            "protecaoLigado": estado.eq_prot_ligado, "varistorCorrente": estado.eq_var_corrente,
            "gapCorrente": estado.eq_gap_corrente, "gapTipo": estado.eq_gap_tipo,
            "gapEnergia": estado.eq_gap_energia, "gapPotencia": estado.eq_gap_potencia,
            "capacidadesDe": estado.eq_cap_de.to_vec(), "capacidadesPara": estado.eq_cap_para.to_vec(),
        }),
        "Trafo de Aterramento" => json!({
            "barra": estado.eq_barra, "capacidade": estado.eq_capacidade,
            "R1": estado.eq_r1, "X1": estado.eq_x1, "Rn": estado.eq_rn, "Xn": estado.eq_xn,
            "capacidadesDe": estado.eq_cap_de.to_vec(),
        }),
        "Gerador Fonte de Corrente" => json!({
            "barra": estado.eq_barra, "potencia": estado.eq_pot,
            "Imax": estado.eq_imax, "FPpre": estado.eq_fp_pre, "Vmin": estado.eq_vmin, "Pinic": estado.eq_pinic,
            "Vmax": estado.eq_vmax, "FPcc": estado.eq_fp_cc, "usarCurva": estado.eq_curva,
            "VP1": estado.eq_vp1, "VP2": estado.eq_vp2, "capacidade": estado.eq_capacidade,
        }),
        _ => json!({
            "barra": estado.eq_barra, "potencia": estado.eq_pot, "X": estado.eq_x1, "R0": estado.eq_r0,
            "conexao": estado.eq_conexao, "Rn": estado.eq_rn, "Xn": estado.eq_xn,
            "capacidadesDe": estado.eq_cap_de.to_vec(),
        }),
    };
    if let (Some(dst), Some(src)) = (dados.as_object_mut(), comuns.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    dados
}

/// Janela `editar_dados_equipamento` do diálogo AnaCAD.
pub(super) fn janela_editar_dados_equipamento(
    app: &mut CadApp,
    ui: &mut Ui,
    estado: &mut Estado,
    fechar: &mut bool,
    notificacao: &mut Option<String>,
) {
    let rotulo = estado.eq_tipo.clone();
    let handle = abrir_dados_equipamento(app, estado, &rotulo);
    egui::ScrollArea::vertical().max_height(540.0).auto_shrink([false, false]).show(ui, |ui| match estado.eq_tipo.as_str() {
        "Carga" => grupos_carga(ui, estado),
        "Shunt de Barra" => grupos_shunt_barra(ui, estado),
        "Shunt de Linha" => grupos_shunt_linha(ui, estado),
        "Motor de Indução" => grupos_motor(ui, estado),
        "Impedância Mútua" => grupos_mutua(ui, estado),
        "Compensador Série" => grupos_comp_serie(ui, estado),
        "Trafo de Aterramento" => grupos_trafo_aterramento(ui, estado),
        "Gerador Fonte de Corrente" => grupos_gerador_fonte(ui, estado),
        "Compensador Estático" => grupos_comp_estatico(ui, estado),
        _ => {}
    });
    // O rodapé fica fora da rolagem, como no Transformador:
    // dentro dela os botões ficavam escondidos no fim do conteúdo.
    let dados = dados_equipamento(estado);
    rodape_equipamento(app, ui, estado, &handle, &rotulo, &dados, fechar, notificacao);
}
