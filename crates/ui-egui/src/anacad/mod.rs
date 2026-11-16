//! Painel AnaCAD: consulta e edição rápida do banco de projeto (`.db`).
//!
//! É um painel **dockado** do cadcraft (não uma janela flutuante): entra na
//! mesma prancheta de Layers/Properties, abre pelo menu `AnaCAD ▸ Painel`, pelo
//! botão da barra de ferramentas ou por `ui.anacad`.
//!
//! A UI é fina: os botões chamam os comandos do motor (`app.run`) e o estado da
//! tela (caminho do banco, documento, último resultado) vive na memória do egui.

pub mod contrib;

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::CadApp;

const ID_ESTADO: &str = "anacad.estado";
/// Marca, na memória do egui, qual desenho já teve a configuração carregada.
const ID_CFG: &str = "anacad.cfg";

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
    imp_unidade: String,
    imp_um_gerador: bool,
    sup_limite: f64,
    sup_flags: [bool; 6],
    sup_lista: Vec<String>,
    sup_resumo: String,
    sup_barras: Vec<i64>,
    sup_nova: i64,
    sup_sel: Option<usize>,
    sup_painel: u8,
    sup_arquivo: String,
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
    ed_autotrafo: bool,
    ed_circ_eq: bool,
    ed_existe: bool,
    ed_listas_em: String,
    barras_opcoes: Vec<(i64, String)>,
    vizinhos: Vec<(i64, String)>,
    viz_codigo: i64,
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
    eq_circ_eq: bool,
    eq_unid_oper: i64,
    eq_cap_de: [f64; 3],
    eq_cap_para: [f64; 3],
    eq_def_expl: bool,
    eq_existe: bool,
    eq_sincrono: bool,
    eq_assoc_trafo: bool,
    eq_barra_alta: i64,
    eq_unid_por_trafo: i64,
    eq_agreg_comp: String,
    eq_agreg_trafo3: String,
    eq_tipo_gerador: String,
    eq_estado_oper: String,
    eq_prot_ligado: bool,
    eq_var_corrente: f64,
    eq_gap_corrente: f64,
    eq_gap_tipo: String,
    eq_gap_energia: f64,
    eq_gap_potencia: f64,
    eq_xm: f64,
    eq_base_hp: f64,
    eq_mut_de1: i64,
    eq_mut_para1: i64,
    eq_mut_nc1: i64,
    eq_mut_ini1: f64,
    eq_mut_fim1: f64,
    eq_mut_de2: i64,
    eq_mut_para2: i64,
    eq_mut_nc2: i64,
    eq_mut_ini2: f64,
    eq_mut_fim2: f64,
    eq_mut_rm: f64,
    eq_mut_xm: f64,
    eq_lado: String,
    eq_bypass: bool,
    eq_imax: f64,
    eq_fp_pre: f64,
    eq_vmin: f64,
    eq_pinic: f64,
    eq_vmax: f64,
    eq_fp_cc: f64,
    eq_curva: bool,
    eq_vp1: f64,
    eq_vp2: f64,
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
    mensagem: String,
    gd_tabela: String,
    gd_colunas: Vec<String>,
    gd_linhas: Vec<Vec<String>>,
    gd_em: String,
    gd_erro: String,
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
            imp_unidade: "Polar".into(),
            imp_um_gerador: false,
            sup_limite: 100.0,
            sup_flags: [true, true, false, false, false, false],
            sup_lista: Vec::new(),
            sup_resumo: String::new(),
            sup_barras: Vec::new(),
            sup_nova: 0,
            sup_sel: None,
            sup_painel: 0,
            sup_arquivo: "barras-superacao.txt".into(),
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
            ed_autotrafo: false,
            ed_circ_eq: false,
            ed_existe: false,
            ed_listas_em: String::new(),
            barras_opcoes: Vec::new(),
            vizinhos: Vec::new(),
            viz_codigo: 0,
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
            eq_circ_eq: false,
            eq_unid_oper: 0,
            eq_cap_de: [0.0; 3],
            eq_cap_para: [0.0; 3],
            eq_def_expl: false,
            eq_existe: false,
            eq_sincrono: false,
            eq_assoc_trafo: false,
            eq_barra_alta: 0,
            eq_unid_por_trafo: 0,
            eq_agreg_comp: String::new(),
            eq_agreg_trafo3: String::new(),
            eq_tipo_gerador: String::new(),
            eq_estado_oper: "Ligado".into(),
            eq_prot_ligado: false,
            eq_var_corrente: 0.0,
            eq_gap_corrente: 0.0,
            eq_gap_tipo: String::new(),
            eq_gap_energia: 0.0,
            eq_gap_potencia: 0.0,
            eq_xm: 0.0,
            eq_base_hp: 0.0,
            eq_mut_de1: 0,
            eq_mut_para1: 0,
            eq_mut_nc1: 1,
            eq_mut_ini1: 0.0,
            eq_mut_fim1: 100.0,
            eq_mut_de2: 0,
            eq_mut_para2: 0,
            eq_mut_nc2: 1,
            eq_mut_ini2: 0.0,
            eq_mut_fim2: 100.0,
            eq_mut_rm: 0.0,
            eq_mut_xm: 0.0,
            eq_lado: "De".into(),
            eq_bypass: false,
            eq_imax: 0.0,
            eq_fp_pre: 0.0,
            eq_vmin: 0.0,
            eq_pinic: 0.0,
            eq_vmax: 0.0,
            eq_fp_cc: 0.0,
            eq_curva: false,
            eq_vp1: 0.0,
            eq_vp2: 0.0,
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
            mensagem: String::new(),
            gd_tabela: "Rede".into(),
            gd_colunas: Vec::new(),
            gd_linhas: Vec::new(),
            gd_em: String::new(),
            gd_erro: String::new(),
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

/// Conecta um banco (cria ou abre), guarda-o na configuração do projeto do
/// desenho e atualiza o status. É essa gravação que faz o banco voltar sozinho
/// na próxima vez que o desenho for aberto.
pub fn conectar_banco(app: &mut CadApp, cmd: &str, arquivo: &str) -> Result<Value, String> {
    let v = app.run(cmd, json!({ "arquivo": arquivo }))?;
    app.ui.anacad_db = arquivo.to_string();
    if let Some(doc) = v.get("primeiroDocumento").and_then(Value::as_i64) {
        app.ui.anacad_documento = doc;
    }
    let nome = v.get("nome").and_then(Value::as_str).unwrap_or(arquivo).to_string();
    let banco = app.ui.anacad_db.clone();
    let documento = app.ui.anacad_documento;
    // Só o banco gravado no desenho (`ANACAD_CFG`) vale como o projeto dele: é
    // o XData que habilita as ferramentas depois de trocar de desenho.
    if app.run("anacad.cfg.gravar", json!({ "banco": banco.clone(), "documento": documento })).is_ok() {
        app.ui.anacad_db_desenho = banco.clone();
    }
    // O banco conectado vale já para o próximo comando: os `anacad.*` leem
    // banco/documento do estado da sessão, não dos parâmetros.
    contexto(app);
    app.status = Some((format!("Banco: {nome}"), crate::now_ms()));
    Ok(v)
}

/// Abre a janela **Configuração do Projeto** com o que está gravado no desenho
/// (XData `ANACAD_CFG`): banco, caderno, texto/traço de barra e linha e os
/// grupos base de tensão.
pub fn abrir_cfg(app: &mut CadApp) -> Result<Value, String> {
    if let Ok(v) = app.run("anacad.cfg.ler", Value::Null) {
        let banco = v.get("banco").and_then(Value::as_str).unwrap_or_default().to_string();
        if !banco.is_empty() {
            app.ui.anacad_db = banco;
        }
        app.ui.anacad_caderno = v.get("caderno").and_then(Value::as_str).unwrap_or_default().to_string();
        app.ui.anacad_barra_texto = f64_do_json(&v, "barra_texto", 2.5);
        app.ui.anacad_barra_cor = cor_do_json(&v, "barra_cor");
        app.ui.anacad_barra_lar = f64_do_json(&v, "barra_lar", 0.3);
        app.ui.anacad_linha_texto = f64_do_json(&v, "linha_texto", 2.5);
        app.ui.anacad_linha_cor = cor_do_json(&v, "linha_cor");
        app.ui.anacad_linha_lar = f64_do_json(&v, "linha_lar", 0.0);
        app.ui.anacad_grupos = grupos_do_json(&v);
        app.ui.anacad_textos_barra = textos_no_desenho(app, "barra");
        app.ui.anacad_textos_linha = textos_no_desenho(app, "linha");
        if let Some(doc) = v.get("documento").and_then(Value::as_i64) {
            app.ui.anacad_documento = doc.max(1);
        }
    }
    app.ui.anacad_dialogo = "cfg_projeto".into();
    Ok(Value::Null)
}

/// Cor vinda do `anacad.cfg.ler`: `"3"` (ACI) ou `"255,128,0"` (true color).
fn cor_do_json(v: &Value, campo: &str) -> cadcraft_color::Color {
    let Some(c) = v.get(campo) else { return cadcraft_color::Color::Index(7) };
    let texto = c.as_str().map(str::to_string).unwrap_or_else(|| c.to_string());
    cor_do_texto(&texto)
}

/// Número vindo do `anacad.cfg.ler` (altura do texto ou espessura do traço).
fn f64_do_json(v: &Value, campo: &str, padrao: f64) -> f64 {
    v.get(campo).and_then(Value::as_f64).unwrap_or(padrao)
}

/// Cor a partir do texto guardado no desenho, no formato do `ANACAD_CFG`.
fn cor_do_texto(texto: &str) -> cadcraft_color::Color {
    use cadcraft_color::Color;
    match Color::parse(texto) {
        Some(cor @ (Color::Index(_) | Color::True(_))) => cor,
        _ => Color::Index(7),
    }
}

/// Texto da cor para o `anacad.cfg.gravar` (`"3"` ou `"255,128,0"`).
fn texto_cor(cor: cadcraft_color::Color) -> String {
    use cadcraft_color::Color;
    match cor {
        Color::Index(i) => i.to_string(),
        Color::True(rgb) => format!("{},{},{}", rgb.0, rgb.1, rgb.2),
        _ => "7".to_string(),
    }
}

/// Botão de cor com o seletor do CAD (paleta ACI e true color), o equivalente
/// do `CColorDialog` do ANAFAS em &Ferramentas > Definir &Cores.
fn botao_cor(ui: &mut Ui, id: &str, cor: &mut cadcraft_color::Color) {
    if let Some(nova) = crate::layers::color_button(ui, ui.id().with(id), *cor, false, 170.0) {
        *cor = nova;
    }
}

/// Bloco de um elemento (barra ou linha) do grupo **Texto e Traço de Barras e
/// Linhas**: tamanho e cor do texto de identificação e espessura do traço.
fn estilo_elemento(
    ui: &mut Ui,
    rotulo: &str,
    id: &str,
    texto: &mut f64,
    cor: &mut cadcraft_color::Color,
    lar: &mut f64,
) {
    ui.label(egui::RichText::new(rotulo).strong());
    ui.horizontal(|ui| {
        ui.label("Tamanho do texto:");
        ui.add(egui::DragValue::new(texto).range(0.5..=50.0).speed(0.1));
        ui.label("Cor do texto:");
        botao_cor(ui, id, cor);
    });
    ui.horizontal(|ui| {
        ui.label("Espessura da linha:");
        ui.add(egui::DragValue::new(lar).range(0.0..=5.0).speed(0.05));
    });
}

