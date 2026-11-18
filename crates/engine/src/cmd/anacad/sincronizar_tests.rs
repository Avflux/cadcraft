use cadcraft_doc::{HAlign, VAlign};

use super::*;

fn traco(s: &mut Session, a: (f64, f64), b: (f64, f64)) -> Handle {
    let poly = LwPolyline {
        vertices: vec![
            PolyVertex { p: Vec2::new(a.0, a.1), ..PolyVertex::default() },
            PolyVertex { p: Vec2::new(b.0, b.1), ..PolyVertex::default() },
        ],
        closed: false,
        const_width: 0.3,
        elevation: 0.0,
        plinegen: false,
    };
    s.add_entity(EntityKind::LwPolyline(poly)).unwrap()
}

fn texto(s: &mut Session, app: &str, campos: &[(&str, &str)]) -> Handle {
    let texto = Text {
        insert: Vec3::new(0.0, 0.0, 0.0),
        align_pt: None,
        height: 2.5,
        value: "B1".into(),
        rotation: 0.0,
        width_factor: 1.0,
        oblique: 0.0,
        style: "Standard".into(),
        halign: HAlign::Left,
        valign: VAlign::Baseline,
    };
    let handle = s.add_entity(EntityKind::Text(texto)).unwrap();
    marcar(s, handle, app, campos).unwrap();
    handle
}

/// Barra desenhada com o texto de identificação, como `anacad.barra.criar`.
fn barra(s: &mut Session, codigo: i64) -> Handle {
    let handle = traco(s, (0.0, 0.0), (10.0, 0.0));
    let codigo_txt = codigo.to_string();
    marcar(s, handle, APP_BARRA, &[("id", "1"), ("codigo", &codigo_txt), ("vbase", "13.8")]).unwrap();
    let hex = handle.hex();
    texto(s, APP_TEXTO_BARRA, &[("campo", "identificador"), ("handle", &hex)]);
    handle
}

/// Linha desenhada entre duas barras (códigos no XData) com o texto do circuito.
fn linha(s: &mut Session, de: i64, para: i64) -> Handle {
    let handle = traco(s, (5.0, 0.0), (25.0, 0.0));
    let (de_txt, para_txt) = (de.to_string(), para.to_string());
    marcar(s, handle, APP_LINHA, &[("id", "1"), ("de", &de_txt), ("para", &para_txt)]).unwrap();
    let hex = handle.hex();
    texto(s, APP_TEXTO_LINHA, &[("campo", "circuito"), ("handle", &hex)]);
    handle
}

#[test]
fn apagar_barra_leva_as_linhas_e_os_textos_pelo_desenho() {
    let mut s = Session::new();
    let apagada = barra(&mut s, 10001);
    let outra = barra(&mut s, 10002);
    let presa = linha(&mut s, 10001, 10002);
    let antes = s.doc().unwrap().clone();
    s.doc_mut().unwrap().remove_entity(apagada);
    // Comando interativo (`ERASE` com seleção): o `uid` chega vazio.
    let saidas = cascata_barras_apagadas(&mut s, Some(&antes), None);
    assert_eq!(saidas, 3, "traco da linha, texto do circuito e texto da barra");
    let doc = s.doc().unwrap();
    assert!(doc.entity(presa).is_none(), "a linha presa a barra apagada sai");
    assert!(doc.entity(outra).is_some(), "a barra da outra ponta fica");
}

#[test]
fn apagar_barra_desvincula_componente_e_equipamento() {
    let mut s = Session::new();
    let apagada = barra(&mut s, 10001);
    let componente = traco(&mut s, (0.0, 0.0), (1.0, 0.0));
    marcar(&mut s, componente, "ANACAD_COMP", &[("id", "1"), ("tipo", "1"), ("barra", "10001")]).unwrap();
    let equipamento = traco(&mut s, (0.0, 5.0), (1.0, 5.0));
    marcar(&mut s, equipamento, "ANACAD_EQ", &[("id", "2"), ("tipo", "Carga"), ("dados", "{\"barra\":10001,\"P\":5.0}")]).unwrap();
    let antes = s.doc().unwrap().clone();
    s.doc_mut().unwrap().remove_entity(apagada);
    cascata_barras_apagadas(&mut s, Some(&antes), None);
    let doc = s.doc().unwrap();
    let comp = doc.entity(componente).unwrap();
    assert!(campo_xdata(comp, "barra").is_none(), "o componente perde o vinculo");
    assert_eq!(campo_xdata(comp, "tipo"), Some("1"), "e continua componente");
    let eq = doc.entity(equipamento).unwrap();
    let dados: Value = serde_json::from_str(campo_xdata(eq, "dados").unwrap()).unwrap();
    assert_eq!(dados.get("barra").and_then(Value::as_i64), Some(0), "o equipamento fica sem barra");
    assert_eq!(dados.get("P").and_then(Value::as_f64), Some(5.0), "e mantem os outros dados");
}

