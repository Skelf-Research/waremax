import { ref, onMounted, onUnmounted } from 'vue'
import type { MapData, NodeData, RobotState, StationState } from '../stores/simulation'

const ROBOT_COLORS: Record<string, string> = {
  Idle: '#94a3b8',
  Moving: '#3b82f6',
  Servicing: '#22c55e',
  WaitingNode: '#f59e0b',
  WaitingEdge: '#f59e0b',
  Charging: '#a855f7',
  Failed: '#ef4444',
  Maintenance: '#f97316',
}

const NODE_COLORS: Record<string, string> = {
  Aisle: '#334155',
  Rack: '#92400e',      // Amber/brown for storage racks - distinct from aisles
  StationPick: '#22c55e',
  StationDrop: '#3b82f6',
  StationInbound: '#f59e0b',
  StationOutbound: '#a855f7',
  Charging: '#eab308',
  Maintenance: '#f97316',
}

export function useWarehouseCanvas(
  canvasRef: ReturnType<typeof ref<HTMLCanvasElement | null>>,
  mapData: ReturnType<typeof ref<MapData | null>>,
  robots: ReturnType<typeof ref<RobotState[]>>,
  stations: ReturnType<typeof ref<StationState[]>>,
) {
  let animationId: number | null = null
  let nodePositions = new Map<number, { x: number; y: number }>()
  let stationNodeSet = new Set<number>()

  const padding = 40
  let scale = 1
  let offsetX = 0
  let offsetY = 0

  function computeTransform(canvas: HTMLCanvasElement, map: MapData) {
    const bounds = map.bounds
    const mapW = bounds.max_x - bounds.min_x
    const mapH = bounds.max_y - bounds.min_y

    const canvasW = canvas.width - padding * 2
    const canvasH = canvas.height - padding * 2

    scale = Math.min(canvasW / mapW, canvasH / mapH)
    offsetX = padding + (canvasW - mapW * scale) / 2
    offsetY = padding + (canvasH - mapH * scale) / 2

    // Cache node positions
    nodePositions.clear()
    for (const node of map.nodes) {
      nodePositions.set(node.id, {
        x: offsetX + (node.x - bounds.min_x) * scale,
        y: offsetY + (node.y - bounds.min_y) * scale,
      })
    }
  }

  function render() {
    const canvas = canvasRef.value
    const map = mapData.value

    // Always schedule next frame to keep the loop alive
    animationId = requestAnimationFrame(render)

    // Early return if no canvas or map, but loop continues
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Handle high-DPI displays
    const dpr = window.devicePixelRatio || 1
    const rect = canvas.getBoundingClientRect()

    // Skip rendering if canvas has no size yet
    if (rect.width === 0 || rect.height === 0) return

    canvas.width = rect.width * dpr
    canvas.height = rect.height * dpr
    ctx.scale(dpr, dpr)

    // Scale back for drawing
    const drawScale = 1 / dpr
    ctx.save()
    ctx.scale(drawScale, drawScale)

    // Clear with dark background
    ctx.fillStyle = '#0f172a'
    ctx.fillRect(0, 0, canvas.width, canvas.height)

    // If no map data yet, show loading state
    if (!map || map.nodes.length === 0) {
      ctx.fillStyle = '#64748b'
      ctx.font = '16px sans-serif'
      ctx.textAlign = 'center'
      ctx.fillText('Waiting for map data...', canvas.width / (2 * dpr), canvas.height / (2 * dpr))
      ctx.restore()
      return
    }

    computeTransform(canvas, map)

    // Draw edges
    drawEdges(ctx, map)

    // Draw nodes
    drawNodes(ctx, map)

    // Draw station labels
    drawStationLabels(ctx)

    // Draw robots (animated)
    drawRobots(ctx)

    ctx.restore()
  }

  function drawEdges(ctx: CanvasRenderingContext2D, map: MapData) {
    ctx.strokeStyle = '#1e293b'
    ctx.lineWidth = 2

    for (const edge of map.edges) {
      const from = nodePositions.get(edge.from)
      const to = nodePositions.get(edge.to)
      if (!from || !to) continue

      ctx.beginPath()
      ctx.moveTo(from.x, from.y)
      ctx.lineTo(to.x, to.y)
      ctx.stroke()
    }
  }

  function drawNodes(ctx: CanvasRenderingContext2D, map: MapData) {
    stationNodeSet.clear()
    for (const s of stations.value) {
      stationNodeSet.add(s.node_id)
    }

    for (const node of map.nodes) {
      const pos = nodePositions.get(node.id)
      if (!pos) continue

      const isStation = stationNodeSet.has(node.id)
      const radius = isStation ? 10 : 4
      const color = NODE_COLORS[node.node_type] || '#334155'

      ctx.beginPath()
      ctx.arc(pos.x, pos.y, radius, 0, Math.PI * 2)

      if (isStation) {
        // Station nodes get a glow effect
        ctx.fillStyle = color
        ctx.fill()
        ctx.strokeStyle = color
        ctx.lineWidth = 2
        ctx.globalAlpha = 0.3
        ctx.beginPath()
        ctx.arc(pos.x, pos.y, radius + 4, 0, Math.PI * 2)
        ctx.stroke()
        ctx.globalAlpha = 1
      } else {
        ctx.fillStyle = color
        ctx.fill()
      }
    }
  }

  function drawStationLabels(ctx: CanvasRenderingContext2D) {
    ctx.font = '10px monospace'
    ctx.textAlign = 'center'
    ctx.fillStyle = '#94a3b8'

    for (const s of stations.value) {
      const pos = nodePositions.get(s.node_id)
      if (!pos) continue

      ctx.fillText(s.name, pos.x, pos.y - 16)

      // Queue indicator
      if (s.queue_length > 0) {
        ctx.fillStyle = '#f59e0b'
        ctx.fillText(`Q:${s.queue_length}`, pos.x, pos.y + 20)
        ctx.fillStyle = '#94a3b8'
      }
    }
  }

  function drawRobots(ctx: CanvasRenderingContext2D) {
    for (const robot of robots.value) {
      // Use exact node position (no interpolation - shows true simulation state)
      const pos = nodePositions.get(robot.node_id)
      if (!pos) continue

      const state = robot.is_failed ? 'Failed' : robot.state
      const color = ROBOT_COLORS[state] || '#94a3b8'
      const radius = 7

      // Robot body
      ctx.beginPath()
      ctx.arc(pos.x, pos.y, radius, 0, Math.PI * 2)
      ctx.fillStyle = color
      ctx.fill()

      // Robot border
      ctx.strokeStyle = '#fff'
      ctx.lineWidth = 1.5
      ctx.stroke()

      // Robot ID
      ctx.font = 'bold 8px monospace'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillStyle = '#fff'
      ctx.fillText(String(robot.id), pos.x, pos.y)

      // Battery indicator (small bar below robot)
      if (robot.battery_soc !== null) {
        const barW = 12
        const barH = 3
        const barX = pos.x - barW / 2
        const barY = pos.y + radius + 3

        ctx.fillStyle = '#1e293b'
        ctx.fillRect(barX, barY, barW, barH)

        const soc = robot.battery_soc
        const barColor = soc > 0.5 ? '#22c55e' : soc > 0.2 ? '#f59e0b' : '#ef4444'
        ctx.fillStyle = barColor
        ctx.fillRect(barX, barY, barW * soc, barH)
      }
    }
  }

  function startRendering() {
    if (animationId) cancelAnimationFrame(animationId)
    render()
  }

  function stopRendering() {
    if (animationId) {
      cancelAnimationFrame(animationId)
      animationId = null
    }
  }

  onMounted(() => {
    startRendering()
  })

  onUnmounted(() => {
    stopRendering()
  })

  return { startRendering, stopRendering }
}
