//! Configuração do projeto guardada no desenho (`ANACAD_CFG`).

use super::*;

// ---------- configuração do projeto ----------
// Mesmo modelo do Eletron: a configuração do projeto anda com o desenho. Lá o
// `DirProjeto` (o `.db`) e o nome do caderno (`Conf.Local`) ficam no XData do
// DWG (`GravarXDataConfiguracao2`) e o add-in relê na abertura
// (`LerXDataConfiguracao2`), então o usuário não escolhe o banco a cada
// sessão. O cadcraft guarda XData por entidade, então a configuração mora num
// POINT oculto na origem (`ANACAD_CFG`), que viaja no DXF como o resto.

/// App name do XData que guarda a configuração do projeto no desenho.
pub(super) const APP_CFG: &str = "ANACAD_CFG";

/// Configuração do projeto: o banco usado, o nome do caderno e o estilo de
/// desenho das barras e das linhas.
///
/// O estilo é o que se precisa para desenhar a identificação do diagrama —
/// tamanho do texto, cor do texto e espessura do traço —, um conjunto para a
/// barra e outro para a linha. Tamanho e espessura são em unidades do desenho;
/// a cor pode ser um índice ACI ou um true color RGB, como no `CColorDialog`
/// do ANAFAS.
///
/// `grupos` são os **grupos base de tensão** do projeto (o `G` do `.lst`:
/// `ten, lar, R, G, B`): tensão em kV e cor — a espessura `lar` não é editada
/// aqui, porque o traço da barra e o da linha têm uma espessura só, no estilo
/// do desenho. No ANAFAS a cor de cada elemento vem do grupo da tensão base
/// dele — 500 kV vermelho `255,0,0`, 345 kV azul `0,128,255`, 138 kV e 13,8 kV
/// laranja `255,165,0` no desenho de exemplo —, não do enrolamento do trafo nem
/// do tipo do elemento. Aqui os grupos são editados no CFG, com os valores do
/// exemplo como padrão.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::cmd::anacad) struct CfgProjeto {
    pub banco: String,
    pub caderno: String,
    pub documento: i64,
    /// Barra: altura e cor do texto de identificação e espessura do traço.
    pub barra_texto: f64,
    pub barra_cor: Color,
    pub barra_lar: f64,
    /// Linha: o mesmo para a identificação e o traço dela.
    pub linha_texto: f64,
    pub linha_cor: Color,
    pub linha_lar: f64,
    /// Escala com que os blocos dos equipamentos entram no desenho (fator de
    /// escala da referência INSERT, 1 = tamanho natural do bloco).
    pub equip_escala: f64,
    /// Altura do texto dos atributos do bloco do equipamento (CODIGO, NOME,
    /// TENSAO, T1, T2), em unidades do desenho — independente da escala do
    /// bloco, como o tamanho do texto da barra e o da linha.
    pub equip_texto: f64,
    pub grupos: Vec<GrupoCor>,
}

/// Grupo base de tensão: a tensão em kV e a cor com que os elementos daquele
/// nível entram no desenho. A espessura (`lar`, o deslizante Espessura da
/// janela 210) não é do grupo: o traço da barra e o da linha têm uma espessura
/// só, no grupo **Texto e Traço de Barras e Linhas**.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GrupoCor {
    pub tensao: f64,
    pub cor: Color,
}

/// Grupos base de tensão do desenho de exemplo do ANAFAS, os seis do cabeçalho
/// do `.lst` (`G ten lar R G B`): 500 kV vermelho, 345 kV azul, 138 kV,
/// 13,8 kV e 4,16 kV laranja e 0 kV preto. A lista não é fixa — cada projeto
/// tem os seus grupos, com tensão e cor livres (janela 210 — Dados de Grupo
/// Base de Tensão).
pub(crate) fn grupos_padrao() -> Vec<GrupoCor> {
    let laranja = Color::True(cadcraft_doc::color::Rgb(255, 165, 0));
    vec![
        GrupoCor { tensao: 500.0, cor: Color::True(cadcraft_doc::color::Rgb(255, 0, 0)) },
        GrupoCor { tensao: 345.0, cor: Color::True(cadcraft_doc::color::Rgb(0, 128, 255)) },
        GrupoCor { tensao: 138.0, cor: laranja },
        GrupoCor { tensao: 13.8, cor: laranja },
        GrupoCor { tensao: 4.16, cor: laranja },
        GrupoCor { tensao: 0.0, cor: Color::True(cadcraft_doc::color::Rgb(0, 0, 0)) },
    ]
}