/// Tensão como o ANAFAS escreve na tela (vírgula decimal).
fn texto_tensao(tensao: f64) -> String {
    format!("{tensao}").replace('.', ",")
}

/// Grupos base de tensão do `anacad.cfg.ler`:
/// `[{ "tensao": 500.0, "cor": "255,0,0" }, …]` (tensão e cor, como o `G` do
/// `.lst`; a espessura do traço é a do estilo, uma só).
fn grupos_do_json(v: &Value) -> Vec<(f64, cadcraft_color::Color)> {
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

/// Grupos base de tensão padrão, os seis do desenho de exemplo do ANAFAS (o
/// cabeçalho do `.lst`): 500 kV vermelho, 345 kV azul, 138 kV, 13,8 kV e
/// 4,16 kV laranja e 0 kV preto. A lista não é fixa: cada projeto tem os seus
/// grupos, com tensão e cor da janela 210.
fn grupos_padrao_ui() -> Vec<(f64, cadcraft_color::Color)> {
    use cadcraft_color::{Color, Rgb};
    vec![
        (500.0, Color::True(Rgb(255, 0, 0))),
        (345.0, Color::True(Rgb(0, 128, 255))),
        (138.0, Color::True(Rgb(255, 165, 0))),
        (13.8, Color::True(Rgb(255, 165, 0))),
        (4.16, Color::True(Rgb(255, 165, 0))),
        (0.0, Color::True(Rgb(0, 0, 0))),
    ]
}

/// Chave do desenho ativo: o arquivo, ou a aba e o título de um desenho novo.
fn chave_do_desenho(app: &CadApp) -> String {
    match app.session.docs.get(app.session.active) {
        Some(d) => d.path.clone().filter(|p| !p.is_empty()).unwrap_or_else(|| format!("#{} {}", app.session.active, d.title)),
        None => String::new(),
    }
}

/// Relê o projeto do desenho ativo, uma vez por troca de aba: o banco vem do
/// `ANACAD_CFG` do próprio desenho — o `DirProjeto`/`LerXDataConfiguracao2` do
/// Eletron —, junto do caderno e do estilo, e é ele, não o da sessão anterior,
/// que decide se as ferramentas estão habilitadas. Desenho sem banco no XData
/// (ou com um `.db` que não existe mais) não libera ferramenta nenhuma.
pub(crate) fn sincronizar_desenho(app: &mut CadApp, ctx: &egui::Context) {
    let desenho = chave_do_desenho(app);
    if desenho.is_empty() {
        return;
    }
    let carregado: String = ctx.data_mut(|d| d.get_temp_mut_or_default::<String>(egui::Id::new(ID_CFG)).clone());
    if carregado == desenho {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID_CFG), desenho));
    // O banco do desenho substitui o da sessão anterior: cada desenho tem o seu.
    let no_desenho = cadcraft_engine::cmd::anacad::banco_do_desenho(&app.session);
    app.ui.anacad_db_desenho = no_desenho.clone();
    app.ui.anacad_db.clear();
    app.ui.anacad_documento = 1;
    let Ok(v) = app.run("anacad.cfg.ler", Value::Null) else { return };
    app.ui.anacad_caderno = v.get("caderno").and_then(Value::as_str).unwrap_or_default().to_string();
    app.ui.anacad_barra_texto = f64_do_json(&v, "barra_texto", 2.5);
    app.ui.anacad_barra_cor = cor_do_json(&v, "barra_cor");
    app.ui.anacad_barra_lar = f64_do_json(&v, "barra_lar", 0.3);
    app.ui.anacad_linha_texto = f64_do_json(&v, "linha_texto", 2.5);
    app.ui.anacad_linha_cor = cor_do_json(&v, "linha_cor");
    app.ui.anacad_linha_lar = f64_do_json(&v, "linha_lar", 0.0);
    app.ui.anacad_grupos = grupos_do_json(&v);
    if let Some(doc) = v.get("documento").and_then(Value::as_i64) {
        app.ui.anacad_documento = doc.max(1);
    }
    // O banco do desenho substitui o da sessão anterior: cada desenho tem o seu.
    if no_desenho.trim().is_empty() {
        app.set_status("Este desenho não tem banco: AnaCAD ▸ Configurações ▸ Configuração do Projeto");
        return;
    }
    if !std::path::Path::new(&no_desenho).exists() {
        app.set_status(format!("Banco {no_desenho} não existe; aponte o .db do projeto"));
        return;
    }
    match conectar_banco(app, "anacad.banco.abrir", &no_desenho) {
        Ok(_) => app.set_status(format!("Banco: {no_desenho}")),
        Err(e) => app.set_status(format!("Banco: {e}")),
    }
}

/// Janela dos diálogos do domínio, com tamanho inicial proporcional ao
/// diálogo do ANAFAS (1 DLU ≈ 1,5 px na horizontal e ≈ 1,6 px na vertical).
fn com_altura<'a>(janela: egui::Window<'a>, altura: f32) -> egui::Window<'a> {
    if altura > 0.0 { janela.default_height(altura) } else { janela }
}

/// Tipo (`Transformador`, `Gerador`…) gravado no XData `ANACAD_EQ`.
fn tipo_do_equipamento(app: &CadApp, handle: &str) -> String {
    dados_entidade(app, handle).and_then(|d| d.get("tipo").and_then(Value::as_str).map(str::to_string)).unwrap_or_default()
}

