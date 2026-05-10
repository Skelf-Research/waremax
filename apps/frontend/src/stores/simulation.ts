import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

const API_BASE = (import.meta.env.PUBLIC_API_URL || '').replace(/\/$/, '')
const WS_BASE = API_BASE.replace(/^http/, 'ws')

export interface NodeData {
  id: number
  name: string
  x: number
  y: number
  node_type: string
}

export interface EdgeData {
  id: number
  from: number
  to: number
  length: number
  bidirectional: boolean
}

export interface MapBounds {
  min_x: number
  max_x: number
  min_y: number
  max_y: number
}

export interface MapData {
  nodes: NodeData[]
  edges: EdgeData[]
  bounds: MapBounds
}

export interface RobotState {
  id: number
  node_id: number
  state: string
  battery_soc: number | null
  current_task: number | null
  is_failed: boolean
}

export interface StationState {
  id: number
  name: string
  node_id: number
  station_type: string
  queue_length: number
  serving_count: number
  concurrency: number
}

export interface MetricsSnapshot {
  throughput_per_hour: number
  orders_completed: number
  orders_pending: number
  robot_utilization: number
  station_utilization: number
  avg_cycle_time_s: number
  late_orders: number
}

export interface SimulationState {
  status: string
  time_s: number
  speed: number
  events_processed: number
  orders_completed: number
  robots: RobotState[]
  stations: StationState[]
  metrics: MetricsSnapshot
}

export interface PresetInfo {
  name: string
  description: string
  robots: number
  stations: number
  order_rate: number
  duration_minutes: number
  grid_size: string
}

export type AppStatus = 'disconnected' | 'connecting' | 'idle' | 'running' | 'paused' | 'finished'

