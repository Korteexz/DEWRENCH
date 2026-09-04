# Credenciais

> Estado: `[PARTIAL]` — o DEWRENCH **não implementa autenticação própria** e
> continua sem OAuth/PAT. A integração com o GitHub existe e é feita pela CLI
> `gh`, que resolve a autenticação inteiramente por conta própria: o DEWRENCH
> observa apenas o exit code de `gh auth status` e nunca lê, grava, exibe ou
> transporta token. Autenticação por variável de ambiente (`GH_TOKEN`,
> `GITHUB_TOKEN`, `GH_ENTERPRISE_TOKEN`) continua funcionando porque essas
> variáveis são deliberadamente mantidas na herança de ambiente — o valor é
> consumido pela `gh`, não pelo DEWRENCH. As demais `GH_*` que redirecionam ou
> executam programas são removidas (ver `enforcement-state.md`).

## Princípios

- Não armazenar token em plaintext.
- Não colocar segredo em frontend sem necessidade.
- Não registrar Authorization, token, senha ou URL autenticada.
- Preferir Git Credential Manager/helper e armazenamento seguro do sistema.
- Solicitar apenas escopos necessários.
- Permitir revogação e troca de conta.
- Diferenciar autenticação do Git remoto e autenticação de API GitHub.

## URLs

Remote URL pode conter usuário ou informação sensível. Sanitizar antes de logs e previews; ainda assim mostrar host/owner/repo suficientes para o usuário confirmar o destino.

## Erros

Mensagem de autenticação deve explicar qual serviço/remote falhou sem repetir credencial. Nunca sugerir colar token em arquivo do repositório.

## Futuro GitHub

O que existe hoje delega tudo à `gh`. Antes de implementar OAuth/PAT próprios:

1. escrever ADR do método;
2. modelar storage e revogação;
3. auditar scopes;
4. testar vazamento em logs/erros;
5. separar permissões de leitura e escrita;
6. garantir que Git local funcione sem conta GitHub.