/// Barra fictícia/auxiliar: a linha vira o INSERT do bloco `ANACAD_DONUT`
/// (diâmetros interno 0 / externo 2,0, centro fechado) no centro dela,
/// no mesmo handle; voltar para Normal restaura o traço. O donut 8/4
/// antigo (CIRCLE + interno no campo `donut`) migra para o bloco.
#[test]
fn barra_ficticia_vira_donut_e_normal_restaura_o_traco() {
    use super::esquematico::{BLOCO_DONUT, RAIO_DONUT, trocar_simbolo_barra};
    let mut s = Session::new();
    let handle = barra(&mut s, 10001);
    let donut = trocar_simbolo_barra(&mut s, handle, true, (5.0, 0.0), (0.0, 0.0), (10.0, 0.0), 0.3).unwrap();
    assert!(donut.is_none(), "o donut em bloco nao tem circulo interno");
    let doc = s.doc().unwrap();
    let e = doc.entity(handle).unwrap();
    assert!(matches!(e.kind, EntityKind::Insert(_)), "a barra ficticia e o bloco donut");
    if let EntityKind::Insert(ins) = &e.kind {
        assert_eq!(ins.block, BLOCO_DONUT);
        assert_eq!((ins.insert.x, ins.insert.y), (5.0, 0.0), "o donut fica no centro da barra");
    }
    // A geometria do bloco e a do comando DONUT: polilinha fechada com
    // bulge, raio de centro 0,5 e largura 1,0 (interno 0, externo 2,0).
    let bloco = doc.blocks.get(BLOCO_DONUT).expect("bloco donut definido");
    assert_eq!(bloco.entities.len(), 1);
    if let Some(ent) = bloco.entities.iter().next()
        && let EntityKind::LwPolyline(p) = &ent.kind
    {
        assert!(p.closed, "o donut e fechado");
        assert_eq!(p.const_width, RAIO_DONUT, "largura = (externo - interno) / 2 = 1,0");
        assert_eq!(p.vertices.len(), 2);
        assert_eq!(p.vertices[0].p.x, -RAIO_DONUT / 2.0);
        assert_eq!(p.vertices[0].bulge, 1.0);
    } else {
        panic!("a geometria do bloco donut e uma polilinha");
    }
    // Reedição sem trocar o tipo: continua o INSERT do bloco.
    let outro = trocar_simbolo_barra(&mut s, handle, true, (5.0, 0.0), (0.0, 0.0), (10.0, 0.0), 0.3).unwrap();
    assert!(outro.is_none());
    assert!(matches!(s.doc().unwrap().entity(handle).unwrap().kind, EntityKind::Insert(_)), "o donut continua apos reedicao");
    // Voltar para Normal: o traco volta no mesmo handle.
    trocar_simbolo_barra(&mut s, handle, false, (5.0, 0.0), (0.0, 0.0), (10.0, 0.0), 0.3).unwrap();
    let doc = s.doc().unwrap();
    let e = doc.entity(handle).unwrap();
    assert!(matches!(e.kind, EntityKind::LwPolyline(_)), "a barra Normal volta a ser linha");
    if let EntityKind::LwPolyline(p) = &e.kind {
        assert_eq!((p.vertices[0].p.x, p.vertices[0].p.y), (0.0, 0.0));
        assert_eq!((p.vertices[1].p.x, p.vertices[1].p.y), (10.0, 0.0));
    }
}

