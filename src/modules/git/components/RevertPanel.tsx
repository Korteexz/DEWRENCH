import type { GitGraphCommit } from '../types/repository'
import type {
  GitFailure,
  GitRevertOutcome,
  GitRevertPreview,
} from '../types/revert'
import { describeFailure, isGitOperationError } from '../types/revert'

interface RevertPanelProps {
  commit: GitGraphCommit
  preview: GitRevertPreview | null
  loading: boolean
  busy: boolean
  failure: GitFailure | null
  outcome: GitRevertOutcome | null
  onRequestPreview: () => void
  onCancel: () => void
  onConfirm: () => void
}

function describeFile(status: string): string {
  if (status.startsWith('A')) {
    return 'adicionado'
  }

  if (status.startsWith('D')) {
    return 'removido'
  }

  if (status.startsWith('R')) {
    return 'renomeado'
  }

  return 'modificado'
}

/**
 * Confirmação de Revert dentro do inspetor de commit.
 *
 * O painel não executa Git: ele apenas apresenta o preview calculado pelo
 * backend e devolve a intenção do usuário. Nenhum estado é fabricado aqui.
 */
export default function RevertPanel({
  commit,
  preview,
  loading,
  busy,
  failure,
  outcome,
  onRequestPreview,
  onCancel,
  onConfirm,
}: RevertPanelProps) {
  const typedFailure = isGitOperationError(failure) ? failure : null
  const failureMessage = failure === null ? null : describeFailure(failure)

  return (
    <section className="inspector-section">
      <div className="inspector-section__heading">
        <h2>Revert</h2>
        <button
          type="button"
          onClick={onRequestPreview}
          disabled={busy || loading || preview !== null}
        >
          {loading ? 'Analisando…' : 'Reverter commit'}
        </button>
      </div>

      {loading && (
        <p className="inspector-empty">Lendo consequências reais no repositório…</p>
      )}

      {preview && (
        <div className="revert-panel" role="group" aria-label="Confirmação de revert">
          <strong className="revert-panel__title">
            Reverter commit {preview.short_hash}
          </strong>

          <p className="revert-panel__note">
            Um novo commit será criado desfazendo as alterações introduzidas por este commit.
          </p>
          <p className="revert-panel__note">
            O commit original e o restante do histórico serão preservados.
          </p>

          {preview.warnings.map((warning) => (
            <p className="revert-panel__warning" key={warning}>{warning}</p>
          ))}

          <h3 className="revert-panel__label">
            Arquivos afetados ({preview.affected_files.length})
          </h3>
          {preview.affected_files.length === 0 ? (
            <p className="inspector-empty">Este commit não altera arquivos.</p>
          ) : (
            <ul className="revert-panel__list">
              {preview.affected_files.map((file) => (
                <li key={`${file.status}:${file.path}`}>
                  <code>{describeFile(file.status)}</code>
                  <span title={file.path}>{file.path}</span>
                </li>
              ))}
            </ul>
          )}

          <h3 className="revert-panel__label">
            Alterações locais não relacionadas ({preview.preserved_local_changes.length})
          </h3>
          {preview.preserved_local_changes.length === 0 ? (
            <p className="inspector-empty">Nenhuma. O working tree está limpo.</p>
          ) : (
            <ul className="revert-panel__list">
              {preview.preserved_local_changes.map((path) => (
                <li key={path}>
                  <code>preservado</code>
                  <span title={path}>{path}</span>
                </li>
              ))}
            </ul>
          )}

          <div className="revert-panel__actions">
            <button type="button" onClick={onCancel} disabled={busy}>
              Cancelar
            </button>
            <button
              className="inspector-button--primary"
              type="button"
              onClick={onConfirm}
              disabled={busy}
            >
              {busy ? 'Revertendo…' : 'Criar commit de Revert'}
            </button>
          </div>
        </div>
      )}

      {outcome && (
        <div className="revert-panel revert-panel--success" role="status">
          <strong className="revert-panel__title">
            Revert criado: {outcome.new_commit_short_hash}
          </strong>
          <p className="revert-panel__note">{outcome.new_commit_subject}</p>
          <p className="revert-panel__note">
            O commit {outcome.reverted_short_hash} continua no histórico.
          </p>
        </div>
      )}

      {failureMessage && (
        <div className="inspector-error" role="alert">
          <p>{failureMessage}</p>

          {typedFailure && (
            <>
              <code className="revert-error__code">{typedFailure.code}</code>

              {typedFailure.affectedFiles.length > 0 && (
                <ul className="revert-panel__list">
                  {typedFailure.affectedFiles.map((path) => (
                    <li key={path}><span title={path}>{path}</span></li>
                  ))}
                </ul>
              )}

              {typedFailure.suggestedAction && (
                <p className="revert-error__action">{typedFailure.suggestedAction}</p>
              )}

              {typedFailure.details && (
                <details className="revert-error__details">
                  <summary>Detalhes técnicos</summary>
                  <pre>{typedFailure.details}</pre>
                </details>
              )}
            </>
          )}
        </div>
      )}

      <p className="revert-panel__hint">
        Equivalente a <code>git revert --no-edit {commit.short_hash}</code>.
      </p>
    </section>
  )
}