impl Default for CfgProjeto {
    fn default() -> Self {
        Self {
            banco: String::new(),
            caderno: String::new(),
            documento: 1,
            // 2,5 de altura e 7 = branco/preto, a cor de quem não escolheu
            // outra; a barra nasce com 0,3 de espessura e a linha com 0 (fio).
            barra_texto: 2.5,
            barra_cor: Color::Index(7),
            barra_lar: 0.3,
            linha_texto: 2.5,
            linha_cor: Color::Index(7),
            linha_lar: 0.0,
            // Tamanho natural do bloco do equipamento, sem escala, e o texto
            // dos atributos com a altura da definição do bloco (2,0).
            equip_escala: 1.0,
            equip_texto: 2.0,
            // Os três RGBs do desenho de exemplo do ANAFAS (grupos base de
            // 500 kV, 13,8 kV e 345 kV).
            grupos: grupos_padrao(),
        }
    }
}

/// Entidades que carregam a configuração do projeto (normalmente uma só).
pub(super) fn entidades_cfg(doc: &cadcraft_doc::Drawing) -> Vec<Handle> {
    doc.model
        .iter()
        .filter_map(|e| match e.common.xdata.first() {
            Some(t) if t.code == 1001 && t.value == APP_CFG => Some(e.handle),
            _ => None,
        })
        .collect()
}

/// Cor lida do `ANACAD_CFG`: índice ACI ("3") ou true color ("255,128,0",
/// "#ff8000", "red"); o que não for cor volta ao branco.
pub(super) fn cor_do_texto(valor: &str) -> Color {
    match Color::parse(valor) {
        Some(cor @ (Color::Index(_) | Color::True(_))) => cor,
        _ => Color::Index(7),
    }
}

/// Texto gravado no `ANACAD_CFG` para a cor: o índice ACI ("3", grupo 62 do
/// DXF) ou o RGB ("255,128,0", grupo 420).
pub(super) fn texto_cor(cor: Color) -> String {
    match cor {
        Color::Index(i) => i.to_string(),
        Color::True(rgb) => format!("{},{},{}", rgb.0, rgb.1, rgb.2),
        _ => "7".to_string(),
    }
}

/// Cor vinda de um parâmetro JSON: número (ACI) ou texto em qualquer formato
/// que o desenho aceita.
pub(super) fn cor_do_valor(v: &Value) -> Option<Color> {
    if let Some(n) = v.as_u64() {
        return Some(cor_do_texto(&n.to_string()));
    }
    v.as_str().map(cor_do_texto)
}

/// Altura de texto lida do `ANACAD_CFG`, no intervalo aceito.
pub(super) fn altura_do_texto(valor: &str) -> f64 {
    valor.parse::<f64>().unwrap_or(2.5).clamp(0.1, 100.0)
}

/// Espessura de traço lida do `ANACAD_CFG`, no intervalo aceito.
pub(super) fn lar_do_texto(valor: &str) -> f64 {
    valor.parse::<f64>().unwrap_or(0.0).clamp(0.0, 10.0)
}

/// Escala do bloco do equipamento lida do `ANACAD_CFG`, no intervalo aceito.
pub(super) fn escala_do_texto(valor: &str) -> f64 {
    valor.parse::<f64>().unwrap_or(1.0).clamp(0.1, 10.0)
}

