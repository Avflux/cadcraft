//! Tabela de comandos `anacad.*` (`CommandSpec`) registrada pelo domínio.

use super::*;

pub(super) fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("anacad", "AnaCAD (diagnostico/importar)", run_anacad)
            .menu(&[])
            .params("{file?: .lst para diagnostico, test_block?: bool, import?: .lst para importar, db?: .db, nome?: nome do documento} -> {...}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.consultar", "AnaCAD: Consultar Banco", run_consultar)
            .menu(&[])
            .params("{db, documento?: id} -> {documentos, documento?, equipamentos, componentes, linhas, ligacoes, contagens}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.mover", "AnaCAD: Mover Componente", run_mover)
            .menu(&[])
            .params("{db, handle, dx?, dy?} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.apagar", "AnaCAD: Apagar Componente/Ligacao", run_apagar)
            .menu(&[])
            .params("{db, handle} -> {removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.comprimento", "AnaCAD: Calcular Comprimentos", run_comprimento)
            .menu(&[])
            .params("{db, documento?} -> {resultado, ligacoes, comprimento_ligacoes, comprimento_linhas, comprimento_total}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.verificar", "AnaCAD: Verificar .LST", run_verificar)
            .menu(&["AnaCAD", "Arquivo", "Verificar .LST"])
            .params("{arquivo, db?} -> {igual, barras, componentes, linhas, ligacoes, bytes_exportados}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana.caso", "AnaCAD: Abrir .ANA como caso", run_ana_caso)
            .menu(&["AnaCAD", "Arquivo", "Abrir .ANA como caso"])
            .params("{arquivo, db, nome?} -> {db, documento, importacao}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana.importar", "AnaCAD: Importar .ANA (DBAR)", run_ana_importar)
            .menu(&[])
            .params("{arquivo, db, documento?} -> {barras, atualizadas, sem_barra, sem_vbase}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana", "AnaCAD: Diagnostico .ANA", run_ana)
            .menu(&["AnaCAD", "Arquivo", "Diagnóstico .ANA"])
            .params("{arquivo} -> {tipo, blocos: [{nome, tipo, linhas, cabecalho?}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.curto", "AnaCAD: Curto-Circuito", run_curto)
            .menu(&[])
            .params("{db, documento?, barra (codigo), referencia (codigo)?, sbase?} -> {resultado, barra, referencia, zth, i_pu, vbase, i_ka}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.referencia", "AnaCAD: Comparar com ANAFAS", run_referencia)
            .menu(&["AnaCAD", "Dados", "Comparar com ANAFAS"])
            .params("{arquivo (.OUT), db?, documento?, barra?} -> {referencias, comparacoes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.relatorio", "AnaCAD: Relatorio", run_relatorio)
            .menu(&[])
            .params("{db, documento?, arquivo?} -> {arquivo, bytes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.resultados", "AnaCAD: Resultados", run_resultados)
            .menu(&[])
            .params("{db, documento?, limite?} -> {resultados: [{id, tipo, saida, criado_em}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ybus", "AnaCAD: Montar Ybus", run_ybus)
            .menu(&[])
            .params("{db, documento?} -> {resultado, barras, circuitos_usados, circuitos_sem_barra, pares_nao_nulos, simetrica, maior_diagonal}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.calcular", "AnaCAD: Calcular Topologia", run_calcular)
            .menu(&[])
            .params("{db, documento?} -> {resultado, barras, linhas, ilhas, barras_isoladas, grau_max, grau_medio}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.validar", "AnaCAD: Validar Handles", run_validar)
            .menu(&["AnaCAD", "Dados", "Validar Handles"])
            .params("{db} -> {componentes, componentes_ok, componentes_sem_entidade, ligacoes, ligacoes_ok, ligacoes_sem_entidade}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.criar", "AnaCAD: Criar Legenda", run_criar_legenda)
            .menu(&[])
            .params("{db, documento?, tipo?: X|Y, numero, x, y, canal?} -> {legenda, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.mover", "AnaCAD: Mover Legenda", run_mover_legenda)
            .menu(&[])
            .params("{db, handle, dx?, dy?} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.criar", "AnaCAD: Criar Barra", run_criar_barra)
            .menu(&[])
            .params("{db, documento?, codigo, x?, y?, x2?, y2?, comprimento?, nome?} -> {barra, handle, x1, y1, x2, y2, largura}")
            .interactive(|_| Ok(Box::new(BarraM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.criar", "AnaCAD: Criar Linha", run_criar_linha)
            .menu(&[])
            .params("{db, documento?, de (codigo da barra), para (codigo da barra)} -> {linha, handle, x1, y1, x2, y2, largura}")
            .interactive(|_| Ok(Box::new(LinhaM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.atribuir", "AnaCAD: Atribuir Barra à Polilinha", run_atribuir_barra)
            .menu(&[])
            .params("{db, documento?, handle, codigo, nome?} -> {barra, handle, x1, y1, x2, y2, largura}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.atribuir", "AnaCAD: Atribuir Linha à Polilinha", run_atribuir_linha)
            .menu(&[])
            .params("{db, documento?, handle Polilinha, de, para} -> {linha, handle, x1, y1, x2, y2, largura}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.editar", "AnaCAD: Editar Barra (janela)", run_editar_barra)
            .menu(&[])
            .params("{db, documento?, handle, codigo?, nome?, dados} -> {barra, handle}")
            .interactive(|_| Ok(Box::new(EditorM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.textos", "AnaCAD: Ligar/Desligar Textos de Barras e Linhas", run_textos_barra)
            .menu(&[])
            .params("{handle?, visivel?} -> {visivel, textos, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.editar", "AnaCAD: Editar Linha (janela)", run_editar_linha)
            .menu(&[])
            .params("{db, documento?, handle, de?, para?, dados} -> {linha, handle}")
            .interactive(|_| Ok(Box::new(EditorM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.dados.excluir", "AnaCAD: Excluir Dados Elétricos da Barra", run_barra_dados_excluir)
            .menu(&[])
            .params("{db, documento?, handle, codigo?} -> {barra, handle, removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.dados.excluir", "AnaCAD: Excluir Dados Elétricos da Linha", run_linha_dados_excluir)
            .menu(&[])
            .params("{db, documento?, handle} -> {linha, handle, removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.banco.criar", "AnaCAD: Novo Banco", run_banco_criar)
            .menu(&[])
            .params("{arquivo} -> {arquivo, nome, versao, documentos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.banco.abrir", "AnaCAD: Abrir Banco", run_banco_abrir)
            .menu(&[])
            .params("{arquivo} -> {arquivo, nome, versao, documentos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.cfg.ler", "AnaCAD: Ler Configuração do Projeto", run_cfg_ler)
            .menu(&[])
            .params("{} -> {banco, caderno, documento, barra_texto, barra_cor, barra_lar, linha_texto, linha_cor, linha_lar, grupos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.cfg.gravar", "AnaCAD: Gravar Configuração do Projeto", run_cfg_gravar)
            .menu(&[])
            .params("{banco?, caderno?, documento?, barra_texto?, barra_cor?, barra_lar?, linha_texto?, linha_cor?, linha_lar?, grupos?} -> {banco, caderno, documento, barra_texto, barra_cor, barra_lar, linha_texto, linha_cor, linha_lar, grupos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.sincronizar", "AnaCAD: Sincronizar XData com o Banco", run_sincronizar)
            .menu(&[])
            .params("{db, documento?, remover?} -> {documento, removidos, total, desenho, sem_banco}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.dados.tabelas", "AnaCAD: Tabelas do Banco", run_dados_tabelas)
            .menu(&[])
            .params("{db, documento?} -> {itens: [{item, total}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.dados.tabela", "AnaCAD: Ler Item do Gerenciador", run_dados_tabela)
            .menu(&[])
            .params("{db, documento?, item, limite?} -> {item, colunas, linhas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.documento.listar", "AnaCAD: Listar Documentos", run_documento_listar)
            .menu(&[])
            .params("{db} -> {documentos: [{id, nome, pagina}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.documento.criar", "AnaCAD: Novo Documento", run_documento_criar)
            .menu(&[])
            .params("{db, nome, projeto?} -> {documento, projeto, nome, documentos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.inserir", "AnaCAD: Inserir Equipamento", run_equipamento_inserir)
            .menu(&[])
            .params("{db, documento, tipo, x, y, codigo?} -> {equipamento, handle, alvo?, tipo_alvo?, vinculo?, conexao?}")
            .enabled(equipamento_escolhido)
            .noundo()
            .interactive(|s| {
                let tipo = st_get(s, |state| state.equip_tipo.clone()).unwrap_or_default();
                if tipo.is_empty() {
                    return Err(crate::EngineError::Other("escolha um equipamento".into()));
                }
                Ok(Box::new(EquipM::novo(tipo)))
            }),
        CommandSpec::new("anacad.equipamento.selecionar", "AnaCAD: Selecionar Equipamento", run_equipamento_selecionar)
            .menu(&[])
            .params("{} -> {handle}")
            .interactive(|_| Ok(Box::new(EquipSelM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.artefato.selecionar", "AnaCAD: Configurações do Artefato", run_artefato_selecionar)
            .menu(&[])
            .params("{} -> {handle}")
            .interactive(|_| Ok(Box::new(ArtefatoSelM::default())))
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.editar", "AnaCAD: Editar Equipamento", run_equipamento_editar)
            .menu(&[])
            .params("{db, documento, handle, dados} -> {equipamento, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.dados.excluir", "AnaCAD: Excluir Dados Elétricos do Equipamento", run_equipamento_dados_excluir)
            .menu(&[])
            .params("{db, documento?, handle} -> {equipamento, handle, removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.componentes", "AnaCAD: Componentes da Barra", run_barra_componentes)
            .menu(&[])
            .params("{db, documento, codigo} -> {componentes:[{id,nome}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.separar", "AnaCAD: Separar Barra", run_barra_separar)
            .menu(&[])
            .params("{db, documento, atual, nova, nome, componentes?, jumper?} -> {barra, componentes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.unir", "AnaCAD: Unir Barras", run_barra_unir)
            .menu(&[])
            .params("{db, documento, final, eliminada} -> {removidos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.linha.seccionar", "AnaCAD: Seccionar Linha", run_linha_seccionar)
            .menu(&[])
            .params("{db, documento, de, para, nova, nome, percentual, reatores?} -> {barra, circuitos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.renumerar", "AnaCAD: Renumerar Barra", run_barra_renumerar)
            .menu(&[])
            .params("{db, documento, atual, nova} -> {barra, nova}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.barra.localizar", "AnaCAD: Localizar Barra", run_barra_localizar)
            .menu(&[])
            .params("{db, documento, codigo} -> {codigo, x, y, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.terminal.listar", "AnaCAD: Listar Terminais", run_terminal_listar)
            .menu(&[])
            .params("{db, documento} -> {terminais:[{id,nome,x,y,handle}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.terminal.renomear", "AnaCAD: Renomear Terminal", run_terminal_renomear)
            .menu(&[])
            .params("{db, id, nome} -> {id, nome}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ponto.inserir", "AnaCAD: Inserir Ponto", run_ponto_inserir)
            .menu(&[])
            .params("{db, documento, tipo, x, y, nome?} -> {componente, handle}")
            .enabled(ponto_escolhido)
            .noundo()
            .interactive(|s| {
                let tipo = st_get(s, |state| state.equip_tipo.clone()).unwrap_or_default();
                if tipo.is_empty() {
                    return Err(crate::EngineError::Other("escolha o tipo de ponto".into()));
                }
                Ok(Box::new(PontoM { tipo }))
            }),
        CommandSpec::new("anacad.barra.detalhar", "AnaCAD: Detalhar Barra", run_barra_detalhar)
            .menu(&[])
            .params("{db, documento, codigo} -> {barra, componentes, circuitos, vizinhos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.anotar.curto", "AnaCAD: Anotar Curto no Diagrama", run_anotar_curto)
            .menu(&[])
            .params("{db, documento?, limite?} -> {anotacoes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.texto.inserir", "AnaCAD: Texto Livre", run_texto_inserir)
            .menu(&[])
            .params("{x, y, texto, altura?, cor?} -> {handle}")
            .enabled(always)
            .noundo()
            .interactive(|s| {
                let (texto, altura, cor) = st_get(s, |state| (state.texto.clone(), state.texto_altura, state.texto_cor)).unwrap_or_default();
                Ok(Box::new(TextoM { texto, altura, cor }))
            }),
        CommandSpec::new("anacad.legenda.config.ler", "AnaCAD: Ler Opções de Legenda", run_legenda_config_ler)
            .menu(&[])
            .params("{db} -> {dados}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.legenda.config.gravar", "AnaCAD: Gravar Opções de Legenda", run_legenda_config_gravar)
            .menu(&[])
            .params("{db, dados} -> {ok}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.niveis.curto", "AnaCAD: Níveis de Curto", run_niveis_curto)
            .menu(&[])
            .params("{db, documento?, sbase?, referencia?} -> {niveis:[{barra,i3f_ka,i1f_ka,i2f_ka,xr}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.evolucao.niveis", "AnaCAD: Evolução de Níveis", run_evolucao_niveis)
            .menu(&[])
            .params("{db, documento?, comparar, sbase?} -> {barras:[{barra,i3f_base_ka,i3f_novo_ka,delta_ka,delta_pct}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.macro", "AnaCAD: Estudo Macro (barras × contingências)", run_macro)
            .menu(&[])
            .params("{db, documento?, sbase?, barras?, casos:[{nome?, abrir?:[[de,para]]}]} -> {casos:[{nome, abrir, niveis}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.superacao", "AnaCAD: Superação de Disjuntores", run_superacao)
            .menu(&[])
            .params("{db, documento?, sbase?} -> {barras:[{barra,i3f_ka,capacidade_ka,margem,supera}], superam, sem_capacidade}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equivalente", "AnaCAD: Equivalente de Rede", run_equivalente)
            .menu(&[])
            .params("{db, documento?, barras:[codigos], sbase?} -> {barras, pares, simetrica}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.zbus", "AnaCAD: Matriz Zbarra", run_zbus)
            .menu(&[])
            .params("{db, documento?, referencia?} -> {barras, diagonal}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.impedancia.barras", "AnaCAD: Impedância entre Barras", run_impedancia_barras)
            .menu(&[])
            .params("{db, documento?, de, para} -> {de, para, z_re, z_im, modulo, angulo_graus}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.curto.simultaneo", "AnaCAD: Faltas Simultâneas", run_curto_simultaneo)
            .menu(&[])
            .params("{db, documento?, barras:[codigos], referencia?} -> {correntes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.ana.exportar", "AnaCAD: Salvar .ANA", run_ana_exportar)
            .menu(&[])
            .params("{db, documento?, arquivo} -> {arquivo, bytes, barras, circuitos}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.bloco", "AnaCAD: Definir Bloco do Equipamento", run_equipamento_bloco)
            .menu(&[])
            .params("{db, tipo, bloco?} -> {tipo, slug, bloco}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.teste.bloco", "AnaCAD: Bloco de Teste", run_teste_bloco)
            .menu(&[])
            .params("{nome?, inserir?, semente?} -> {bloco, handle, entidades}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.criar", "AnaCAD: Conectar Terminais", run_conexao_criar)
            .menu(&[])
            .params("{db, documento?, de_handle, de_terminal?, para_handle, para_terminal?, potencial?, desenhar?} -> {conexao, handle}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.validar", "AnaCAD: Validar Conexões", run_conexao_validar)
            .menu(&[])
            .params("{db, documento?, tolerancia?} -> {conexoes:[{id,ok,...}]}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.topologia", "AnaCAD: Topologia das Conexões", run_conexao_topologia)
            .menu(&[])
            .params("{db, documento?, tolerancia?, apenas_validas?} -> {componentes, nos, arestas}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.mover", "AnaCAD: Mover Equipamento", run_equipamento_mover)
            .menu(&[])
            .params("{db, handle, dx, dy} -> {handle, x, y}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.transformar", "AnaCAD: Rotacionar/Escalar Equipamento", run_equipamento_transformar)
            .menu(&[])
            .params("{db, handle, rotacao?, escala?} -> {handle, rotacao, escala}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.equipamento.sincronizar", "AnaCAD: Sincronizar Equipamentos", run_equipamento_sincronizar)
            .menu(&[])
            .params("{db, documento?} -> {atualizados, ausentes}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("anacad.conexao.comparar", "AnaCAD: Comparar Topologia CAD × Banco", run_conexao_comparar)
            .menu(&[])
            .params("{db, documento?} -> {cad, banco, faltando_em_cad, extras_cad}")
            .enabled(always)
            .noundo(),
        CommandSpec::new(
            "anacad.equipamento.sincronizar_conexoes",
            "AnaCAD: Sincronizar De/Para pelas Conexões",
            run_equipamento_sincronizar_conexoes,
        )
        .menu(&[])
        .params("{db, documento?} -> {atualizados, ignoradas}")
        .enabled(always)
        .noundo(),
        CommandSpec::new("anacad.exportar", "AnaCAD: Exportar .lst", run_exportar)
            .menu(&[])
            .params("{db, documento, saida} -> {saida, registros, mensagem}")
            .enabled(always)
            .noundo(),
    ]
}
