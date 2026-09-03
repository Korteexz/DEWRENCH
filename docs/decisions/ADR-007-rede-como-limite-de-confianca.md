# ADR-007 — A rede como limite de confiança

- Status: aceito
- Data: 2026-09-03
- Contexto de código: `frontend/fui-redesign`

## Contexto

Até esta sessão nenhuma operação do DEWRENCH saía da máquina. Push, fetch e
pull mudam isso: passam a existir credenciais, um host remoto que responde o
que quiser, e mensagens de erro que podem conter token embutido em URL
(`https://user:token@host/...`).

Além disso, `git remote add` aceita URLs que não são endereços: remote helpers
como `ext::` fazem o Git **executar um comando arbitrário** durante um fetch.
Um remote malicioso configurado uma vez transforma qualquer sincronização em
execução remota de código.

## Decisão

1. **Allowlist de protocolo.** Só `https`, `http`, `ssh`, `git`, `file` e
   caminhos locais são aceitos como URL de remote. Qualquer coisa com `::` é
   recusada explicitamente, com a razão dita ao usuário.
2. **Validação contra injeção de argumento.** Nome e URL não podem começar por
   `-`: o Git leria o valor como opção de linha de comando.
3. **Saneamento obrigatório de saída.** Todo texto vindo de operação de rede
   passa por `errors::sanitize` antes de cruzar o IPC. O texto técnico continua
   acessível na interface — normalizar não pode significar esconder.
4. **Nenhuma operação de rede sem plano.** Push e pull têm preflight read-only
   que devolve origem, destino, contagens e a lista real de commits.
5. **Nenhuma estratégia implícita.** O pull recebe a estratégia escolhida pelo
   usuário. O backend informa quais são possíveis e recomenda uma; nunca
   executa `git pull` genérico, cujo comportamento depende de configuração
   global invisível (`pull.rebase`).
6. **Nenhum estado intermediário órfão.** Integração que entra em conflito é
   desfeita (`merge --abort` / `rebase --abort`) e reportada com os arquivos
   conflitantes. O DEWRENCH ainda não resolve conflito, e deixar o repositório
   parado num merge pela metade seria pior do que não ter feito nada.
7. **Force push fora de escopo.** Não existe como ação nesta versão.

## Consequências

**Positivas**: a superfície de rede tem uma porta só (`sync.rs`), erros são
classificados em códigos estáveis, e a interface pode explicar consequência
antes de agir.

**Negativas**: caminhos locais e `file://` são aceitos, o que é necessário para
os laboratórios de teste e legítimo para mirrors — mas amplia o que pode ser
configurado como destino. Operações de rede ainda não têm timeout nem
cancelamento.

**Trabalho decorrente**: timeout e cancelamento; migrar os 11 commands antigos
de `String` para erro tipado; resolução de conflito dentro do produto.
