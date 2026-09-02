# Módulo Git — roadmap

> Não há datas prometidas. As versões expressam capacidade, não calendário.

## 0.5 — entende Git

Status: `[PARTIAL]`

- abrir e classificar repositório;
- ler status, branches, commits e topologia;
- representar projeto/commit/branch;
- corrigir stage individual;
- estabilizar parsing e estados vazios;
- adicionar testes da leitura.

## 0.6 — manipula histórico

Status: `[PARTIAL]`

- stage/unstage/commit;
- criar e trocar branch;
- diff por arquivo com parser visual;
- merge com fast-forward/commit/conflito/abort;
- operações locais documentadas e testadas.

## 0.7 — conversa com o mundo

Status: `[PLANNED]`

- remote e upstream;
- fetch;
- pull com estratégia explícita;
- push;
- ahead/behind;
- autenticação segura;
- GitHub e PR/MR depois do Git remoto genérico.

## 0.8 — desfaz e recupera

Status: `[PLANNED]`

- revert;
- recovery center;
- conflito visível;
- abort/continue seguros;
- preview de consequências;
- histórico de operações do DEWRENCH.

## 0.9 — polimento do frontend

Status: `[EXPERIMENTAL/PARTIAL]`

- consolidar design system CRT técnico;
- motion sem prejudicar legibilidade;
- responsividade;
- acessibilidade e reduced motion;
- múltiplas visualizações;
- performance em repositórios maiores;
- estados vazios/loading/error completos.

## 1.0 — Git Module Complete

Status: `[PLANNED]`

- regressão automatizada;
- modelo de erro tipado;
- segurança e permissões revisadas;
- operações locais/remotas essenciais estáveis;
- recuperação documentada;
- documentação alinhada ao código;
- empacotamento multiplataforma validado.

## Depois de 1.0

Docker, Database Viewer, RRF, CI/CD, Kubernetes, Terraform, ferramentas do sistema, plugins e IA local opcional. Cada módulo entra verticalmente, sem simular funcionalidade inexistente.
