export interface FocusableGraphEdge {
  id: string
  source: string
  target: string
}

export type GraphFocusLevel = 'selected' | 'connected' | 'dimmed' | null

export interface GraphFocusState {
  selectedNodeId: string
  connectedNodeIds: ReadonlySet<string>
  connectedEdgeIds: ReadonlySet<string>
}

const INTERACTION_CLASSES = new Set([
  'graph-focus--selected',
  'graph-focus--connected',
  'graph-focus--dimmed',
  'graph-label--nearby',
])

/**
 * Derive one-hop focus strictly from rendered graph edges. No visual
 * relationship can appear unless it already exists in the Git graph adapter.
 */
export function createGraphFocus(
  selectedNodeId: string | null,
  edges: FocusableGraphEdge[],
): GraphFocusState | null {
  if (!selectedNodeId) {
    return null
  }

  const connectedNodeIds = new Set<string>()
  const connectedEdgeIds = new Set<string>()

  for (const edge of edges) {
    if (edge.source === selectedNodeId) {
      connectedNodeIds.add(edge.target)
      connectedEdgeIds.add(edge.id)
    } else if (edge.target === selectedNodeId) {
      connectedNodeIds.add(edge.source)
      connectedEdgeIds.add(edge.id)
    }
  }

  return { selectedNodeId, connectedNodeIds, connectedEdgeIds }
}

export function getNodeFocusLevel(
  nodeId: string,
  focus: GraphFocusState | null,
): GraphFocusLevel {
  if (!focus) {
    return null
  }
  if (nodeId === focus.selectedNodeId) {
    return 'selected'
  }
  return focus.connectedNodeIds.has(nodeId) ? 'connected' : 'dimmed'
}

export function getEdgeFocusLevel(
  edgeId: string,
  focus: GraphFocusState | null,
): GraphFocusLevel {
  if (!focus) {
    return null
  }
  return focus.connectedEdgeIds.has(edgeId) ? 'connected' : 'dimmed'
}

/** Replace only interaction classes, preserving semantic edge/node classes. */
export function withGraphInteractionClasses(
  className: string | undefined,
  focusLevel: GraphFocusLevel,
  labelNearby = false,
): string {
  const classes = (className ?? '')
    .split(/\s+/)
    .filter((value) => value && !INTERACTION_CLASSES.has(value))

  if (focusLevel) {
    classes.push(`graph-focus--${focusLevel}`)
  }
  if (labelNearby) {
    classes.push('graph-label--nearby')
  }

  return classes.join(' ')
}
