# Credenciais

> Estado: `[PLANNED]` — o módulo atual não implementa autenticação própria nem GitHub API.

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

Antes de implementar OAuth/PAT:

1. escrever ADR do método;
2. modelar storage e revogação;
3. auditar scopes;
4. testar vazamento em logs/erros;
5. separar permissões de leitura e escrita;
6. garantir que Git local funcione sem conta GitHub.
