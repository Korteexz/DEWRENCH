# DEWRENCH — documentação do projeto

> Baseline documental: `main@eb90e2e` (`stage_all_made`, 2026-08-30)  
> Documento iniciado em: 2026-09-02  
> Fase do produto: pré-1.0, módulo Git ativo

## O que é o DEWRENCH

DEWRENCH é uma bancada desktop local-first para visualizar, compreender e operar ferramentas de desenvolvimento tradicionalmente controladas por terminal. Ele não reimplementa Git, Docker, bancos de dados ou Kubernetes: ele executa ferramentas reais, traduz seus resultados para modelos internos e oferece representações visuais manipuláveis.

```text
ferramenta real → backend nativo → contrato IPC → modelo do frontend → visualização
Git CLI         → Rust           → Tauri       → TypeScript        → grafo/inspetor
```

A interface é parte do produto, não apenas decoração. Relações, cores, formas, foco e movimento devem representar causalidade e estado reais.

## Estado atual

| Área | Status | Observação |
|---|---|---|
| Shell desktop Tauri | `[IMPLEMENTED]` | Aplicação Tauri 2 com frontend React/Vite. |
| Seleção de pasta local | `[IMPLEMENTED]` | Usa o seletor nativo de diretórios. |
| Módulo Git local | `[IMPLEMENTED]` | Leitura, grafo, mutações locais, revert e diff. |
| Docker | `[PLANNED]` | Botão visível e inativo (`SOON`). |
| Database Viewer | `[PLANNED]` | Botão visível e inativo. |
| RRF | `[PLANNED]` | Botão visível e inativo. |
| Git remotes | `[IMPLEMENTED]` | Listar, adicionar, renomear, trocar URL, remover. |
| Push / fetch / pull | `[IMPLEMENTED]` | Com preflight, erros tipados e estratégia explícita. |
| Remote-tracking branches | `[IMPLEMENTED]` | Upstream, ahead/behind e divergência no índice. |
| GitHub (provider) | `[PARTIAL]` | Via `gh` CLI: contexto, PRs, abrir no navegador, criar PR. |
| Atividade / Temporal Matrix | `[IMPLEMENTED]` | Eventos do Git agregados em ano/mês/dia/hora. |
| Plugins | `[PLANNED]` | Direção arquitetural, sem runtime de plugins. |
| IA/RAG local | `[PLANNED]` | Futuro plugin opcional; não é dependência do produto. |

## Vocabulário de status

- `[IMPLEMENTED]`: existe no código atual e possui fluxo utilizável.
- `[PARTIAL]`: existe, mas faltam casos, interface, validação ou robustez.
- `[EXPERIMENTAL]`: implementado para avaliação; contrato e comportamento podem mudar.
- `[PLANNED]`: direção aceita, ainda não implementada.
- `[STUB]`: arquivo ou ponto de extensão criado, mas vazio/inativo.
- `[KNOWN DEFECT]`: problema observado e reproduzível na baseline.
- `[REQUIRES AUDIT]`: conclusão que deve ser confirmada antes de mudança sensível.

## Princípios do produto

1. Local-first por padrão.
2. Open source e inspecionável.
3. Abstrair complexidade sem esconder causalidade.
4. O estado visual deve derivar do estado real da ferramenta.
5. A mesma informação pode ter múltiplas visualizações: grafo, fluxo, blocos, tabela ou terminal.
6. Complexidade progressiva: acesso simples para iniciantes e detalhes para usuários avançados.
7. Operações perigosas devem explicar consequências, não apenas pedir confirmação.
8. Módulos devem ser peças substituíveis, evitando dependências diretas entre Git, Docker, DB, RRF e futuros módulos.
9. Automação ou IA futura deve ser opcional e removível.

## Stack verificada

- Tauri 2 + Rust 2021;
- React 19 + TypeScript 6;
- Vite 8;
- XYFlow/React Flow 12;
- Git CLI instalado na máquina do usuário;
- Oxlint.

As versões exatas ficam nos manifests `package.json` e `src-tauri/Cargo.toml`.

## Fluxo principal atual

1. O usuário escolhe uma pasta local na Home.
2. `open_project` canonicaliza a pasta e classifica o Git em `not_repository`, `unborn_repository` ou `repository`.
3. Uma pasta sem Git abre o fluxo de criação de repositório e commit inicial.
4. Um repositório sem commit abre uma tela informativa.
5. Um repositório válido abre o workspace Git.
6. O frontend carrega detalhes e grafo em paralelo.
7. Após uma mutação bem-sucedida, o frontend relê o estado do backend antes de atualizar a visualização.

## Limites conhecidos da baseline

- Somente Git está ativo.
- Merge de branches locais, resolução de conflito e force push não estão implementados.
- O grafo lê até 80 commits e apenas branches locais.
- O diff tem parser próprio e leitura unificada ou lado a lado; ainda não permite selecionar arquivo específico no backend.
- Erros são strings, normalmente derivadas do `stderr` do Git.
- Testes: unitários em `src-tauri/src/modules/**` e integração em `src-tauri/tests/git_network.rs` (laboratórios Git em diretório temporário). O frontend ainda não tem suíte.
- `core/*`, `git/merge.rs`, `git/parser.rs` e dois arquivos de `github/` (`auth.rs`, `pull_request.rs`) continuam stubs vazios e não declarados.

## Como navegar nesta documentação

- [`architecture/`](architecture/overview.md): como o sistema está dividido.
- [`modules/git/`](modules/git/overview.md): comportamento do módulo ativo.
- [`security/`](security/threat-model.md): riscos, limites de confiança e política de ações.
- [`product/`](product/vision.md): visão, UX, design e roadmap.
- [`decisions/`](decisions/ADR-001-tauri.md): decisões arquiteturais e seus motivos.
- [`contributing/`](contributing/agent-guidelines.md): regras para humanos e agentes de código.

## Regra para agentes

Documentação é contexto, não prova final. Antes de modificar o DEWRENCH, compare este material com o código da branch ativa. Ao encontrar divergência, pare, registre a diferença e trate o código executável como estado atual e a documentação como intenção a ser revisada.