/// O donut entra na ponta da identificação (o ponto inicial da barra):
/// no eixo horizontal o extremo de menor `x`, no vertical o de cima,
/// independente do sentido em que a barra foi desenhada.
#[test]
fn donut_entra_na_ponta_do_texto_da_barra() {
    use super::esquematico::ponta_do_texto;
    assert_eq!(ponta_do_texto((0.0, 0.0), (10.0, 0.0)), (0.0, 0.0));
    assert_eq!(ponta_do_texto((10.0, 0.0), (0.0, 0.0)), (0.0, 0.0), "o sentido do traco nao muda a ponta");
    assert_eq!(ponta_do_texto((0.0, 0.0), (0.0, 10.0)), (0.0, 10.0), "na vertical o donut fica no topo, junto do texto");
    assert_eq!(ponta_do_texto((2.0, 3.0), (2.0, 0.0)), (2.0, 3.0));
}

/// Migração do donut 8/4 antigo: CIRCLE com círculo interno no campo
/// `donut` do XData vira o INSERT do bloco e o interno sai junto.
#[test]
fn donut_antigo_migra_para_o_bloco() {
    use super::esquematico::{BLOCO_DONUT, trocar_simbolo_barra};
    let mut s = Session::new();
    let handle = barra(&mut s, 10001);
    s.doc_mut()
        .unwrap()
        .modify_entity(handle, |e| {
            e.kind = EntityKind::Circle(Circle { center: Vec3::new(5.0, 0.0, 0.0), radius: 8.0 });
        })
        .unwrap();
    let interno = s.add_entity(EntityKind::Circle(Circle { center: Vec3::new(5.0, 0.0, 0.0), radius: 4.0 })).unwrap();
    marcar(
        &mut s,
        handle,
        APP_BARRA,
        &[("id", "1"), ("codigo", "10001"), ("x1", "0"), ("y1", "0"), ("x2", "10"), ("y2", "0"), ("donut", &interno.hex())],
    )
    .unwrap();
    trocar_simbolo_barra(&mut s, handle, true, (5.0, 0.0), (0.0, 0.0), (10.0, 0.0), 0.3).unwrap();
    let doc = s.doc().unwrap();
    assert!(matches!(doc.entity(handle).unwrap().kind, EntityKind::Insert(ref ins) if ins.block == BLOCO_DONUT), "o donut antigo vira o bloco");
    assert!(doc.entity(interno).is_none(), "o circulo interno antigo sai junto");
}

#[test]
fn troca_de_desenho_nao_e_cascata() {
    let mut s = Session::new();
    let apagada = barra(&mut s, 10001);
    let presa = linha(&mut s, 10001, 0);
    let antes = s.doc().unwrap().clone();
    s.doc_mut().unwrap().remove_entity(apagada);
    let uid = s.state().unwrap().uid;
    assert_eq!(cascata_barras_apagadas(&mut s, Some(&antes), Some(uid + 1)), 0);
    assert!(s.doc().unwrap().entity(presa).is_some());
}

#[test]
fn apagar_linha_leva_o_texto_do_circuito_na_hora() {
    let mut s = Session::new();
    let de = barra(&mut s, 10001);
    barra(&mut s, 10002);
    let apagada = linha(&mut s, 10001, 10002);
    let antes = s.doc().unwrap().clone();
    s.doc_mut().unwrap().remove_entity(apagada);
    // O `ERASE` com seleção chega sem `uid`.
    assert_eq!(cascata_linhas_apagadas(&mut s, Some(&antes), None), 1, "o texto do circuito");
    let doc = s.doc().unwrap();
    assert!(textos_ligados(doc, APP_TEXTO_LINHA, apagada).is_empty(), "nao sobra texto de circuito sem a linha");
    assert!(doc.entity(de).is_some(), "as barras das pontas ficam");
}

