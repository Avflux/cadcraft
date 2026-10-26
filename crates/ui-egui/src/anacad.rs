//! Painel AnaCAD: consulta e edição rápida do banco de projeto (`.db`).
//!
//! É um painel **dockado** do cadcraft (não uma janela flutuante): entra na
//! mesma prancheta de Layers/Properties, abre pelo menu `AnaCAD ▸ Painel`, pelo
//! botão da barra de ferramentas ou por `ui.anacad`.
//!
//! A UI é fina: os botões chamam os comandos do motor (`app.run`) e o estado da
//! tela (caminho do banco, documento, último resultado) vive na memória do egui.

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::CadApp;

const ID_ESTADO: &str = "anacad.estado";

#[derive(Clone)]
struct Estado {
    db: String,
    documento: i64,
    barra_codigo: i64,
    barra_x: f64,
    barra_y: f64,
    barra_x2: f64,
    barra_y2: f64,
    linha_de: i64,
    linha_para: i64,
    legenda_numero: i64,
    legenda_x: f64,
    legenda_y: f64,
    comp_handle: String,
    comp_dx: f64,
    comp_dy: f64,
    curto_barra: i64,
    curto_ref: i64,
    curto_tipo: String,
    curto_abrir: String,
    curto_zf: String,
    curto_contrib: bool,
    curto_tensoes: bool,
    ana_arquivo: String,
    lst_arquivo: String,
    saida_arquivo: String,
    ed_nome: String,
    ed_area: String,
    ed_vbase: f64,
    ed_tipo: String,
    ed_capacidade: f64,
    ed_cap_am: [f64; 3],
    ed_pre_mod: f64,
    ed_pre_ang: f64,
    ed_disjuntor: bool,
    ed_status: bool,
    ed_circuito: i64,
    ed_potencia: f64,
    ed_comprimento: f64,
    ed_r1: f64,
    ed_x1: f64,
    ed_s1: f64,
    ed_r0: f64,
    ed_x0: f64,
    ed_s0: f64,
    ed_cap_de: [f64; 3],
    ed_cap_para: [f64; 3],
    eq_tipo: String,
    eq_codigo: i64,
    eq_nome: String,
    eq_de: i64,
    eq_para: i64,
    eq_barra: i64,
    eq_pot: f64,
    eq_r1: f64,
    eq_x1: f64,
    eq_r0: f64,
    eq_x0: f64,
    eq_tap: f64,
    eq_p: f64,
    eq_q: f64,
    eq_status: bool,
    eq_grupo: String,
    eq_area: String,
    eq_circuito: i64,
    eq_capacidade: f64,
    eq_defasamento: f64,
    eq_conexao: String,
    eq_conexao2: String,
    eq_rn: f64,
    eq_xn: f64,
    eq_rn2: f64,
    eq_xn2: f64,
    eq_unidades: i64,
    top_atual: i64,
    top_nova: i64,
    top_nome: String,
    top_jumper: bool,
    top_final: i64,
    top_eliminada: i64,
    top_comp_barra: i64,
    top_componentes: Vec<(i64, String, bool)>,
    sec_de: i64,
    sec_para: i64,
    sec_nova: i64,
    sec_nome: String,
    sec_percentual: f64,
    sec_reatores: bool,
    ren_atual: i64,
    ren_nova: i64,
    loc_codigo: i64,
    term_lista: Vec<(i64, String, f64, f64)>,
    term_carregada: bool,
    term_id: i64,
    term_nome: String,
    det_codigo: i64,
    det_resumo: String,
    texto_conteudo: String,
    texto_altura: f64,
    texto_cor: u8,
    niv_resumo: String,
    eqv_barras: String,
    zbus_resumo: String,
    sim_barras: String,
    sim_resumo: String,
    eqv_resumo: String,
    eqv_validar: bool,
    leg_modulo_corrente: String,
    leg_angulo_corrente: String,
    leg_modulo_tensao: String,
    leg_angulo_tensao: String,
    leg_barras_nome: bool,
    leg_equip_numero: bool,
    leg_trafo_tap: bool,
    leg_trafo_defasamento: bool,
    leg_carregada: bool,
    ed_handle: String,
    doc_pagina: String,
    doc_lista: Vec<(i64, String, String)>,
    doc_lista_db: String,
    mensagem: String,
}

impl Default for Estado {
    fn default() -> Self {
        Estado {
            db: String::new(),
            documento: 1,
            barra_codigo: 0,
            barra_x: 0.0,
            barra_y: 0.0,
            barra_x2: 0.0,
            barra_y2: 0.0,
            linha_de: 0,
            linha_para: 0,
            legenda_numero: 0,
            legenda_x: 0.0,
            legenda_y: 0.0,
            comp_handle: String::new(),
            comp_dx: 0.0,
            comp_dy: 0.0,
            curto_barra: 0,
            curto_ref: 0,
            curto_tipo: "trifasica".into(),
            curto_abrir: String::new(),
            curto_zf: "0".into(),
            curto_contrib: false,
            curto_tensoes: false,
            ana_arquivo: String::new(),
            lst_arquivo: String::new(),
            saida_arquivo: String::new(),
            ed_nome: String::new(),
            ed_area: String::new(),
            ed_vbase: 0.0,
            ed_tipo: "Normal".into(),
            ed_capacidade: 0.0,
            ed_cap_am: [0.0; 3],
            ed_pre_mod: 1.0,
            ed_pre_ang: 0.0,
            ed_disjuntor: false,
            ed_status: true,
            ed_circuito: 0,
            ed_potencia: 0.0,
            ed_comprimento: 0.0,
            ed_r1: 0.0,
            ed_x1: 0.0,
            ed_s1: 0.0,
            ed_r0: 0.0,
            ed_x0: 0.0,
            ed_s0: 0.0,
            ed_cap_de: [0.0; 3],
            ed_cap_para: [0.0; 3],
            eq_tipo: String::new(),
            eq_codigo: 0,
            eq_nome: String::new(),
            eq_de: 0,
            eq_para: 0,
            eq_barra: 0,
            eq_pot: 0.0,
            eq_r1: 0.0,
            eq_x1: 0.0,
            eq_r0: 0.0,
            eq_x0: 0.0,
            eq_tap: 0.0,
            eq_p: 0.0,
            eq_q: 0.0,
            eq_status: true,
            eq_grupo: String::new(),
            eq_area: String::new(),
            eq_circuito: 0,
            eq_capacidade: 0.0,
            eq_defasamento: 0.0,
            eq_conexao: "Estrela aterrado".into(),
            eq_conexao2: "Estrela aterrado".into(),
            eq_rn: 0.0,
            eq_xn: 0.0,
            eq_rn2: 0.0,
            eq_xn2: 0.0,
            eq_unidades: 1,
            top_atual: 0,
            top_nova: 0,
            top_nome: "Nova barra".into(),
            top_jumper: false,
            top_final: 0,
            top_eliminada: 0,
            top_comp_barra: 0,
            top_componentes: Vec::new(),
            sec_de: 0,
            sec_para: 0,
            sec_nova: 0,
            sec_nome: "Barra de seccionamento".into(),
            sec_percentual: 50.0,
            sec_reatores: false,
            ren_atual: 0,
            ren_nova: 0,
            loc_codigo: 0,
            term_lista: Vec::new(),
            term_carregada: false,
            term_id: 0,
            term_nome: String::new(),
            det_codigo: 0,
            det_resumo: String::new(),
            texto_conteudo: String::new(),
            texto_altura: 2.5,
            texto_cor: 7,
            niv_resumo: String::new(),
            eqv_barras: String::new(),
            zbus_resumo: String::new(),
            sim_barras: String::new(),
            sim_resumo: String::new(),
            eqv_resumo: String::new(),
            eqv_validar: false,
            leg_modulo_corrente: "kA".into(),
            leg_angulo_corrente: "grau".into(),
            leg_modulo_tensao: "kV".into(),
            leg_angulo_tensao: "grau".into(),
            leg_barras_nome: true,
            leg_equip_numero: true,
            leg_trafo_tap: true,
            leg_trafo_defasamento: true,
            leg_carregada: false,
            ed_handle: String::new(),
            doc_pagina: String::new(),
            doc_lista: Vec::new(),
            doc_lista_db: String::new(),
            mensagem: String::new(),
        }
    }
}

/// Abre (ou ativa) o painel dockado, como o QuickCalc faz.
pub fn open(app: &mut CadApp) -> Result<Value, String> {
    let panel = crate::docking::Panel::Anacad;
    crate::docking::command(app, false, &json!({ "operation": "open", "panel": panel.id() }))
}

