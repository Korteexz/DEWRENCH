# Sistema de módulos

> Estado: navegação `[IMPLEMENTED]`; contrato formal/runtime `[PLANNED]`.

## O que existe hoje

`ModuleNavigation.tsx` possui um registro local:

- Git: disponível e ativo;
- Docker: indisponível;
- Database Viewer: indisponível;
- RRF: indisponível.

Botões futuros são clicáveis para feedback visual e acessível, mas não trocam viewport nem executam ação.

## O que ainda não existe

- carregamento dinâmico de módulos;
- manifesto de módulo;
- lifecycle formal;
- event bus;
- marketplace ou instalação;
- isolamento de permissões por módulo;
- viewport/routing dos módulos futuros.

## Direção do contrato

```ts
interface DewrenchModule {
  id: string
  metadata: ModuleMetadata
  capabilities: Capability[]
  routes: ModuleRoute[]
  commands: CommandContract[]
  events: EventContract[]
  riskPolicy: RiskPolicy
}
```

Este contrato é conceitual. Não deve ser criado antecipadamente sem um segundo módulo real para validar a abstração.

## Regras de modularidade

1. O módulo exporta sua viewport e seus contratos; o shell fornece chrome e infraestrutura.
2. Módulos não acessam internals uns dos outros.
3. Integrações passam por eventos ou contratos do Core.
4. Cada módulo documenta operações, erros e segurança próprios.
5. Permissões são mínimas por capacidade.
6. Remover um módulo não deve quebrar os demais.
7. Dados compartilhados representam o projeto, não a implementação interna de uma ferramenta.

## Modelo de crescimento

Começar como monólito modular evita a complexidade prematura de plugins binários, processos separados ou microserviços. Quando Docker/RRF existir de verdade, usar o atrito observado para definir o primeiro contrato formal.

## IA futura

IA/RAG local deve ser um módulo ou plugin opcional. Ela não pode ser requisito para abrir projetos, executar Git ou compreender operações básicas.
