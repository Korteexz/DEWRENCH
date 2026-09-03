import { useState } from 'react'

import { Button, SectionHeader, TechnicalLabel } from '../../../design'
import {
  addRemote,
  removeRemote,
  renameRemote,
  setRemoteUrl,
} from '../services/gitServices'
import type { GitRemote, GitRemotesView } from '../types/remote'
import {
  describeFailure,
  isGitOperationError,
  toGitFailure,
  type GitFailure,
} from '../types/revert'

interface RemotesPanelProps {
  projectPath: string
  remotes: GitRemotesView | null
  busy: boolean
  onChanged: () => void
}

type Draft =
  | { kind: 'none' }
  | { kind: 'add' }
  | { kind: 'rename'; remote: string }
  | { kind: 'url'; remote: string }
  | { kind: 'remove'; remote: string }

/**
 * Configuração de remotes.
 *
 * Duas regras de segurança são visíveis na interface, não só no backend:
 * mudança permanente exige ação explícita (nada é salvo enquanto se digita), e
 * alterar o destino de `origin` mostra a consequência antes de aplicar — é a
 * configuração que decide para onde todo push futuro vai.
 */
export default function RemotesPanel({
  projectPath,
  remotes,
  busy,
  onChanged,
}: RemotesPanelProps) {
  const [draft, setDraft] = useState<Draft>({ kind: 'none' })
  const [name, setName] = useState('')
  const [url, setUrl] = useState('')
  const [running, setRunning] = useState(false)
  const [failure, setFailure] = useState<GitFailure | null>(null)

  const list = remotes?.remotes ?? []
  const disabled = busy || running

  function reset() {
    setDraft({ kind: 'none' })
    setName('')
    setUrl('')
    setFailure(null)
  }

  async function run(operation: () => Promise<void>) {
    setRunning(true)
    setFailure(null)

    try {
      await operation()
      reset()
      onChanged()
    } catch (error) {
      setFailure(toGitFailure(error))
    } finally {
      setRunning(false)
    }
  }

  function startEdit(next: Draft, remote?: GitRemote) {
    setFailure(null)
    setDraft(next)
    setName(remote?.name ?? '')
    setUrl(remote?.fetch_url ?? '')
  }

  return (
    <section className="inspector-section">
      <SectionHeader
        title="Remotes"
        readout={String(list.length).padStart(2, '0')}
        actions={(
          <Button
            onClick={() => (draft.kind === 'add' ? reset() : startEdit({ kind: 'add' }))}
            disabled={disabled}
          >
            {draft.kind === 'add' ? 'Cancelar' : 'Adicionar'}
          </Button>
        )}
      />

      {list.length === 0 && draft.kind !== 'add' && (
        <p className="inspector-empty">
          Nenhum remote configurado neste repositório.
        </p>
      )}

      <ul className="remote-list">
        {list.map((remote) => (
          <li className="remote-item" key={remote.name}>
            <div className="remote-item__head">
              <strong>{remote.name}</strong>
              {remote.is_origin && <span className="remote-item__tag">ORIGIN</span>}
              {remote.is_upstream && (
                <span className="remote-item__tag" data-tone="upstream">UPSTREAM</span>
              )}
              {remote.identity.provider !== 'unknown' && (
                <span className="remote-item__provider">{remote.identity.provider}</span>
              )}
            </div>

            <dl className="remote-item__urls">
              <div>
                <dt>fetch</dt>
                <dd title={remote.fetch_url}>{remote.fetch_url}</dd>
              </div>
              {remote.push_url !== remote.fetch_url && (
                <div>
                  <dt>push</dt>
                  <dd title={remote.push_url}>{remote.push_url}</dd>
                </div>
              )}
            </dl>

            <div className="remote-item__actions">
              <Button onClick={() => startEdit({ kind: 'rename', remote: remote.name }, remote)} disabled={disabled}>
                Renomear
              </Button>
              <Button onClick={() => startEdit({ kind: 'url', remote: remote.name }, remote)} disabled={disabled}>
                Trocar URL
              </Button>
              <Button variant="danger" onClick={() => startEdit({ kind: 'remove', remote: remote.name }, remote)} disabled={disabled}>
                Remover
              </Button>
            </div>

            {draft.kind === 'rename' && draft.remote === remote.name && (
              <div className="remote-form">
                <TechnicalLabel tone="low" size="micro">Novo nome</TechnicalLabel>
                <input
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="nome-do-remote"
                  disabled={disabled}
                />
                <div className="remote-form__actions">
                  <Button onClick={reset} disabled={disabled}>Cancelar</Button>
                  <Button
                    variant="primary"
                    onClick={() => void run(() => renameRemote(projectPath, remote.name, name))}
                    disabled={disabled || name.trim().length === 0 || name === remote.name}
                  >
                    Renomear
                  </Button>
                </div>
              </div>
            )}

            {draft.kind === 'url' && draft.remote === remote.name && (
              <div className="remote-form">
                <TechnicalLabel tone="low" size="micro">Nova URL</TechnicalLabel>
                <input
                  value={url}
                  onChange={(event) => setUrl(event.target.value)}
                  placeholder="https://github.com/owner/repo.git"
                  disabled={disabled}
                />
                {remote.is_origin && (
                  <p className="revert-panel__warning">
                    Isto muda o destino de todo push e fetch futuros deste repositório.
                  </p>
                )}
                <div className="remote-form__actions">
                  <Button onClick={reset} disabled={disabled}>Cancelar</Button>
                  <Button
                    variant="primary"
                    onClick={() => void run(() => setRemoteUrl(projectPath, remote.name, url, false))}
                    disabled={disabled || url.trim().length === 0 || url === remote.fetch_url}
                  >
                    Aplicar
                  </Button>
                </div>
              </div>
            )}

            {draft.kind === 'remove' && draft.remote === remote.name && (
              <div className="remote-form remote-form--danger">
                <p className="revert-panel__note">
                  Remover <strong>{remote.name}</strong> apaga a configuração local
                  do destino e as refs remotas conhecidas. Nada é apagado no
                  servidor, e os commits locais permanecem.
                </p>
                <div className="remote-form__actions">
                  <Button onClick={reset} disabled={disabled}>Cancelar</Button>
                  <Button
                    variant="danger"
                    onClick={() => void run(() => removeRemote(projectPath, remote.name))}
                    disabled={disabled}
                  >
                    Remover {remote.name}
                  </Button>
                </div>
              </div>
            )}
          </li>
        ))}
      </ul>

      {draft.kind === 'add' && (
        <div className="remote-form">
          <TechnicalLabel tone="low" size="micro">Nome</TechnicalLabel>
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="origin"
            disabled={disabled}
          />
          <TechnicalLabel tone="low" size="micro">URL</TechnicalLabel>
          <input
            value={url}
            onChange={(event) => setUrl(event.target.value)}
            placeholder="https://github.com/owner/repo.git"
            disabled={disabled}
          />
          <div className="remote-form__actions">
            <Button onClick={reset} disabled={disabled}>Cancelar</Button>
            <Button
              variant="primary"
              onClick={() => void run(() => addRemote(projectPath, name, url))}
              disabled={disabled || name.trim().length === 0 || url.trim().length === 0}
            >
              Adicionar remote
            </Button>
          </div>
        </div>
      )}

      {failure && (
        <div className="inspector-error" role="alert">
          <p>{describeFailure(failure)}</p>
          {isGitOperationError(failure) && (
            <>
              <code className="revert-error__code">{failure.code}</code>
              {failure.suggestedAction && (
                <p className="revert-error__action">{failure.suggestedAction}</p>
              )}
            </>
          )}
        </div>
      )}
    </section>
  )
}
