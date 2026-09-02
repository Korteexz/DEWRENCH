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
| Command injection | execução arbitrária | mitigação parcial: sem shell |
| Path traversal/escape | arquivo fora do repo | validação insuficiente |
| Symlink abuse | operação fora do escopo aparente | sem política formal |
| Operação Git destrutiva | perda de trabalho/histórico | operações críticas ainda ausentes |
| Credential leakage | acesso indevido a remotes | sem integração própria; logs devem ser auditados |
| Remote inesperado | push para destino errado | remote ainda não implementado |
| Race/double execution | estado inconsistente | busy UI parcial; sem lock backend |
| Estado intermediário oculto | usuário agrava conflito/falha | modelo ainda não existe |
| UI/backend divergence | visual mente sobre repo | refresh implementado após mutações |
| Git output não confiável | erro de parsing/render | texto tratado como dado; parser é simples |
| Dependência/webview | exploração da aplicação | CSP atual está desativada |

## Suposições

- O usuário escolhe voluntariamente a pasta.
- O Git utilizado é o binário disponível no PATH.
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
