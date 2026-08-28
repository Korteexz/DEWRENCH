import type { ReactNode } from 'react'

import type { ProjectOpenResult } from '../types/project'
import type { GitRepositoryDetails } from '../types/repository'

interface GitInspectorPaneProps {
  project: ProjectOpenResult
  details: GitRepositoryDetails | null
  children?: ReactNode
}

/** Persistent inspector geometry prevents contextual details from feeling like a floating card. */
export default function GitInspectorPane({
  project,
  details,
  children,
}: GitInspectorPaneProps) {
  return (
    <aside className="git-inspector-pane" aria-label="Inspetor contextual">
      <div className="panel-terminal-label">
        <span>03</span>
        <strong>CONTEXT INSPECTOR</strong>
        <i className="status-dot status-dot--amber" />
      </div>

      {children ?? (
        <div className="inspector-standby">
          <span className="inspector-standby__reticle" aria-hidden="true"><i /></span>
          <p>SELECT A GRAPH OBJECT</p>
          <small>Commit, branch and repository metadata will resolve here.</small>

          <dl>
            <div><dt>PROJECT</dt><dd>{project.name}</dd></div>
            <div><dt>BRANCH</dt><dd>{details?.branch ?? '—'}</dd></div>
            <div><dt>CHANGES</dt><dd>{details?.files.length ?? '—'}</dd></div>
            <div><dt>COMMITS</dt><dd>{details?.commits.length ?? '—'}</dd></div>
          </dl>
        </div>
      )}
    </aside>
  )
}