export const useSimulationStore = defineStore('simulation', () => {
  // Session state
  const sessionId = ref<string | null>(null)
  const appStatus = ref<AppStatus>('disconnected')
  const speed = ref(1.0)
  const simTime = ref(0)
  const eventsProcessed = ref(0)

  // Map
  const mapData = ref<MapData | null>(null)

  // Entities
  const robots = ref<RobotState[]>([])
  const stations = ref<StationState[]>([])

  // Metrics
  const metrics = ref<MetricsSnapshot>({
    throughput_per_hour: 0,
    orders_completed: 0,
    orders_pending: 0,
    robot_utilization: 0,
    station_utilization: 0,
    avg_cycle_time_s: 0,
    late_orders: 0,
  })

  // Presets
  const presets = ref<PresetInfo[]>([])

  // Event log (last N events for display)
  const eventLog = ref<string[]>([])
  const maxLogEntries = 100

  // WebSocket
  let ws: WebSocket | null = null

  // Computed
  const isConnected = computed(() => sessionId.value !== null)
  const isRunning = computed(() => appStatus.value === 'running')
  const isPaused = computed(() => appStatus.value === 'paused')
  const isFinished = computed(() => appStatus.value === 'finished')

  const robotCounts = computed(() => {
    const counts: Record<string, number> = {}
    for (const r of robots.value) {
      const state = r.is_failed ? 'Failed' : r.state
      counts[state] = (counts[state] || 0) + 1
    }
    return counts
  })

  const formattedTime = computed(() => {
    const totalSeconds = Math.floor(simTime.value)
    const hours = Math.floor(totalSeconds / 3600)
    const minutes = Math.floor((totalSeconds % 3600) / 60)
    const seconds = totalSeconds % 60
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
  })

  // Actions
  async function loadPresets() {
    try {
      const res = await fetch(`${API_BASE}/api/presets`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      presets.value = await res.json()
    } catch (e) {
      console.error('Failed to load presets:', e)
      addLog(`Error: Failed to load presets`)
    }
  }

  async function createSession(config: {
    preset: string
    robot_count?: number
    order_rate?: number
    duration_minutes?: number
    grid_rows?: number
    grid_cols?: number
  }) {
    const previousStatus = appStatus.value
    try {
      appStatus.value = 'connecting'
      const res = await fetch(`${API_BASE}/api/session`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      })
      if (!res.ok) {
        const err = await res.json().catch(() => ({}))
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      const data = await res.json()
      if (!data.session_id) throw new Error('No session_id in response')
      sessionId.value = data.session_id

      // Load map
      const mapRes = await fetch(`${API_BASE}/api/session/${sessionId.value}/map`)
      if (!mapRes.ok) throw new Error(`Map fetch failed: HTTP ${mapRes.status}`)
      mapData.value = await mapRes.json()

      // Connect WebSocket
      connectWebSocket()

      appStatus.value = 'paused'
      addLog('Session created')
    } catch (e) {
      appStatus.value = previousStatus === 'disconnected' ? 'disconnected' : 'idle'
      const msg = e instanceof Error ? e.message : String(e)
      addLog(`Error: Failed to create session: ${msg}`)
      console.error('Failed to create session:', e)
    }
  }

  function connectWebSocket() {
    if (!sessionId.value) return
    const wsUrl = `${WS_BASE}/ws/${sessionId.value}`
    ws = new WebSocket(wsUrl)

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data)
      handleWsMessage(msg)
    }

    ws.onclose = () => {
      if (appStatus.value !== 'disconnected') {
        addLog('WebSocket disconnected')
      }
      sessionId.value = null
      appStatus.value = 'disconnected'
      robots.value = []
      stations.value = []
      mapData.value = null
    }

    ws.onerror = () => {
      addLog('WebSocket error')
      sessionId.value = null
      appStatus.value = 'disconnected'
    }
  }

  function handleWsMessage(msg: any) {
    switch (msg.type) {
      case 'Connected':
        addLog(`Connected to session`)
        break

      case 'StateSync':
        updateState(msg.state)
        break

      case 'RobotMoved': {
        const robot = robots.value.find(r => r.id === msg.robot_id)
        if (robot) {
          robot.node_id = msg.to_node
        }
        break
      }

      case 'RobotStateChanged': {
        const robot = robots.value.find(r => r.id === msg.robot_id)
        if (robot) {
          robot.state = msg.new_state
        }
        break
      }

      case 'OrderCompleted':
        addLog(`Order #${msg.order_id} completed (${msg.cycle_time_s.toFixed(1)}s${msg.on_time ? '' : ', LATE'})`)
        break

      case 'Finished':
        appStatus.value = 'finished'
        metrics.value = msg.final_metrics
        addLog('Simulation finished')
        break

      case 'Error':
        appStatus.value = 'disconnected'
        addLog(`Error: ${msg.message}`)
        break
    }
  }

  function updateState(state: SimulationState) {
    simTime.value = state.time_s
    speed.value = state.speed
    eventsProcessed.value = state.events_processed
    robots.value = state.robots
    stations.value = state.stations
    metrics.value = state.metrics

    switch (state.status) {
      case 'running': appStatus.value = 'running'; break
      case 'paused': appStatus.value = 'paused'; break
      case 'finished': appStatus.value = 'finished'; break
      case 'idle': appStatus.value = 'idle'; break
      case 'error': appStatus.value = 'disconnected'; break
    }
  }

  async function start() {
    if (!sessionId.value) return
    try {
      const res = await fetch(`${API_BASE}/api/session/${sessionId.value}/start`, { method: 'POST' })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      appStatus.value = 'running'
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      addLog(`Error: Failed to start: ${msg}`)
    }
  }

  async function pause() {
    if (!sessionId.value) return
    try {
      const res = await fetch(`${API_BASE}/api/session/${sessionId.value}/pause`, { method: 'POST' })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      addLog(`Error: Failed to pause: ${msg}`)
    }
  }

  async function resume() {
    if (!sessionId.value) return
    try {
      const res = await fetch(`${API_BASE}/api/session/${sessionId.value}/resume`, { method: 'POST' })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      addLog(`Error: Failed to resume: ${msg}`)
    }
  }

  async function setSpeed(newSpeed: number) {
    if (!sessionId.value) return
    const previousSpeed = speed.value
    speed.value = newSpeed
    try {
      const res = await fetch(`${API_BASE}/api/session/${sessionId.value}/speed`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ speed: newSpeed }),
      })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      if (typeof data.speed === 'number') {
        speed.value = data.speed
      }
    } catch (e) {
      speed.value = previousSpeed
      const msg = e instanceof Error ? e.message : String(e)
      addLog(`Error: Failed to set speed: ${msg}`)
    }
  }

  async function step() {
    if (!sessionId.value) return
    try {
      const res = await fetch(`${API_BASE}/api/session/${sessionId.value}/step`, { method: 'POST' })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      addLog(`Error: Failed to step: ${msg}`)
    }
  }

  async function addRobot(nodeId?: number) {
    if (!sessionId.value) return
    try {
      const res = await fetch(`${API_BASE}/api/session/${sessionId.value}/add-robot`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ node_id: nodeId ?? null }),
      })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      addLog('Robot added')
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      addLog(`Error: Failed to add robot: ${msg}`)
    }
  }

  async function reset() {
    const sid = sessionId.value
    if (sid) {
      ws?.close()
      try {
        await fetch(`${API_BASE}/api/session/${sid}`, { method: 'DELETE' })
      } catch (e) {
        // ignore cleanup errors
      }
    }
    sessionId.value = null
    appStatus.value = 'disconnected'
    robots.value = []
    stations.value = []
    mapData.value = null
    simTime.value = 0
    eventsProcessed.value = 0
    eventLog.value = []
    metrics.value = {
      throughput_per_hour: 0, orders_completed: 0, orders_pending: 0,
      robot_utilization: 0, station_utilization: 0, avg_cycle_time_s: 0, late_orders: 0,
    }
  }

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString()
    eventLog.value.unshift(`[${timestamp}] ${message}`)
    if (eventLog.value.length > maxLogEntries) {
      eventLog.value.pop()
    }
  }

  return {
    // State
    sessionId, appStatus, speed, simTime, eventsProcessed,
    mapData, robots, stations, metrics, presets, eventLog,
    // Computed
    isConnected, isRunning, isPaused, isFinished, robotCounts, formattedTime,
    // Actions
    loadPresets, createSession, start, pause, resume, setSpeed,
    step, addRobot, reset, addLog,
  }
})
