# Estado da aplicação de segurança

> Escopo desta página: o que o Security Core do DEWRENCH **faz hoje**, o que
> está **preparado mas não ligado**, e o que **não existe**. Ela é atualizada
> por sessão de trabalho de segurança, e cada linha responde a uma pergunta só:
> *isso acontece quando o app roda?*
>
> Nada aqui afirma que o DEWRENCH é seguro. As afirmações são sobre controles
> específicos, testados de formas específicas, numa versão específica.

**Última atualização:** sessão de Security Core + Red Team (branch
`cybersec_test/core`).
**Plataforma dos testes:** Linux x86_64, git 2.x. Windows e macOS **não foram
testados** — ver *Não testado*.

---

## ENFORCED NOW

Controles que estão no caminho de execução real e que um teste automatizado
falha se forem removidos.

### Fronteira de processo — `core::process`

| Controle | O que faz | Onde é exercido |
|---|---|---|
| Sem shell | Nenhuma invocação usa `sh -c`, `bash -c` ou `cmd /C`. Argumento é argumento. | `tests/red_team.rs` (separadores em branch, mensagem e caminho) |
| Allowlist de executável | `ProgramId` é um enum fechado (`Git`, `Gh`). Não existe `run(program: &str)`. | tipo; não há caminho para violar |
| `stdin` fechado | `Stdio::null()`: nenhum prompt interativo trava o app. | `core::process` |
| Ambiente higienizado | 22 variáveis removidas da herança, incluindo `GIT_EXTERNAL_DIFF`, `GIT_DIR`, `LD_PRELOAD`, `DYLD_*`. | `tests/red_team_env.rs` |
| Prelúdio de config | `core.fsmonitor`, `diff.external` e `core.pager` zerados em toda invocação do git. | `tests/red_team.rs` |
| Hooks desligados | `core.hooksPath` apontado para diretório vazio enquanto a confiança do workspace for menor que `ExecutableContent`. | `tests/red_team.rs` (post-checkout, pre-commit, `core.hooksPath`) |
| Precedência de auth | `core.sshCommand` e `credential.helper` do repositório perdem para a config global/de sistema do usuário. | `tests/red_team.rs` |
| Tempo limite | 60s local, 180s rede; o processo é **encerrado**, não só abandonado. | `core::process` |
| Teto de saída | 24 MiB stdout / 256 KiB stderr, com sinal de truncamento. | `core::process` |
| Recusa de operando | Valor que começa com `-` não chega ao git como dado. | `core::process`, `tests/red_team.rs` |

### Fronteira de autoridade — `core::state`

| Controle | O que faz |
|---|---|
| Deny-by-default | Um caminho que o usuário nunca abriu é recusado, em leitura e em mutação. |
| Identidade canônica | Symlink, alias e diferença de maiúsculas resolvem para o **mesmo** workspace — dois nomes não viram duas autoridades. |
| Autoridade por workspace | Abrir A não concede nada sobre B. |
| Caminho do registro | A operação executa contra a raiz canônica registrada, não contra a string recebida do IPC. |
| Lock RAII | Liberado em retorno antecipado, `?` e panic. |

Exercido por `tests/security_boundary.rs`.

### Fronteira de caminho — `core::path_security`

Resolução canônica com verificação componente a componente, cobrindo `..`,
caminho absoluto, symlink para fora, symlink de diretório no meio do caminho,
symlink encadeado, alvo inexistente e irmão com prefixo textual igual
(`projeto` vs `projeto-malicioso`).

### Redação — `core::events`

Blocos PEM, credencial embutida em URL, tokens com prefixo conhecido
(`ghp_`, `github_pat_`, `glpat-`, `xox*`, `AKIA`, `sk-`, `AIza`) e pares
`chave=valor` sensíveis são removidos antes de virar erro ou evento de
auditoria. `modules::git::errors::sanitize` delega para cá.

