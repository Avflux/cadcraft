use super::*;

fn barra_em(s: &mut Session, codigo: i64, a: Vec2, b: Vec2) -> Handle {
    let handle = s
        .add_entity(EntityKind::LwPolyline(LwPolyline {
            vertices: vec![PolyVertex { p: a, ..PolyVertex::default() }, PolyVertex { p: b, ..PolyVertex::default() }],
            closed: false,
            const_width: 0.3,
            elevation: 0.0,
            plinegen: false,
        }))
        .unwrap();
    let codigo = codigo.to_string();
    marcar(s, handle, APP_BARRA, &[("id", "1"), ("codigo", &codigo), ("vbase", "13.8")]).unwrap();
    handle
}

#[test]
fn transformador_e_compensador_gravam_as_barras_de_t1_e_t2() {
    for (indice, (tipo, distancia_t2)) in [("transformador", 40.54), ("compensador_serie", 32.86)].into_iter().enumerate() {
        let caminho = std::env::temp_dir().join(format!("cadcraft-anacad-equipamento-{}-{indice}.db", std::process::id()));
        let _ = std::fs::remove_file(&caminho);
        let db = caminho.to_string_lossy().to_string();
        let mut s = Session::new();
        let documento = anacad_data::Banco::abrir(&caminho).unwrap().garantir_documento(1).unwrap();
        set_banco(&mut s, &db, documento);

        barra_em(&mut s, 10001, Vec2::new(90.0, 100.0), Vec2::new(110.0, 100.0));
        barra_em(&mut s, 10002, Vec2::new(90.0 + distancia_t2, 100.0), Vec2::new(110.0 + distancia_t2, 100.0));
        let mut maquina = EquipM::novo(tipo.to_string());
        maquina.begin(&mut s).unwrap();
        maquina.input(&mut s, Input::Enter).unwrap();
        maquina.input(&mut s, Input::Point(Vec2::new(100.0, 100.0))).unwrap();
        assert!(anacad_data::Banco::abrir(&caminho).unwrap().equipamentos(documento, 100).unwrap().is_empty());

        assert_eq!(maquina.input(&mut s, Input::Point(Vec2::new(100.0 + distancia_t2, 100.0))).unwrap(), Step::Done);
        let handle = s.doc().unwrap().model.iter().find(|e| app_xdata(e) == Some("ANACAD_EQ")).unwrap().handle;

        let equipamento = s.doc().unwrap().entity(handle).unwrap();
        let dados: Value = serde_json::from_str(campo_xdata(equipamento, "dados").unwrap()).unwrap();
        assert_eq!(dados["de"], 10001);
        assert_eq!(dados["para"], 10002);

        let banco = anacad_data::Banco::abrir(&caminho).unwrap();
        assert_eq!(banco.equipamentos(documento, 100).unwrap().len(), 1);
        let conexoes = banco.conexoes(documento).unwrap();
        assert_eq!(conexoes.len(), 2);
        assert!(conexoes.iter().any(|c| c.para_terminal.as_deref() == Some("T1")));
        assert!(conexoes.iter().any(|c| c.para_terminal.as_deref() == Some("T2")));
        drop(banco);
        let _ = std::fs::remove_file(&caminho);
    }
}