/// Árvore do **Gerenciador de Dados** no ANAFAS (menu Dados): `Rede CA` com os
/// elementos da rede. A chave é o item lido do banco (a lista
/// `anacad_data::ITENS_GERENCIADOR`) e o rótulo é o do ANAFAS — os dados do
/// caso, não as tabelas do `.db`.
const GERENCIADOR_REDE: &[(&str, &str, crate::icons::Icon)] = &[
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
const GERENCIADOR_RAIZ: (&str, &str, crate::icons::Icon) =
    ("Rede", "Quantidade", crate::icons::Icon::AnacadRede);
const GERENCIADOR_FORA: &[(&str, &str, crate::icons::Icon)] =
    &[("Area", "Área", crate::icons::Icon::Layers)];

/// Carrega o item escolhido no Gerenciador de Dados: os campos da janela
/// `Dados de …` do elemento e até 500 registros do documento ativo. Quem lê o
/// banco é o motor (`anacad.dados.tabela`).
fn carregar_tabela(app: &mut CadApp, estado: &mut Estado) {
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

pub use contrib::{ANACAD_UI, AnacadUi};

/// Hand the UI's connected database and active document to the engine before starting a command
/// that reads them (the interactive `anacad.*` commands take them from the session).
pub(crate) fn contexto(app: &mut CadApp) {
    cadcraft_engine::cmd::anacad::set_banco(&mut app.session, &app.ui.anacad_db, app.ui.anacad_documento);
}

/// Diálogos das ferramentas abertos pelo tool set.
pub fn dialogo(app: &mut CadApp, ctx: &egui::Context) {
    if let Some(handle) = cadcraft_engine::cmd::anacad::take_equip_handle(&mut app.session) {
        app.ui.anacad_handle = handle;
        app.ui.anacad_dialogo = "editar_equipamento".into();
    }
    if let Some(handle) = cadcraft_engine::cmd::anacad::take_edit_handle(&mut app.session) {
        let tipo = tipo_da_entidade(app, &handle).unwrap_or("editar_barra");
        app.ui.anacad_handle = handle;
        app.ui.anacad_dialogo = tipo.into();
    }
    // Seletor **Configurações**: o handle pode ser barra, linha ou equipamento.
    if let Some(handle) = cadcraft_engine::cmd::anacad::take_artefato_handle(&mut app.session)
        && !abrir_artefato(app, &handle)
    {
        app.set_status("Selecione uma barra, uma linha ou um equipamento do AnaCAD");
    }
    // Cada equipamento abre a sua janela do ANAFAS (`Dados de …`), não uma
    // janela genérica por tipo (janelas-ui §2.1).
    if app.ui.anacad_dialogo == "editar_equipamento" {
        let handle = if !app.ui.anacad_handle.is_empty() {
            app.ui.anacad_handle.clone()
        } else {
            app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
        };
        match tipo_do_equipamento(app, &handle).as_str() {
            "Transformador" => app.ui.anacad_dialogo = "editar_transformador".into(),
            "Gerador" => app.ui.anacad_dialogo = "editar_gerador".into(),
            "Carga"
            | "Shunt de Barra"
            | "Shunt de Linha"
            | "Motor de Indução"
            | "Impedância Mútua"
            | "Compensador Série"
            | "Trafo de Aterramento"
            | "Gerador Fonte de Corrente"
            | "Compensador Estático" => app.ui.anacad_dialogo = "editar_dados_equipamento".into(),
            _ => {}
        }
    }
    let tipo_dados = if app.ui.anacad_dialogo == "editar_dados_equipamento" {
        let handle = if !app.ui.anacad_handle.is_empty() {
            app.ui.anacad_handle.clone()
        } else {
            app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
        };
        tipo_do_equipamento(app, &handle)
    } else {
        String::new()
    };
    if app.ui.anacad_dialogo.is_empty() {
        return;
    }
    let acao = app.ui.anacad_dialogo.clone();
    let titulo = match acao.as_str() {
        "cfg_projeto" => "Configuração do Projeto",
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
        "gerenciador" => "Gerenciador de Dados",
        "criar_barra" => "Criar barra (LWPOLYLINE 0,3)",
        "criar_linha" => "Criar linha (LWPOLYLINE 0,0)",
        "criar_legenda" => "Criar legenda (X)",
        "editar_barra" => "Dados de Barra",
        "editar_linha" => "Dados de Linha de Transmissão",
        "editar_equipamento" => "Dados do Equipamento",
        "editar_transformador" => "Dados de Transformador",
        "editar_gerador" => "Dados de Gerador",
        "editar_dados_equipamento" => caption_equipamento(&tipo_dados),
        "impedancia" => "Impedância Equivalente entre 2 Barras",
        "superacao" => "Superação de Disjuntores",
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
    let largura = match acao.as_str() {
        "editar_barra" => 470.0,
        "editar_linha" => 620.0,
        "editar_transformador" => 520.0,
        "editar_gerador" => 640.0,
        "editar_dados_equipamento" => 600.0,
        "superacao" => 560.0,
        "gerenciador" => 780.0,
        "cfg_projeto" => 520.0,
        _ if acao.starts_with("editar_") => 660.0,
        _ => 440.0,
    };
    let altura = match acao.as_str() {
        "editar_barra" => 540.0,
        "editar_linha" => 680.0,
        "editar_transformador" => 720.0,
        "editar_gerador" => 760.0,
        "editar_dados_equipamento" => 660.0,
        "superacao" => 560.0,
        "gerenciador" => 480.0,
        _ => 0.0,
    };
    let centro = if altura > 0.0 {
        let y = ((ctx.content_rect().height() - altura) / 2.0).max(60.0);
        egui::pos2(ctx.content_rect().center().x - largura / 2.0, y)
    } else {
        ctx.content_rect().center() - egui::vec2(largura / 2.0, 220.0)
    };
    let mut janela = com_altura(egui::Window::new(titulo), altura)
        .id(egui::Id::new("anacad.dialogo"))
        .default_width(largura)
        .default_pos(centro)
        .collapsible(false)
        // Dados de Barra tem altura fixa, como o DIALOGEX do ANAFAS; redimensionar
        // a janela deixaria o rodapé solto no meio do vazio.
        .resizable(acao != "editar_barra")
        .open(&mut aberto);
    // O Gerenciador de Dados mede a grade e a coluna de ícones: fica com
    // tamanho fixo, como o DIALOGEX do ANAFAS, senão o conteúdo (que ocupa a
    // altura disponível) estica a janela pela tela inteira.
    if acao == "gerenciador" {
        janela = janela.fixed_size(egui::vec2(largura, altura));
    }
    janela
        .show(ctx, |ui| {
            match acao.as_str() {
                "cfg_projeto" => {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Banco de Dados").strong());
                        ui.horizontal(|ui| {
                            ui.label("Banco (.db):");
                            ui.add(egui::TextEdit::singleline(&mut app.ui.anacad_db).desired_width(300.0));
                            // Padrão de símbolos do Eletron: a pasta abre e já conecta o `.db`
                            // escolhido; o "+" cria um banco novo.
                            if crate::icons::button(ui, crate::icons::Icon::Open, 22.0, "Abrir banco", false).clicked() {
                                let escolhido = if app.services.pick_db_open.is_some() {
                                    app.services.pick_db_open.as_ref().and_then(|f| f())
                                } else if app.ui.anacad_db.trim().is_empty() {
                                    None
                                } else {
                                    Some(app.ui.anacad_db.clone())
                                };
                                if let Some(arquivo) = escolhido {
                                    app.ui.anacad_db = arquivo.clone();
                                    estado.mensagem = match conectar_banco(app, "anacad.banco.abrir", &arquivo) {
                                        Ok(_) => "banco conectado".to_string(),
                                        Err(e) => format!("erro: {e}"),
                                    };
                                    // A janela grava `estado` de volta em `app.ui` no fim do
                                    // quadro; sem esta cópia o caminho recém-aberto se perdia.
                                    estado.db = app.ui.anacad_db.clone();
                                    estado.documento = app.ui.anacad_documento;
                                }
                            }
                            if crate::icons::button(ui, crate::icons::Icon::Plus, 22.0, "Novo banco", false).clicked() {
                                let escolhido = if app.services.pick_db_save.is_some() {
                                    app.services.pick_db_save.as_ref().and_then(|f| f("anacad.db"))
                                } else if app.ui.anacad_db.trim().is_empty() {
                                    None
                                } else {
                                    Some(app.ui.anacad_db.clone())
                                };
                                if let Some(arquivo) = escolhido {
                                    app.ui.anacad_db = arquivo.clone();
                                    estado.mensagem = match conectar_banco(app, "anacad.banco.criar", &arquivo) {
                                        Ok(_) => "banco criado e conectado".to_string(),
                                        Err(e) => format!("erro: {e}"),
                                    };
                                    estado.db = app.ui.anacad_db.clone();
                                    estado.documento = app.ui.anacad_documento;
                                }
                            }
                        });
                    });
                    // O que o desenho aponta no XData: sem essa gravação as
                    // ferramentas ficam desabilitadas, mesmo com texto na caixa.
                    let no_desenho = app.ui.anacad_db_desenho.clone();
                    ui.small(if no_desenho.trim().is_empty() {
                        "No desenho (ANACAD_CFG): nenhum banco".to_string()
                    } else {
                        format!("No desenho (ANACAD_CFG): {no_desenho}")
                    });
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Projeto").strong());
                        ui.horizontal(|ui| {
                            ui.label("Caderno:");
                            ui.add(egui::TextEdit::singleline(&mut app.ui.anacad_caderno).desired_width(300.0));
                        });
                    });
                    // Texto e traço dos elementos: o ANAFAS pede as três cores
                    // da identificação em &Ferramentas > Definir &Cores; aqui o CFG
                    // guarda um estilo por elemento (acréscimo do CAD, §4).
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Texto e Traço de Barras e Linhas").strong());
                        estilo_elemento(
                            ui,
                            "Barra",
                            "cfg.estilo.barra",
                            &mut app.ui.anacad_barra_texto,
                            &mut app.ui.anacad_barra_cor,
                            &mut app.ui.anacad_barra_lar,
                        );
                        estilo_elemento(
                            ui,
                            "Linha",
                            "cfg.estilo.linha",
                            &mut app.ui.anacad_linha_texto,
                            &mut app.ui.anacad_linha_cor,
                            &mut app.ui.anacad_linha_lar,
                        );
                        // Liga/desliga da identificação, um switch por elemento
                        // (era na janela Barras): grava `Visible` nos TEXT de
                        // todas as barras / de todas as linhas do desenho.
                        ui.horizontal(|ui| {
                            ui.label("Mostrar textos:");
                            if ui
                                .checkbox(&mut app.ui.anacad_textos_barra, "Barra")
                                .on_hover_text("Mostra ou oculta a identificação de todas as barras")
                                .changed()
                            {
                                let quer = app.ui.anacad_textos_barra;
                                estado.mensagem = alternar_textos(app, "barra", Some(quer));
                            }
                            if ui
                                .checkbox(&mut app.ui.anacad_textos_linha, "Linha")
                                .on_hover_text("Mostra ou oculta o número do circuito de todas as linhas")
                                .changed()
                            {
                                let quer = app.ui.anacad_textos_linha;
                                estado.mensagem = alternar_textos(app, "linha", Some(quer));
                            }
                        });
                    });
                    // Grupos base de tensão (o `G` do `.lst`): o ANAFAS desenha
                    // cada elemento na cor do nível de tensão dele — 500 kV vermelho,
                    // 345 kV azul, 138/13,8/4,16 kV laranja no exemplo. A espessura
                    // não é do grupo: é a do estilo, uma só para barras e linhas.
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Grupos Base de Tensão").strong());
                        for (i, (tensao, cor)) in app.ui.anacad_grupos.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{} kV:", texto_tensao(*tensao)));
                                botao_cor(ui, &format!("cfg.cor.grupo{i}"), cor);
                            });
                        }
                    });
                    ui.separator();
                    // Rodapé de símbolos, como as janelas do Eletron: o check grava e o X da
                    // barra de título fecha.
                    ui.vertical_centered(|ui| {
                        if crate::icons::button(ui, crate::icons::Icon::Check, 24.0, "Gravar configuração", false).clicked() {
                            let banco = app.ui.anacad_db.clone();
                            let params = json!({
                                "banco": banco,
                                "caderno": app.ui.anacad_caderno,
                                "documento": app.ui.anacad_documento,
                                "barra_texto": app.ui.anacad_barra_texto,
                                "barra_cor": texto_cor(app.ui.anacad_barra_cor),
                                "barra_lar": app.ui.anacad_barra_lar,
                                "linha_texto": app.ui.anacad_linha_texto,
                                "linha_cor": texto_cor(app.ui.anacad_linha_cor),
                                "linha_lar": app.ui.anacad_linha_lar,
                                "grupos": app.ui.anacad_grupos.iter().map(|(tensao, cor)| json!({"tensao": tensao, "cor": texto_cor(*cor)})).collect::<Vec<_>>(),
                            });
                            match app.run("anacad.cfg.gravar", params) {
                                Ok(_) => {
                                    // O banco que vale é o gravado no desenho.
                                    app.ui.anacad_db_desenho = banco.trim().to_string();
                                    if !banco.trim().is_empty() && std::path::Path::new(&banco).exists() {
                                        match conectar_banco(app, "anacad.banco.abrir", &banco) {
                                            Ok(_) => estado.mensagem = "configuração gravada; banco conectado".to_string(),
                                            Err(e) => estado.mensagem = format!("configuração gravada; banco: {e}"),
                                        }
                                    } else {
                                        estado.mensagem = "configuração gravada no desenho".to_string();
                                    }
                                }
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                    });
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
                        cadcraft_engine::cmd::anacad::set_texto(
                            &mut app.session,
                            estado.texto_conteudo.clone(),
                            estado.texto_altura,
                            estado.texto_cor,
                        );
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
                    let handle = if !estado.comp_handle.trim().is_empty() {
                        estado.comp_handle.clone()
                    } else {
                        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                    };
                    estado.comp_handle = handle.clone();
                    let chave = format!("b|{}|{}|{}", estado.db, estado.documento, handle);
                    if estado.ed_listas_em != chave {
                        if estado.ed_handle != handle {
                            if let Some(dados) = dados_entidade(app, &handle) {
                                preencher_estado(&mut estado, &dados);
                            }
                            estado.ed_handle = handle.clone();
                        }
                        recarregar_listas_dados(app, &mut estado, &handle, "barra");
                        estado.ed_listas_em = chave;
                    }
                    let mut selecionar = false;
                    let largura = ui.available_width().max(440.0);
                    // Cabeçalho: handle à esquerda e a "mão" (selecionar no desenho)
                    // no fim da linha, como nas ferramentas do Eletron (`Select_32`).
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if crate::icons::button(ui, crate::icons::Icon::SelectObjects, 20.0, "Selecionar no desenho", false)
                                .on_hover_text("Selecionar a barra no desenho")
                                .clicked()
                            {
                                selecionar = true;
                            }
                        });
                    });
                    // Conteúdo rola dentro de uma altura fixa; o rodapé de ações fica
                    // fixo no fim da janela. Não usar `ui.available_height()` aqui: numa
                    // janela de tamanho automático ele vale a altura da tela, e a janela
                    // crescia até o limite do app.
                    egui::ScrollArea::vertical().max_height(470.0).auto_shrink([false, false]).show(ui, |ui| {
                        let largura_grupo = (largura - 18.0).max(380.0);
                        ui.group(|ui| {
                            ui.set_min_width(largura_grupo);
                            ui.label(egui::RichText::new("Identificação").strong());
                            egui::Grid::new("anacad.barra.identificacao").num_columns(4).spacing([8.0, 6.0]).show(ui, |ui| {
                                ui.label("Número:");
                                campo_numero(ui, "anacad.barra.numero", &mut estado.barra_codigo, &estado.barras_opcoes);
                                ui.label("Nome:");
                                campo_nome_editavel(ui, "anacad.barra.nome", &mut estado.ed_nome, &mut estado.barra_codigo, &estado.barras_opcoes);
                                ui.end_row();
                                ui.label("Área:");
                                ui.add(egui::TextEdit::singleline(&mut estado.ed_area).desired_width(80.0));
                                ui.checkbox(&mut estado.ed_status, "Ligado");
                                ui.end_row();
                            });
                        });
                        ui.horizontal_top(|ui| {
                            // `ui.group` herda o layout do pai (aqui horizontal), então o
                            // empilhamento dos rádios precisa de um `vertical` explícito;
                            // sem ele os itens saem lado a lado e as legendas se sobrepõem.
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new("Tipo").strong());
                                    ui.radio_value(&mut estado.ed_tipo, "Normal".to_string(), "Normal");
                                    ui.radio_value(&mut estado.ed_tipo, "Fictícia de transformador".to_string(), "Fictícia de transformador");
                                    ui.radio_value(&mut estado.ed_tipo, "Auxiliar".to_string(), "Auxiliar");
                                    ui.checkbox(&mut estado.ed_autotrafo, "Autotrafo");
                                });
                            });
                            // "Base de Tensão (kV):" fica na coluna da direita, logo
                            // acima e com a mesma largura do grupo "Tensão Pré-Falta"
                            // (ANAFAS 149: coluna x=139, 140 DLU de largura).
                            ui.vertical(|ui| {
                                ui.set_width(240.0);
                                ui.horizontal(|ui| {
                                    ui.label("Base de Tensão (kV):");
                                    ui.add_sized([90.0, 20.0], egui::DragValue::new(&mut estado.ed_vbase).speed(0.1));
                                });
                                ui.group(|ui| {
                                    ui.set_min_width(220.0);
                                    ui.label(egui::RichText::new("Tensão Pré-Falta").strong());
                                    egui::Grid::new("anacad.barra.prefalta").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                                        ui.label("Módulo (pu):");
                                        ui.add_sized([90.0, 20.0], egui::DragValue::new(&mut estado.ed_pre_mod).speed(0.01));
                                        ui.end_row();
                                        ui.label("Ângulo (°):");
                                        ui.add_sized([90.0, 20.0], egui::DragValue::new(&mut estado.ed_pre_ang).speed(0.1));
                                        ui.end_row();
                                    });
                                });
                            });
                        });
                        ui.group(|ui| {
                            ui.set_min_width(largura_grupo);
                            ui.checkbox(&mut estado.ed_disjuntor, "Existe disjuntor na SE");
                            ui.horizontal(|ui| {
                                ui.label("Menor Capacidade de Interrupção (kA):");
                                ui.add_sized([80.0, 20.0], egui::DragValue::new(&mut estado.ed_capacidade).speed(0.1));
                            });
                            ui.label("Capacidades de Interrupção Simétrica dos disjuntores de amarre (kA):");
                            ui.horizontal(|ui| {
                                ui.add_sized([80.0, 20.0], egui::DragValue::new(&mut estado.ed_cap_am[0]).speed(0.1));
                                ui.add_sized([80.0, 20.0], egui::DragValue::new(&mut estado.ed_cap_am[1]).speed(0.1));
                            });
                        });
                        ui.group(|ui| {
                            ui.set_min_width(largura_grupo);
                            ui.label(egui::RichText::new("Barras Vizinhas").strong());
                            egui::Grid::new("anacad.barra.vizinhas").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                                ui.label("Número:");
                                combo_codigos(ui, "anacad.barra.vizinho.numero", &mut estado.viz_codigo, &estado.vizinhos);
                                ui.end_row();
                                ui.label("Nome:");
                                ui.horizontal(|ui| {
                                    combo_codigos(ui, "anacad.barra.vizinho.nome", &mut estado.viz_codigo, &estado.vizinhos);
                                    if ui.add_enabled(estado.viz_codigo != 0, egui::Button::new("Mostrar")).clicked() {
                                        let cmd = json!({
                                            "db": estado.db,
                                            "documento": estado.documento,
                                            "codigo": estado.viz_codigo,
                                        });
                                        estado.mensagem = match app.run("anacad.barra.localizar", cmd) {
                                            Ok(v) => format!(
                                                "barra {} em x={:.3} y={:.3}",
                                                estado.viz_codigo,
                                                v.get("x").and_then(Value::as_f64).unwrap_or(0.0),
                                                v.get("y").and_then(Value::as_f64).unwrap_or(0.0),
                                            ),
                                            Err(e) => format!("erro: {e}"),
                                        };
                                    }
                                    ui.add_enabled(false, egui::Button::new("Trocar"))
                                        .on_disabled_hover_text("comportamento do ANAFAS (149) não levantado; pendência na paridade §4");
                                });
                                ui.end_row();
                            });
                        });
                        if !estado.mensagem.is_empty() {
                            ui.add_space(2.0);
                            ui.label(&estado.mensagem);
                        }
                    });
                    // Rodapé de ações (ANAFAS 149): alinhado ao fim da janela.
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Fechar").clicked() {
                                fechar = true;
                            }
                            let existe = estado.ed_existe;
                            if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
                                let cmd = json!({
                                    "db": estado.db,
                                    "documento": estado.documento,
                                    "handle": handle,
                                    "codigo": estado.barra_codigo,
                                });
                                match app.run("anacad.barra.dados.excluir", cmd) {
                                    Ok(_) => {
                                        estado.ed_existe = false;
                                        estado.ed_listas_em.clear();
                                        estado.mensagem = format!("dados elétricos da barra {} removidos", estado.barra_codigo);
                                    }
                                    Err(e) => estado.mensagem = format!("erro: {e}"),
                                }
                            }
                            if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
                                aplicar_barra(app, &mut estado, &handle);
                                if !estado.mensagem.starts_with("erro") {
                                    notificacao = Some(format!("barra {} atualizada", estado.barra_codigo));
                                    fechar = true;
                                }
                            }
                            if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
                                aplicar_barra(app, &mut estado, &handle);
                                if !estado.mensagem.starts_with("erro") {
                                    notificacao = Some(format!("barra {} inserida", estado.barra_codigo));
                                    fechar = true;
                                }
                            }
                        });
                    });
                    if selecionar {
                        app.ui.anacad_dialogo.clear();
                        app.ui.anacad_handle.clear();
                        app.start("anacad.barra.editar");
                    }
                }
                "editar_linha" => {
                    let handle = if !estado.comp_handle.trim().is_empty() {
                        estado.comp_handle.clone()
                    } else {
                        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                    };
                    estado.comp_handle = handle.clone();
                    let chave = format!("l|{}|{}|{}", estado.db, estado.documento, handle);
                    if estado.ed_listas_em != chave {
                        if estado.ed_handle != handle {
                            if let Some(dados) = dados_entidade(app, &handle) {
                                preencher_estado(&mut estado, &dados);
                            }
                            estado.ed_handle = handle.clone();
                        }
                        recarregar_listas_dados(app, &mut estado, &handle, "linha");
                        estado.ed_listas_em = chave;
                    }
                    let mut selecionar = false;
                    let largura = ui.available_width().max(560.0);
                    let largura_grupo = (largura - 20.0).max(520.0);
                    // No ANAFAS as barras De/Para não se trocam depois de
                    // preenchidas: elas vêm das extremidades do traço.
                    let travado = estado.ed_existe;
                    // Cabeçalho com o handle e a "mão" (selecionar no desenho), como na
                    // janela Dados de Barra (149): o rodapé fica só com as ações.
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if crate::icons::button(ui, crate::icons::Icon::SelectObjects, 20.0, "Selecionar no desenho", false)
                                .on_hover_text("Selecionar a linha no desenho")
                                .clicked()
                            {
                                selecionar = true;
                            }
                        });
                    });
                    // Os campos têm largura fixa (`add_sized`) e os grupos largura
                    // mínima: sem isso o conteúdo passa da janela e as legendas saem
                    // cortadas. Sem área rolável, como no DIALOGEX do ANAFAS: o
                    // formulário inteiro cabe na janela.
                    ui.group(|ui| {
                        ui.set_min_width(largura_grupo);
                        ui.label(egui::RichText::new("Identificação").strong());
                        egui::Grid::new("anacad.linha.identificacao").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
                            ui.label("Barra De:");
                            campo_barra_da_linha(ui, "anacad.linha.de", &mut estado.linha_de, &estado.barras_opcoes, !travado);
                            ui.label("Nome:");
                            combo_nome_da_linha(ui, "anacad.linha.de.nome", &mut estado.linha_de, &estado.barras_opcoes, !travado);
                            ui.end_row();
                            ui.label("Barra Para:");
                            campo_barra_da_linha(ui, "anacad.linha.para", &mut estado.linha_para, &estado.barras_opcoes, !travado);
                            ui.label("Nome:");
                            combo_nome_da_linha(ui, "anacad.linha.para.nome", &mut estado.linha_para, &estado.barras_opcoes, !travado);
                            ui.end_row();
                            ui.label("Número do Circuito:");
                            ui.add_sized([90.0, 20.0], egui::DragValue::new(&mut estado.ed_circuito).speed(1.0));
                            ui.label("Área:");
                            ui.horizontal(|ui| {
                                ui.add_sized([70.0, 20.0], egui::TextEdit::singleline(&mut estado.ed_area));
                                ui.checkbox(&mut estado.ed_status, "Ligado");
                            });
                            ui.end_row();
                            ui.label("Nome:");
                            ui.add_sized([180.0, 20.0], egui::TextEdit::singleline(&mut estado.ed_nome));
                            ui.checkbox(&mut estado.ed_circ_eq, "Circuito Equivalente");
                            ui.end_row();
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_width(largura_grupo);
                        ui.horizontal(|ui| {
                            ui.label("Potência Nominal (MVA):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_potencia).speed(1.0));
                            ui.label("Comprimento (km):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_comprimento).speed(0.1));
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_width(largura_grupo);
                        ui.label(egui::RichText::new("Sequência Positiva").strong());
                        egui::Grid::new("anacad.linha.seq1").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
                            ui.label("Resistência (R1 %):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_r1).speed(0.1));
                            ui.label("Reatância (X1 %):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_x1).speed(0.1));
                            ui.end_row();
                            ui.label("Susceptância (S1 Mvar):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_s1).speed(0.1));
                            ui.end_row();
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_width(largura_grupo);
                        ui.label(egui::RichText::new("Sequência Zero").strong());
                        egui::Grid::new("anacad.linha.seq0").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
                            ui.label("Resistência (R0 %):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_r0).speed(0.1));
                            ui.label("Reatância (X0 %):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_x0).speed(0.1));
                            ui.end_row();
                            ui.label("Susceptância (S0 Mvar):");
                            ui.add_sized([110.0, 20.0], egui::DragValue::new(&mut estado.ed_s0).speed(0.1));
                            ui.end_row();
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_width(largura_grupo);
                        ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
                        // De e Para em linhas separadas: as seis capacidades lado a
                        // lado não cabem na largura do diálogo.
                        ui.horizontal(|ui| {
                            ui.label("De:");
                            for (i, v) in estado.ed_cap_de.iter_mut().enumerate() {
                                ui.label(format!("{}:", i + 1));
                                ui.add_sized([80.0, 20.0], egui::DragValue::new(v).speed(0.1));
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Para:");
                            for (i, v) in estado.ed_cap_para.iter_mut().enumerate() {
                                ui.label(format!("{}:", i + 1));
                                ui.add_sized([80.0, 20.0], egui::DragValue::new(v).speed(0.1));
                            }
                        });
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        let existe = estado.ed_existe;
                        if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
                            aplicar_linha(app, &mut estado, &handle);
                            if !estado.mensagem.starts_with("erro") {
                                notificacao = Some(format!("linha {handle} inserida"));
                                fechar = true;
                            }
                        }
                        if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
                            aplicar_linha(app, &mut estado, &handle);
                            if !estado.mensagem.starts_with("erro") {
                                notificacao = Some(format!("linha {handle} atualizada"));
                                fechar = true;
                            }
                        }
                        if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
                            let cmd = json!({
                                "db": estado.db,
                                "documento": estado.documento,
                                "handle": handle,
                            });
                            match app.run("anacad.linha.dados.excluir", cmd) {
                                Ok(_) => {
                                    estado.ed_existe = false;
                                    estado.ed_listas_em.clear();
                                    estado.mensagem = "dados elétricos da linha removidos".to_string();
                                }
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                        if ui.button("Fechar").clicked() {
                            fechar = true;
                        }
                    });
                    if selecionar {
                        app.ui.anacad_dialogo.clear();
                        app.ui.anacad_handle.clear();
                        app.start("anacad.linha.editar");
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
                            campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
                            ui.label("secundário");
                            campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
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
                            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
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
                            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
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
                            campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
                            ui.label("para");
                            campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
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
                            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
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
                            campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
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
                            campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
                            ui.label("para");
                            campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
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
                                campo_conexao(ui, "eq.de", &mut estado.eq_de, &estado.barras_opcoes);
                                ui.label("para");
                                campo_conexao(ui, "eq.para", &mut estado.eq_para, &estado.barras_opcoes);
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("barra");
                                campo_conexao(ui, "eq.barra", &mut estado.eq_barra, &estado.barras_opcoes);
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
                "editar_transformador" => {
                    let handle = if !estado.comp_handle.trim().is_empty() {
                        estado.comp_handle.clone()
                    } else {
                        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                    };
                    estado.comp_handle = handle.clone();
                    let chave = format!("t|{}|{}|{}", estado.db, estado.documento, handle);
                    if estado.ed_listas_em != chave {
                        if estado.ed_handle != handle {
                            if let Some(dados) = dados_entidade(app, &handle) {
                                preencher_equipamento(&mut estado, &dados);
                            }
                            estado.ed_handle = handle.clone();
                        }
                        carregar_barras_opcoes(app, &mut estado);
                        estado.ed_listas_em = chave;
                    }
                    egui::ScrollArea::vertical().max_height(520.0).auto_shrink([false, false]).show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Identificação").strong());
                            egui::Grid::new("anacad.trafo.identificacao").num_columns(4).show(ui, |ui| {
                                ui.label("Primário:");
                                campo_conexao(ui, "anacad.trafo.de", &mut estado.eq_de, &estado.barras_opcoes);
                                ui.label("Nome:");
                                combo_conexao(ui, "anacad.trafo.de.nome", &mut estado.eq_de, &estado.barras_opcoes);
                                ui.end_row();
                                ui.label("Secundário:");
                                campo_conexao(ui, "anacad.trafo.para", &mut estado.eq_para, &estado.barras_opcoes);
                                ui.label("Nome:");
                                combo_conexao(ui, "anacad.trafo.para.nome", &mut estado.eq_para, &estado.barras_opcoes);
                                ui.end_row();
                                ui.label("Número do Circuito:");
                                ui.add(egui::DragValue::new(&mut estado.eq_circuito).speed(1.0));
                                ui.label("Nome:");
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut estado.eq_nome).desired_width(110.0));
                                    ui.checkbox(&mut estado.eq_status, "Ligado");
                                });
                                ui.end_row();
                                ui.label("Área:");
                                ui.add(egui::TextEdit::singleline(&mut estado.eq_area).desired_width(70.0));
                                ui.horizontal(|ui| {
                                    ui.label("Capacidade (MVA):");
                                    ui.add(egui::DragValue::new(&mut estado.eq_capacidade).speed(0.1));
                                });
                                ui.checkbox(&mut estado.eq_circ_eq, "Circuito Equivalente");
                                ui.end_row();
                            });
                        });
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Sequência Positiva").strong());
                            ui.horizontal(|ui| {
                                ui.label("Resistência (R1 %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(0.1));
                                ui.label("Reatância (X1 %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(0.1));
                            });
                        });
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Sequência Zero").strong());
                            ui.horizontal(|ui| {
                                ui.label("Resistência (R0 %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(0.1));
                                ui.label("Reatância (X0 %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(0.1));
                                ui.add_enabled(false, egui::Button::new("Converter"))
                                    .on_disabled_hover_text("conversão de trafo/shunt (790/791) sem fórmula no reverso; paridade §4");
                            });
                        });
                        grupo_conexao(ui, "Tipo de Conexão do Primário", &mut estado.eq_conexao, &mut estado.eq_rn, &mut estado.eq_xn);
                        grupo_conexao(ui, "Tipo de Conexão do Secundário", &mut estado.eq_conexao2, &mut estado.eq_rn2, &mut estado.eq_xn2);
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Defasamento (°)");
                                ui.add(egui::DragValue::new(&mut estado.eq_defasamento).speed(0.1));
                                ui.checkbox(&mut estado.eq_def_expl, "Explícito");
                            });
                            ui.label(egui::RichText::new("Obs: def. do secundário em relação ao primário.").small().weak());
                        });
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Tap (pu):");
                                ui.add(egui::DragValue::new(&mut estado.eq_tap).speed(0.001));
                            });
                        });
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Número de Unidades:");
                                ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
                                ui.label("Número de Unidades em Operação:");
                                ui.add(egui::DragValue::new(&mut estado.eq_unid_oper).range(0..=9999));
                            });
                        });
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
                            ui.horizontal(|ui| {
                                ui.label("De");
                                for (i, v) in estado.eq_cap_de.iter_mut().enumerate() {
                                    ui.label(format!("{}:", i + 1));
                                    ui.add(egui::DragValue::new(v).speed(0.1));
                                }
                                ui.label("Para");
                                for (i, v) in estado.eq_cap_para.iter_mut().enumerate() {
                                    ui.label(format!("{}:", i + 1));
                                    ui.add(egui::DragValue::new(v).speed(0.1));
                                }
                            });
                        });
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        let existe = estado.eq_existe;
                        if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
                            aplicar_equipamento_trafo(app, &mut estado, &handle);
                            if !estado.mensagem.starts_with("erro") {
                                notificacao = Some("dados do transformador inseridos".to_string());
                                fechar = true;
                            }
                        }
                        if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
                            aplicar_equipamento_trafo(app, &mut estado, &handle);
                            if !estado.mensagem.starts_with("erro") {
                                notificacao = Some("transformador atualizado".to_string());
                                fechar = true;
                            }
                        }
                        if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
                            let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle});
                            match app.run("anacad.equipamento.dados.excluir", cmd) {
                                Ok(_) => {
                                    estado.eq_existe = false;
                                    estado.mensagem = "dados elétricos do transformador removidos".to_string();
                                }
                                Err(e) => estado.mensagem = format!("erro: {e}"),
                            }
                        }
                        if ui.button("Fechar").clicked() {
                            fechar = true;
                        }
                    });
                    ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
                }

                "editar_gerador" => {
                    let handle = if !estado.comp_handle.trim().is_empty() {
                        estado.comp_handle.clone()
                    } else {
                        app.session.selection().first().map(|h| h.hex()).unwrap_or_default()
                    };
                    estado.comp_handle = handle.clone();
                    let chave = format!("g|{}|{}|{}", estado.db, estado.documento, handle);
                    if estado.ed_listas_em != chave {
                        if estado.ed_handle != handle {
                            if let Some(dados) = dados_entidade(app, &handle) {
                                preencher_equipamento(&mut estado, &dados);
                            }
                            estado.ed_handle = handle.clone();
                        }
                        carregar_barras_opcoes(app, &mut estado);
                        estado.ed_listas_em = chave;
                    }
                    egui::ScrollArea::vertical().max_height(600.0).auto_shrink([false, false]).show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Identificação").strong());
                            ui.horizontal(|ui| {
                                ui.label("Barra:");
                                campo_conexao(ui, "anacad.ger.barra", &mut estado.eq_barra, &estado.barras_opcoes);
                                ui.label("Nome:");
                                combo_conexao(ui, "anacad.ger.barra.nome", &mut estado.eq_barra, &estado.barras_opcoes);
                                ui.label("Área:");
                                ui.add(egui::TextEdit::singleline(&mut estado.eq_area).desired_width(60.0));
                                ui.checkbox(&mut estado.eq_status, "Ligado");
                            });
                            ui.horizontal(|ui| {
                                ui.label("Grupo:");
                                ui.add(egui::TextEdit::singleline(&mut estado.eq_grupo).desired_width(60.0));
                                ui.label("Nome:");
                                ui.add(egui::TextEdit::singleline(&mut estado.eq_nome).desired_width(120.0));
                                ui.label("Tipo:");
                                ui.add(egui::TextEdit::singleline(&mut estado.eq_tipo_gerador).desired_width(80.0));
                                ui.checkbox(&mut estado.eq_sincrono, "Síncrono");
                                ui.checkbox(&mut estado.eq_circ_eq, "Circuito Equivalente");
                            });
                        });
                        ui.horizontal_top(|ui| {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Potência Nominal (MVA):");
                                    ui.add(egui::DragValue::new(&mut estado.eq_pot).speed(1.0));
                                });
                            });
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Barra de Alta:");
                                    campo_numero(ui, "anacad.ger.barra_alta", &mut estado.eq_barra_alta, &estado.barras_opcoes);
                                });
                                ui.checkbox(&mut estado.eq_assoc_trafo, "Associar a Transformador Elevador");
                                ui.horizontal(|ui| {
                                    ui.label("Num. Circuito:");
                                    ui.add(egui::DragValue::new(&mut estado.eq_circuito).speed(1.0));
                                });
                            });
                        });
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Sequência Positiva").strong());
                            ui.horizontal(|ui| {
                                ui.label("Resistência (R1 %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_r1).speed(0.1));
                                ui.label("Reatância Subtransitória (X''d %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_x1).speed(0.1));
                            });
                        });
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Sequência Zero").strong());
                            ui.horizontal(|ui| {
                                ui.label("Resistência (R0 %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_r0).speed(0.1));
                                ui.label("Reatância (X0 %):");
                                ui.add(egui::DragValue::new(&mut estado.eq_x0).speed(0.1));
                            });
                        });
                        grupo_conexao(ui, "Tipo de Conexão", &mut estado.eq_conexao, &mut estado.eq_rn, &mut estado.eq_xn);
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Despacho").strong());
                            ui.horizontal(|ui| {
                                ui.label("Geração Ativa (MW):");
                                ui.add(egui::DragValue::new(&mut estado.eq_p).speed(1.0));
                                ui.label("Geração Reativa (MVar):");
                                ui.add(egui::DragValue::new(&mut estado.eq_q).speed(1.0));
                            });
                        });
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Número de Unidades:");
                                ui.add(egui::DragValue::new(&mut estado.eq_unidades).range(1..=9999));
                                ui.label("Número de Unidades em Operação:");
                                ui.add(egui::DragValue::new(&mut estado.eq_unid_oper).range(0..=9999));
                                ui.label("Unidades por Transformador:");
                                ui.add(egui::DragValue::new(&mut estado.eq_unid_por_trafo).range(0..=9999));
                            });
                        });
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Capacidades de Interrupção Simétrica (kA)").strong());
                            ui.horizontal(|ui| {
                                ui.label("1:");
                                ui.add(egui::DragValue::new(&mut estado.eq_cap_de[0]).speed(0.1));
                                ui.label("2:");
                                ui.add(egui::DragValue::new(&mut estado.eq_cap_de[1]).speed(0.1));
                                ui.label("3:");
                                ui.add(egui::DragValue::new(&mut estado.eq_cap_de[2]).speed(0.1));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Agregador para Compensador Síncrono:");
                                ui.add(egui::TextEdit::singleline(&mut estado.eq_agreg_comp).desired_width(80.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Agregador para Transformador de 3 Enrolamentos:");
                                ui.add(egui::TextEdit::singleline(&mut estado.eq_agreg_trafo3).desired_width(80.0));
                            });
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            let existe = estado.eq_existe;
                            if ui.add_enabled(!existe, egui::Button::new("Inserir")).clicked() {
                                aplicar_equipamento_gerador(app, &mut estado, &handle);
                                if !estado.mensagem.starts_with("erro") {
                                    notificacao = Some("dados do gerador inseridos".to_string());
                                    fechar = true;
                                }
                            }
                            if ui.add_enabled(existe, egui::Button::new("Alterar")).clicked() {
                                aplicar_equipamento_gerador(app, &mut estado, &handle);
                                if !estado.mensagem.starts_with("erro") {
                                    notificacao = Some("gerador atualizado".to_string());
                                    fechar = true;
                                }
                            }
                            if ui.add_enabled(existe, egui::Button::new("Remover")).clicked() {
                                let cmd = json!({"db": estado.db, "documento": estado.documento, "handle": handle});
                                match app.run("anacad.equipamento.dados.excluir", cmd) {
                                    Ok(_) => {
                                        estado.eq_existe = false;
                                        estado.mensagem = "dados elétricos do gerador removidos".to_string();
                                    }
                                    Err(e) => estado.mensagem = format!("erro: {e}"),
                                }
                            }
                            if ui.button("Fechar").clicked() {
                                fechar = true;
                            }
                        });
                        ui.label(egui::RichText::new(format!("handle: {handle}")).small().weak());
                    });
                }

                "editar_dados_equipamento" => {
                    let rotulo = estado.eq_tipo.clone();
                    let handle = abrir_dados_equipamento(app, &mut estado, &rotulo);
                    egui::ScrollArea::vertical().max_height(540.0).auto_shrink([false, false]).show(ui, |ui| {
                        match estado.eq_tipo.as_str() {
                            "Carga" => grupos_carga(ui, &mut estado),
                            "Shunt de Barra" => grupos_shunt_barra(ui, &mut estado),
                            "Shunt de Linha" => grupos_shunt_linha(ui, &mut estado),
                            "Motor de Indução" => grupos_motor(ui, &mut estado),
                            "Impedância Mútua" => grupos_mutua(ui, &mut estado),
                            "Compensador Série" => grupos_comp_serie(ui, &mut estado),
                            "Trafo de Aterramento" => grupos_trafo_aterramento(ui, &mut estado),
                            "Gerador Fonte de Corrente" => grupos_gerador_fonte(ui, &mut estado),
                            "Compensador Estático" => grupos_comp_estatico(ui, &mut estado),
                            _ => {}
                        }
                        let dados = dados_equipamento(&estado);
                        rodape_equipamento(app, ui, &mut estado, &handle, &rotulo, &dados, &mut fechar, &mut notificacao);
                    });
                }

                "impedancia" => {
                    let chave = format!("imp|{}|{}", estado.db, estado.documento);
                    if estado.ed_listas_em != chave {
                        carregar_barras_opcoes(app, &mut estado);
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
                        ui.add_enabled(
                            false,
                            egui::Checkbox::new(&mut estado.imp_um_gerador, "Desconsiderar todos os geradores do sistema exceto um"),
                        )
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
                            fechar = true;
                        }
                    });
                }

                "superacao" => {
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
                                        let codigos: Vec<i64> = texto
                                            .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
                                            .filter_map(|t| t.parse::<i64>().ok())
                                            .collect();
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
                                    texto.push_str(&format!("{codigo}\\n"));
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
                            fechar = true;
                        }
                    });
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
                // Gerenciador de Dados (333/336): a árvore de tabelas do ANAFAS
                // vira a coluna de ícones da esquerda e a grade dos registros
                // fica à direita. Os registros vêm do `.db` conectado, nunca do
                // desenho.
                "gerenciador" => {
                    let chave = format!("{}|{}|{}", estado.db, estado.documento, estado.gd_tabela);
                    if estado.gd_em != chave {
                        carregar_tabela(app, &mut estado);
                        estado.gd_em = chave;
                    }
                    let mut escolhida: Option<&'static str> = None;
                    let mut recarregar = false;
                    // Só a coluna dos ícones rola; a grade ocupa o resto.
                    let altura_icones = (ui.available_height() - 44.0).max(120.0);
                    ui.horizontal_top(|ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("anacad.gerenciador.tabelas")
                            .max_height(altura_icones)
                            .auto_shrink([true, false])
                            .show(ui, |ui| {
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
                            egui::ScrollArea::both()
                                .id_salt("anacad.gerenciador.grade")
                                .max_height(altura_grade)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
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
                                    fechar = true;
                                }
                            });
                        });
                    });
                    if recarregar {
                        estado.gd_em.clear();
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
/// Botão Alterar/Inserir da janela Dados de Gerador (163).
fn aplicar_equipamento_gerador(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "codigo": estado.eq_codigo,
        "nome": estado.eq_nome,
        "tipo": "Gerador",
        "barra": estado.eq_barra,
        "grupo": estado.eq_grupo,
        "area": estado.eq_area,
        "status": estado.eq_status,
        "tipoGerador": estado.eq_tipo_gerador,
        "sincrono": estado.eq_sincrono,
        "circuitoEquivalente": estado.eq_circ_eq,
        "potencia": estado.eq_pot,
        "barraAlta": estado.eq_barra_alta,
        "associarTrafoElevador": estado.eq_assoc_trafo,
        "numeroCircuito": estado.eq_circuito,
        "R1": estado.eq_r1,
        "X1": estado.eq_x1,
        "R0": estado.eq_r0,
        "X0": estado.eq_x0,
        "conexao": estado.eq_conexao,
        "Rn": estado.eq_rn,
        "Xn": estado.eq_xn,
        "P": estado.eq_p,
        "Q": estado.eq_q,
        "unidades": estado.eq_unidades,
        "unidadesOperacao": estado.eq_unid_oper,
        "unidadesPorTransformador": estado.eq_unid_por_trafo,
        "capacidadesDe": estado.eq_cap_de.to_vec(),
        "agregadorCompensador": estado.eq_agreg_comp,
        "agregadorTrafo3": estado.eq_agreg_trafo3,
    });
    let cmd = json!({
        "db": estado.db,
        "documento": estado.documento,
        "handle": handle,
        "dados": dados,
    });
    estado.mensagem = match app.run("anacad.equipamento.editar", cmd) {
        Ok(_) => String::new(),
        Err(e) => format!("erro: {e}"),
    };
}

