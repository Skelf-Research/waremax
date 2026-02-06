<script setup lang="ts">
import { onMounted } from 'vue'
import { useSimulationStore } from './stores/simulation'
import WarehouseCanvas from './components/WarehouseCanvas.vue'
import ControlPanel from './components/ControlPanel.vue'
import ConfigPanel from './components/ConfigPanel.vue'
import MetricsPanel from './components/MetricsPanel.vue'
import EventLog from './components/EventLog.vue'

const store = useSimulationStore()

onMounted(() => {
  store.loadPresets()
})
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden">
    <!-- Header -->
    <header class="flex items-center justify-between px-6 py-3 bg-warehouse-surface border-b border-warehouse-border">
      <div class="flex items-center gap-3">
        <h1 class="text-xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
          Waremax Simulation
        </h1>
        <span
          v-if="store.isConnected"
          class="badge"
          :class="{
            'bg-green-500/20 text-green-400': store.isRunning,
            'bg-yellow-500/20 text-yellow-400': store.isPaused,
            'bg-blue-500/20 text-blue-400': store.isFinished,
            'bg-gray-500/20 text-gray-400': store.appStatus === 'idle',
          }"
        >
          {{ store.appStatus.toUpperCase() }}
        </span>
      </div>

      <div class="flex items-center gap-4 text-sm text-gray-400">
        <span v-if="store.isConnected" class="font-mono">
          {{ store.formattedTime }}
        </span>
        <span v-if="store.isConnected" class="font-mono">
          Speed: {{ store.speed }}x
        </span>
        <span v-if="store.isConnected" class="text-xs">
          {{ store.eventsProcessed.toLocaleString() }} events
        </span>
      </div>
    </header>

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left Sidebar: Config -->
      <aside class="w-72 border-r border-warehouse-border overflow-y-auto flex-shrink-0">
        <ConfigPanel />
      </aside>

      <!-- Center: Canvas -->
      <main class="flex-1 flex flex-col overflow-hidden min-w-0">
        <div class="flex-1 relative min-h-0">
          <WarehouseCanvas />

          <!-- Overlay when disconnected -->
          <div
            v-if="!store.isConnected"
            class="absolute inset-0 flex items-center justify-center bg-warehouse-bg/80 backdrop-blur-sm"
          >
            <div class="text-center">
              <div class="text-6xl mb-4 opacity-30">&#x1F3ED;</div>
              <p class="text-gray-400 text-lg mb-2">Configure and start a simulation</p>
              <p class="text-gray-500 text-sm">Select a preset from the left panel</p>
            </div>
          </div>
        </div>

        <!-- Controls Bar - Always visible with fixed height -->
        <div class="flex-shrink-0">
          <ControlPanel />
        </div>
      </main>

      <!-- Right Sidebar: Metrics + Events -->
      <aside class="w-80 border-l border-warehouse-border overflow-y-auto flex-shrink-0 flex flex-col">
        <MetricsPanel />
        <EventLog />
      </aside>
    </div>
  </div>
</template>
