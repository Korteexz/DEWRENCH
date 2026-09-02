# Módulo Git — grafo visual

> Estado: topologia `[IMPLEMENTED]`; física `[EXPERIMENTAL]`.

## Semântica atual

| Objeto | Representação |
|---|---|
| Projeto atual | orb central com capability `GIT` |
| Commit | pequeno anel com núcleo e label |
| Merge commit | commit com cor/label de merge e quantidade de pais |
| Branch | losango com nome; branch atual recebe estado destacado |
| Ancestralidade | linha contínua cinza-esverdeada |
| Relação de merge | linha contínua amarelo discreto |
| Branch → head | linha laranja tracejada |
| Seleção | glow/foco e redução de elementos distantes |

## Origem dos dados

O backend retorna commits e parents reais. O adapter cria:

- um nó de projeto;
- um nó por commit;
- uma edge por parent presente na janela carregada;
- um nó por branch local;
- uma edge branch-head quando o head está na janela.

O grafo não cria edge para parent fora dos 80 commits carregados.

## Layout

`layoutConstellation` calcula posições iniciais; `layoutWorkspaceGraph` aplica essas posições e traduz relações em edges XYFlow. O projeto funciona como âncora visual, mas não representa relação Git de ancestralidade.

## Interação

- click seleciona;
- click no vazio limpa seleção;
- clique direito abre ações contextuais;
- sidebar e canvas compartilham `selectedNodeId`;
- drag ativa a física e perturba o grid deformável;
- nós próximos do ponteiro revelam labels;
- seleção destaca o nó e relações de um salto.

## Regras

1. Relação visual precisa corresponder ao Git real.
2. Física pode mover geometria, nunca alterar semântica.
3. Cor e forma devem permanecer distinguíveis sem depender apenas de glow.
4. O layout deve ser determinístico o bastante para não desorientar após refresh.
5. Limites de truncamento precisam ser comunicados.
6. Múltiplas visualizações futuras devem consumir o mesmo modelo semântico.

## Evolução planejada

- working tree e staging como estados visuais;
- ahead/behind e upstream;
- commits não enviados;
- caminhos de merge/revert;
- filtro e busca;
- modo fluxograma/tabela/log;
- destaque temporal da operação executada, inspirado no princípio “mostrar qual parte do fluxo acabou de rodar”.
