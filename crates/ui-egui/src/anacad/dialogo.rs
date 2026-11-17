//! Despachante das janelas do AnaCAD: título, tamanho e a janela de cada ação.

use super::*;

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
    janela.show(ctx, |ui| {
        match acao.as_str() {
            "cfg_projeto" => projeto::janela_cfg_projeto(app, ui, &mut estado),
            "criar_barra" => desenho::janela_criar_barra(app, ui, &mut estado),
            "criar_linha" => desenho::janela_criar_linha(app, ui, &mut estado),
            "criar_legenda" => desenho::janela_criar_legenda(app, ui, &mut estado),
            "opcoes_legenda" => desenho::janela_opcoes_legenda(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "equivalente" => analise::janela_equivalente(app, ui, &mut estado),
            "simultaneo" => analise::janela_simultaneo(app, ui, &mut estado),
            "zbus" => analise::janela_zbus(app, ui, &mut estado),
            "niveis_curto" => analise::janela_niveis_curto(app, ui, &mut estado),
            "texto_livre" => desenho::janela_texto_livre(app, ui, &mut estado),
            "detalhar_barra" => desenho::janela_detalhar_barra(app, ui, &mut estado),
            "renomear_terminal" | "localizar_terminal" => desenho::janela_terminal(app, ui, &mut estado, &acao, &mut fechar, &mut notificacao),
            "renumerar_barra" => desenho::janela_renumerar_barra(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "localizar_barra" => desenho::janela_localizar_barra(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "seccionar_linha" => desenho::janela_seccionar_linha(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "separar_barra" => desenho::janela_separar_barra(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "unir_barras" => desenho::janela_unir_barras(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "editar_barra" => barra::janela_editar_barra(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "editar_linha" => linha::janela_editar_linha(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "editar_equipamento" => equipamento::janela_editar_equipamento(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "editar_transformador" => transformador::janela_editar_transformador(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "editar_gerador" => gerador::janela_editar_gerador(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "editar_dados_equipamento" => dados_equipamento::janela_editar_dados_equipamento(app, ui, &mut estado, &mut fechar, &mut notificacao),
            "impedancia" => analise::janela_impedancia(app, ui, &mut estado, &mut fechar),
            "superacao" => analise::janela_superacao(app, ui, &mut estado, &mut fechar),
            "atribuir_barra" | "atribuir_linha" | "mover" | "apagar" => operacoes::janela_por_handle(app, ui, &mut estado, &acao),
            "importar_lst" | "importar_ana" => operacoes::janela_importar(app, ui, &mut estado, &acao),
            "exportar_ana" => operacoes::janela_exportar_ana(app, ui, &mut estado),
            "exportar" => operacoes::janela_exportar(app, ui, &mut estado),
            "curto" => analise::janela_curto(app, ui, &mut estado),
            "gerenciador" => gerenciador::janela_gerenciador(app, ui, &mut estado, &mut fechar),
            _ => operacoes::janela_executar(app, ui, &mut estado, &acao),
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