/// Apagar o equipamento no desenho leva o circuito que ele criou no banco
/// na hora, sem esperar o Sincronizar — como o apagar da barra leva as
/// linhas dela.
#[test]
fn apagar_equipamento_leva_o_circuito_no_banco() {
    let caminho = std::env::temp_dir().join(format!("cadcraft-anacad-eq-{}-{}.db", std::process::id(), line!()));
    let _ = std::fs::remove_file(&caminho);
    let mut s = Session::new();
    let equipamento = traco(&mut s, (0.0, 0.0), (1.0, 0.0));
    let hex = equipamento.hex();
    let (id, doc) = {
        let banco = anacad_data::Banco::abrir(&caminho).unwrap();
        let doc = banco.garantir_documento(1).unwrap();
        let id = banco.criar_equipamento(doc, "Transformador", 1, "Transformador", 0.0, 0.0).unwrap();
        banco.definir_handle("Equipamento", id, &hex).unwrap();
        let dados = json!({"de": 10001, "para": 10002, "X1": 5.0});
        banco.atualizar_dados_equipamento(&hex, &dados.to_string()).unwrap();
        banco.sincronizar_equipamento(doc, "Transformador", &dados).unwrap();
        (id, doc)
    };
    marcar(
        &mut s,
        equipamento,
        "ANACAD_EQ",
        &[("id", &id.to_string()), ("tipo", "Transformador"), ("codigo", "1"), ("dados", "{\"de\":10001,\"para\":10002}")],
    )
    .unwrap();
    set_banco(&mut s, caminho.to_string_lossy().as_ref(), doc);
    // O caminho do usuário: o `ERASE` do equipamento no desenho. A cascata
    // roda no `after_command` do módulo, não é chamada à mão.
    s.execute("erase", &json!({"handles": [hex]})).unwrap();
    assert!(s.doc().unwrap().entity(equipamento).is_none(), "o equipamento saiu do desenho");

    let banco = anacad_data::Banco::abrir(&caminho).unwrap();
    assert!(banco.circuitos(doc, 100).unwrap().is_empty(), "o circuito do trafo ficou no banco");
    assert!(banco.equipamentos(doc, 100).unwrap().is_empty(), "o equipamento ficou no banco");
    drop(banco);
    let _ = std::fs::remove_file(&caminho);
}

/// Apagar o equipamento no desenho leva a conexão (o circuito que liga o equipamento
/// à barra ou linha) no desenho e no banco.
#[test]
fn apagar_equipamento_leva_a_conexao_do_desenho_e_do_banco() {
    let caminho = std::env::temp_dir().join(format!("cadcraft-anacad-eq-conn-{}-{}.db", std::process::id(), line!()));
    let _ = std::fs::remove_file(&caminho);
    let mut s = Session::new();
    let b = barra(&mut s, 10001);
    let equipamento = traco(&mut s, (0.0, 5.0), (1.0, 5.0));
    let eq_hex = equipamento.hex();
    let b_hex = b.hex();
    let (id, doc) = {
        let banco = anacad_data::Banco::abrir(&caminho).unwrap();
        let doc = banco.garantir_documento(1).unwrap();
        let id = banco.criar_equipamento(doc, "Carga", 1, "Carga", 0.0, 5.0).unwrap();
        banco.definir_handle("Equipamento", id, &eq_hex).unwrap();
        (id, doc)
    };
    marcar(&mut s, equipamento, "ANACAD_EQ", &[("id", &id.to_string()), ("tipo", "Carga"), ("codigo", "1"), ("dados", "{\"barra\":10001}")]).unwrap();

    // Traço da conexão que liga o equipamento à barra:
    let conexao = traco(&mut s, (0.0, 0.0), (0.0, 5.0));
    let conn_hex = conexao.hex();
    marcar(&mut s, conexao, APP_CONEXAO, &[("de", &b_hex), ("para", &eq_hex), ("de_terminal", "INICIO"), ("para_terminal", "T1")]).unwrap();
    {
        let banco = anacad_data::Banco::abrir(&caminho).unwrap();
        banco
            .inserir_conexao(&anacad_data::ConexaoNova {
                documento: Some(doc),
                de_tipo: "Barra".to_string(),
                de_handle: b_hex.clone(),
                de_terminal: "INICIO".to_string(),
                para_tipo: "Equipamento".to_string(),
                para_handle: eq_hex.clone(),
                para_terminal: "T1".to_string(),
                potencial: None,
                handle: Some(conn_hex.clone()),
            })
            .unwrap();
    }
    set_banco(&mut s, caminho.to_string_lossy().as_ref(), doc);

    // Apagar o equipamento no desenho:
    s.execute("erase", &json!({"handles": [eq_hex]})).unwrap();
    assert!(s.doc().unwrap().entity(equipamento).is_none(), "o equipamento saiu");
    assert!(s.doc().unwrap().entity(conexao).is_none(), "a conexao para a barra saiu junto");
    assert!(s.doc().unwrap().entity(b).is_some(), "a barra continua");

    let banco = anacad_data::Banco::abrir(&caminho).unwrap();
    assert!(banco.conexoes(doc).unwrap().is_empty(), "a conexao saiu do banco");
    drop(banco);
    let _ = std::fs::remove_file(&caminho);
}

