use super::*;

/// Posição do `attdef` `tag` na definição do bloco.
fn terminal(s: &Session, bloco: &str, tag: &str) -> Option<(f64, f64)> {
    s.doc().ok()?.blocks.get(bloco)?.entities.iter().find_map(|e| match &e.kind {
        EntityKind::AttDef(a) if a.tag == tag => Some((a.text.insert.x, a.text.insert.y)),
        _ => None,
    })
}

/// O bloco leva os terminais do símbolo nas mesmas posições.
#[test]
fn bloco_com_os_terminais_do_simbolo() {
    let mut s = Session::new();
    for (slug, _) in anacad_app::EQUIPAMENTOS {
        let nome = bloco_equipamento(&mut s, slug).unwrap();
        assert_eq!(terminal(&s, &nome, "T1"), Some((0.0, 0.0)), "{slug}: T1 fora da origem");
    }
    assert_eq!(terminal(&s, "ANACAD_EQ_transformador", "T2"), Some((0.0, -40.54)));
    assert_eq!(terminal(&s, "ANACAD_EQ_carga", "T2"), None);
}

/// Salvar a ligação dos enrolamentos troca o bloco da referência, mantendo os
/// atributos.
#[test]
fn editar_transformador_troca_o_bloco() {
    let caminho = std::env::temp_dir().join(format!("cadcraft-anacad-simbolos-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&caminho);
    let db = caminho.to_string_lossy().to_string();
    let mut s = Session::new();
    let documento = anacad_data::Banco::abrir(&caminho).unwrap().garantir_documento(1).unwrap();
    set_banco(&mut s, &db, documento);
    let saida =
        run_equipamento_inserir(&mut s, &json!({ "db": db, "documento": documento, "tipo": "transformador", "x": 100.0, "y": 100.0 })).unwrap();
    let handle_txt = saida["handle"].as_str().unwrap().to_string();
    assert_eq!(saida["bloco"], "ANACAD_EQ_transformador");
    let dados = json!({ "conexao": "Delta", "conexao2": "Estrela não aterrado" });
    run_equipamento_editar(&mut s, &json!({ "db": db, "documento": documento, "handle": handle_txt, "dados": dados })).unwrap();
    let handle = Handle::parse_hex(&handle_txt).unwrap();
    let EntityKind::Insert(ins) = &s.doc().unwrap().entity(handle).unwrap().kind else { panic!("o equipamento nao e bloco") };
    assert_eq!(ins.block, "ANACAD_EQ_transformador_D_Y");
    let t2 = ins.attribs.iter().find(|a| a.tag == "T2").unwrap();
    assert_eq!((t2.text.insert.x, t2.text.insert.y), (100.0, 100.0 - 40.54));
    let _ = std::fs::remove_file(&caminho);
}
