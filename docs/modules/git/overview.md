# Módulo Git — visão geral

> Estado: `[PARTIAL]`  
> Escopo atual: repositórios locais

O módulo Git é o primeiro vertical slice real do DEWRENCH. Ele usa o Git instalado no sistema e transforma repositório, branches e commits em um workspace visual.

## Capacidades

| Capacidade | Status | Observação |
|---|---|---|
| Escolher pasta | `[IMPLEMENTED]` | Diálogo nativo. |
| Detectar estado Git | `[PARTIAL]` | `.git` diretório + validação de `HEAD`. |
| Criar repo + commit inicial | `[IMPLEMENTED]` | `init -b`, `add .`, `commit -m`; sem rollback. |
| Ler branch atual | `[IMPLEMENTED]` | `branch --show-current`. |
| Ler working tree | `[PARTIAL]` | Porcelain v1 sem `-z`. |
| Stage all | `[IMPLEMENTED]` | `git add -A`. |
| Stage por arquivo | `[KNOWN DEFECT]` | Backend existe; UI está ligada ao unstage. |
| Unstage por arquivo | `[IMPLEMENTED]` | `git restore --staged -- file`. |
| Criar commit | `[IMPLEMENTED]` | Somente conteúdo staged. |
| Grafo de commits | `[IMPLEMENTED]` | Até 80 commits, todas as branches locais. |
| Detalhes recentes | `[IMPLEMENTED]` | Até 10 commits; sidebar mostra até 18 do grafo. |
| Criar branch | `[IMPLEMENTED]` | A partir de branch/ref ou commit. |
| Trocar branch | `[IMPLEMENTED]` | `git switch`. |
| Diff de commit | `[PARTIAL]` | Patch bruto completo no inspetor. |
| Merge | `[STUB]` | Arquivo vazio e não exportado. |
| Histórico avançado | `[STUB]` | Arquivo vazio. |
| Remote/fetch/pull/push | `[STUB/PLANNED]` | Sem contrato ativo. |
| Revert/undo | `[PLANNED]` | Sem implementação. |
| GitHub/PR | `[STUB/PLANNED]` | Módulo GitHub vazio. |

## Objetivo educacional

O módulo deve tornar causalidade observável: qual ação alterou staging, histórico, branch, remote ou working tree. A interface não deve reduzir Git a botões mágicos.

## Fora do escopo atual

- rebase, cherry-pick, stash e tags;
- submodules e worktrees;
- LFS;
- credenciais próprias;
- hospedagem remota específica;
- resolução visual de conflitos;
- colaboração em tempo real.

## Definição de pronto para 1.0 do módulo

Leitura e mutações locais estáveis; operações remotas fundamentais; visualização de divergência; recuperação/revert; modelo de erro tipado; segurança e confirmações; testes de regressão; UI acessível e documentação atualizada.