/// Cascata inversa: apagar o circuito de ligação (a conexão) apaga o
/// equipamento ligado a ele, no desenho e no banco — o espelho de
/// `apagar_equipamento_leva_a_conexao_do_desenho_e_do_banco`.
#[test]
fn apagar_conexao_leva_o_equipamento_do_desenho_e_do_banco() {
    let caminho = std::env::temp_dir().join(format!("cadcraft-anacad-conn-eq-{}-{}.db", std::process::id(), line!()));
    let _ = std::fs::remove_file(&caminho);
    let mut s = Session::new();
    let b = barra(&mut s, 10001);
    let equipamento = traco(&mut s, (0.0, 5.0), (1.0, 5.0));
    let eq_hex = equipamento.hex();
    let b_hex = b.hex();
    let (id, doc) = {
        let banco = anacad_data::Banco::abrir(&caminho).unwrap();
        let doc = banco.garantir_documento(1).unwrap();
        let id = banco.criar_equipamento(doc, "Carga", 1, "Carga", 0.0, 5.0).unwrap();
        banco.definir_handle("Equipamento", id, &eq_hex).unwrap();
        (id, doc)
    };
    marcar(&mut s, equipamento, "ANACAD_EQ", &[("id", &id.to_string()), ("tipo", "Carga"), ("codigo", "1"), ("dados", "{\"barra\":10001}")]).unwrap();
    let conexao = traco(&mut s, (0.0, 0.0), (0.0, 5.0));
    let conn_hex = conexao.hex();
    marcar(&mut s, conexao, APP_CONEXAO, &[("de", &b_hex), ("para", &eq_hex), ("de_terminal", "INICIO"), ("para_terminal", "T1")]).unwrap();
    {
        let banco = anacad_data::Banco::abrir(&caminho).unwrap();
        banco
            .inserir_conexao(&anacad_data::ConexaoNova {
                documento: Some(doc),
                de_tipo: "Barra".to_string(),
                de_handle: b_hex.clone(),
                de_terminal: "INICIO".to_string(),
                para_tipo: "Equipamento".to_string(),
                para_handle: eq_hex.clone(),
                para_terminal: "T1".to_string(),
                potencial: None,
                handle: Some(conn_hex.clone()),
            })
            .unwrap();
    }
    set_banco(&mut s, caminho.to_string_lossy().as_ref(), doc);

    // Apagar o circuito de ligação no desenho:
    s.execute("erase", &json!({"handles": [conn_hex]})).unwrap();
    assert!(s.doc().unwrap().entity(conexao).is_none(), "a conexao saiu");
    assert!(s.doc().unwrap().entity(equipamento).is_none(), "o equipamento saiu junto");
    assert!(s.doc().unwrap().entity(b).is_some(), "a barra continua");

    let banco = anacad_data::Banco::abrir(&caminho).unwrap();
    assert!(banco.conexoes(doc).unwrap().is_empty(), "a conexao saiu do banco");
    assert!(banco.equipamentos(doc, 100).unwrap().is_empty(), "o equipamento saiu do banco");
    drop(banco);
    let _ = std::fs::remove_file(&caminho);
}

/// Apagar a barra que tem um circuito (conexão) com o equipamento apaga
/// o equipamento junto: a conexão sai na cascata da barra (depois da
/// inversa), e o equipamento ligado a ela sai junto.
#[test]
fn apagar_barra_com_conexao_apaga_o_equipamento() {
    let mut s = Session::new();
    let b = barra(&mut s, 10001);
    let b_hex = b.hex();
    let equipamento = traco(&mut s, (0.0, 5.0), (1.0, 5.0));
    let eq_hex = equipamento.hex();
    marcar(&mut s, equipamento, "ANACAD_EQ", &[("id", "1"), ("tipo", "Carga"), ("codigo", "1"), ("dados", "{\"barra\":10001}")]).unwrap();
    let conexao = traco(&mut s, (0.0, 0.0), (0.0, 5.0));
    marcar(&mut s, conexao, APP_CONEXAO, &[("de", &b_hex), ("para", &eq_hex), ("de_terminal", "INICIO"), ("para_terminal", "T1")]).unwrap();
    s.execute("erase", &json!({"handles": [b_hex]})).unwrap();
    assert!(s.doc().unwrap().entity(conexao).is_none(), "a conexao saiu com a barra");
    assert!(s.doc().unwrap().entity(equipamento).is_none(), "o equipamento ligado a ela saiu junto");
}

