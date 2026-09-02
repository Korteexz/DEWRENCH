import { useMemo, useState } from 'react'

import { Button, TechnicalLabel } from '../../../design'
import {
  parseUnifiedDiff,
  toSplitRows,
  type DiffFile,
  type DiffHunk,
  type DiffLine,
} from '../view/diff'

interface DiffViewProps {
  source: string | null
}

type DiffMode = 'unified' | 'split'

/**
 * Teto de linhas renderizadas por arquivo. Um commit grande pode trazer
 * dezenas de milhares de linhas, e cada linha é um nó no DOM: sem teto, abrir
 * o diff travaria a janela. O usuário expande arquivo a arquivo.
 */
const LINE_BUDGET = 320

function statusLabel(file: DiffFile): string {
  switch (file.status) {
    case 'added': return 'NOVO'
    case 'deleted': return 'REMOVIDO'
    case 'renamed': return 'RENOMEADO'
    case 'binary': return 'BINÁRIO'
    default: return 'MODIFICADO'
  }
}

function lineNumber(value: number | null): string {
  return value === null ? '' : String(value)
}

function UnifiedRow({ line }: { line: DiffLine }) {
  return (
    <div className="diff-row" data-kind={line.kind}>
      <span className="diff-row__num">{lineNumber(line.oldNumber)}</span>
      <span className="diff-row__num">{lineNumber(line.newNumber)}</span>
      <span className="diff-row__sign" aria-hidden="true">
        {line.kind === 'add' ? '+' : line.kind === 'del' ? '−' : ' '}
      </span>
      <span className="diff-row__text">{line.text || ' '}</span>
    </div>
  )
}

function SplitRows({ hunk }: { hunk: DiffHunk }) {
  return (
    <>
      {toSplitRows(hunk).map((row, index) => (
        <div className="diff-split-row" key={index}>
          <span className="diff-row__num">{lineNumber(row.left?.oldNumber ?? null)}</span>
          <span
            className="diff-row__text diff-row__text--side"
            data-kind={row.left ? (row.left.kind === 'add' ? 'context' : row.left.kind) : 'empty'}
          >
            {row.left && row.left.kind !== 'add' ? row.left.text || ' ' : ''}
          </span>
          <span className="diff-row__num">{lineNumber(row.right?.newNumber ?? null)}</span>
          <span
            className="diff-row__text diff-row__text--side"
            data-kind={row.right ? (row.right.kind === 'del' ? 'context' : row.right.kind) : 'empty'}
          >
            {row.right && row.right.kind !== 'del' ? row.right.text || ' ' : ''}
          </span>
        </div>
      ))}
    </>
  )
}

function FileDiff({ file, mode }: { file: DiffFile; mode: DiffMode }) {
  const [expanded, setExpanded] = useState(false)
  const total = file.hunks.reduce((sum, hunk) => sum + hunk.lines.length, 0)
  const overBudget = total > LINE_BUDGET && !expanded

  let remaining = overBudget ? LINE_BUDGET : Number.POSITIVE_INFINITY
  const hunks: DiffHunk[] = []
  for (const hunk of file.hunks) {
    if (remaining <= 0) {
      break
    }
    hunks.push(
      hunk.lines.length <= remaining
        ? hunk
        : { ...hunk, lines: hunk.lines.slice(0, remaining) },
    )
    remaining -= hunk.lines.length
  }

  return (
    <section className="diff-file">
      <header className="diff-file__head">
        <span className="diff-file__path" title={file.path}>
          {file.previousPath && file.previousPath !== file.path && (
            <em>{file.previousPath} → </em>
          )}
          {file.path}
        </span>
        <span className="diff-file__status">{statusLabel(file)}</span>
        <span className="diff-file__stat">
          <b data-kind="add">+{file.additions}</b>
          <b data-kind="del">−{file.deletions}</b>
        </span>
      </header>

      {file.status === 'binary' ? (
        <p className="diff-file__note">Conteúdo binário: o Git não gera diff textual.</p>
      ) : (
        <div className="diff-file__body">
          {hunks.map((hunk) => (
            <div className="diff-hunk" key={hunk.range + hunk.context}>
              <div className="diff-hunk__range">
                <code>{hunk.range}</code>
                {hunk.context && <span>{hunk.context}</span>}
              </div>

              {mode === 'split' ? (
                <SplitRows hunk={hunk} />
              ) : (
                hunk.lines.map((line, index) => (
                  <UnifiedRow key={index} line={line} />
                ))
              )}
            </div>
          ))}
        </div>
      )}

      {overBudget && (
        <div className="diff-file__more">
          <Button onClick={() => setExpanded(true)}>
            Mostrar as {total - LINE_BUDGET} linhas restantes
          </Button>
        </div>
      )}
    </section>
  )
}

/**
 * Leitura do diff em dois modos, como as ferramentas que o usuário já conhece:
 * unificado (uma coluna, cabe no inspetor estreito) e lado a lado (antes à
 * esquerda, depois à direita — precisa de largura, e a junta do deck agora
 * permite conseguir essa largura).
 *
 * Nada aqui interpreta Git: o parser é puro e o componente só desenha.
 */
export default function DiffView({ source }: DiffViewProps) {
  const [mode, setMode] = useState<DiffMode>('unified')
  const parsed = useMemo(() => parseUnifiedDiff(source), [source])

  if (parsed.empty) {
    return (
      <p className="diff-empty">
        Nenhuma alteração de conteúdo neste commit.
      </p>
    )
  }

  return (
    <div className="diff-view">
      <div className="diff-view__bar">
        <span className="diff-view__totals">
          <b data-kind="add">+{parsed.additions}</b>
          <b data-kind="del">−{parsed.deletions}</b>
          <TechnicalLabel tone="faint" size="micro">
            {parsed.files.length === 1
              ? '1 arquivo'
              : `${parsed.files.length} arquivos`}
          </TechnicalLabel>
        </span>

        <span className="diff-view__modes" role="group" aria-label="Modo de leitura do diff">
          <button
            className="diff-view__mode"
            type="button"
            data-active={mode === 'unified'}
            aria-pressed={mode === 'unified'}
            onClick={() => setMode('unified')}
          >
            UNIFICADO
          </button>
          <button
            className="diff-view__mode"
            type="button"
            data-active={mode === 'split'}
            aria-pressed={mode === 'split'}
            onClick={() => setMode('split')}
          >
            LADO A LADO
          </button>
        </span>
      </div>

      {parsed.files.map((file) => (
        <FileDiff key={file.path + file.previousPath} file={file} mode={mode} />
      ))}
    </div>
  )
}
