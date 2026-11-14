//! Leitura e escrita dos arquivos `.ANA`/`.LST` do irmao (`anacad-core::ana`).

use super::*;

pub(super) fn run_ana_importar(_s: &mut Session, p: &Value) -> Result<Value> {
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.ana.importar", "`arquivo` is required"))?;
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ana.importar", "`db` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let bytes = ler_arquivo(arquivo)?;
    let ana = anacad_core::ana::ArquivoAna::ler(&bytes).map_err(|e| bad("anacad.ana.importar", format!("{arquivo}: {e}")))?;
    let barras = ana.ler_dbar();
    let circuitos = ana.ler_dcir();
    let shunts = ana.ler_dshl();
    let geradores = ana.ler_deol();
    let mutuas = ana.ler_dmut();
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ana.importar", format!("{db}: {e}")))?;
    let mut atualizadas = 0usize;
    let mut sem_barra = 0usize;
    let mut sem_vbase = 0usize;
    for barra in &barras {
        banco.atualizar_barra_ficticia(Some(documento), barra.codigo, barra.ficticia).map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        let Some(vbase) = barra.vbase else {
            sem_vbase += 1;
            continue;
        };
        let linhas = banco.atualizar_barra_vbase(Some(documento), barra.codigo, vbase).map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        if linhas == 0 {
            sem_barra += 1;
        } else {
            atualizadas += linhas;
        }
    }
    let _ = banco.apagar_circuitos(Some(documento));
    banco.apagar_ana_extras(Some(documento)).map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
    let mut circuitos_gravados = 0usize;
    for circuito in &circuitos {
        banco.inserir_circuito_ana(Some(documento), circuito).map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        circuitos_gravados += 1;
    }
    let dados_gravados = gravar_capacidades_das_barras(&banco, documento, &barras, &circuitos);
    let mut shunts_gravados = 0usize;
    for shunt in &shunts {
        banco
            .inserir_shunt(Some(documento), shunt.de, shunt.para, shunt.qpos, shunt.rn, shunt.xn)
            .map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        shunts_gravados += 1;
    }
    let mut mutuas_gravadas = 0usize;
    for mutua in &mutuas {
        banco
            .inserir_mutua(&anacad_data::MutuaNova {
                documento: Some(documento),
                de1: mutua.de1,
                para1: mutua.para1,
                de2: mutua.de2,
                para2: mutua.para2,
                rm: mutua.rm,
                xm: mutua.xm,
            })
            .map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        mutuas_gravadas += 1;
    }
    let mut geradores_gravados = 0usize;
    for gerador in &geradores {
        banco
            .inserir_gerador(Some(documento), gerador.nb, gerador.unidades, gerador.p_inic, gerador.i_max, gerador.v_min, gerador.v_max)
            .map_err(|e| bad("anacad.ana.importar", e.to_string()))?;
        geradores_gravados += 1;
    }
    Ok(json!({
        "arquivo": arquivo,
        "barras": barras.len(),
        "atualizadas": atualizadas,
        "sem_barra": sem_barra,
        "sem_vbase": sem_vbase,
        "circuitos": circuitos_gravados,
        "dados_barras": dados_gravados,
        "desligados": circuitos.iter().filter(|c| !c.ligado).count(),
        "shunts": shunts_gravados,
        "geradores": geradores_gravados,
        "mutuas": mutuas_gravadas,
    }))
}

