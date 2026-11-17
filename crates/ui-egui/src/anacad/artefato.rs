//! Seletor de artefato: identifica barra, linha ou equipamento e preenche a janela.

use super::*;

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

pub(super) fn tipo_da_entidade(app: &CadApp, handle_txt: &str) -> Option<&'static str> {
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

pub(super) fn dados_entidade(app: &CadApp, handle_txt: &str) -> Option<serde_json::Map<String, Value>> {
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

pub(super) fn preencher_equipamento(estado: &mut Estado, dados: &serde_json::Map<String, Value>) {
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

pub(super) fn preencher_estado(estado: &mut Estado, dados: &serde_json::Map<String, Value>) {
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
