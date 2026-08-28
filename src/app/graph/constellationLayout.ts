export interface ConstellationCommit {
  id: string
  parentIds: string[]
}

export interface ConstellationBranch {
  id: string
  headId: string
  current: boolean
}

export interface ConstellationPoint {
  x: number
  y: number
}

interface SimulatedPoint extends ConstellationPoint {
  vx: number
  vy: number
}

export interface ConstellationLayoutResult {
  commitPositions: Map<string, ConstellationPoint>
  branchPositions: Map<string, ConstellationPoint>
  projectPosition: ConstellationPoint
}

const EDGE_LENGTH = 112
const NODE_CLEARANCE = 126
const LANE_WIDTH = 118
const RANK_HEIGHT = 82
const ITERATIONS = 150

function laneNumber(index: number): number {
  const distance = Math.floor(index / 2) + 1
  return index % 2 === 0 ? distance : -distance
}

function deterministicNoise(value: string, salt: number): number {
  let hash = 2166136261 ^ salt

  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }

  return ((hash >>> 0) / 4294967295) * 2 - 1
}

/**
 * Produce a compact, deterministic topology without inventing edges.
 *
 * Real parent relationships act as springs, while rank/lane anchors are weak
 * enough to let long merge paths fold into a constellation instead of forcing
 * a wide flowchart. No animation runs here; this is a bounded layout pass when
 * Git data changes.
 */