/// Carrega handle/estado comum das janelas `Dados de …` de equipamento.
fn abrir_dados_equipamento(app: &mut CadApp, estado: &mut Estado, chave_tipo: &str) -> String {
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
fn caption_equipamento(tipo: &str) -> &'static str {
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
fn rodape_equipamento(
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
fn grupo_capacidades3(ui: &mut Ui, capacidades: &mut [f64; 3]) {
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
fn grupo_radios_conexao(ui: &mut Ui, conexao: &mut String) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Tipo de Conexão").strong());
        for c in ["Delta", "Estrela não aterrado", "Estrela aterrado"] {
            ui.radio_value(conexao, c.to_string(), c);
        }
    });
}

fn grupos_carga(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.carga.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.carga.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Área:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(60.0));
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_grupo).desired_width(70.0));
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

fn grupos_shunt_barra(ui: &mut Ui, e: &mut Estado) {
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
            ui.add(egui::TextEdit::singleline(&mut e.eq_grupo).desired_width(70.0));
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(130.0));
        });
        ui.horizontal(|ui| {
            ui.label("Área:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(60.0));
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

fn grupos_shunt_linha(ui: &mut Ui, e: &mut Estado) {
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
            ui.add(egui::TextEdit::singleline(&mut e.eq_grupo).desired_width(70.0));
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(110.0));
        });
        ui.horizontal(|ui| {
            ui.label("Área:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(60.0));
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

fn grupos_motor(ui: &mut Ui, e: &mut Estado) {
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
            ui.add(egui::TextEdit::singleline(&mut e.eq_grupo).desired_width(70.0));
            ui.label("Área:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(60.0));
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

fn grupos_mutua(ui: &mut Ui, e: &mut Estado) {
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
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(60.0));
            ui.checkbox(&mut e.eq_status, "Ligado");
        });
    });
}

fn grupos_comp_serie(ui: &mut Ui, e: &mut Estado) {
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
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(60.0));
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
        ui.horizontal(|ui| {
            ui.label("De");
            for (i, v) in e.eq_cap_de.iter_mut().enumerate() {
                ui.label(format!("{}:", i + 1));
                ui.add(egui::DragValue::new(v).speed(0.1));
            }
            ui.label("Para");
            for (i, v) in e.eq_cap_para.iter_mut().enumerate() {
                ui.label(format!("{}:", i + 1));
                ui.add(egui::DragValue::new(v).speed(0.1));
            }
        });
    });
}