/// Apagar a linha que tem um circuito (conexão) com o equipamento apaga
/// o equipamento junto; as barras das pontas ficam.
#[test]
fn apagar_linha_com_conexao_apaga_o_equipamento() {
    let mut s = Session::new();
    barra(&mut s, 10001);
    barra(&mut s, 10002);
    let b1 = barra(&mut s, 10003);
    let linha_apagada = linha(&mut s, 10001, 10002);
    let linha_hex = linha_apagada.hex();
    let equipamento = traco(&mut s, (0.0, 5.0), (1.0, 5.0));
    let eq_hex = equipamento.hex();
    marcar(&mut s, equipamento, "ANACAD_EQ", &[("id", "1"), ("tipo", "Carga"), ("codigo", "1"), ("dados", "{\"de\":10001,\"para\":10002}")]).unwrap();
    let conexao = traco(&mut s, (0.0, 0.0), (0.0, 5.0));
    marcar(&mut s, conexao, APP_CONEXAO, &[("de", &linha_hex), ("para", &eq_hex), ("de_terminal", "INICIO"), ("para_terminal", "T1")]).unwrap();
    s.execute("erase", &json!({"handles": [linha_hex]})).unwrap();
    assert!(s.doc().unwrap().entity(conexao).is_none(), "a conexao saiu com a linha");
    assert!(s.doc().unwrap().entity(equipamento).is_none(), "o equipamento ligado a ela saiu junto");
    assert!(s.doc().unwrap().entity(b1).is_some(), "a barra fica");
}

/// Sessão com banco conectado e o tipo de equipamento escolhido, como a
/// UI deixa ao clicar no ícone do equipamento.
fn sessao_com_equipamento(nome: &str) -> (Session, std::path::PathBuf, i64) {
    let caminho = std::env::temp_dir().join(format!("cadcraft-anacad-{nome}-{}-{}.db", std::process::id(), line!()));
    let _ = std::fs::remove_file(&caminho);
    let mut s = Session::new();
    let doc = {
        let banco = anacad_data::Banco::abrir(&caminho).unwrap();
        banco.garantir_documento(1).unwrap()
    };
    set_banco(&mut s, caminho.to_string_lossy().as_ref(), doc);
    crate::cmd::anacad::st(&mut s, |state| state.equip_tipo = "Carga".to_string());
    (s, caminho, doc)
}

fn equipamentos_no_desenho(s: &Session) -> usize {
    s.doc().unwrap().model.iter().filter(|e| app_xdata(e) == Some("ANACAD_EQ")).count()
}

/// Cancelar a inserção do equipamento com ESC depois do primeiro clique
/// (sem a ligação) não deixa o bloco no desenho nem o registro no banco:
/// a inserção só acontece no segundo clique, com a ligação.
#[test]
fn esc_antes_da_ligacao_nao_insere_o_equipamento() {
    let (mut s, caminho, doc) = sessao_com_equipamento("eq-esc");
    barra(&mut s, 10001);
    s.start("anacad.equipamento.inserir").unwrap();
    s.input(crate::Input::Enter).unwrap();
    // Primeiro clique longe da barra: não acha alvo, espera a ligação.
    s.input(crate::Input::Point(Vec2::new(100.0, 100.0))).unwrap();
    assert_eq!(equipamentos_no_desenho(&s), 0, "o bloco entrou no desenho antes da ligacao");
    // ESC cancela o comando.
    s.cancel();
    assert_eq!(equipamentos_no_desenho(&s), 0, "o ESC deixou o bloco no desenho");
    let banco = anacad_data::Banco::abrir(&caminho).unwrap();
    assert!(banco.equipamentos(doc, 100).unwrap().is_empty(), "o ESC deixou o registro no banco");
    drop(banco);
    let _ = std::fs::remove_file(&caminho);
}

