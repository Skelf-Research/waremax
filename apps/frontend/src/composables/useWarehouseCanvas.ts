import { ref, onMounted, onUnmounted, watch, type Ref } from 'vue'
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
  Rack: '#92400e',
  StationPick: '#22c55e',
  StationDrop: '#3b82f6',
  StationInbound: '#f59e0b',
  StationOutbound: '#a855f7',
  Charging: '#eab308',
  Maintenance: '#f97316',
}

export function useWarehouseCanvas(
  canvasRef: Ref<HTMLCanvasElement | null>,
  mapData: Ref<MapData | null>,
  robots: Ref<RobotState[]>,
  stations: Ref<StationState[]>,
) {
  let animationId: number | null = null
  let nodePositions = new Map<number, { x: number; y: number }>()
  let stationNodeSet = new Set<number>()
  let resizeObserver: ResizeObserver | null = null

  const padding = 40
  let scale = 1
  let offsetX = 0
  let offsetY = 0
  let canvasWidth = 0
  let canvasHeight = 0
  let dpr = 1

  function computeTransform(map: MapData) {
    const bounds = map.bounds
    const mapW = bounds.max_x - bounds.min_x
    const mapH = bounds.max_y - bounds.min_y

    const canvasW = canvasWidth - padding * 2
    const canvasH = canvasHeight - padding * 2

    scale = Math.min(canvasW / mapW, canvasH / mapH)
    offsetX = padding + (canvasW - mapW * scale) / 2
    offsetY = padding + (canvasH - mapH * scale) / 2

    nodePositions.clear()
    for (const node of map.nodes) {
      nodePositions.set(node.id, {
        x: offsetX + (node.x - bounds.min_x) * scale,
        y: offsetY + (node.y - bounds.min_y) * scale,
      })
    }
  }

  function resizeCanvas() {
    const canvas = canvasRef.value
    if (!canvas) return

    const rect = canvas.getBoundingClientRect()
    if (rect.width === 0 || rect.height === 0) return

    dpr = window.devicePixelRatio || 1
    canvasWidth = rect.width
    canvasHeight = rect.height

    if (canvas.width !== Math.floor(canvasWidth * dpr) || canvas.height !== Math.floor(canvasHeight * dpr)) {
      canvas.width = Math.floor(canvasWidth * dpr)
      canvas.height = Math.floor(canvasHeight * dpr)
    }
  }

  function render() {
    animationId = requestAnimationFrame(render)

    const canvas = canvasRef.value
    const map = mapData.value
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    if (canvasWidth === 0 || canvasHeight === 0) {
      resizeCanvas()
      if (canvasWidth === 0 || canvasHeight === 0) return
    }

    ctx.save()
    ctx.scale(dpr, dpr)

    ctx.fillStyle = '#0f172a'
    ctx.fillRect(0, 0, canvasWidth, canvasHeight)

    if (!map || map.nodes.length === 0) {
      ctx.fillStyle = '#64748b'
      ctx.font = '16px sans-serif'
      ctx.textAlign = 'center'
      ctx.fillText('Waiting for map data...', canvasWidth / 2, canvasHeight / 2)
      ctx.restore()
      return
    }

    computeTransform(map)
    drawEdges(ctx, map)
    drawNodes(ctx, map)
    drawStationLabels(ctx)
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
        ctx.fillStyle = color
        ctx.fill()
        ctx.save()
        ctx.strokeStyle = color
        ctx.lineWidth = 2
        ctx.globalAlpha = 0.3
        ctx.beginPath()
        ctx.arc(pos.x, pos.y, radius + 4, 0, Math.PI * 2)
        ctx.stroke()
        ctx.restore()
      } else {
        ctx.fillStyle = color
        ctx.fill()
      }
    }
  }

  function drawStationLabels(ctx: CanvasRenderingContext2D) {
    ctx.font = '10px monospace'
    ctx.textAlign = 'center'
    ctx.textBaseline = 'alphabetic'
    ctx.fillStyle = '#94a3b8'

    for (const s of stations.value) {
      const pos = nodePositions.get(s.node_id)
      if (!pos) continue

      ctx.fillText(s.name, pos.x, pos.y - 16)

      if (s.queue_length > 0) {
        ctx.fillStyle = '#f59e0b'
        ctx.fillText(`Q:${s.queue_length}`, pos.x, pos.y + 20)
        ctx.fillStyle = '#94a3b8'
      }
    }
  }

  function drawRobots(ctx: CanvasRenderingContext2D) {
    for (const robot of robots.value) {
      let pos = nodePositions.get(robot.node_id)
      if (!pos) continue

      const state = robot.is_failed ? 'Failed' : robot.state
      const color = ROBOT_COLORS[state] || '#94a3b8'
      const radius = 7

      ctx.beginPath()
      ctx.arc(pos.x, pos.y, radius, 0, Math.PI * 2)
      ctx.fillStyle = color
      ctx.fill()

      ctx.strokeStyle = '#fff'
      ctx.lineWidth = 1.5
      ctx.stroke()

      ctx.font = 'bold 8px monospace'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillStyle = '#fff'
      ctx.fillText(String(robot.id), pos.x, pos.y)

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
    resizeCanvas()
    render()
  }

  function stopRendering() {
    if (animationId) {
      cancelAnimationFrame(animationId)
      animationId = null
    }
  }

  onMounted(() => {
    const canvas = canvasRef.value
    if (canvas && 'ResizeObserver' in window) {
      resizeObserver = new ResizeObserver(() => {
        resizeCanvas()
      })
      resizeObserver.observe(canvas)
    }
    requestAnimationFrame(() => {
      resizeCanvas()
      startRendering()
    })
  })

  onUnmounted(() => {
    stopRendering()
    if (resizeObserver) {
      resizeObserver.disconnect()
      resizeObserver = null
    }
  })

  return { startRendering, stopRendering }
}