fn grupos_trafo_aterramento(ui: &mut Ui, e: &mut Estado) {
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
            ui.add(egui::TextEdit::singleline(&mut e.eq_grupo).desired_width(60.0));
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(100.0));
            ui.label("Área:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(50.0));
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

fn grupos_gerador_fonte(ui: &mut Ui, e: &mut Estado) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Identificação").strong());
        ui.horizontal(|ui| {
            ui.label("Barra:");
            campo_conexao(ui, "anacad.gf.barra", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Nome:");
            combo_conexao(ui, "anacad.gf.barra.nome", &mut e.eq_barra, &e.barras_opcoes);
            ui.label("Área:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(50.0));
        });
        ui.horizontal(|ui| {
            ui.label("Grupo:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_grupo).desired_width(70.0));
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

fn grupos_comp_estatico(ui: &mut Ui, e: &mut Estado) {
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
            ui.add(egui::TextEdit::singleline(&mut e.eq_grupo).desired_width(70.0));
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_nome).desired_width(130.0));
        });
        ui.horizontal(|ui| {
            ui.label("Área:");
            ui.add(egui::TextEdit::singleline(&mut e.eq_area).desired_width(60.0));
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
fn dados_equipamento(estado: &Estado) -> Value {
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

/// Lista de barras do banco para os combos das janelas de equipamento.
fn carregar_barras_opcoes(app: &mut CadApp, estado: &mut Estado) {
    estado.barras_opcoes.clear();
    let cmd = json!({"db": estado.db, "documento": estado.documento});
    if let Ok(resposta) = app.run("anacad.consultar", cmd)
        && let Some(lista) = resposta.get("barras").and_then(Value::as_array)
    {
        for b in lista {
            if let Some(codigo) = b.get("codigo").and_then(Value::as_i64) {
                let nome = b.get("nome").and_then(Value::as_str).unwrap_or("").to_string();
                estado.barras_opcoes.push((codigo, nome));
            }
        }
        estado.barras_opcoes.sort_by_key(|(codigo, _)| *codigo);
    }
}

/// Grupo "Tipo de Conexão" do transformador/gerador: rádios + Rn/Xn.
fn grupo_conexao(ui: &mut Ui, titulo: &str, conexao: &mut String, rn: &mut f64, xn: &mut f64) {
    ui.group(|ui| {
        ui.label(egui::RichText::new(titulo).strong());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for c in ["Delta", "Estrela não aterrado", "Estrela aterrado"] {
                    ui.radio_value(conexao, c.to_string(), c);
                }
            });
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Resistência de Aterramento (Rn %):");
                    ui.add(egui::DragValue::new(rn).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Reatância de Aterramento (Xn %):");
                    ui.add(egui::DragValue::new(xn).speed(0.1));
                });
            });
        });
    });
}

