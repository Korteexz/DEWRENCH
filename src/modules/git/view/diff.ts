/**
 * Parser do diff unificado do Git.
 *
 * O backend entrega o texto bruto de `git show --unified=3`. Traduzir esse
 * texto em estrutura é trabalho de apresentação, não de domínio: nada aqui
 * decide o que o Git fez, apenas descreve o que ele reportou. Por isso vive
 * em `view/` e é uma função pura — dá para testar sem React e sem Tauri.
 */

export type DiffLineKind = 'context' | 'add' | 'del'

export interface DiffLine {
  kind: DiffLineKind
  text: string
  /** Número na versão anterior; nulo quando a linha não existia. */
  oldNumber: number | null
  /** Número na versão nova; nulo quando a linha deixou de existir. */
  newNumber: number | null
}

export interface DiffHunk {
  /** O cabeçalho `@@ -a,b +c,d @@` como o Git escreveu. */
  range: string
  /** Contexto que o Git anexa depois do `@@` (assinatura de função, etc). */
  context: string
  lines: DiffLine[]
}

export type DiffFileStatus =
  | 'modified'
  | 'added'
  | 'deleted'
  | 'renamed'
  | 'binary'

export interface DiffFile {
  path: string
  previousPath: string | null
  status: DiffFileStatus
  hunks: DiffHunk[]
  additions: number
  deletions: number
}

export interface ParsedDiff {
  files: DiffFile[]
  additions: number
  deletions: number
  /** Diff vazio é um resultado válido: commit sem mudança de conteúdo. */
  empty: boolean
}

const HUNK_PATTERN = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@(.*)$/

function createFile(path: string): DiffFile {
  return {
    path,
    previousPath: null,
    status: 'modified',
    hunks: [],
    additions: 0,
    deletions: 0,
  }
}

/** `a/src/lib.rs` -> `src/lib.rs`; `/dev/null` sinaliza criação/remoção. */
function stripPrefix(raw: string): string | null {
  const value = raw.trim()
  if (value === '/dev/null') {
    return null
  }
  return value.replace(/^[ab]\//, '')
}

export function parseUnifiedDiff(source: string | null): ParsedDiff {
  const files: DiffFile[] = []
  let current: DiffFile | null = null
  let hunk: DiffHunk | null = null
  let oldLine = 0
  let newLine = 0

  for (const line of (source ?? '').split('\n')) {
    if (line.startsWith('diff --git ')) {
      // Caminhos com espaço tornam esta linha ambígua; ela só abre o arquivo.
      // Os nomes confiáveis vêm de `---`/`+++` logo abaixo.
      current = createFile('')
      hunk = null
      files.push(current)
      continue
    }

    if (!current) {
      continue
    }

    if (line.startsWith('--- ')) {
      const path = stripPrefix(line.slice(4))
      if (path === null) {
        current.status = 'added'
      } else if (current.status === 'renamed') {
        current.previousPath = path
      } else {
        current.previousPath = path
      }
      continue
    }

    if (line.startsWith('+++ ')) {
      const path = stripPrefix(line.slice(4))
      if (path === null) {
        current.status = 'deleted'
        current.path = current.previousPath ?? current.path
      } else {
        current.path = path
      }
      continue
    }

    if (line.startsWith('rename from ')) {
      current.status = 'renamed'
      current.previousPath = line.slice(12).trim()
      continue
    }

    if (line.startsWith('rename to ')) {
      current.status = 'renamed'
      current.path = line.slice(10).trim()
      continue
    }

    if (line.startsWith('new file mode')) {
      current.status = 'added'
      continue
    }

    if (line.startsWith('deleted file mode')) {
      current.status = 'deleted'
      continue
    }

    if (line.startsWith('Binary files ')) {
      current.status = 'binary'
      continue
    }

    const hunkMatch = HUNK_PATTERN.exec(line)
    if (hunkMatch) {
      oldLine = Number(hunkMatch[1])
      newLine = Number(hunkMatch[2])
      hunk = {
        range: line.slice(0, line.indexOf('@@', 2) + 2),
        context: hunkMatch[3].trim(),
        lines: [],
      }
      current.hunks.push(hunk)
      continue
    }

    if (!hunk) {
      continue
    }

    if (line.startsWith('\\')) {
      // "\ No newline at end of file" descreve a linha anterior, não é conteúdo.
      continue
    }

    if (line.startsWith('+')) {
      hunk.lines.push({
        kind: 'add',
        text: line.slice(1),
        oldNumber: null,
        newNumber: newLine,
      })
      newLine += 1
      current.additions += 1
      continue
    }

    if (line.startsWith('-')) {
      hunk.lines.push({
        kind: 'del',
        text: line.slice(1),
        oldNumber: oldLine,
        newNumber: null,
      })
      oldLine += 1
      current.deletions += 1
      continue
    }

    if (line.startsWith(' ') || line === '') {
      hunk.lines.push({
        kind: 'context',
        text: line.slice(1),
        oldNumber: oldLine,
        newNumber: newLine,
      })
      oldLine += 1
      newLine += 1
    }
  }

  const additions = files.reduce((total, file) => total + file.additions, 0)
  const deletions = files.reduce((total, file) => total + file.deletions, 0)

  return {
    files,
    additions,
    deletions,
    empty: files.length === 0,
  }
}

export interface DiffSplitRow {
  left: DiffLine | null
  right: DiffLine | null
}

/**
 * Emparelha remoções e adições para a leitura lado a lado.
 *
 * Blocos consecutivos de `-` e `+` são pareados por posição: é a mesma
 * heurística que o Git usa para exibir substituição, e ela erra do lado
 * seguro — sobra vira linha vazia em vez de alinhar coisas não relacionadas.
 */
export function toSplitRows(hunk: DiffHunk): DiffSplitRow[] {
  const rows: DiffSplitRow[] = []
  let removals: DiffLine[] = []
  let additions: DiffLine[] = []

  function flush(): void {
    const pairs = Math.max(removals.length, additions.length)
    for (let index = 0; index < pairs; index += 1) {
      rows.push({
        left: removals[index] ?? null,
        right: additions[index] ?? null,
      })
    }
    removals = []
    additions = []
  }

  for (const line of hunk.lines) {
    if (line.kind === 'del') {
      removals.push(line)
      continue
    }
    if (line.kind === 'add') {
      additions.push(line)
      continue
    }
    flush()
    rows.push({ left: line, right: line })
  }

  flush()
  return rows
}