pub(super) fn run_ana_exportar(_s: &mut Session, p: &Value) -> Result<Value> {
    let db = str_param(p, "db").ok_or_else(|| bad("anacad.ana.exportar", "`db` is required"))?;
    let arquivo = str_param(p, "arquivo").ok_or_else(|| bad("anacad.ana.exportar", "`arquivo` is required"))?;
    let documento = p.get("documento").and_then(Value::as_i64).unwrap_or(1);
    let banco = anacad_data::Banco::abrir(db).map_err(|e| bad("anacad.ana.exportar", format!("{db}: {e}")))?;
    let barras = banco.barras(documento, 100_000).map_err(|e| bad("anacad.ana.exportar", e.to_string()))?;
    let circuitos = banco.circuitos(documento, 100_000).map_err(|e| bad("anacad.ana.exportar", e.to_string()))?;
    let shunts = banco.shunts(documento, 100_000).map_err(|e| bad("anacad.ana.exportar", e.to_string()))?;
    let geradores = banco.geradores(documento, 100_000).map_err(|e| bad("anacad.ana.exportar", e.to_string()))?;
    let mutuas = banco.mutuas(documento).map_err(|e| bad("anacad.ana.exportar", e.to_string()))?;
    let mut out = String::new();
    out.push_str("TIPO\nP\nTITU     1\nExportado pelo anacad\nCMNT     1\nBanco: ");
    out.push_str(db);
    out.push('\n');
    out.push_str("DBAR\n(NB  C M      BN      VPRE ANG VBAS   DISJUN\n(--- - - ------------ ----==== ----\n");
    for b in &barras {
        let codigo = b.codigo.unwrap_or(0).to_string();
        let nome = b.nome.clone().unwrap_or_default();
        let vbase = banco.barra_vbase(documento, b.codigo.unwrap_or(0)).map_err(|e| bad("anacad.ana.exportar", e.to_string()))?.unwrap_or(0.0);
        let mut campos = vec![(0, format!("{codigo:>5}")), (9, nome), (31, format!("{vbase:.5}"))];
        // Pré-falta do formato `A`: VPRE (`F4.3`) e ANG (`F4.1`), casas implícitas.
        if let Some(prefalta) =
            banco.dados_barra(documento, b.codigo.unwrap_or(0)).ok().flatten().and_then(|texto| serde_json::from_str::<Value>(&texto).ok())
        {
            if let Some(modulo) = prefalta.get("preFaltaModulo").and_then(Value::as_f64) {
                campos.push((22, format!("{:>4}", (modulo * 1000.0).round() as i64)));
            }
            if let Some(angulo) = prefalta.get("preFaltaAngulo").and_then(Value::as_f64) {
                campos.push((26, format!("{:>5}", (angulo * 10.0).round() as i64)));
            }
            // Menor capacidade de interrupção da barra, alternativa ao `DJ_BF/DJ_BT`.
            if let Some(capacidade) = prefalta.get("capacidadeInterrupcao").and_then(Value::as_f64) {
                campos.push((38, anacad_core::ana::real_campo(capacidade, 6)));
            }
        }
        if b.ficticia {
            campos.push((7, "1".to_string()));
        }
        out.push_str(&ana_linha(&campos));
        out.push('\n');
    }
    out.push_str("99999\n");
    out.push_str("DCIR\n(BF     BT    NC R1    X1    R0    X0       CN S1   S0   TAP         IA DEF   DJ_BF DJ_BT\n(----=-=====  --=------======------======------           -----       ---===-\n");
    for c in &circuitos {
        out.push_str(&anacad_core::ana::linha_dcir(&anacad_core::ana::CircuitoAna {
            de: c.de.unwrap_or(0),
            para: c.para.unwrap_or(0),
            tipo: c.tipo.clone().unwrap_or_else(|| "1L".to_string()),
            ligado: c.ligado,
            r1: c.r1,
            x1: c.x1,
            r0: c.r0,
            x0: c.x0,
            ia: c.ia,
            dj_bf: None,
            dj_bt: None,
            s1: None,
            s0: None,
            tap: c.tap,
            defasamento: c.defasamento,
        }));
        out.push('\n');
    }
    out.push_str("99999\n");
    if !shunts.is_empty() {
        out.push_str("DSHL\n(BF  CE  BT   NC  Qpos   Rn    Xn\n(----=-=====  ==------=====------\n");
        for s in &shunts {
            let de = s.de.unwrap_or(0);
            let para = s.para.unwrap_or(0);
            let campos =
                vec![(0, format!("{de:>5}")), (5, format!("{para:>7}")), (20, ana_num(s.qpos, 5)), (30, ana_num(s.rn, 5)), (36, ana_num(s.xn, 5))];
            out.push_str(&ana_linha(&campos));
            out.push('\n');
        }
        out.push_str("99999\n");
    }
    if !geradores.is_empty() {
        out.push_str("DEOL\n(NB    P_inic       I_max  V_min         NunNop          V_max\n(--- -------------- ----- --------- ----            -------------------\n");
        for g in &geradores {
            let nb = g.nb.unwrap_or(0);
            let campos = vec![
                (0, format!("{nb:>5}")),
                (17, ana_num(g.p_inic, 5)),
                (30, ana_num(g.i_max, 5)),
                (35, ana_num(g.v_min, 5)),
                (48, ana_num(g.unidades, 1)),
                (62, ana_num(g.v_max, 5)),
            ];
            out.push_str(&ana_linha(&campos));
            out.push('\n');
        }
        out.push_str("99999\n");
    }
    if !mutuas.is_empty() {
        out.push_str("DMUT\n(BF1 CE BT1  NC1 BF2   BT2  NC2  RM    XM\n(----=-=====----=====  =====----------======\n");
        for m in &mutuas {
            out.push_str(&anacad_core::ana::linha_dmut(&anacad_core::ana::MutuaAna {
                de1: m.de1,
                para1: m.para1,
                de2: m.de2,
                para2: m.para2,
                rm: m.rm,
                xm: m.xm,
            }));
            out.push('\n');
        }
        out.push_str("99999\n");
    }
    std::fs::write(arquivo, out.as_bytes()).map_err(|e| bad("anacad.ana.exportar", format!("{arquivo}: {e}")))?;
    Ok(json!({
        "arquivo": arquivo,
        "bytes": out.len(),
        "barras": barras.len(),
        "circuitos": circuitos.len(),
        "shunts": shunts.len(),
        "geradores": geradores.len(),
        "mutuas": mutuas.len(),
    }))
}