/// Botão Alterar/Inserir da janela Dados de Transformador (176).
fn aplicar_equipamento_trafo(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "codigo": estado.eq_codigo,
        "nome": estado.eq_nome,
        "tipo": "Transformador",
        "de": estado.eq_de,
        "para": estado.eq_para,
        "numeroCircuito": estado.eq_circuito,
        "area": estado.eq_area,
        "capacidade": estado.eq_capacidade,
        "circuitoEquivalente": estado.eq_circ_eq,
        "status": estado.eq_status,
        "R1": estado.eq_r1,
        "X1": estado.eq_x1,
        "R0": estado.eq_r0,
        "X0": estado.eq_x0,
        "conexao": estado.eq_conexao,
        "Rn": estado.eq_rn,
        "Xn": estado.eq_xn,
        "conexao2": estado.eq_conexao2,
        "Rn2": estado.eq_rn2,
        "Xn2": estado.eq_xn2,
        "defasamento": estado.eq_defasamento,
        "defasamentoExplicito": estado.eq_def_expl,
        "tap": estado.eq_tap,
        "unidades": estado.eq_unidades,
        "unidadesOperacao": estado.eq_unid_oper,
        "capacidadesDe": estado.eq_cap_de.to_vec(),
        "capacidadesPara": estado.eq_cap_para.to_vec(),
    });
    let cmd = json!({
        "db": estado.db,
        "documento": estado.documento,
        "handle": handle,
        "dados": dados,
    });
    estado.mensagem = match app.run("anacad.equipamento.editar", cmd) {
        Ok(_) => String::new(),
        Err(e) => format!("erro: {e}"),
    };
}