/// Aplica um modo de exibição ao desenho atual.
pub fn aplicar_exibicao(app: &mut CadApp, modo: &str) -> String {
    let Ok(doc) = app.session.doc() else { return "sem desenho".to_string() };
    let mut alvos: Vec<(cadcraft_doc::Handle, bool)> = Vec::new();
    for h in doc.model.handles() {
        let Some(e) = doc.entity(h) else { continue };
        let app_name = e.common.xdata.iter().find(|t| t.code == 1001).map(|t| t.value.as_str()).unwrap_or("");
        let bloco = match &e.kind {
            cadcraft_doc::EntityKind::Insert(i) => i.block.as_str(),
            _ => "",
        };
        let equipamento = bloco.starts_with("ANACAD_EQ_") || bloco.starts_with("ANACAD_SIMB_");
        let barra_linha = matches!(e.kind, cadcraft_doc::EntityKind::LwPolyline(_)) && matches!(app_name, "ANACAD_BARRA" | "ANACAD_LINHA");
        let mut visivel = true;
        match modo {
            "esquematico" => visivel = !barra_linha,
            "mutuas" if equipamento && bloco != "ANACAD_EQ_impedancia_mutua" => visivel = false,
            _ => {}
        }
        alvos.push((h, visivel));
    }
    let Ok(doc) = app.session.doc_mut() else { return "sem desenho".to_string() };
    for (h, visivel) in alvos {
        let _ = doc.modify_entity(h, |e| e.common.visible = visivel);
    }
    match modo {
        "esquematico" => "modo esquemático: barras/linhas ocultas".to_string(),
        "mutuas" => "modo mútuas: só impedâncias mútuas entre os equipamentos".to_string(),
        "unifilar" => "modo unifilar (o desenho já é unifilar)".to_string(),
        "nao_desenhados" => "todos os elementos do banco já estão desenhados".to_string(),
        _ => "desenho normal".to_string(),
    }
}

/// Conecta um banco (cria ou abre) e atualiza o status./// Conecta um banco (cria ou abre) e atualiza o status.
pub fn conectar_banco(app: &mut CadApp, cmd: &str, arquivo: &str) -> Result<Value, String> {
    let v = app.run(cmd, json!({ "arquivo": arquivo }))?;
    app.ui.anacad_db = arquivo.to_string();
    if let Some(doc) = v.get("primeiroDocumento").and_then(Value::as_i64) {
        app.ui.anacad_documento = doc;
    }
    let nome = v.get("nome").and_then(Value::as_str).unwrap_or(arquivo).to_string();
    app.status = Some((format!("Banco: {nome}"), crate::now_ms()));
    Ok(v)
}