export function layoutConstellation(
  commits: ConstellationCommit[],
  branches: ConstellationBranch[],
): ConstellationLayoutResult {
  const commitById = new Map(commits.map((commit) => [commit.id, commit]))
  const rankByCommit = new Map<string, number>()

  function getRank(commitId: string, visiting = new Set<string>()): number {
    const cachedRank = rankByCommit.get(commitId)
    if (cachedRank !== undefined) {
      return cachedRank
    }

    if (visiting.has(commitId)) {
      return 0
    }

    const commit = commitById.get(commitId)
    if (!commit) {
      return 0
    }

    const nextVisiting = new Set(visiting).add(commitId)
    const parentRanks = commit.parentIds
      .filter((parentId) => commitById.has(parentId))
      .map((parentId) => getRank(parentId, nextVisiting))
    const rank = parentRanks.length === 0 ? 0 : Math.max(...parentRanks) + 1
    rankByCommit.set(commitId, rank)
    return rank
  }

  commits.forEach((commit) => getRank(commit.id))

  const laneByCommit = new Map<string, number>()
  let allocatedLaneCount = 0

  function assignFirstParentRoute(startId: string, lane: number): void {
    let currentId: string | undefined = startId

    while (currentId && commitById.has(currentId) && !laneByCommit.has(currentId)) {
      laneByCommit.set(currentId, lane)
      currentId = commitById.get(currentId)?.parentIds[0]
    }
  }

  const orderedBranches = [...branches].sort((left, right) => {
    if (left.current !== right.current) {
      return left.current ? -1 : 1
    }

    return left.id.localeCompare(right.id)
  })
  const principalHead = orderedBranches.find((branch) => branch.current)?.headId
    ?? commits[0]?.id

  if (principalHead) {
    assignFirstParentRoute(principalHead, 0)
  }

  for (const branch of orderedBranches) {
    if (commitById.has(branch.headId) && !laneByCommit.has(branch.headId)) {
      assignFirstParentRoute(branch.headId, laneNumber(allocatedLaneCount++))
    }
  }

  const newestFirst = [...commits].sort((left, right) => (
    getRank(right.id) - getRank(left.id) || left.id.localeCompare(right.id)
  ))

  for (const commit of newestFirst) {
    for (const secondaryParentId of commit.parentIds.slice(1)) {
      if (commitById.has(secondaryParentId) && !laneByCommit.has(secondaryParentId)) {
        assignFirstParentRoute(
          secondaryParentId,
          laneNumber(allocatedLaneCount++),
        )
      }
    }
  }

  for (const commit of newestFirst) {
    if (!laneByCommit.has(commit.id)) {
      assignFirstParentRoute(commit.id, laneNumber(allocatedLaneCount++))
    }
  }

  const points = new Map<string, SimulatedPoint>()
  for (const commit of commits) {
    const rank = getRank(commit.id)
    const lane = laneByCommit.get(commit.id) ?? 0
    points.set(commit.id, {
      x: lane * LANE_WIDTH
        + rank * 16
        + deterministicNoise(commit.id, 17) * 15,
      y: -rank * RANK_HEIGHT
        + deterministicNoise(commit.id, 41) * 13,
      vx: 0,
      vy: 0,
    })
  }

  const realEdges = commits.flatMap((commit) => commit.parentIds
    .filter((parentId) => commitById.has(parentId))
    .map((parentId) => ({ parentId, childId: commit.id })))

  for (let iteration = 0; iteration < ITERATIONS; iteration += 1) {
    const forceById = new Map(
      commits.map((commit) => [commit.id, { x: 0, y: 0 }]),
    )

    // Pull every real parent/child pair toward a compact, uniform segment.
    for (const edge of realEdges) {
      const parent = points.get(edge.parentId)
      const child = points.get(edge.childId)
      const parentForce = forceById.get(edge.parentId)
      const childForce = forceById.get(edge.childId)
      if (!parent || !child || !parentForce || !childForce) {
        continue
      }

      const dx = child.x - parent.x
      const dy = child.y - parent.y
      const distance = Math.max(1, Math.hypot(dx, dy))
      const spring = Math.max(-7, Math.min(7, (distance - EDGE_LENGTH) * 0.055))
      const fx = (dx / distance) * spring
      const fy = (dy / distance) * spring
      parentForce.x += fx
      parentForce.y += fy
      childForce.x -= fx
      childForce.y -= fy

      // Children should remain generally above parents, but only a small gap is
      // enforced so unequal merge paths can fold rather than create giant arcs.
      const upwardGap = parent.y - child.y
      if (upwardGap < 30) {
        const correction = (30 - upwardGap) * 0.045
        parentForce.y += correction
        childForce.y -= correction
      }
    }

    // Local repulsion protects small labels without expanding the whole graph.
    for (let leftIndex = 0; leftIndex < commits.length; leftIndex += 1) {
      for (let rightIndex = leftIndex + 1; rightIndex < commits.length; rightIndex += 1) {
        const left = points.get(commits[leftIndex].id)
        const right = points.get(commits[rightIndex].id)
        const leftForce = forceById.get(commits[leftIndex].id)
        const rightForce = forceById.get(commits[rightIndex].id)
        if (!left || !right || !leftForce || !rightForce) {
          continue
        }

        let dx = right.x - left.x
        let dy = right.y - left.y
        let distance = Math.hypot(dx, dy)

        if (distance < 0.01) {
          dx = deterministicNoise(commits[leftIndex].id, rightIndex) || 0.5
          dy = deterministicNoise(commits[rightIndex].id, leftIndex) || -0.5
          distance = Math.hypot(dx, dy)
        }

        if (distance >= NODE_CLEARANCE) {
          continue
        }

        const repulsion = Math.min(5, (NODE_CLEARANCE - distance) * 0.052)
        const fx = (dx / distance) * repulsion
        const fy = (dy / distance) * repulsion
        leftForce.x -= fx
        leftForce.y -= fy
        rightForce.x += fx
        rightForce.y += fy
      }
    }

    for (const commit of commits) {
      const point = points.get(commit.id)
      const force = forceById.get(commit.id)
      if (!point || !force) {
        continue
      }

      const rank = getRank(commit.id)
      const lane = laneByCommit.get(commit.id) ?? 0
      const laneAnchor = lane * LANE_WIDTH + rank * 16
      const rankAnchor = -rank * RANK_HEIGHT
      force.x += (laneAnchor - point.x) * 0.008
      force.y += (rankAnchor - point.y) * 0.006
      force.x += -point.x * 0.0015
      force.y += -point.y * 0.0015

      point.vx = (point.vx + force.x) * 0.76
      point.vy = (point.vy + force.y) * 0.76
      const speed = Math.hypot(point.vx, point.vy)
      if (speed > 9) {
        point.vx = (point.vx / speed) * 9
        point.vy = (point.vy / speed) * 9
      }
      point.x += point.vx
      point.y += point.vy
    }
  }

  const commitPositions = new Map<string, ConstellationPoint>()
  const centerX = points.size > 0
    ? [...points.values()].reduce((sum, point) => sum + point.x, 0) / points.size
    : 0
  const centerY = points.size > 0
    ? [...points.values()].reduce((sum, point) => sum + point.y, 0) / points.size
    : 0

  for (const [id, point] of points) {
    commitPositions.set(id, { x: point.x - centerX, y: point.y - centerY })
  }

  const branchesByHead = new Map<string, ConstellationBranch[]>()
  for (const branch of orderedBranches) {
    const siblings = branchesByHead.get(branch.headId) ?? []
    siblings.push(branch)
    branchesByHead.set(branch.headId, siblings)
  }

  const branchPositions = new Map<string, ConstellationPoint>()
  for (const [headId, headBranches] of branchesByHead) {
    const head = commitPositions.get(headId) ?? { x: 0, y: 0 }
    headBranches.forEach((branch, index) => {
      const offset = index - (headBranches.length - 1) / 2
      branchPositions.set(branch.id, {
        x: head.x + offset * 42 - 12,
        y: head.y - 72 - Math.abs(offset) * 18,
      })
    })
  }

  const positionedCommits = [...commitPositions.values()]
  const minimumX = positionedCommits.length > 0
    ? Math.min(...positionedCommits.map((point) => point.x))
    : 0
  const verticalCenter = positionedCommits.length > 0
    ? (
      Math.min(...positionedCommits.map((point) => point.y))
      + Math.max(...positionedCommits.map((point) => point.y))
    ) / 2
    : 0

  return {
    commitPositions,
    branchPositions,
    projectPosition: { x: minimumX - 172, y: verticalCenter },
  }
}