/// Configuração gravada no desenho; sem XData, os padrões.
pub(in crate::cmd::anacad) fn cfg_do_desenho(doc: &cadcraft_doc::Drawing) -> CfgProjeto {
    let mut cfg = CfgProjeto::default();
    let mut grupos_lidos = false;
    for handle in entidades_cfg(doc) {
        let Some(entity) = doc.entity(handle) else { continue };
        let pares: Vec<&str> = entity.common.xdata.iter().filter(|t| t.code == 1000).map(|t| t.value.as_str()).collect();
        for par in pares.chunks(2) {
            if par.len() != 2 {
                continue;
            }
            match par[0] {
                "banco" => cfg.banco = par[1].to_string(),
                "caderno" => cfg.caderno = par[1].to_string(),
                "documento" => cfg.documento = par[1].parse().unwrap_or(1).max(1),
                "barra_texto" => cfg.barra_texto = altura_do_texto(par[1]),
                "barra_cor" => cfg.barra_cor = cor_do_texto(par[1]),
                "barra_lar" => cfg.barra_lar = lar_do_texto(par[1]),
                "linha_texto" => cfg.linha_texto = altura_do_texto(par[1]),
                "linha_cor" => cfg.linha_cor = cor_do_texto(par[1]),
                "linha_lar" => cfg.linha_lar = lar_do_texto(par[1]),
                "equip_escala" => cfg.equip_escala = escala_do_texto(par[1]),
                "equip_texto" => cfg.equip_texto = altura_do_texto(par[1]),
                // Desenho da versão 4: as três cores da identificação valem
                // para o texto da barra e o da linha.
                "cor_identificador" | "cor_numero" | "cor_outros" => {
                    let cor = cor_do_texto(par[1]);
                    cfg.barra_cor = cor;
                    cfg.linha_cor = cor;
                }
                "grupo" => {
                    // Os pares `grupo` do desenho substituem a lista padrão.
                    if !grupos_lidos {
                        cfg.grupos.clear();
                        grupos_lidos = true;
                    }
                    if let Some(grupo) = grupo_do_texto(par[1]) {
                        cfg.grupos.push(grupo);
                    }
                }
                _ => {}
            }
        }
    }
    cfg
}

/// Grava a configuração no desenho, criando o marcador (POINT oculto na
/// origem) quando ele ainda não existe.
pub(in crate::cmd::anacad) fn gravar_cfg(s: &mut Session, cfg: &CfgProjeto) -> Result<()> {
    let handle = match entidades_cfg(s.doc()?).into_iter().next() {
        Some(handle) => handle,
        None => {
            let handle = s.add_entity(EntityKind::Point(Point { p: Vec3::ZERO, angle: 0.0 }))?;
            s.doc_mut()?.modify_entity(handle, |e| e.common.visible = false)?;
            handle
        }
    };
    let (barra_cor, linha_cor) = (texto_cor(cfg.barra_cor), texto_cor(cfg.linha_cor));
    let (barra_texto, barra_lar) = (cfg.barra_texto.to_string(), cfg.barra_lar.to_string());
    let (linha_texto, linha_lar) = (cfg.linha_texto.to_string(), cfg.linha_lar.to_string());
    let equip_escala = cfg.equip_escala.to_string();
    let equip_texto = cfg.equip_texto.to_string();
    let documento = cfg.documento.to_string();
    let grupos: Vec<String> = cfg.grupos.iter().map(texto_grupo).collect();
    let mut campos: Vec<(&str, &str)> = vec![
        ("versao", "6"),
        ("banco", &cfg.banco),
        ("caderno", &cfg.caderno),
        ("documento", &documento),
        ("barra_texto", &barra_texto),
        ("barra_cor", &barra_cor),
        ("barra_lar", &barra_lar),
        ("linha_texto", &linha_texto),
        ("linha_cor", &linha_cor),
        ("linha_lar", &linha_lar),
        ("equip_escala", &equip_escala),
        ("equip_texto", &equip_texto),
    ];
    campos.extend(grupos.iter().map(|grupo| ("grupo", grupo.as_str())));
    marcar(s, handle, APP_CFG, &campos)
}