/// Recarrega barras, linhas e circuitos do banco para as janelas de dados
/// (Dados de Barra 149 e Dados de Linha 164) e resolve o que só o banco sabe:
/// se o elemento já tem dados elétricos e quais são as barras vizinhas.
fn recarregar_listas_dados(app: &mut CadApp, estado: &mut Estado, handle: &str, tipo: &str) {
    estado.barras_opcoes.clear();
    estado.vizinhos.clear();
    let cmd = json!({"db": estado.db, "documento": estado.documento});
    let Ok(resposta) = app.run("anacad.consultar", cmd) else {
        return;
    };
    let mut id_codigo: Vec<(i64, i64)> = Vec::new();
    let mut barra_do_handle: Option<(i64, bool)> = None;
    let mut dados_por_codigo: Vec<(i64, bool)> = Vec::new();
    if let Some(lista) = resposta.get("barras").and_then(Value::as_array) {
        for b in lista {
            let Some(codigo) = b.get("codigo").and_then(Value::as_i64) else {
                continue;
            };
            let nome = b.get("nome").and_then(Value::as_str).unwrap_or("").to_string();
            if let Some(id) = b.get("id").and_then(Value::as_i64) {
                id_codigo.push((id, codigo));
            }
            let tem_dados = b.get("dados").and_then(Value::as_str).is_some() || b.get("vbase").and_then(Value::as_f64).is_some();
            dados_por_codigo.push((codigo, tem_dados));
            if b.get("handle").and_then(Value::as_str) == Some(handle) {
                barra_do_handle = Some((codigo, tem_dados));
            }
            estado.barras_opcoes.push((codigo, nome));
        }
    }
    estado.barras_opcoes.sort_by_key(|(codigo, _)| *codigo);
    if tipo == "barra" {
        let alvo = barra_do_handle
            .or_else(|| dados_por_codigo.iter().find(|(codigo, _)| *codigo == estado.barra_codigo).map(|(codigo, tem)| (*codigo, *tem)));
        match alvo {
            Some((codigo, tem_dados)) => {
                if estado.barra_codigo == 0 {
                    estado.barra_codigo = codigo;
                }
                estado.ed_existe = tem_dados;
            }
            None => estado.ed_existe = false,
        }
        let mut vizinhos: Vec<i64> = Vec::new();
        if let Some(lista) = resposta.get("circuitos").and_then(Value::as_array) {
            for c in lista {
                let (Some(de), Some(para)) = (c.get("de").and_then(Value::as_i64), c.get("para").and_then(Value::as_i64)) else {
                    continue;
                };
                if de == estado.barra_codigo && para != de {
                    vizinhos.push(para);
                }
                if para == estado.barra_codigo && de != para {
                    vizinhos.push(de);
                }
            }
        }
        if let Some(lista) = resposta.get("linhas").and_then(Value::as_array) {
            for l in lista {
                let de = l.get("de").and_then(Value::as_i64).and_then(|id| id_codigo.iter().find(|(i, _)| *i == id).map(|(_, c)| *c));
                let para = l.get("para").and_then(Value::as_i64).and_then(|id| id_codigo.iter().find(|(i, _)| *i == id).map(|(_, c)| *c));
                let (Some(de), Some(para)) = (de, para) else {
                    continue;
                };
                if de == estado.barra_codigo && para != de {
                    vizinhos.push(para);
                }
                if para == estado.barra_codigo && de != para {
                    vizinhos.push(de);
                }
            }
        }
        vizinhos.sort_unstable();
        vizinhos.dedup();
        for codigo in vizinhos {
            let nome = nome_da_barra(&estado.barras_opcoes, codigo);
            estado.vizinhos.push((codigo, nome));
        }
        if !estado.vizinhos.iter().any(|(c, _)| *c == estado.viz_codigo) {
            estado.viz_codigo = estado.vizinhos.first().map(|(c, _)| *c).unwrap_or(0);
        }
    } else {
        let mut tem_dados = false;
        let mut de_codigo = None;
        let mut para_codigo = None;
        if let Some(lista) = resposta.get("linhas").and_then(Value::as_array) {
            for l in lista {
                if l.get("handle").and_then(Value::as_str) != Some(handle) {
                    continue;
                }
                tem_dados = l.get("dados").and_then(Value::as_str).is_some();
                de_codigo = l.get("de").and_then(Value::as_i64).and_then(|id| id_codigo.iter().find(|(i, _)| *i == id).map(|(_, c)| *c));
                para_codigo = l.get("para").and_then(Value::as_i64).and_then(|id| id_codigo.iter().find(|(i, _)| *i == id).map(|(_, c)| *c));
            }
        }
        estado.ed_existe = tem_dados;
        if estado.linha_de == 0
            && let Some(c) = de_codigo
        {
            estado.linha_de = c;
        }
        if estado.linha_para == 0
            && let Some(c) = para_codigo
        {
            estado.linha_para = c;
        }
    }
}

fn nome_da_barra(opcoes: &[(i64, String)], codigo: i64) -> String {
    opcoes.iter().find(|(c, _)| *c == codigo).map(|(_, nome)| nome.clone()).unwrap_or_default()
}

/// Lista suspensa editável do ANAFAS: campo de texto com o número e um popup
/// com as barras do banco para escolher.
fn campo_numero(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let mut texto = if *codigo == 0 { String::new() } else { codigo.to_string() };
    if ui.add(egui::TextEdit::singleline(&mut texto).desired_width(52.0)).changed() {
        *codigo = texto.trim().parse().unwrap_or(0);
    }
    popup_codigos(ui, id, codigo, opcoes);
}

fn popup_codigos(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    egui::ComboBox::from_id_salt(id).width(14.0).selected_text("").show_ui(ui, |ui| {
        for (c, nome) in opcoes {
            if ui.selectable_label(*codigo == *c, format!("{c} — {nome}")).clicked() {
                *codigo = *c;
            }
        }
    });
}

/// Lista suspensa editável do campo Nome: texto livre + popup com os nomes do
/// banco; escolher um nome sincroniza o número.
fn campo_nome_editavel(ui: &mut Ui, id: &str, nome: &mut String, codigo: &mut i64, opcoes: &[(i64, String)]) {
    ui.add(egui::TextEdit::singleline(nome).desired_width(150.0));
    egui::ComboBox::from_id_salt(id).width(14.0).selected_text("").show_ui(ui, |ui| {
        for (c, n) in opcoes {
            if ui.selectable_label(*codigo == *c, n).clicked() {
                *codigo = *c;
                *nome = n.clone();
            }
        }
    });
}

/// Lista suspensa (não editável) com os nomes das barras; escolher o nome
/// preenche o número correspondente.
fn combo_nome(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let atual = nome_da_barra(opcoes, *codigo);
    egui::ComboBox::from_id_salt(id).width(200.0).selected_text(atual).show_ui(ui, |ui| {
        for (c, nome) in opcoes {
            if ui.selectable_label(*codigo == *c, nome).clicked() {
                *codigo = *c;
            }
        }
    });
}

/// Campo de conexão do equipamento (a barra, ou o **De**/**Para** da linha)
/// travado: o vínculo vem da ligação criada ao inserir o bloco, como no ANAFAS,
/// e digitar aqui criaria um vínculo que o desenho não tem.
fn campo_conexao(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let r = ui.add_enabled_ui(false, |ui| campo_numero(ui, id, codigo, opcoes));
    r.response.on_hover_text("A barra vem da ligação do equipamento no desenho");
}

/// Nome da barra travado junto com o número do campo de conexão.
fn combo_conexao(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let r = ui.add_enabled_ui(false, |ui| combo_nome(ui, id, codigo, opcoes));
    r.response.on_hover_text("A barra vem da ligação do equipamento no desenho");
}

/// Campo **Barra De**/**Barra Para** da janela Dados de Linha (164). No ANAFAS,
/// depois de gravadas as barras não mudam na janela: elas vêm das extremidades
/// do traço (o XData `de`/`para` da linha).
fn campo_barra_da_linha(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)], habilitado: bool) {
    let r = ui.add_enabled_ui(habilitado, |ui| campo_numero(ui, id, codigo, opcoes));
    if !habilitado {
        r.response.on_hover_text("De/Para vêm das barras nas extremidades da linha");
    }
}