/// O caminho normal continua: primeiro clique longe, segundo na barra —
/// aí o bloco entra, com a conexão.
#[test]
fn segundo_clique_na_barra_insere_e_liga() {
    let (mut s, caminho, doc) = sessao_com_equipamento("eq-dois-cliques");
    barra(&mut s, 10001);
    s.start("anacad.equipamento.inserir").unwrap();
    s.input(crate::Input::Enter).unwrap();
    s.input(crate::Input::Point(Vec2::new(100.0, 100.0))).unwrap();
    assert_eq!(equipamentos_no_desenho(&s), 0);
    s.input(crate::Input::Point(Vec2::new(5.0, 0.0))).unwrap();
    assert_eq!(equipamentos_no_desenho(&s), 1, "o bloco nao entrou no segundo clique");
    let banco = anacad_data::Banco::abrir(&caminho).unwrap();
    assert_eq!(banco.equipamentos(doc, 100).unwrap().len(), 1, "o registro nao entrou no banco");
    assert_eq!(banco.conexoes(doc).unwrap().len(), 1, "a conexao nao foi criada");
    drop(banco);
    let _ = std::fs::remove_file(&caminho);
}

/// Primeiro clique já sobre a barra: insere e liga na hora, como antes.
#[test]
fn primeiro_clique_na_barra_insere_na_hora() {
    let (mut s, caminho, doc) = sessao_com_equipamento("eq-um-clique");
    barra(&mut s, 10001);
    s.start("anacad.equipamento.inserir").unwrap();
    s.input(crate::Input::Enter).unwrap();
    s.input(crate::Input::Point(Vec2::new(5.0, 0.0))).unwrap();
    assert_eq!(equipamentos_no_desenho(&s), 1, "o bloco nao entrou no primeiro clique");
    let banco = anacad_data::Banco::abrir(&caminho).unwrap();
    assert_eq!(banco.conexoes(doc).unwrap().len(), 1, "a conexao nao foi criada");
    drop(banco);
    let _ = std::fs::remove_file(&caminho);
}

#[test]
fn inserir_equipamento_com_angulo_personalizado() {
    let (mut s, caminho, _doc) = sessao_com_equipamento("eq-ang");
    barra(&mut s, 10001);
    s.start("anacad.equipamento.inserir").unwrap();
    let p = s.current_prompt().unwrap();
    assert_eq!(p.message, "Ângulo de rotação do bloco");
    assert_eq!(p.default.as_deref(), Some("0"));
    s.input(crate::Input::Text("90".into())).unwrap();
    assert_eq!(s.current_prompt().unwrap().message, "Ponto de inserção do equipamento");
    s.input(crate::Input::Point(Vec2::new(5.0, 0.0))).unwrap();
    assert_eq!(equipamentos_no_desenho(&s), 1);
    let bloco_inserido = s
        .doc()
        .unwrap()
        .model
        .iter()
        .find_map(|e| if let cadcraft_doc::EntityKind::Insert(ins) = &e.kind { Some(ins.rotation) } else { None })
        .unwrap();
    assert!((bloco_inserido - 90.0_f64.to_radians()).abs() < 1e-6);
    let banco = anacad_data::Banco::abrir(&caminho).unwrap();
    drop(banco);
    let _ = std::fs::remove_file(&caminho);
}

/// O banco de um desenho é o que o XData dele aponta: é esse o verificador
/// que a UI consulta ao trocar de aba (o `DirProjeto` do Eletron, relido do
/// DWG em `LerXDataConfiguracao2`). Sem ele, as ferramentas não liberam.
#[test]
fn banco_do_desenho_vem_do_xdata_da_configuracao() {
    use crate::cmd::anacad::{banco_do_desenho, esquematico};
    let mut s = Session::new();
    assert!(banco_do_desenho(&s).is_empty(), "desenho novo nao tem banco");
    esquematico::gravar_cfg(&mut s, &esquematico::CfgProjeto { banco: "C:/proj/projeto.db".into(), ..Default::default() }).unwrap();
    assert_eq!(banco_do_desenho(&s), "C:/proj/projeto.db");
    // Trocar o banco no desenho troca o que o verificador devolve.
    esquematico::gravar_cfg(&mut s, &esquematico::CfgProjeto { banco: "D:/outro.db".into(), ..Default::default() }).unwrap();
    assert_eq!(banco_do_desenho(&s), "D:/outro.db");
}