### Guarda arquitetural

`tests/security_boundary.rs` falha se qualquer arquivo sob `src/modules/`
voltar a conter `Command::new`. É a única defesa contra a regressão mais
provável desta arquitetura.

---

## ARCHITECTURALLY PREPARED — não ligado

Existe, tem teste de unidade, e **nenhum fluxo do app o consulta hoje**.
Listado aqui para que ninguém confunda "o tipo existe" com "a regra vale".

| Componente | Estado |
|---|---|
| `core::policy` — catálogo de ações, capability, risco, decisão | Avalia corretamente; nenhum command chama `authorize()`. |
| `core::approval` — snapshot de preflight, digest, TTL, staleness | Detecta aprovação obsoleta e expirada; nenhum fluxo emite token. |
| `core::authority::WorkspaceTrust` | Usado hoje **apenas** para decidir hooks e precedência de auth. Nenhuma interface concede `ExecutableContent`. |
| `core::events::AuditEvent` + journal | Grava e redige; nada escreve nele ainda. |
| `RecoveryKind` | Declarado por ação; nenhuma decisão usa. |

---

## KNOWN RESIDUAL RISK

Aceito conscientemente nesta versão, com o motivo.

1. **Confiança de workspace nasce em `Opened` e nunca sobe.** Não há interface
   para o usuário declarar que confia no conteúdo executável de um repositório.
   Consequência prática: hooks legítimos (pre-commit, husky) **não rodam** pelo
   DEWRENCH. Isso é uma quebra de comportamento deliberada — ver
   *Breaking changes* no relatório da sessão.
2. **Config local de auth é ignorada.** Uma chave de deploy por repositório
   (`core.sshCommand` local) deixa de valer. O ganho é fechar um RCE
   reproduzido; o custo é real e conhecido.
3. **Aprovação não está no caminho.** Push, pull e revert executam sem token de
   aprovação vinculado ao preflight. O tipo existe; a ligação não.
4. **Lock não cobre o IPC inteiro.** `core::state::acquire` existe e é testado,
   mas os commands ainda não o adquirem: duas mutações simultâneas sobre o
   mesmo repositório disputam no nível do próprio git (`index.lock`), não no
   nível do DEWRENCH.
5. **`git` e `gh` são resolvidos pelo `PATH`.** Um `PATH` comprometido troca o
   executável. Resolver caminho absoluto na inicialização é possível e não foi
   feito.
6. **Diretório de hooks vazio vive no temporário.** Quem já tiver escrita nesse
   diretório pode plantar hooks. Quem tem essa escrita já executa código como o
   usuário, então o ganho de blindar isso é pequeno — mas não é zero.
7. **CSP do Tauri não foi revisada nesta sessão.** Documentado, não corrigido.

---

## NÃO TESTADO

Não é "seguro" nem "inseguro". É desconhecido.

- **Windows**: junction, caminho verbatim `\\?\`, nomes 8.3, comparação de
  maiúsculas em NTFS, ADS (`arquivo.txt:stream`). A lógica existe e é
  case-insensitive sob `cfg!(windows)`, mas **nunca foi executada** em Windows.
- **macOS**: normalização Unicode do HFS+/APFS (NFD vs NFC) em nome de
  arquivo e de branch.
- **Repositórios grandes**: comportamento do teto de saída em `git log` de
  repositório com centenas de milhares de commits.
- **Submódulos**: nenhum fluxo foi exercitado com submódulo.
- **`credential.helper` executando de fato**: a precedência foi verificada
  automaticamente; a não-execução do helper hostil foi verificada **à mão**,
  não por teste de regressão.

---

## FORA DE ESCOPO desta sessão

Docker, Kubernetes, Terraform, banco de dados, Redis, Kafka, Jenkins,
Prometheus; cofre de segredos; broker de privilégio; sandbox; engine de
recuperação e rollback; merge de PR; qualquer mudança de frontend.
