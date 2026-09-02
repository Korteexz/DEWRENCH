# ADR-001 — Tauri 2

- Status: Accepted
- Data da decisão: 2026-08 (aproximada)

## Contexto

DEWRENCH precisa de UI rica, acesso nativo controlado, execução de ferramentas locais e distribuição multiplataforma, sem carregar um Chromium completo por aplicativo.

## Decisão

Usar Tauri 2 com frontend web React/TypeScript e backend Rust.

## Motivos

- integração nativa e comandos IPC;
- WebView do sistema;
- footprint potencialmente menor que Electron;
- Rust adequado para operações de sistema;
- suporte a Windows, Linux e macOS;
- capabilities e plugins controláveis.

## Consequências

- diferenças entre WebView2 e WebKitGTK precisam ser testadas;
- contratos Rust ↔ TypeScript precisam de disciplina;
- build exige toolchains web e Rust;
- capabilities/CSP fazem parte da segurança;
- código nativo eleva o impacto de validação incorreta.

## Alternativas consideradas

- Electron: ecossistema maduro, porém maior bundle/memória.
- UI totalmente nativa: integração forte, mas menor reaproveitamento web e custo maior de interface dinâmica.

## Revisão futura

Reavaliar apenas com evidência de limitação técnica real, não por preferência abstrata de framework.