/// Lista do **Nome** de De/Para, travada junto com o número.
fn combo_nome_da_linha(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)], habilitado: bool) {
    let r = ui.add_enabled_ui(habilitado, |ui| combo_nome(ui, id, codigo, opcoes));
    if !habilitado {
        r.response.on_hover_text("De/Para vêm das barras nas extremidades da linha");
    }
}

/// Lista suspensa com `código — nome` (Barras Vizinhas).
fn combo_codigos(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let atual = opcoes.iter().find(|(c, _)| c == codigo).map(|(c, n)| format!("{c} — {n}")).unwrap_or_default();
    egui::ComboBox::from_id_salt(id).width(150.0).selected_text(atual).show_ui(ui, |ui| {
        for (c, nome) in opcoes {
            if ui.selectable_label(*codigo == *c, format!("{c} — {nome}")).clicked() {
                *codigo = *c;
            }
        }
    });
}

/// Estado do liga/desliga dos textos de identificação (`"barra"` | `"linha"`)
/// no desenho aberto: `true` quando todas as identificações estão visíveis.
fn textos_no_desenho(app: &mut CadApp, tipo: &str) -> bool {
    app.run("anacad.barra.textos", json!({ "tipo": tipo, "consultar": true }))
        .ok()
        .and_then(|v| v.get("visivel").and_then(Value::as_bool))
        .unwrap_or(true)
}

/// Liga/desliga os TEXT de identificação de todas as barras (`"barra"`) ou de
/// todas as linhas (`"linha"`) pelo comando que grava `Visible` nos textos.
fn alternar_textos(app: &mut CadApp, tipo: &str, visivel: Option<bool>) -> String {
    let mut cmd = json!({ "tipo": tipo });
    if let Some(v) = visivel {
        cmd["visivel"] = json!(v);
    }
    match app.run("anacad.barra.textos", cmd) {
        Ok(v) => {
            let visivel = v.get("visivel").and_then(Value::as_bool).unwrap_or(false);
            let textos = v.get("textos").and_then(Value::as_i64).unwrap_or(0);
            let nome = if tipo == "linha" { "linha" } else { "barra" };
            format!("textos de {nome} {}: {textos}", if visivel { "ligados" } else { "desligados" })
        }
        Err(e) => format!("erro: {e}"),
    }
}

/// Botão Alterar/Inserir da janela Dados de Barra (149).
fn aplicar_barra(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "codigo": estado.barra_codigo,
        "nome": estado.ed_nome,
        "area": estado.ed_area,
        "vbase": estado.ed_vbase,
        "tipo": estado.ed_tipo,
        "autotrafo": estado.ed_autotrafo,
        "capacidadeInterrupcao": estado.ed_capacidade,
        "capacidadesAmarre": [estado.ed_cap_am[0], estado.ed_cap_am[1]],
        "preFaltaModulo": estado.ed_pre_mod,
        "preFaltaAngulo": estado.ed_pre_ang,
        "disjuntorSE": estado.ed_disjuntor,
        "status": estado.ed_status,
    });
    let cmd = json!({
        "db": estado.db,
        "documento": estado.documento,
        "handle": handle,
        "codigo": estado.barra_codigo,
        "dados": dados,
    });
    estado.mensagem = match app.run("anacad.barra.editar", cmd) {
        Ok(_) => String::new(),
        Err(e) => format!("erro: {e}"),
    };
}

/// Botão Alterar/Inserir da janela Dados de Linha (164).
fn aplicar_linha(app: &mut CadApp, estado: &mut Estado, handle: &str) {
    let dados = json!({
        "de": estado.linha_de,
        "para": estado.linha_para,
        "nome": estado.ed_nome,
        "area": estado.ed_area,
        "numeroCircuito": estado.ed_circuito,
        "potenciaNominal": estado.ed_potencia,
        "comprimento": estado.ed_comprimento,
        "R1": estado.ed_r1,
        "X1": estado.ed_x1,
        "S1": estado.ed_s1,
        "R0": estado.ed_r0,
        "X0": estado.ed_x0,
        "S0": estado.ed_s0,
        "capacidadesDe": estado.ed_cap_de.to_vec(),
        "capacidadesPara": estado.ed_cap_para.to_vec(),
        "circuitoEquivalente": estado.ed_circ_eq,
        "status": estado.ed_status,
    });
    let cmd = json!({
        "db": estado.db,
        "documento": estado.documento,
        "handle": handle,
        "de": estado.linha_de,
        "para": estado.linha_para,
        "dados": dados,
    });
    estado.mensagem = match app.run("anacad.linha.editar", cmd) {
        Ok(_) => String::new(),
        Err(e) => format!("erro: {e}"),
    };
}

pub fn panel(app: &mut CadApp, ui: &mut Ui) {
    let mut estado: Estado = ui.ctx().data_mut(|d| d.get_temp_mut_or_default::<Estado>(egui::Id::new(ID_ESTADO)).clone());
    // O painel só mostra o banco do desenho: quem o troca é a Configuração do
    // Projeto, e o valor vem do `ANACAD_CFG`, relido a cada troca de aba. Sem
    // este espelho o painel devolveria à sessão o banco do desenho anterior.
    estado.db = app.ui.anacad_db.clone();
    estado.documento = app.ui.anacad_documento.max(1);
    // A barra do módulo mora aqui, não numa linha da barra do CAD, e fica fixa
    // no alto da janela: barra, linha e equipamentos abrem suas janelas daqui.
    crate::anacad::contrib::ferramentas(app, ui);
    ui.separator();
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

/// Abre a janela de dados do artefato apontado pelo handle — `Dados de Barra`,
/// `Dados de Linha de Transmissão` ou a janela `Dados de …` do equipamento —,
/// que é o que o seletor **Configurações** do tool set faz. Devolve `false`
/// quando o objeto não é artefato do domínio.
pub(crate) fn abrir_artefato(app: &mut CadApp, handle_txt: &str) -> bool {
    if let Some(janela) = tipo_da_entidade(app, handle_txt) {
        app.ui.anacad_handle = handle_txt.to_string();
        app.ui.anacad_dialogo = janela.into();
        return true;
    }
    let equipamento = cadcraft_doc::Handle::parse_hex(handle_txt).is_some_and(|handle| {
        app.session.doc().ok().and_then(|doc| doc.entity(handle)).is_some_and(|e| {
            let marca = e.common.xdata.iter().find(|t| t.code == 1001).map(|t| t.value.as_str()).unwrap_or("");
            let bloco = match &e.kind {
                cadcraft_doc::EntityKind::Insert(ins) => ins.block.as_str(),
                _ => "",
            };
            marca == "ANACAD_EQ" || bloco.starts_with("ANACAD_EQ_") || bloco.starts_with("ANACAD_SIMB_")
        })
    });
    if equipamento {
        app.ui.anacad_handle = handle_txt.to_string();
        app.ui.anacad_dialogo = "editar_equipamento".into();
    }
    equipamento
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
    estado.eq_circ_eq = dados.get("circuitoEquivalente").and_then(Value::as_bool).unwrap_or(false);
    estado.eq_unid_oper = i("unidadesOperacao");
    estado.eq_def_expl = dados.get("defasamentoExplicito").and_then(Value::as_bool).unwrap_or(false);
    let arr = |k: &str| {
        let mut v = [0.0; 3];
        if let Some(a) = dados.get(k).and_then(Value::as_array) {
            for (i, x) in a.iter().take(3).enumerate() {
                v[i] = x.as_f64().unwrap_or(0.0);
            }
        }
        v
    };
    estado.eq_cap_de = arr("capacidadesDe");
    estado.eq_cap_para = arr("capacidadesPara");
    estado.eq_existe = dados.get("dados").and_then(Value::as_str).is_some();
    estado.eq_sincrono = dados.get("sincrono").and_then(Value::as_bool).unwrap_or(false);
    estado.eq_assoc_trafo = dados.get("associarTrafoElevador").and_then(Value::as_bool).unwrap_or(false);
    estado.eq_barra_alta = i("barraAlta");
    estado.eq_unid_por_trafo = i("unidadesPorTransformador");
    estado.eq_agreg_comp = s("agregadorCompensador");
    estado.eq_agreg_trafo3 = s("agregadorTrafo3");
    estado.eq_tipo_gerador = s("tipoGerador");
    estado.eq_estado_oper = if s("estadoOperativo").is_empty() { "Ligado".into() } else { s("estadoOperativo") };
    estado.eq_prot_ligado = dados.get("protecaoLigado").and_then(Value::as_bool).unwrap_or(false);
    estado.eq_var_corrente = f("varistorCorrente", 0.0);
    estado.eq_gap_corrente = f("gapCorrente", 0.0);
    estado.eq_gap_tipo = s("gapTipo");
    estado.eq_gap_energia = f("gapEnergia", 0.0);
    estado.eq_gap_potencia = f("gapPotencia", 0.0);
    estado.eq_xm = f("Xm", 0.0);
    estado.eq_base_hp = f("baseHP", 0.0);
    estado.eq_mut_de1 = i("mutDe1");
    estado.eq_mut_para1 = i("mutPara1");
    estado.eq_mut_nc1 = i("mutNC1").max(1);
    estado.eq_mut_ini1 = f("mutIni1", 0.0);
    estado.eq_mut_fim1 = f("mutFim1", 100.0);
    estado.eq_mut_de2 = i("mutDe2");
    estado.eq_mut_para2 = i("mutPara2");
    estado.eq_mut_nc2 = i("mutNC2").max(1);
    estado.eq_mut_ini2 = f("mutIni2", 0.0);
    estado.eq_mut_fim2 = f("mutFim2", 100.0);
    estado.eq_mut_rm = f("Rm", 0.0);
    estado.eq_mut_xm = f("mutXm", 0.0);
    estado.eq_lado = if s("lado").is_empty() { "De".into() } else { s("lado") };
    estado.eq_bypass = dados.get("bypass").and_then(Value::as_bool).unwrap_or(false);
    estado.eq_imax = f("Imax", 0.0);
    estado.eq_fp_pre = f("FPpre", 0.0);
    estado.eq_vmin = f("Vmin", 0.0);
    estado.eq_pinic = f("Pinic", 0.0);
    estado.eq_vmax = f("Vmax", 0.0);
    estado.eq_fp_cc = f("FPcc", 0.0);
    estado.eq_curva = dados.get("usarCurva").and_then(Value::as_bool).unwrap_or(false);
    estado.eq_vp1 = f("VP1", 0.0);
    estado.eq_vp2 = f("VP2", 0.0);
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
    estado.ed_autotrafo = dados.get("autotrafo").and_then(Value::as_bool).unwrap_or(false);
    if estado.ed_tipo == "Autotrafo" {
        // valor gravado antes da janela 149 separar o rádio do checkbox
        estado.ed_tipo = "Normal".into();
        estado.ed_autotrafo = true;
    }
    estado.ed_circ_eq = dados.get("circuitoEquivalente").and_then(Value::as_bool).unwrap_or(false);
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