/// `anacad.cfg.ler`: a configuração gravada no desenho, já aplicada à sessão
/// (banco, documento e cores), para o próximo comando usá-la.
pub(in crate::cmd::anacad) fn run_cfg_ler(s: &mut Session, _p: &Value) -> Result<Value> {
    let mut cfg = cfg_do_desenho(s.doc()?);
    // Sem grupos no desenho, valem os do projeto: o `Grupos` do documento, o
    // mesmo lugar em que o `G` do `.lst` fica no ANAFAS.
    if !tem_grupos_no_desenho(s.doc()?)
        && let Some(grupos) = grupos_do_documento(&cfg)
        && !grupos.is_empty()
    {
        cfg.grupos = grupos;
    }
    guardar_cfg_na_sessao(s, &cfg);
    Ok(resumo_cfg(&cfg))
}

/// `anacad.cfg.gravar`: grava no desenho os campos recebidos (os ausentes
/// ficam como estão) e recolore a identificação das barras já desenhada.
pub(in crate::cmd::anacad) fn run_cfg_gravar(s: &mut Session, p: &Value) -> Result<Value> {
    let mut cfg = cfg_do_desenho(s.doc()?);
    if let Some(v) = p.get("banco").and_then(Value::as_str) {
        cfg.banco = v.trim().to_string();
    }
    if let Some(v) = p.get("caderno").and_then(Value::as_str) {
        cfg.caderno = v.trim().to_string();
    }
    if let Some(v) = p.get("documento").and_then(Value::as_i64) {
        cfg.documento = v.max(1);
    }
    if let Some(v) = p.get("barra_texto").and_then(Value::as_f64) {
        cfg.barra_texto = v.clamp(0.1, 100.0);
    }
    if let Some(cor) = p.get("barra_cor").and_then(cor_do_valor) {
        cfg.barra_cor = cor;
    }
    if let Some(v) = p.get("barra_lar").and_then(Value::as_f64) {
        cfg.barra_lar = v.clamp(0.0, 10.0);
    }
    if let Some(v) = p.get("linha_texto").and_then(Value::as_f64) {
        cfg.linha_texto = v.clamp(0.1, 100.0);
    }
    if let Some(cor) = p.get("linha_cor").and_then(cor_do_valor) {
        cfg.linha_cor = cor;
    }
    if let Some(v) = p.get("linha_lar").and_then(Value::as_f64) {
        cfg.linha_lar = v.clamp(0.0, 10.0);
    }
    if let Some(v) = p.get("equip_escala").and_then(Value::as_f64) {
        cfg.equip_escala = v.clamp(0.1, 10.0);
    }
    if let Some(v) = p.get("equip_texto").and_then(Value::as_f64) {
        cfg.equip_texto = v.clamp(0.1, 100.0);
    }
    // Cores dos grupos base de tensão: repintam o que já está desenhado com a
    // cor do nível de tensão de cada elemento, como no ANAFAS.
    let mut grupos: Option<Vec<GrupoCor>> = None;
    if let Some(lista) = p.get("grupos").and_then(Value::as_array) {
        grupos = Some(
            lista
                .iter()
                .filter_map(|item| {
                    let tensao = item.get("tensao").and_then(Value::as_f64)?;
                    let cor = item.get("cor").and_then(cor_do_valor)?;
                    Some(GrupoCor { tensao, cor })
                })
                .collect(),
        );
    }
    // Sem `grupos` no pedido, o desenho herda os do projeto na primeira vez: o
    // `Grupos` do documento, como o `G` do `.lst`.
    if grupos.is_none()
        && !tem_grupos_no_desenho(s.doc()?)
        && let Some(do_banco) = grupos_do_documento(&cfg)
        && !do_banco.is_empty()
    {
        cfg.grupos = do_banco;
    }
    let mudou_grupos = grupos.is_some();
    if let Some(lista) = grupos {
        cfg.grupos = lista;
    }
    gravar_cfg(s, &cfg)?;
    guardar_cfg_na_sessao(s, &cfg);
    if mudou_grupos {
        gravar_grupos_no_documento(&cfg);
        repintar_por_tensao(s, &cfg)?;
    }
    // O estilo vale também para o que já está desenhado, sem esperar a
    // próxima edição da barra ou da linha.
    aplicar_estilo(s, &cfg)?;
    Ok(resumo_cfg(&cfg))
}

