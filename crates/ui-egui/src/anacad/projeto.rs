//! Configuração do Projeto: banco do desenho, exibição e carga da configuração.

use super::*;

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
        app.ui.anacad_equip_escala = f64_do_json(&v, "equip_escala", 1.0);
        app.ui.anacad_equip_texto = f64_do_json(&v, "equip_texto", 2.0);
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

/// Chave do desenho ativo: o arquivo, ou a aba e o título de um desenho novo.
pub(super) fn chave_do_desenho(app: &CadApp) -> String {
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
    app.ui.anacad_equip_escala = f64_do_json(&v, "equip_escala", 1.0);
    app.ui.anacad_equip_texto = f64_do_json(&v, "equip_texto", 2.0);
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

/// Estado do liga/desliga dos textos de identificação (`"barra"` | `"linha"`)
/// no desenho aberto: `true` quando todas as identificações estão visíveis.
pub(super) fn textos_no_desenho(app: &mut CadApp, tipo: &str) -> bool {
    app.run("anacad.barra.textos", json!({ "tipo": tipo, "consultar": true }))
        .ok()
        .and_then(|v| v.get("visivel").and_then(Value::as_bool))
        .unwrap_or(true)
}

/// Liga/desliga os TEXT de identificação de todas as barras (`"barra"`) ou de
/// todas as linhas (`"linha"`) pelo comando que grava `Visible` nos textos.
pub(super) fn alternar_textos(app: &mut CadApp, tipo: &str, visivel: Option<bool>) -> String {
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

/// Janela `cfg_projeto` do diálogo AnaCAD.
pub(super) fn janela_cfg_projeto(app: &mut CadApp, ui: &mut Ui, estado: &mut Estado) {
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
        estilo_elemento(ui, "Barra", "cfg.estilo.barra", &mut app.ui.anacad_barra_texto, &mut app.ui.anacad_barra_cor, &mut app.ui.anacad_barra_lar);
        estilo_elemento(ui, "Linha", "cfg.estilo.linha", &mut app.ui.anacad_linha_texto, &mut app.ui.anacad_linha_cor, &mut app.ui.anacad_linha_lar);
        // Liga/desliga da identificação, um switch por elemento
        // (era na janela Barras): grava `Visible` nos TEXT de
        // todas as barras / de todas as linhas do desenho.
        ui.horizontal(|ui| {
            ui.label("Mostrar textos:");
            if ui.checkbox(&mut app.ui.anacad_textos_barra, "Barra").on_hover_text("Mostra ou oculta a identificação de todas as barras").changed()
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
    // Escala dos blocos dos equipamentos: fator com que o bloco
    // entra no desenho ao inserir equipamento (1 = tamanho
    // natural do bloco). Vale para os próximos equipamentos;
    // um bloco já inserido muda a escala dele em
    // `anacad.equipamento.transformar`.
    ui.group(|ui| {
        ui.label(egui::RichText::new("Equipamentos").strong());
        ui.horizontal(|ui| {
            ui.label("Escala dos blocos:");
            ui.add(egui::DragValue::new(&mut app.ui.anacad_equip_escala).range(0.1..=10.0).speed(0.05))
                .on_hover_text("Fator de escala do bloco do equipamento ao inserir (1 = tamanho natural)");
        });
        ui.horizontal(|ui| {
            ui.label("Tamanho do texto:");
            ui.add(egui::DragValue::new(&mut app.ui.anacad_equip_texto).range(0.5..=50.0).speed(0.1))
                .on_hover_text("Altura do texto dos atributos do bloco (CODIGO, NOME, TENSAO, T1, T2), independente da escala do bloco");
        });
        ui.small("Vale para os equipamentos inseridos a partir da gravação.");
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
                "equip_escala": app.ui.anacad_equip_escala,
                "equip_texto": app.ui.anacad_equip_texto,
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
