# Threat model

> Estado: primeira versão `[DRAFT]`. Deve ser revisada antes de 1.0 e a cada nova capacidade nativa/remota.

## Pergunta central

O que pode dar errado quando uma ferramenta local recebe permissão intencional para ler e modificar projetos, Git, filesystem, processos e serviços remotos?

## Ativos

- código-fonte e documentos do usuário;
- histórico Git e branches;
- working tree e staging area;
- credenciais, tokens e helpers;
- remotes e repositórios hospedados;
- variáveis de ambiente;
- configurações Git/DEWRENCH;
- disponibilidade e integridade da máquina;
- intenção e confiança do usuário.

## Atores/fontes de risco

- erro acidental do usuário;
- bug do DEWRENCH;
- repository malicioso ou não confiável;
- remote comprometido;
- entrada manipulada pelo frontend;
- dependência vulnerável;
- agente de código ou plugin excessivamente autorizado;
- processo concorrente alterando o mesmo repo.

## Ameaças prioritárias

| Ameaça | Impacto | Estado atual |
|---|---|---|
| Command injection | execução arbitrária | sem shell em nenhuma invocação; testado com separadores, crase, `$()` e newline em branch, mensagem e caminho |
| Argument injection | a operação vira outra operação | **era explorável** (`switch --orphan=<x>` movia o HEAD); operandos iniciados por `-` recusados no Core + separador `--` onde o subcomando aceita |
| Hooks do repositório | execução arbitrária ao abrir/trocar branch/commitar | **era explorável** (`post-checkout`, `pre-commit`, `core.hooksPath`); desligados enquanto a confiança for menor que `ExecutableContent` |
| Config hostil apontando para programa | execução arbitrária | `core.fsmonitor`, `diff.external`, `core.pager` zerados; `core.sshCommand` e `credential.helper` **eram exploráveis** e agora perdem para a config do usuário |
| Env poisoning | execução arbitrária / leitura redirecionada | 22 variáveis removidas da herança; testado com `GIT_EXTERNAL_DIFF` e `GIT_DIR` |
| Path traversal/escape | arquivo fora do repo | resolução canônica componente a componente; `..`, absoluto e irmão com prefixo igual testados |
| Symlink abuse | operação fora do escopo aparente | symlink para fora não herda autoridade; symlink para o próprio workspace resolve para a MESMA identidade |
| Caminho arbitrário via IPC | operar sobre qualquer pasta da máquina | **era possível**; agora só caminho correspondente a workspace registrado |
| Operação Git destrutiva | perda de trabalho/histórico | preflight do Revert existe; aprovação vinculada ao estado **não está ligada** |
| Credential leakage | acesso indevido a remotes | redação central (`core::events`) cobre PEM, URL com credencial, tokens com prefixo e pares chave=valor |
| Remote inesperado | push para destino errado | nome de remote precisa existir; URL por allowlist de protocolo; `ext::`/`fd::` recusados |
| Race/double execution | estado inconsistente | lock RAII existe e é testado, **mas os commands ainda não o adquirem** |
| Git output não confiável | erro de parsing/render | texto tratado como dado; teto de saída de 24 MiB |
| Dependência/webview | exploração da aplicação | CSP **não revisada nesta sessão** |

O detalhamento por controle, com o que é aplicado hoje e o que é só fundação,
está em `enforcement-state.md`.

## Suposições

- O usuário escolhe voluntariamente a pasta.
- O Git utilizado é o binário disponível no PATH — o que é, por si só, um risco
  residual registrado: um PATH comprometido troca o executável.
- Local-first não significa que toda entrada local é confiável.
- Repositórios podem conter nomes de arquivos, config e conteúdo hostis.
- Remote e plugins ampliarão drasticamente a superfície de ataque.

## Objetivos de segurança

1. Executar somente a operação que o usuário pretendeu.
2. Limitar o escopo ao projeto selecionado.
3. Tornar consequência e destino visíveis.
4. Preservar recuperação sempre que possível.
5. Proteger segredos em trânsito, armazenamento, logs e UI.
6. Falhar de modo seguro e explícito.
7. Manter permissões Tauri mínimas.

## Fora do modelo atual

Sandbox forte contra código arbitrário do próprio projeto, execução de builds de terceiros e proteção contra comprometimento total do sistema operacional não são garantias atuais.