/// Aplica ao desenho o estilo da configuração: altura e cor dos TEXT de
/// identificação da barra e espessura do traço das barras e das linhas (com o
/// campo `largura` do XData acompanhando a espessura).
pub(super) fn aplicar_estilo(s: &mut Session, cfg: &CfgProjeto) -> Result<()> {
    let alvos: Vec<(Handle, &'static str)> = s
        .doc()?
        .model
        .iter()
        .filter_map(|e| {
            let app = match e.common.xdata.first() {
                Some(t) if t.code == 1001 => t.value.as_str(),
                _ => return None,
            };
            match app {
                APP_TEXTO_BARRA => Some((e.handle, APP_TEXTO_BARRA)),
                APP_TEXTO_LINHA => Some((e.handle, APP_TEXTO_LINHA)),
                APP_BARRA => Some((e.handle, APP_BARRA)),
                APP_LINHA => Some((e.handle, APP_LINHA)),
                _ => None,
            }
        })
        .collect();
    // Posição nova de cada texto de circuito, achada no dono dele (o `handle`
    // do XData é o da linha): sem isso um desenho antigo continuaria com o
    // número afastado do traço, só com a altura nova.
    let mut posicoes: Vec<(Handle, Vec3)> = Vec::new();
    for e in s.doc()?.model.iter() {
        if app_xdata(e) != Some(APP_TEXTO_LINHA) {
            continue;
        }
        let Some(dono) = campo_xdata(e, "handle").and_then(Handle::parse_hex) else { continue };
        let Some((a, b)) = s.doc()?.entity(dono).and_then(|linha| extremos_da_linha(linha)) else { continue };
        posicoes.push((e.handle, posicao_texto_linha(a, b, cfg.linha_texto)));
    }
    for (handle, app) in alvos {
        let (texto, cor, lar) = match app {
            APP_LINHA | APP_TEXTO_LINHA => (cfg.linha_texto, cfg.linha_cor, cfg.linha_lar),
            _ => (cfg.barra_texto, cfg.barra_cor, cfg.barra_lar),
        };
        let mut largura = None;
        let posicao = posicoes.iter().find(|(dono, _)| *dono == handle).map(|(_, pos)| *pos);
        s.doc_mut()?.modify_entity(handle, |e| {
            match &mut e.kind {
                EntityKind::Text(t) => {
                    t.height = texto;
                    // O texto do circuito é reassentado na altura nova: no meio
                    // da linha, centrado e por cima do traço.
                    if let Some(pos) = posicao {
                        t.insert = pos;
                        t.align_pt = Some(pos);
                        t.halign = HAlign::Center;
                        t.valign = VAlign::Bottom;
                    }
                    e.common.color = cor;
                }
                EntityKind::LwPolyline(p) => {
                    p.const_width = lar;
                    largura = Some(lar.to_string());
                }
                _ => {}
            }
            if let Some(valor) = &largura {
                let tags = &mut e.common.xdata;
                for i in 0..tags.len().saturating_sub(1) {
                    if tags[i].code == 1000 && tags[i].value == "largura" && tags[i + 1].code == 1000 {
                        tags[i + 1].value = valor.clone();
                        break;
                    }
                }
            }
        })?;
    }
    Ok(())
}

pub(super) fn guardar_cfg_na_sessao(s: &mut Session, cfg: &CfgProjeto) {
    st(s, |state| {
        state.db = cfg.banco.clone();
        state.documento = cfg.documento;
        state.grupos = cfg.grupos.clone();
    });
}

pub(super) fn resumo_cfg(cfg: &CfgProjeto) -> Value {
    json!({
        "banco": cfg.banco,
        "caderno": cfg.caderno,
        "documento": cfg.documento,
        "barra_texto": cfg.barra_texto,
        "barra_cor": texto_cor(cfg.barra_cor),
        "barra_lar": cfg.barra_lar,
        "linha_texto": cfg.linha_texto,
        "linha_cor": texto_cor(cfg.linha_cor),
        "linha_lar": cfg.linha_lar,
        "equip_escala": cfg.equip_escala,
        "equip_texto": cfg.equip_texto,
        "grupos": cfg.grupos.iter().map(|grupo| json!({"tensao": grupo.tensao, "cor": texto_cor(grupo.cor)})).collect::<Vec<_>>(),
    })
}
