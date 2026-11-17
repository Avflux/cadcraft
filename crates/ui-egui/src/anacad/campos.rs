//! Campos compartilhados (barra, conexão, área, grupo, circuito) e a carga das listas de opções.

use super::*;

/// Lista de barras do banco para os combos das janelas de equipamento.
pub(super) fn carregar_barras_opcoes(app: &mut CadApp, estado: &mut Estado) {
    estado.barras_opcoes.clear();
    let cmd = json!({"db": estado.db, "documento": estado.documento});
    if let Ok(resposta) = app.run("anacad.consultar", cmd) {
        if let Some(lista) = resposta.get("barras").and_then(Value::as_array) {
            for b in lista {
                if let Some(codigo) = b.get("codigo").and_then(Value::as_i64) {
                    let nome = b.get("nome").and_then(Value::as_str).unwrap_or("").to_string();
                    estado.barras_opcoes.push((codigo, nome));
                }
            }
            estado.barras_opcoes.sort_by_key(|(codigo, _)| *codigo);
        }
        carregar_sugestoes(&resposta, estado);
    }
}

/// Listas de sugestão das janelas de dados: áreas e grupos do documento, que o
/// `anacad.consultar` devolve já agregados (o `DARE` das áreas e o `Grupo` dos
/// equipamentos).
pub(super) fn carregar_sugestoes(resposta: &Value, estado: &mut Estado) {
    estado.areas_opcoes =
        resposta.get("areas").and_then(Value::as_array).map(|lista| lista.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    estado.grupos_opcoes = resposta
        .get("grupos")
        .and_then(Value::as_array)
        .map(|lista| lista.iter().filter_map(|g| g.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    estado.grupos_opcoes.sort();
}

/// Recarrega barras, linhas e circuitos do banco para as janelas de dados
/// (Dados de Barra 149 e Dados de Linha 164) e resolve o que só o banco sabe:
/// se o elemento já tem dados elétricos e quais são as barras vizinhas.
pub(super) fn recarregar_listas_dados(app: &mut CadApp, estado: &mut Estado, handle: &str, tipo: &str) {
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
    carregar_sugestoes(&resposta, estado);
    // Circuitos por par de barras: a lista do "Número do Circuito:" da janela
    // 164 mostra os que já existem entre o De e o Para.
    estado.circuitos_par.clear();
    if let Some(lista) = resposta.get("linhas").and_then(Value::as_array) {
        for l in lista {
            let codigo_de = |id: Option<i64>| id.and_then(|id| id_codigo.iter().find(|(i, _)| *i == id).map(|(_, c)| *c));
            let de = codigo_de(l.get("de").and_then(Value::as_i64));
            let para = codigo_de(l.get("para").and_then(Value::as_i64));
            let nsg = l.get("nsg").and_then(Value::as_i64).unwrap_or(0);
            if let (Some(de), Some(para)) = (de, para)
                && nsg > 0
            {
                estado.circuitos_par.push((de, para, nsg));
            }
        }
    }
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

pub(super) fn nome_da_barra(opcoes: &[(i64, String)], codigo: i64) -> String {
    opcoes.iter().find(|(c, _)| *c == codigo).map(|(_, nome)| nome.clone()).unwrap_or_default()
}

/// Lista suspensa editável do ANAFAS: campo de texto com o número e um popup
/// com as barras do banco para escolher.
pub(super) fn campo_numero(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let mut texto = if *codigo == 0 { String::new() } else { codigo.to_string() };
    if ui.add(egui::TextEdit::singleline(&mut texto).desired_width(52.0)).changed() {
        *codigo = texto.trim().parse().unwrap_or(0);
    }
    popup_codigos(ui, id, codigo, opcoes);
}

pub(super) fn popup_codigos(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
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
pub(super) fn campo_nome_editavel(ui: &mut Ui, id: &str, nome: &mut String, codigo: &mut i64, opcoes: &[(i64, String)]) {
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
pub(super) fn combo_nome(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
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
pub(super) fn campo_conexao(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let r = ui.add_enabled_ui(false, |ui| campo_numero(ui, id, codigo, opcoes));
    r.response.on_hover_text("A barra vem da ligação do equipamento no desenho");
}

/// Nome da barra travado junto com o número do campo de conexão.
pub(super) fn combo_conexao(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let r = ui.add_enabled_ui(false, |ui| combo_nome(ui, id, codigo, opcoes));
    r.response.on_hover_text("A barra vem da ligação do equipamento no desenho");
}

/// Campo **Barra De**/**Barra Para** da janela Dados de Linha (164). No ANAFAS,
/// depois de gravadas as barras não mudam na janela: elas vêm das extremidades
/// do traço (o XData `de`/`para` da linha).
pub(super) fn campo_barra_da_linha(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)], habilitado: bool) {
    let r = ui.add_enabled_ui(habilitado, |ui| campo_numero(ui, id, codigo, opcoes));
    if !habilitado {
        r.response.on_hover_text("De/Para vêm das barras nas extremidades da linha");
    }
}

/// Lista do **Nome** de De/Para, travada junto com o número.
pub(super) fn combo_nome_da_linha(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)], habilitado: bool) {
    let r = ui.add_enabled_ui(habilitado, |ui| combo_nome(ui, id, codigo, opcoes));
    if !habilitado {
        r.response.on_hover_text("De/Para vêm das barras nas extremidades da linha");
    }
}

/// Lista suspensa editável da **Área** (ANAFAS: “Área:” é `ComboBox`
/// `CBS_DROPDOWN`, editável): texto livre + popup com as áreas do documento.
pub(super) fn campo_area(ui: &mut Ui, area: &mut String, opcoes: &[i64]) {
    ui.add(egui::TextEdit::singleline(area).desired_width(52.0));
    egui::ComboBox::from_id_salt(ui.next_auto_id()).width(14.0).selected_text("").show_ui(ui, |ui| {
        for a in opcoes {
            let texto = a.to_string();
            if ui.selectable_label(area.trim() == texto, &texto).clicked() {
                *area = texto;
            }
        }
    });
}

/// Lista suspensa editável do **Grupo** dos equipamentos (mesma forma do
/// “Área”: texto livre + popup com os grupos já usados no documento).
pub(super) fn campo_grupo(ui: &mut Ui, grupo: &mut String, opcoes: &[String]) {
    ui.add(egui::TextEdit::singleline(grupo).desired_width(60.0));
    egui::ComboBox::from_id_salt(ui.next_auto_id()).width(14.0).selected_text("").show_ui(ui, |ui| {
        for g in opcoes {
            if ui.selectable_label(grupo.trim() == g, g).clicked() {
                *grupo = g.clone();
            }
        }
    });
}

/// Lista suspensa editável do **Número do Circuito** (ANAFAS: `ComboBox`
/// editável; o combo é recarregado para o par De/Para — `FUN_0076ffd0`,
/// documentado em `COMPORTAMENTO.md` §793): número livre + popup com os
/// circuitos que já existem entre as duas barras.
pub(super) fn campo_circuito(ui: &mut Ui, numero: &mut i64, de: i64, para: i64, opcoes: &[(i64, i64, i64)]) {
    ui.add(egui::DragValue::new(numero).speed(1.0).range(1..=9999));
    let do_par: Vec<i64> = opcoes.iter().filter(|(d, p, _)| (*d == de && *p == para) || (*d == para && *p == de)).map(|(_, _, nsg)| *nsg).collect();
    if do_par.is_empty() {
        return;
    }
    egui::ComboBox::from_id_salt(ui.next_auto_id()).width(14.0).selected_text("").show_ui(ui, |ui| {
        for nsg in do_par {
            if ui.selectable_label(*numero == nsg, nsg.to_string()).clicked() {
                *numero = nsg;
            }
        }
    });
}

/// Lista suspensa com `código — nome` (Barras Vizinhas).
pub(super) fn combo_codigos(ui: &mut Ui, id: &str, codigo: &mut i64, opcoes: &[(i64, String)]) {
    let atual = opcoes.iter().find(|(c, _)| c == codigo).map(|(c, n)| format!("{c} — {n}")).unwrap_or_default();
    egui::ComboBox::from_id_salt(id).width(150.0).selected_text(atual).show_ui(ui, |ui| {
        for (c, nome) in opcoes {
            if ui.selectable_label(*codigo == *c, format!("{c} — {nome}")).clicked() {
                *codigo = *c;
            }
        }
    });
}
