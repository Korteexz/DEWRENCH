# Segurança de filesystem

> Estado: abertura canonicalizada; política completa `[PARTIAL]`.

## Implementado

`repository::open` confirma existência, confirma diretório e usa `canonicalize()`. O path canonicalizado volta ao frontend e passa a representar o projeto da sessão.

## Lacunas

- outras operações recebem path novamente pelo IPC e não repetem política central;
- `file` é enviado ao Git como path relativo, sem verificação própria de escape;
- symlinks não possuem política explícita;
- `.git` arquivo/worktrees não são reconhecidos pela checagem atual;
- não há escopo de workspace registrado no backend;
- não há controle para path trocado entre preflight e execução.

## Política alvo

1. Registrar no backend um identificador opaco de workspace após abertura.
2. Usar esse identificador nas operações, reduzindo reenvio de path arbitrário.
3. Canonicalizar e confirmar que alvo pertence ao workspace quando a operação exigir.
4. Definir comportamento para symlink por tipo de operação.
5. Usar argumentos separados e `--` antes de filenames.
6. Preferir formatos Git nul-delimited para filenames.
7. Nunca apagar fora do escopo nem silenciosamente.
8. Tratar erros de permissão e TOCTOU explicitamente.

## Renames e nomes incomuns

O parser atual de `status --porcelain=v1` fatia a string a partir do terceiro byte. Antes de produção, migrar para saída `-z` e parser que preserve unicode, espaços, renames e caracteres especiais com segurança.