/// Diálogos das ferramentas abertos pelo tool set.
pub fn dialogo(app: &mut CadApp, ctx: &egui::Context) {
    if let Some(handle) = app.session.anacad_equip_handle.take() {
        app.ui.anacad_handle = handle;
        app.ui.anacad_dialogo = "editar_equipamento".into();
    }
    if let Some(handle) = app.session.anacad_edit_handle.take() {
        let tipo = tipo_da_entidade(app, &handle).unwrap_or("editar_barra");
        app.ui.anacad_handle = handle;
        app.ui.anacad_dialogo = tipo.into();
    }
    if app.ui.anacad_dialogo.is_empty() {
        return;
    }
    let acao = app.ui.anacad_dialogo.clone();
    let titulo = match acao.as_str() {
        "banco_novo" => "Novo banco",
        "banco_abrir" => "Abrir banco",
        "documento_novo" => "Novo documento",
        "separar_barra" => "Separar Barra",
        "seccionar_linha" => "Seccionamento de Linha",
        "renumerar_barra" => "Renumerar Barra",
        "localizar_barra" => "Localizar Barra",
        "renomear_terminal" => "Renomear Terminal",
        "localizar_terminal" => "Localizar Terminal",
        "detalhar_barra" => "Detalhamento de Conexões de Barra",
        "texto_livre" => "Texto Livre",
        "opcoes_legenda" => "Opções de Legenda",
        "niveis_curto" => "Níveis de Curto-Circuito",
        "equivalente" => "Equivalente de Rede",
        "zbus" => "Matriz Zbarra",
        "simultaneo" => "Faltas Simultâneas",
        "unir_barras" => "Unir Barras",
        "criar_barra" => "Criar barra (LWPOLYLINE 0,3)",
        "criar_linha" => "Criar linha (LWPOLYLINE 0,0)",
        "criar_legenda" => "Criar legenda (X)",
        "editar_barra" => "Dados de Barra",
        "editar_linha" => "Dados de Linha de Transmissão",
        "editar_equipamento" => "Dados do Equipamento",
        "atribuir_barra" => "Atribuir barra a polilinha",
        "atribuir_linha" => "Atribuir linha a polilinha",
        "mover" => "Mover por handle",
        "apagar" => "Apagar por handle",
        "importar_lst" => "Importar .LST",
        "importar_ana" => "Importar .ANA",
        "exportar" => "Exportar .LST",
        "curto" => "Curto-circuito",
        "consultar" => "Consultar banco",
        "topologia" => "Calcular topologia",
        "comprimentos" => "Calcular comprimentos",
        "ybus" => "Montar Ybus",
        "resultados" => "Resultados",
        "relatorio" => "Relatório",
        _ => return,
    };
    let mut estado: Estado = ctx.data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    estado.db = app.ui.anacad_db.clone();
    estado.documento = app.ui.anacad_documento;
    if !app.ui.anacad_handle.is_empty() {
        estado.comp_handle = app.ui.anacad_handle.clone();
        estado.ed_handle.clear();
        app.ui.anacad_handle.clear();
    }
    let mut aberto = true;
    let mut fechar = false;
    let mut notificacao: Option<String> = None;
    let largura = if acao.starts_with("editar_") { 660.0 } else { 440.0 };
    let centro = ctx.content_rect().center() - egui::vec2(largura / 2.0, 220.0);
    egui::Window::new(titulo)
        .id(egui::Id::new("anacad.dialogo"))
        .default_width(largura)
        .default_pos(centro)
        .collapsible(false)
        .resizable(true)
        .open(&mut aberto)
        .show(ctx, |ui| {
            match acao.as_str() {
                "banco_novo" | "banco_abrir" => {
                    let cmd = if acao == "banco_novo" { "anacad.banco.criar" } else { "anacad.banco.abrir" };
                    let tem_picker = if acao == "banco_novo" { app.services.pick_db_save.is_some() } else { app.services.pick_db_open.is_some() };
                    if !estado.db.trim().is_empty() {
                        ui.label("Selecionado:");
                        ui.monospace(&estado.db);
                    }
                    if tem_picker {
                        let rotulo = if acao == "banco_novo" { "Escolher pasta/nome e criar..." } else { "Escolher banco para abrir..." };
                        if ui.button(rotulo).clicked() {
                            let escolhido = if acao == "banco_novo" {
                                app.services.pick_db_save.as_ref().and_then(|f| f("anacad.db"))
                            } else {
                                app.services.pick_db_open.as_ref().and_then(|f| f())
                            };
                            if let Some(arquivo) = escolhido {
                                estado.db = arquivo.clone();
                                match conectar_banco(app, cmd, &arquivo) {
                                    Ok(_) => estado.mensagem = "banco conectado".to_string(),
                                    Err(e) => estado.mensagem = format!("erro: {e}"),
                                }
                            }
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Arquivo .db");
                            ui.add(egui::TextEdit::singleline(&mut estado.db).desired_width(340.0));
                        });
                        let rotulo = if acao == "banco_novo" { "Criar banco" } else { "Abrir banco" };
                        if ui.button(rotulo).clicked() {
                            match conectar_banco(app, cmd, &estado.db.clone()) {
                                Ok(_) => estado.mensagem = "banco conectado".to_string(),
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                    }
                    if !estado.mensagem.is_empty() {
                        ui.separator();
                        ui.label(&estado.mensagem);
                    }
                }
                "documento_novo" => {
                    if estado.db.trim().is_empty() {
                        ui.label("Abra ou crie um banco primeiro.");
                    } else {
                        if estado.doc_lista_db != estado.db {
                            estado.doc_lista = carregar_documentos(app, &estado.db);
                            estado.doc_lista_db = estado.db.clone();
                        }
                        if ui.button("Atualizar lista").clicked() {
                            estado.doc_lista = carregar_documentos(app, &estado.db);
                        }
                        ui.separator();
                        ui.label("Documentos existentes:");
                        let mut escolhido = None;
                        for (id, nome, pagina) in &estado.doc_lista {
                            let pagina = if pagina.is_empty() { String::new() } else { format!(" — pág. {pagina}") };
                            let rotulo = format!("#{id}  {nome}{pagina}");
                            if ui.selectable_label(estado.documento == *id, rotulo).clicked() {
                                escolhido = Some(*id);
                            }
                        }
                        if let Some(id) = escolhido {
                            estado.documento = id;
                            app.ui.anacad_documento = id;
                            estado.mensagem = format!("documento ativo: #{id}");
                        }
                        ui.separator();
                        ui.label("Novo documento:");
                        ui.horizontal(|ui| {
                            ui.label("Nome");
                            ui.add(egui::TextEdit::singleline(&mut estado.ed_nome).desired_width(200.0));
                            ui.label("Página");
                            ui.add(egui::TextEdit::singleline(&mut estado.doc_pagina).desired_width(70.0));
                        });
                        if ui.button("Criar documento").clicked() {
                            match app.run("anacad.documento.criar", json!({ "db": estado.db, "nome": estado.ed_nome, "pagina": estado.doc_pagina })) {
                                Ok(v) => {
                                    if let Some(doc) = v.get("documento").and_then(Value::as_i64) {
                                        app.ui.anacad_documento = doc;
                                        estado.documento = doc;
                                    }
                                    estado.mensagem = format!("documento criado: {}", estado.ed_nome);
                                    estado.doc_lista = carregar_documentos(app, &estado.db);
                                }
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                    }
                    if !estado.mensagem.is_empty() {
                        ui.separator();
                        ui.label(&estado.mensagem);
                    }
                }
                "criar_barra" => {
                    ui.horizontal(|ui| {
                        ui.label("início x");
                        ui.add(egui::DragValue::new(&mut estado.barra_x).speed(1.0));
                        ui.label("y");
                        ui.add(egui::DragValue::new(&mut estado.barra_y).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("fim x");
                        ui.add(egui::DragValue::new(&mut estado.barra_x2).speed(1.0));
                        ui.label("y");
                        ui.add(egui::DragValue::new(&mut estado.barra_y2).speed(1.0));
                    });
                    if ui.button("Criar barra").clicked() {
                        let x2 = if estado.barra_x2 == 0.0 && estado.barra_y2 == 0.0 { estado.barra_x + 1.0 } else { estado.barra_x2 };
                        let y2 = if estado.barra_x2 == 0.0 && estado.barra_y2 == 0.0 { estado.barra_y } else { estado.barra_y2 };
                        estado.mensagem = criar_barra(app, &estado.db, estado.documento, 0, estado.barra_x, estado.barra_y, x2, y2);
                    }
                }
                "criar_linha" => {
                    ui.horizontal(|ui| {
                        ui.label("início x");
                        ui.add(egui::DragValue::new(&mut estado.barra_x).speed(1.0));
                        ui.label("y");
                        ui.add(egui::DragValue::new(&mut estado.barra_y).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("fim x");
                        ui.add(egui::DragValue::new(&mut estado.barra_x2).speed(1.0));
                        ui.label("y");
                        ui.add(egui::DragValue::new(&mut estado.barra_y2).speed(1.0));
                    });
                    if ui.button("Criar linha").clicked() {
                        let cmd = json!({"db": estado.db, "documento": estado.documento,
                        "x1": estado.barra_x, "y1": estado.barra_y,
                        "x2": estado.barra_x2, "y2": estado.barra_y2});
                        estado.mensagem = match app.run("anacad.linha.criar", cmd) {
                            Ok(v) => format!("linha criada: {}", v.get("handle").and_then(Value::as_str).unwrap_or("-")),
                            Err(e) => format!("erro: {e}"),
                        };
                    }
                }
                "criar_legenda" => {
                    ui.horizontal(|ui| {
                        ui.label("número");
                        ui.add(egui::DragValue::new(&mut estado.legenda_numero).speed(1.0));
                        ui.label("x");
                        ui.add(egui::DragValue::new(&mut estado.legenda_x).speed(1.0));
                        ui.label("y");
                        ui.add(egui::DragValue::new(&mut estado.legenda_y).speed(1.0));
                    });
                    if ui.button("Criar legenda").clicked() {
                        estado.mensagem = criar_legenda(app, &estado.db, estado.documento, estado.legenda_numero, estado.legenda_x, estado.legenda_y);
                    }
                }
                "opcoes_legenda" => {
                    if !estado.leg_carregada {
                        if let Ok(v) = app.run("anacad.legenda.config.ler", json!({"db": estado.db}))
                            && let Some(d) = v.get("dados")
                        {
                            if let Some(s) = d.get("moduloCorrente").and_then(Value::as_str) {
                                estado.leg_modulo_corrente = s.into();
                            }
                            if let Some(s) = d.get("anguloCorrente").and_then(Value::as_str) {
                                estado.leg_angulo_corrente = s.into();
                            }
                            if let Some(s) = d.get("moduloTensao").and_then(Value::as_str) {
                                estado.leg_modulo_tensao = s.into();
                            }
                            if let Some(s) = d.get("anguloTensao").and_then(Value::as_str) {
                                estado.leg_angulo_tensao = s.into();
                            }
                            if let Some(b) = d.get("barrasNome").and_then(Value::as_bool) {
                                estado.leg_barras_nome = b;
                            }
                            if let Some(b) = d.get("equipNumero").and_then(Value::as_bool) {
                                estado.leg_equip_numero = b;
                            }
                            if let Some(b) = d.get("trafoTap").and_then(Value::as_bool) {
                                estado.leg_trafo_tap = b;
                            }
                            if let Some(b) = d.get("trafoDefasamento").and_then(Value::as_bool) {
                                estado.leg_trafo_defasamento = b;
                            }
                        }
                        estado.leg_carregada = true;
                    }
                    ui.label(egui::RichText::new("Unidades de Corrente").strong());
                    ui.horizontal(|ui| {
                        ui.label("Módulo");
                        ui.add(egui::TextEdit::singleline(&mut estado.leg_modulo_corrente).desired_width(60.0));
                        ui.label("Ângulo");
                        ui.add(egui::TextEdit::singleline(&mut estado.leg_angulo_corrente).desired_width(60.0));
                    });
                    ui.label(egui::RichText::new("Unidades de Tensão").strong());
                    ui.horizontal(|ui| {
                        ui.label("Módulo");
                        ui.add(egui::TextEdit::singleline(&mut estado.leg_modulo_tensao).desired_width(60.0));
                        ui.label("Ângulo");
                        ui.add(egui::TextEdit::singleline(&mut estado.leg_angulo_tensao).desired_width(60.0));
                    });
                    ui.separator();
                    ui.checkbox(&mut estado.leg_barras_nome, "Identificar barras por nome");
                    ui.checkbox(&mut estado.leg_equip_numero, "Identificar equipamentos por número");
                    ui.checkbox(&mut estado.leg_trafo_tap, "Mostrar tap de transformadores");
                    ui.checkbox(&mut estado.leg_trafo_defasamento, "Mostrar defasamento de transformadores");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Restaurar padrão").clicked() {
                            estado.leg_modulo_corrente = "kA".into();
                            estado.leg_angulo_corrente = "grau".into();
                            estado.leg_modulo_tensao = "kV".into();
                            estado.leg_angulo_tensao = "grau".into();
                            estado.leg_barras_nome = true;
                            estado.leg_equip_numero = true;
                            estado.leg_trafo_tap = true;
                            estado.leg_trafo_defasamento = true;
                        }
                        if ui.button("Salvar").clicked() {
                            let dados = json!({
                                "moduloCorrente": estado.leg_modulo_corrente,
                                "anguloCorrente": estado.leg_angulo_corrente,
                                "moduloTensao": estado.leg_modulo_tensao,
                                "anguloTensao": estado.leg_angulo_tensao,
                                "barrasNome": estado.leg_barras_nome,
                                "equipNumero": estado.leg_equip_numero,
                                "trafoTap": estado.leg_trafo_tap,
                                "trafoDefasamento": estado.leg_trafo_defasamento,
                            });
                            match app.run("anacad.legenda.config.gravar", json!({"db": estado.db, "dados": dados})) {
                                Ok(_) => {
                                    fechar = true;
                                    notificacao = Some("opções de legenda salvas".to_string());
                                }
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                    });
                }
                "equivalente" => {
                    ui.label("Barras a manter (códigos separados por vírgula):");
                    ui.add(egui::TextEdit::singleline(&mut estado.eqv_barras).desired_width(420.0));
                    ui.checkbox(&mut estado.eqv_validar, "validar contra a rede original");
                    if ui.button("Calcular equivalente").clicked() {
                        let barras: Vec<i64> = estado
                            .eqv_barras
                            .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
                            .filter_map(|t| t.trim().parse::<i64>().ok())
                            .collect();
                        match app.run(
                            "anacad.equivalente",
                            json!({"db": estado.db, "documento": estado.documento, "barras": barras, "validar": estado.eqv_validar}),
                        ) {
                            Ok(v) => {
                                estado.eqv_resumo = format!(
                                    "barras: {:?}\npares não nulos: {}\nsimétrica: {}\nmaior diagonal: {:.4}",
                                    v.get("barras")
                                        .and_then(Value::as_array)
                                        .map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                                        .unwrap_or_default(),
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
                "simultaneo" => {
                    ui.label("Barras com falta trifásica simultânea (códigos separados por vírgula):");
                    ui.add(egui::TextEdit::singleline(&mut estado.sim_barras).desired_width(420.0));
                    if ui.button("Calcular faltas simultâneas").clicked() {
                        let barras: Vec<i64> = estado
                            .sim_barras
                            .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
                            .filter_map(|t| t.trim().parse::<i64>().ok())
                            .collect();
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
                "zbus" => {
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
                "niveis_curto" => {
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
                "texto_livre" => {
                    ui.label("Texto:");
                    ui.add(egui::TextEdit::multiline(&mut estado.texto_conteudo).desired_width(420.0).desired_rows(4));
                    ui.horizontal(|ui| {
                        ui.label("Altura");
                        ui.add(egui::DragValue::new(&mut estado.texto_altura).range(0.1..=500.0).speed(0.1));
                        ui.label("Cor ACI");
                        ui.add(egui::DragValue::new(&mut estado.texto_cor).range(1..=255));
                    });
                    ui.separator();
                    if ui.button("Inserir no desenho").clicked() {
                        app.session.anacad_texto = estado.texto_conteudo.clone();
                        app.session.anacad_texto_altura = estado.texto_altura;
                        app.session.anacad_texto_cor = estado.texto_cor;
                        app.ui.anacad_dialogo.clear();
                        app.start("anacad.texto.inserir");
                    }
                }
                "detalhar_barra" => {
                    ui.horizontal(|ui| {
                        ui.label("Código da barra");
                        ui.add(egui::DragValue::new(&mut estado.det_codigo).speed(1.0));
                        if ui.button("Atualizar").clicked() {
                            match app
                                .run("anacad.barra.detalhar", json!({"db": estado.db, "documento": estado.documento, "codigo": estado.det_codigo}))
                            {
                                Ok(v) => {
                                    let componentes = v.get("componentes").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
                                    let circuitos = v.get("circuitos").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
                                    let vizinhos = v
                                        .get("vizinhos")
                                        .and_then(Value::as_array)
                                        .map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                                        .unwrap_or_default();
                                    estado.det_resumo = format!(
                                        "barra {}\ncomponentes: {componentes}\ncircuitos: {circuitos}\nvizinhos: {vizinhos:?}",
                                        estado.det_codigo
                                    );
                                }
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                    });
                    ui.separator();
                    ui.monospace(&estado.det_resumo);
                    if !estado.mensagem.is_empty() {
                        ui.separator();
                        ui.label(&estado.mensagem);
                    }
                }
                "renomear_terminal" | "localizar_terminal" => {
                    if !estado.term_carregada {
                        estado.term_lista = carregar_terminais(app, &estado.db, estado.documento);
                        estado.term_carregada = true;
                        if let Some((id, nome, _, _)) = estado.term_lista.first() {
                            estado.term_id = *id;
                            estado.term_nome = nome.clone();
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Atualizar lista").clicked() {
                            estado.term_lista = carregar_terminais(app, &estado.db, estado.documento);
                        }
                        ui.label(format!("{} terminal(is)", estado.term_lista.len()));
                    });
                    let mut escolhido = None;
                    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        for (id, nome, _, _) in &estado.term_lista {
                            if ui.selectable_label(estado.term_id == *id, format!("#{id} {nome}")).clicked() {
                                escolhido = Some((*id, nome.clone()));
                            }
                        }
                    });
                    if let Some((id, nome)) = escolhido {
                        estado.term_id = id;
                        estado.term_nome = nome;
                    }
                    if acao == "renomear_terminal" {
                        ui.horizontal(|ui| {
                            ui.label("Novo nome");
                            ui.add(egui::TextEdit::singleline(&mut estado.term_nome).desired_width(240.0));
                        });
                        if ui.button("Renomear").clicked() {
                            match app.run("anacad.terminal.renomear", json!({"db": estado.db, "id": estado.term_id, "nome": estado.term_nome})) {
                                Ok(_) => {
                                    estado.term_carregada = false;
                                    fechar = true;
                                    notificacao = Some("terminal renomeado".to_string());
                                }
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                    } else if ui.button("Localizar").clicked()
                        && let Some((_, _, x, y)) = estado.term_lista.iter().find(|(id, _, _, _)| *id == estado.term_id)
                    {
                        let _ = app.run("zoom", json!({"mode": "center", "center": [x, y], "height": 120.0}));
                        fechar = true;
                        notificacao = Some("terminal localizado".to_string());
                    }
                }
                "renumerar_barra" => {
                    ui.horizontal(|ui| {
                        ui.label("Código atual");
                        ui.add(egui::DragValue::new(&mut estado.ren_atual).speed(1.0));
                        ui.label("Novo código");
                        ui.add(egui::DragValue::new(&mut estado.ren_nova).speed(1.0));
                    });
                    ui.separator();
                    if ui.button("Renumerar").clicked() {
                        let cmd = json!({"db": estado.db, "documento": estado.documento,
                        "atual": estado.ren_atual, "nova": estado.ren_nova});
                        match app.run("anacad.barra.renumerar", cmd) {
                            Ok(_) => {
                                fechar = true;
                                notificacao = Some(format!("barra {} renumerada para {}", estado.ren_atual, estado.ren_nova));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "localizar_barra" => {
                    ui.horizontal(|ui| {
                        ui.label("Código da barra");
                        ui.add(egui::DragValue::new(&mut estado.loc_codigo).speed(1.0));
                    });
                    ui.separator();
                    if ui.button("Localizar").clicked() {
                        match app.run("anacad.barra.localizar", json!({"db": estado.db, "documento": estado.documento, "codigo": estado.loc_codigo}))
                        {
                            Ok(v) => {
                                let x = v.get("x").and_then(Value::as_f64).unwrap_or(0.0);
                                let y = v.get("y").and_then(Value::as_f64).unwrap_or(0.0);
                                let _ = app.run("zoom", json!({"mode": "center", "center": [x, y], "height": 120.0}));
                                fechar = true;
                                notificacao = Some(format!("barra {} localizada", estado.loc_codigo));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "seccionar_linha" => {
                    ui.horizontal(|ui| {
                        ui.label("Barra De");
                        ui.add(egui::DragValue::new(&mut estado.sec_de).speed(1.0));
                        ui.label("Barra Para");
                        ui.add(egui::DragValue::new(&mut estado.sec_para).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Nova barra");
                        ui.add(egui::DragValue::new(&mut estado.sec_nova).speed(1.0));
                        ui.label("Nome");
                        ui.add(egui::TextEdit::singleline(&mut estado.sec_nome).desired_width(220.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Percentual a partir da barra De (%)");
                        ui.add(egui::DragValue::new(&mut estado.sec_percentual).range(0.1..=99.9).speed(1.0));
                        ui.checkbox(&mut estado.sec_reatores, "Eliminar reatores");
                    });
                    ui.separator();
                    if ui.button("Executar").clicked() {
                        let cmd = json!({
                            "db": estado.db, "documento": estado.documento,
                            "de": estado.sec_de, "para": estado.sec_para,
                            "nova": estado.sec_nova, "nome": estado.sec_nome,
                            "percentual": estado.sec_percentual, "reatores": estado.sec_reatores,
                        });
                        match app.run("anacad.linha.seccionar", cmd) {
                            Ok(_) => {
                                fechar = true;
                                notificacao = Some(format!("linha seccionada em {}", estado.sec_nova));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "separar_barra" => {
                    ui.horizontal(|ui| {
                        ui.label("Barra atual");
                        ui.add(egui::DragValue::new(&mut estado.top_atual).speed(1.0));
                        ui.label("Nova barra");
                        ui.add(egui::DragValue::new(&mut estado.top_nova).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Nome da nova");
                        ui.add(egui::TextEdit::singleline(&mut estado.top_nome).desired_width(220.0));
                        ui.checkbox(&mut estado.top_jumper, "JUMPER entre as barras");
                    });
                    if estado.top_comp_barra != estado.top_atual {
                        estado.top_componentes = carregar_componentes_barra(app, &estado.db, estado.documento, estado.top_atual);
                        estado.top_comp_barra = estado.top_atual;
                    }
                    ui.separator();
                    ui.label("Componentes da barra atual (marque os que vão para a nova):");
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (id, nome, marcado) in &mut estado.top_componentes {
                            ui.checkbox(marcado, format!("#{id} {nome}"));
                        }
                    });
                    ui.separator();
                    if ui.button("Separar").clicked() {
                        let componentes: Vec<i64> = estado.top_componentes.iter().filter(|(_, _, marcado)| *marcado).map(|(id, _, _)| *id).collect();
                        let cmd = json!({
                            "db": estado.db, "documento": estado.documento,
                            "atual": estado.top_atual, "nova": estado.top_nova,
                            "nome": estado.top_nome, "componentes": componentes,
                            "jumper": estado.top_jumper,
                        });
                        match app.run("anacad.barra.separar", cmd) {
                            Ok(v) => {
                                fechar = true;
                                notificacao = Some(format!(
                                    "barra {} separada ({} componentes)",
                                    estado.top_nova,
                                    v.get("componentes").and_then(Value::as_i64).unwrap_or(0)
                                ));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "unir_barras" => {
                    ui.horizontal(|ui| {
                        ui.label("Barra final");
                        ui.add(egui::DragValue::new(&mut estado.top_final).speed(1.0));
                        ui.label("Barra eliminada");
                        ui.add(egui::DragValue::new(&mut estado.top_eliminada).speed(1.0));
                    });
                    ui.separator();
                    if ui.button("Unir").clicked() {
                        let cmd = json!({
                            "db": estado.db, "documento": estado.documento,
                            "final": estado.top_final, "eliminada": estado.top_eliminada,
                        });
                        match app.run("anacad.barra.unir", cmd) {
                            Ok(_) => {
                                fechar = true;
                                notificacao = Some(format!("barra {} unida à {}", estado.top_eliminada, estado.top_final));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "editar_barra" => {
                    if ui.button("Selecionar no desenho").clicked() {
                        app.ui.anacad_dialogo.clear();
                        app.ui.anacad_handle.clear();
                        app.start("anacad.barra.editar");
                    }
                    let handle = if !estado.comp_handle.trim().is_empty() {
                        estado.comp_handle.clone()
                    } else {
                        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                    };
                    estado.comp_handle = handle.clone();
                    if estado.ed_handle != handle {
                        if let Some(dados) = dados_entidade(app, &handle) {
                            preencher_estado(&mut estado, &dados);
                        }
                        estado.ed_handle = handle.clone();
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Identificação").strong());
                    ui.horizontal(|ui| {
                        ui.label("handle");
                        ui.add(egui::TextEdit::singleline(&mut estado.comp_handle).desired_width(90.0));
                        ui.label("código");
                        ui.add(egui::DragValue::new(&mut estado.barra_codigo).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("nome");
                        ui.add(egui::TextEdit::singleline(&mut estado.ed_nome).desired_width(160.0));
                        ui.label("área");
                        ui.add(egui::TextEdit::singleline(&mut estado.ed_area).desired_width(120.0));
                    });
                    ui.separator();
                    ui.label(egui::RichText::new("Elétrico").strong());
                    ui.horizontal(|ui| {
                        ui.label("base tensão kV");
                        ui.add(egui::DragValue::new(&mut estado.ed_vbase).speed(1.0));
                        ui.label("capac. interrupção kA");
                        ui.add(egui::DragValue::new(&mut estado.ed_capacidade).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("capac. amarra kA");
                        for (i, v) in estado.ed_cap_am.iter_mut().enumerate() {
                            ui.add(egui::DragValue::new(v).speed(1.0).prefix(format!("{}: ", i + 1)));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("tipo");
                        egui::ComboBox::from_id_salt("anacad.ed.tipo").selected_text(&estado.ed_tipo).show_ui(ui, |ui| {
                            for t in ["Normal", "Fictícia de transformador", "Auxiliar", "Autotrafo"] {
                                ui.selectable_value(&mut estado.ed_tipo, t.to_string(), t);
                            }
                        });
                        ui.checkbox(&mut estado.ed_status, "Ligado");
                        ui.checkbox(&mut estado.ed_disjuntor, "Disjuntor na SE");
                    });
                    ui.separator();
                    ui.label(egui::RichText::new("Pré-falta").strong());
                    ui.horizontal(|ui| {
                        ui.label("pré-falta módulo pu");
                        ui.add(egui::DragValue::new(&mut estado.ed_pre_mod).speed(1.0));
                        ui.label("ângulo°");
                        ui.add(egui::DragValue::new(&mut estado.ed_pre_ang).speed(1.0));
                    });
                    ui.separator();
                    if ui.button("Aplicar").clicked() {
                        let dados = json!({"codigo": estado.barra_codigo, "nome": estado.ed_nome, "area": estado.ed_area,
                        "vbase": estado.ed_vbase, "tipo": estado.ed_tipo, "capacidadeInterrupcao": estado.ed_capacidade,
                        "preFaltaModulo": estado.ed_pre_mod, "preFaltaAngulo": estado.ed_pre_ang,
                        "disjuntorSE": estado.ed_disjuntor, "status": estado.ed_status,
                        "capacidadesAmarre": estado.ed_cap_am.to_vec()});
                        let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle,
                        "codigo": estado.barra_codigo, "dados": dados});
                        match app.run("anacad.barra.editar", cmd) {
                            Ok(_) => {
                                fechar = true;
                                notificacao = Some(format!("barra {handle} atualizada"));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "editar_linha" => {
                    if ui.button("Selecionar no desenho").clicked() {
                        app.ui.anacad_dialogo.clear();
                        app.ui.anacad_handle.clear();
                        app.start("anacad.linha.editar");
                    }
                    let handle = if !estado.comp_handle.trim().is_empty() {
                        estado.comp_handle.clone()
                    } else {
                        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                    };
                    estado.comp_handle = handle.clone();
                    if estado.ed_handle != handle {
                        if let Some(dados) = dados_entidade(app, &handle) {
                            preencher_estado(&mut estado, &dados);
                        }
                        estado.ed_handle = handle.clone();
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Identificação").strong());
                    ui.horizontal(|ui| {
                        ui.label("handle");
                        ui.add(egui::TextEdit::singleline(&mut estado.comp_handle).desired_width(90.0));
                        ui.label("de");
                        ui.add(egui::DragValue::new(&mut estado.linha_de).speed(1.0));
                        ui.label("para");
                        ui.add(egui::DragValue::new(&mut estado.linha_para).speed(1.0));
                        ui.label("circuito");
                        ui.add(egui::DragValue::new(&mut estado.ed_circuito).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("nome");
                        ui.add(egui::TextEdit::singleline(&mut estado.ed_nome).desired_width(160.0));
                        ui.label("área");
                        ui.add(egui::TextEdit::singleline(&mut estado.ed_area).desired_width(120.0));
                    });
                    ui.separator();
                    ui.label(egui::RichText::new("Elétrico").strong());
                    ui.horizontal(|ui| {
                        ui.label("potência MVA");
                        ui.add(egui::DragValue::new(&mut estado.ed_potencia).speed(1.0));
                        ui.label("comprimento km");
                        ui.add(egui::DragValue::new(&mut estado.ed_comprimento).speed(1.0));
                        ui.checkbox(&mut estado.ed_status, "Ligado");
                    });
                    ui.separator();
                    ui.label(egui::RichText::new("Sequência positiva").strong());
                    ui.horizontal(|ui| {
                        ui.label("R1%");
                        ui.add(egui::DragValue::new(&mut estado.ed_r1).speed(1.0));
                        ui.label("X1%");
                        ui.add(egui::DragValue::new(&mut estado.ed_x1).speed(1.0));
                        ui.label("S1 Mvar");
                        ui.add(egui::DragValue::new(&mut estado.ed_s1).speed(1.0));
                    });
                    ui.separator();
                    ui.label(egui::RichText::new("Sequência zero").strong());
                    ui.horizontal(|ui| {
                        ui.label("R0%");
                        ui.add(egui::DragValue::new(&mut estado.ed_r0).speed(1.0));
                        ui.label("X0%");
                        ui.add(egui::DragValue::new(&mut estado.ed_x0).speed(1.0));
                        ui.label("S0 Mvar");
                        ui.add(egui::DragValue::new(&mut estado.ed_s0).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("capac. interrupção De kA");
                        for (i, v) in estado.ed_cap_de.iter_mut().enumerate() {
                            ui.add(egui::DragValue::new(v).speed(1.0).prefix(format!("{}: ", i + 1)));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("capac. interrupção Para kA");
                        for (i, v) in estado.ed_cap_para.iter_mut().enumerate() {
                            ui.add(egui::DragValue::new(v).speed(1.0).prefix(format!("{}: ", i + 1)));
                        }
                    });
                    ui.separator();
                    if ui.button("Aplicar").clicked() {
                        let dados = json!({"de": estado.linha_de, "para": estado.linha_para, "nome": estado.ed_nome,
                        "area": estado.ed_area, "numeroCircuito": estado.ed_circuito,
                        "potenciaNominal": estado.ed_potencia, "comprimento": estado.ed_comprimento,
                        "R1": estado.ed_r1, "X1": estado.ed_x1, "S1": estado.ed_s1,
                        "R0": estado.ed_r0, "X0": estado.ed_x0, "S0": estado.ed_s0,
                        "capacidadesDe": estado.ed_cap_de.to_vec(), "capacidadesPara": estado.ed_cap_para.to_vec(),
                        "status": estado.ed_status});
                        let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle,
                        "de": estado.linha_de, "para": estado.linha_para, "dados": dados});
                        match app.run("anacad.linha.editar", cmd) {
                            Ok(_) => {
                                fechar = true;
                                notificacao = Some(format!("linha {handle} atualizada"));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "editar_equipamento" => {
                    let handle = estado.comp_handle.clone();
                    if estado.ed_handle != handle {
                        if let Some(dados) = dados_entidade(app, &handle) {
                            preencher_equipamento(&mut estado, &dados);
                        }
                        estado.ed_handle = handle.clone();
                    }
                    let conexoes = ["Delta", "Estrela não aterrado", "Estrela aterrado"];
                    ui.separator();
                    ui.label(egui::RichText::new("Identificação").strong());
                    ui.horizontal(|ui| {
                        ui.label("equipamento");
                        ui.monospace(if estado.eq_tipo.is_empty() { "-" } else { &estado.eq_tipo });
                        ui.label("código");
                        ui.add(egui::DragValue::new(&mut estado.eq_codigo).speed(1.0));
                        ui.checkbox(&mut estado.eq_status, "Ligado");
                    });
                    ui.horizontal(|ui| {
                        ui.label("nome");
                        ui.add(egui::TextEdit::singleline(&mut estado.eq_nome).desired_width(220.0));
                        ui.label("grupo");
                        ui.add(egui::TextEdit::singleline(&mut estado.eq_grupo).desired_width(100.0));
                        ui.label("área");
                        ui.add(egui::TextEdit::singleline(&mut estado.eq_area).desired_width(80.0));
                    });
                    let tipo = estado.eq_tipo.as_str();
                    if tipo == "Transformador" {
                        ui.separator();
                        ui.label(egui::RichText::new("Enrolamentos").strong());
                        ui.horizontal(|ui| {
                            ui.label("primário");
                            ui.add(egui::DragValue::new(&mut estado.eq_de).speed(1.0));
                            ui.label("secundário");
                            ui.add(egui::DragValue::new(&mut estado.eq_para).speed(1.0));
                            ui.label("circuito");
                            ui.add(egui::DragValue::new(&mut estado.eq_circuito).speed(1.0));
                            ui.label("capacidade MVA");
                            ui.add(egui::DragValue::new(&mut estado.eq_capacidade).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("R1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
                            ui.label("X1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
                            ui.label("R0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
                            ui.label("X0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("conexão primário");
                            egui::ComboBox::from_id_salt("eq.con1").selected_text(&estado.eq_conexao).show_ui(ui, |ui| {
                                for c in conexoes {
                                    ui.selectable_value(&mut estado.eq_conexao, c.to_string(), c);
                                }
                            });
                            ui.label("Rn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
                            ui.label("Xn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("conexão secundário");
                            egui::ComboBox::from_id_salt("eq.con2").selected_text(&estado.eq_conexao2).show_ui(ui, |ui| {
                                for c in conexoes {
                                    ui.selectable_value(&mut estado.eq_conexao2, c.to_string(), c);
                                }
                            });
                            ui.label("Rn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_rn2).speed(1.0));
                            ui.label("Xn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_xn2).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("defasamento°");
                            ui.add(egui::DragValue::new(&mut estado.eq_defasamento).speed(1.0));
                            ui.label("tap");
                            ui.add(egui::DragValue::new(&mut estado.eq_tap).speed(0.001));
                            ui.label("unidades");
                            ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
                        });
                    } else if tipo == "Gerador" || tipo == "Gerador Fonte de Corrente" || tipo == "Motor de Indução" {
                        ui.separator();
                        ui.label(egui::RichText::new("Máquina").strong());
                        ui.horizontal(|ui| {
                            ui.label("barra");
                            ui.add(egui::DragValue::new(&mut estado.eq_barra).speed(1.0));
                            ui.label("potência MVA");
                            ui.add(egui::DragValue::new(&mut estado.eq_pot).speed(1.0));
                            ui.label("unidades");
                            ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
                        });
                        ui.horizontal(|ui| {
                            ui.label("R1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
                            ui.label("X1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
                            ui.label("R0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
                            ui.label("X0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("conexão");
                            egui::ComboBox::from_id_salt("eq.con").selected_text(&estado.eq_conexao).show_ui(ui, |ui| {
                                for c in conexoes {
                                    ui.selectable_value(&mut estado.eq_conexao, c.to_string(), c);
                                }
                            });
                            ui.label("Rn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
                            ui.label("Xn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("P MW");
                            ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
                            ui.label("Q Mvar");
                            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
                        });
                    } else if tipo == "Carga" {
                        ui.separator();
                        ui.label(egui::RichText::new("Carga").strong());
                        ui.horizontal(|ui| {
                            ui.label("barra");
                            ui.add(egui::DragValue::new(&mut estado.eq_barra).speed(1.0));
                            ui.label("P MW");
                            ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
                            ui.label("Q Mvar");
                            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
                            ui.label("unidades");
                            ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
                        });
                        ui.horizontal(|ui| {
                            ui.label("conexão");
                            egui::ComboBox::from_id_salt("eq.con").selected_text(&estado.eq_conexao).show_ui(ui, |ui| {
                                for c in conexoes {
                                    ui.selectable_value(&mut estado.eq_conexao, c.to_string(), c);
                                }
                            });
                        });
                    } else if tipo == "Compensador Série" || tipo == "Impedância Mútua" {
                        ui.separator();
                        ui.label(egui::RichText::new(if tipo == "Impedância Mútua" { "Linhas acopladas" } else { "Circuito série" }).strong());
                        ui.horizontal(|ui| {
                            ui.label("de");
                            ui.add(egui::DragValue::new(&mut estado.eq_de).speed(1.0));
                            ui.label("para");
                            ui.add(egui::DragValue::new(&mut estado.eq_para).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("R1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
                            ui.label("X1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
                            ui.label("R0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
                            ui.label("X0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
                        });
                    } else if tipo == "Trafo de Aterramento" {
                        ui.separator();
                        ui.label(egui::RichText::new("Aterramento").strong());
                        ui.horizontal(|ui| {
                            ui.label("barra");
                            ui.add(egui::DragValue::new(&mut estado.eq_barra).speed(1.0));
                            ui.label("R0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
                            ui.label("X0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Rn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
                            ui.label("Xn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
                        });
                    } else if tipo == "Compensador Estático" {
                        ui.separator();
                        ui.label(egui::RichText::new("Compensador estático").strong());
                        ui.horizontal(|ui| {
                            ui.label("barra");
                            ui.add(egui::DragValue::new(&mut estado.eq_barra).speed(1.0));
                            ui.label("Q Mvar");
                            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("R1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
                            ui.label("X1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
                            ui.label("R0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
                            ui.label("X0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
                        });
                    } else if tipo.contains("Shunt") {
                        ui.separator();
                        ui.label(egui::RichText::new("Shunt").strong());
                        ui.horizontal(|ui| {
                            ui.label("de");
                            ui.add(egui::DragValue::new(&mut estado.eq_de).speed(1.0));
                            ui.label("para");
                            ui.add(egui::DragValue::new(&mut estado.eq_para).speed(1.0));
                            ui.label("Q Mvar");
                            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Rn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_rn).speed(1.0));
                            ui.label("Xn%");
                            ui.add(egui::DragValue::new(&mut estado.eq_xn).speed(1.0));
                        });
                    } else {
                        ui.separator();
                        ui.label(egui::RichText::new("Conexão").strong());
                        let usa_par = matches!(tipo, "Compensador Série" | "Impedância Mútua");
                        if usa_par {
                            ui.horizontal(|ui| {
                                ui.label("de");
                                ui.add(egui::DragValue::new(&mut estado.eq_de).speed(1.0));
                                ui.label("para");
                                ui.add(egui::DragValue::new(&mut estado.eq_para).speed(1.0));
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("barra");
                                ui.add(egui::DragValue::new(&mut estado.eq_barra).speed(1.0));
                            });
                        }
                        ui.horizontal(|ui| {
                            ui.label("potência MVA");
                            ui.add(egui::DragValue::new(&mut estado.eq_pot).speed(1.0));
                            ui.label("R1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(1.0));
                            ui.label("X1%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("R0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(1.0));
                            ui.label("X0%");
                            ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("P MW");
                            ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
                            ui.label("Q Mvar");
                            ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
                        });
                    }
                    ui.separator();
                    if ui.button("Aplicar").clicked() {
                        let dados = json!({
                            "tipo": estado.eq_tipo, "codigo": estado.eq_codigo, "nome": estado.eq_nome,
                            "grupo": estado.eq_grupo, "area": estado.eq_area,
                            "de": estado.eq_de, "para": estado.eq_para, "barra": estado.eq_barra,
                            "numeroCircuito": estado.eq_circuito, "capacidade": estado.eq_capacidade,
                            "potencia": estado.eq_pot, "R1": estado.eq_r1, "X1": estado.eq_x1,
                            "R0": estado.eq_r0, "X0": estado.eq_x0, "tap": estado.eq_tap,
                            "defasamento": estado.eq_defasamento,
                            "conexao": estado.eq_conexao, "conexao2": estado.eq_conexao2,
                            "Rn": estado.eq_rn, "Xn": estado.eq_xn, "Rn2": estado.eq_rn2, "Xn2": estado.eq_xn2,
                            "P": estado.eq_p, "Q": estado.eq_q, "unidades": estado.eq_unidades,
                            "status": estado.eq_status,
                        });
                        let cmd = json!({"db": estado.db, "handle": handle, "dados": dados});
                        match app.run("anacad.equipamento.editar", cmd) {
                            Ok(_) => {
                                fechar = true;
                                notificacao = Some(format!("equipamento {} atualizado", estado.eq_codigo));
                            }
                            Err(e) => estado.mensagem = format!("erro: {e}"),
                        }
                    }
                }
                "atribuir_barra" | "atribuir_linha" | "mover" | "apagar" => {
                    ui.horizontal(|ui| {
                        ui.label("handle");
                        ui.add(egui::TextEdit::singleline(&mut estado.comp_handle).desired_width(90.0));
                    });
                    if acao == "atribuir_barra" {
                        ui.horizontal(|ui| {
                            ui.label("código");
                            ui.add(egui::DragValue::new(&mut estado.barra_codigo).speed(1.0));
                        });
                        if ui.button("Atribuir barra").clicked() {
                            estado.mensagem = atribuir_barra(app, &estado.db, estado.documento, &estado.comp_handle, estado.barra_codigo);
                        }
                    } else if acao == "atribuir_linha" {
                        ui.horizontal(|ui| {
                            ui.label("de");
                            ui.add(egui::DragValue::new(&mut estado.linha_de).speed(1.0));
                            ui.label("para");
                            ui.add(egui::DragValue::new(&mut estado.linha_para).speed(1.0));
                        });
                        if ui.button("Atribuir linha").clicked() {
                            estado.mensagem =
                                atribuir_linha(app, &estado.db, estado.documento, &estado.comp_handle, estado.linha_de, estado.linha_para);
                        }
                    } else if acao == "mover" {
                        ui.horizontal(|ui| {
                            ui.label("dx");
                            ui.add(egui::DragValue::new(&mut estado.comp_dx).speed(1.0));
                            ui.label("dy");
                            ui.add(egui::DragValue::new(&mut estado.comp_dy).speed(1.0));
                        });
                        if ui.button("Mover").clicked() {
                            estado.mensagem = mover_componente(app, &estado.db, &estado.comp_handle, estado.comp_dx, estado.comp_dy);
                        }
                    } else if ui.button("Apagar").clicked() {
                        estado.mensagem = apagar_componente(app, &estado.db, &estado.comp_handle);
                    }
                }
                "importar_lst" | "importar_ana" => {
                    let arquivo = if acao == "importar_lst" { &mut estado.lst_arquivo } else { &mut estado.ana_arquivo };
                    ui.horizontal(|ui| {
                        ui.label("arquivo");
                        ui.add(egui::TextEdit::singleline(arquivo).desired_width(320.0));
                    });
                    if ui.button("Importar").clicked() {
                        estado.mensagem = if acao == "importar_lst" {
                            importar_lst(app, &estado.db, &estado.lst_arquivo)
                        } else {
                            importar_ana(app, &estado.db, estado.documento, &estado.ana_arquivo)
                        };
                    }
                }
                "exportar_ana" => {
                    ui.horizontal(|ui| {
                        ui.label("saída .ANA");
                        ui.add(egui::TextEdit::singleline(&mut estado.ana_arquivo).desired_width(320.0));
                    });
                    if ui.button("Salvar .ANA").clicked() {
                        estado.mensagem = match app
                            .run("anacad.ana.exportar", json!({ "db": estado.db, "documento": estado.documento, "arquivo": estado.ana_arquivo }))
                        {
                            Ok(_) => "arquivo .ANA gravado".to_string(),
                            Err(e) => format!("erro: {e}"),
                        };
                    }
                }
                "exportar" => {
                    ui.horizontal(|ui| {
                        ui.label("saída .lst");
                        ui.add(egui::TextEdit::singleline(&mut estado.saida_arquivo).desired_width(320.0));
                    });
                    if ui.button("Exportar").clicked() {
                        estado.mensagem = match app
                            .run("anacad.exportar", json!({ "db": estado.db, "documento": estado.documento, "saida": estado.saida_arquivo }))
                        {
                            Ok(_) => "tela exportada".to_string(),
                            Err(e) => format!("erro: {e}"),
                        };
                    }
                }
                "curto" => {
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
                                        a.get("modulo")
                                            .and_then(Value::as_f64)
                                            .unwrap_or(9.0)
                                            .total_cmp(&b.get("modulo").and_then(Value::as_f64).unwrap_or(9.0))
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
                _ => {
                    if ui.button("Executar").clicked() {
                        estado.mensagem = match acao.as_str() {
                            "consultar" => consultar(app, &estado.db, estado.documento),
                            "topologia" => calcular(app, &estado.db, estado.documento),
                            "comprimentos" => calcular_comprimentos(app, &estado.db, estado.documento),
                            "ybus" => montar_ybus(app, &estado.db, estado.documento),
                            "resultados" => listar_resultados(app, &estado.db, estado.documento),
                            "relatorio" => gerar_relatorio(app, &estado.db, estado.documento),
                            _ => String::new(),
                        };
                    }
                }
            }
            if !estado.mensagem.is_empty() {
                ui.separator();
                ui.label(&estado.mensagem);
            }
        });
    if fechar {
        app.ui.anacad_dialogo.clear();
    }
    if let Some(msg) = notificacao {
        app.status = Some((msg, crate::now_ms()));
    }
    if !aberto {
        app.ui.anacad_dialogo.clear();
    }
    app.ui.anacad_db = estado.db.clone();
    app.ui.anacad_documento = estado.documento;
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

/// Conteúdo do painel (chamado pelo dock).
pub fn panel(app: &mut CadApp, ui: &mut Ui) {
    let mut estado: Estado = ui.ctx().data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    if !app.ui.anacad_db.is_empty() && app.ui.anacad_db != estado.db {
        estado.db = app.ui.anacad_db.clone();
    }
    estado.documento = app.ui.anacad_documento.max(1);
    egui::ScrollArea::vertical().id_salt("anacad-conteudo").auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        ui.label("Status");
        ui.label("Banco conectado:");
        if estado.db.trim().is_empty() {
            ui.monospace("(nenhum)");
        } else {
            let nome = std::path::Path::new(&estado.db).file_name().and_then(|n| n.to_str()).unwrap_or(estado.db.as_str()).to_string();
            ui.monospace(&nome).on_hover_text(&estado.db);
            ui.small("conectado");
        }
        if !estado.mensagem.is_empty() {
            ui.separator();
            ui.label(&estado.mensagem);
        }
    });
    app.ui.anacad_db = estado.db.clone();
    app.ui.anacad_documento = estado.documento;
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(ID_ESTADO), estado));
}

fn calcular(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.calcular", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "topologia: {} barra(s), {} ilha(s), {} isolada(s), grau máx {}",
            v.get("barras").and_then(Value::as_i64).unwrap_or(0),
            v.get("ilhas").and_then(Value::as_i64).unwrap_or(0),
            v.get("barras_isoladas").and_then(Value::as_i64).unwrap_or(0),
            v.get("grau_max").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn importar_lst(app: &mut CadApp, db: &str, arquivo: &str) -> String {
    if arquivo.trim().is_empty() {
        return "informe o caminho do .lst".to_string();
    }
    match app.run("anacad", json!({ "import": arquivo, "db": db })) {
        Ok(v) => format!(
            "LST: {} barras, {} linhas, {} ligações",
            v.pointer("/banco/barras").and_then(Value::as_i64).unwrap_or(0),
            v.pointer("/banco/linhas").and_then(Value::as_i64).unwrap_or(0),
            v.pointer("/banco/ligacoes").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn importar_ana(app: &mut CadApp, db: &str, documento: i64, arquivo: &str) -> String {
    if arquivo.trim().is_empty() {
        return "informe o caminho do .ANA".to_string();
    }
    match app.run("anacad.ana.importar", json!({ "arquivo": arquivo, "db": db, "documento": documento })) {
        Ok(v) => format!(
            "ANA: {} barras, {} circuitos, {} shunts, {} geradores",
            v.get("barras").and_then(Value::as_i64).unwrap_or(0),
            v.get("circuitos").and_then(Value::as_i64).unwrap_or(0),
            v.get("shunts").and_then(Value::as_i64).unwrap_or(0),
            v.get("geradores").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn atribuir_barra(app: &mut CadApp, db: &str, documento: i64, handle: &str, codigo: i64) -> String {
    match app.run("anacad.barra.atribuir", json!({ "db": db, "documento": documento, "handle": handle, "codigo": codigo })) {
        Ok(v) => format!("barra atribuída: #{} (handle {})", v.get("barra").and_then(Value::as_i64).unwrap_or(0), handle),
        Err(e) => format!("erro: {e}"),
    }
}

fn atribuir_linha(app: &mut CadApp, db: &str, documento: i64, handle: &str, de: i64, para: i64) -> String {
    match app.run("anacad.linha.atribuir", json!({ "db": db, "documento": documento, "handle": handle, "de": de, "para": para })) {
        Ok(v) => format!("linha atribuída: #{} (handle {})", v.get("linha").and_then(Value::as_i64).unwrap_or(0), handle),
        Err(e) => format!("erro: {e}"),
    }
}

fn montar_ybus(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.ybus", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "Ybus: {} barras, {} circuitos, {} pares, simétrica={}",
            v.get("barras").and_then(Value::as_i64).unwrap_or(0),
            v.get("circuitos_usados").and_then(Value::as_i64).unwrap_or(0),
            v.get("pares_nao_nulos").and_then(Value::as_i64).unwrap_or(0),
            v.get("simetrica").and_then(Value::as_bool).unwrap_or(false)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn gerar_relatorio(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.relatorio", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "relatório: {} ({} bytes)",
            v.get("arquivo").and_then(Value::as_str).unwrap_or("-"),
            v.get("bytes").and_then(Value::as_i64).unwrap_or(0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn listar_resultados(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.resultados", json!({ "db": db, "documento": documento, "limite": 5 })) {
        Ok(v) => {
            let lista = v.get("resultados").and_then(Value::as_array);
            match lista {
                Some(itens) if !itens.is_empty() => itens
                    .iter()
                    .map(|r| {
                        format!("#{} {}", r.get("id").and_then(Value::as_i64).unwrap_or(0), r.get("tipo").and_then(Value::as_str).unwrap_or("-"))
                    })
                    .collect::<Vec<_>>()
                    .join(" · "),
                _ => "nenhum resultado".to_string(),
            }
        }
        Err(e) => format!("erro: {e}"),
    }
}

fn calcular_comprimentos(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.comprimento", json!({ "db": db, "documento": documento })) {
        Ok(v) => format!(
            "comprimentos: ligações {:.1}, linhas {:.1}, total {:.1}",
            v.get("comprimento_ligacoes").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("comprimento_linhas").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("comprimento_total").and_then(Value::as_f64).unwrap_or(0.0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn criar_legenda(app: &mut CadApp, db: &str, documento: i64, numero: i64, x: f64, y: f64) -> String {
    match app.run("anacad.legenda.criar", json!({ "db": db, "documento": documento, "tipo": "X", "numero": numero, "x": x, "y": y })) {
        Ok(v) => format!(
            "legenda {} criada (handle {})",
            v.get("legenda").and_then(Value::as_i64).unwrap_or(0),
            v.get("handle").and_then(Value::as_str).unwrap_or("-")
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn mover_componente(app: &mut CadApp, db: &str, handle: &str, dx: f64, dy: f64) -> String {
    match app.run("anacad.mover", json!({ "db": db, "handle": handle, "dx": dx, "dy": dy })) {
        Ok(v) => format!(
            "componente movido para ({}, {})",
            v.get("x").and_then(Value::as_f64).unwrap_or(0.0),
            v.get("y").and_then(Value::as_f64).unwrap_or(0.0)
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn apagar_componente(app: &mut CadApp, db: &str, handle: &str) -> String {
    match app.run("anacad.apagar", json!({ "db": db, "handle": handle })) {
        Ok(v) => format!("apagado ({} linha(s))", v.get("removidos").and_then(Value::as_i64).unwrap_or(0)),
        Err(e) => format!("erro: {e}"),
    }
}

fn criar_barra(app: &mut CadApp, db: &str, documento: i64, codigo: i64, x: f64, y: f64, x2: f64, y2: f64) -> String {
    match app.run("anacad.barra.criar", json!({ "db": db, "documento": documento, "codigo": codigo, "x": x, "y": y, "x2": x2, "y2": y2 })) {
        Ok(v) => format!(
            "barra {} criada (handle {})",
            v.get("barra").and_then(Value::as_i64).unwrap_or(0),
            v.get("handle").and_then(Value::as_str).unwrap_or("-")
        ),
        Err(e) => format!("erro: {e}"),
    }
}

fn consultar(app: &mut CadApp, db: &str, documento: i64) -> String {
    match app.run("anacad.consultar", json!({ "db": db, "documento": documento })) {
        Ok(v) => resumir(&v),
        Err(e) => format!("erro: {e}"),
    }
}

fn carregar_componentes_barra(app: &mut CadApp, db: &str, documento: i64, codigo: i64) -> Vec<(i64, String, bool)> {
    match app.run("anacad.barra.componentes", json!({"db": db, "documento": documento, "codigo": codigo})) {
        Ok(v) => v
            .get("componentes")
            .and_then(Value::as_array)
            .map(|itens| itens.iter().filter_map(|c| Some((c.get("id")?.as_i64()?, c.get("nome")?.as_str()?.to_string(), false))).collect())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn carregar_terminais(app: &mut CadApp, db: &str, documento: i64) -> Vec<(i64, String, f64, f64)> {
    match app.run("anacad.terminal.listar", json!({"db": db, "documento": documento})) {
        Ok(v) => v
            .get("terminais")
            .and_then(Value::as_array)
            .map(|itens| {
                itens
                    .iter()
                    .filter_map(|t| {
                        Some((
                            t.get("id")?.as_i64()?,
                            t.get("nome")?.as_str()?.to_string(),
                            t.get("x").and_then(Value::as_f64).unwrap_or(0.0),
                            t.get("y").and_then(Value::as_f64).unwrap_or(0.0),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn tipo_da_entidade(app: &CadApp, handle_txt: &str) -> Option<&'static str> {
    let handle = cadcraft_doc::Handle::parse_hex(handle_txt)?;
    let doc = app.session.doc().ok()?;
    let entidade = doc.entity(handle)?;
    let app_name = entidade.common.xdata.iter().find(|t| t.code == 1001).map(|t| t.value.as_str())?;
    match app_name {
        "ANACAD_BARRA" => Some("editar_barra"),
        "ANACAD_LINHA" => Some("editar_linha"),
        _ => None,
    }
}

fn dados_entidade(app: &CadApp, handle_txt: &str) -> Option<serde_json::Map<String, Value>> {
    let handle = cadcraft_doc::Handle::parse_hex(handle_txt)?;
    let doc = app.session.doc().ok()?;
    let entidade = doc.entity(handle)?;
    let pares: Vec<&str> = entidade.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
    let mut mapa = serde_json::Map::new();
    let mut i = 0;
    while i + 1 < pares.len() {
        if pares[i] == "dados" {
            if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(pares[i + 1]) {
                for (k, v) in obj {
                    mapa.insert(k, v);
                }
            }
        } else {
            mapa.insert(pares[i].to_string(), Value::String(pares[i + 1].to_string()));
        }
        i += 2;
    }
    Some(mapa)
}

fn preencher_equipamento(estado: &mut Estado, dados: &serde_json::Map<String, Value>) {
    let f = |k: &str, padrao: f64| dados.get(k).and_then(|v| v.as_f64().or_else(|| v.as_str()?.trim().parse().ok())).unwrap_or(padrao);
    let i = |k: &str| dados.get(k).and_then(|v| v.as_i64().or_else(|| v.as_str()?.trim().parse().ok())).unwrap_or(0);
    let s = |k: &str| dados.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    estado.eq_tipo = s("tipo");
    estado.eq_codigo = i("codigo");
    estado.eq_nome = s("nome");
    estado.eq_de = i("de");
    estado.eq_para = i("para");
    estado.eq_barra = i("barra");
    estado.eq_pot = f("potencia", 0.0);
    estado.eq_r1 = f("R1", 0.0);
    estado.eq_x1 = f("X1", 0.0);
    estado.eq_r0 = f("R0", 0.0);
    estado.eq_x0 = f("X0", 0.0);
    estado.eq_tap = f("tap", 1.0);
    estado.eq_p = f("P", 0.0);
    estado.eq_q = f("Q", 0.0);
    estado.eq_status = dados.get("status").and_then(Value::as_bool).unwrap_or(true);
    estado.eq_grupo = s("grupo");
    estado.eq_area = s("area");
    estado.eq_circuito = i("numeroCircuito");
    estado.eq_capacidade = f("capacidade", 0.0);
    estado.eq_defasamento = f("defasamento", 0.0);
    estado.eq_conexao = if s("conexao").is_empty() { "Estrela aterrado".into() } else { s("conexao") };
    estado.eq_conexao2 = if s("conexao2").is_empty() { "Estrela aterrado".into() } else { s("conexao2") };
    estado.eq_rn = f("Rn", 0.0);
    estado.eq_xn = f("Xn", 0.0);
    estado.eq_rn2 = f("Rn2", 0.0);
    estado.eq_xn2 = f("Xn2", 0.0);
    estado.eq_unidades = i("unidades").max(1);
}

fn preencher_estado(estado: &mut Estado, dados: &serde_json::Map<String, Value>) {
    let f = |k: &str| dados.get(k).and_then(|v| v.as_f64().or_else(|| v.as_str()?.trim().parse().ok())).unwrap_or(0.0);
    let i = |k: &str| dados.get(k).and_then(|v| v.as_i64().or_else(|| v.as_str()?.trim().parse().ok())).unwrap_or(0);
    let s = |k: &str| dados.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let b = |k: &str| dados.get(k).and_then(Value::as_bool).unwrap_or(true);
    estado.barra_codigo = i("codigo");
    estado.linha_de = i("de");
    estado.linha_para = i("para");
    estado.ed_nome = s("nome");
    estado.ed_area = s("area");
    estado.ed_vbase = f("vbase");
    estado.ed_tipo = if s("tipo").is_empty() { "Normal".into() } else { s("tipo") };
    estado.ed_capacidade = f("capacidadeInterrupcao");
    let arr = |k: &str| {
        let mut v = [0.0; 3];
        if let Some(a) = dados.get(k).and_then(Value::as_array) {
            for (i, x) in a.iter().take(3).enumerate() {
                v[i] = x.as_f64().unwrap_or(0.0);
            }
        }
        v
    };
    estado.ed_cap_am = arr("capacidadesAmarre");
    estado.ed_cap_de = arr("capacidadesDe");
    estado.ed_cap_para = arr("capacidadesPara");
    estado.ed_pre_mod = if dados.contains_key("preFaltaModulo") { f("preFaltaModulo") } else { 1.0 };
    estado.ed_pre_ang = f("preFaltaAngulo");
    estado.ed_disjuntor = dados.get("disjuntorSE").and_then(Value::as_bool).unwrap_or(false);
    estado.ed_status = b("status");
    estado.ed_circuito = i("numeroCircuito");
    estado.ed_potencia = f("potenciaNominal");
    estado.ed_comprimento = f("comprimento");
    estado.ed_r1 = f("R1");
    estado.ed_x1 = f("X1");
    estado.ed_s1 = f("S1");
    estado.ed_r0 = f("R0");
    estado.ed_x0 = f("X0");
    estado.ed_s0 = f("S0");
}

fn carregar_documentos(app: &mut CadApp, db: &str) -> Vec<(i64, String, String)> {
    match app.run("anacad.documento.listar", json!({ "db": db })) {
        Ok(v) => v
            .get("documentos")
            .and_then(Value::as_array)
            .map(|itens| {
                itens
                    .iter()
                    .filter_map(|d| {
                        Some((
                            d.get("id")?.as_i64()?,
                            d.get("nome")?.as_str()?.to_string(),
                            d.get("pagina").and_then(Value::as_str).unwrap_or("").to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn resumir(v: &Value) -> String {
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
