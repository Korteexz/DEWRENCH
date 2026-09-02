# ADR-004 — Local-first

- Status: Accepted

## Decisão

Projetos e operações locais permanecem na máquina do usuário por padrão. Cloud e contas externas entram apenas quando uma capability exige.

## Motivos

- privacidade e controle;
- baixa latência;
- funcionamento sem cloud;
- compatibilidade com ferramentas locais;
- menor lock-in;
- coerência com software open source.

## Consequências

- cada sistema operacional precisa de testes;
- storage/config local precisa de migração segura;
- integrações remotas são opcionais;
- “local” não é sinônimo de “confiável”;
- backups continuam responsabilidade explícita a comunicar.

## Regra

Nenhum módulo fundamental pode exigir login em serviço externo para operar recursos locais.
