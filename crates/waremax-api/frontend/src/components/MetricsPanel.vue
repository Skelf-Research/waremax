<script setup lang="ts">
import { computed } from 'vue'
import { useSimulationStore } from '../stores/simulation'

const store = useSimulationStore()

const metrics = computed(() => store.metrics)

function formatPercent(value: number): string {
  return (value * 100).toFixed(1) + '%'
}

function formatTime(seconds: number): string {
  if (seconds < 60) return seconds.toFixed(1) + 's'
  const min = Math.floor(seconds / 60)
  const sec = (seconds % 60).toFixed(0)
  return `${min}m ${sec}s`
}
</script>

<template>
  <div class="p-4 flex-1">
    <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">Metrics</h2>

    <div v-if="store.isConnected" class="space-y-4">
      <!-- Throughput -->
      <div class="card">
        <div class="flex justify-between items-center mb-1">
          <span class="text-xs text-gray-500">Throughput</span>
          <span class="text-lg font-bold text-indigo-400">
            {{ metrics.throughput_per_hour.toFixed(1) }}
          </span>
        </div>
        <span class="text-xs text-gray-600">orders/hour</span>
      </div>

      <!-- Orders -->
      <div class="grid grid-cols-2 gap-2">
        <div class="card text-center">
          <div class="text-lg font-bold text-green-400">{{ metrics.orders_completed }}</div>
          <div class="text-xs text-gray-500">Completed</div>
        </div>
        <div class="card text-center">
          <div class="text-lg font-bold" :class="metrics.late_orders > 0 ? 'text-red-400' : 'text-gray-500'">
            {{ metrics.late_orders }}
          </div>
          <div class="text-xs text-gray-500">Late</div>
        </div>
      </div>

      <!-- Utilization Bars -->
      <div class="space-y-3">
        <!-- Robot Utilization -->
        <div>
          <div class="flex justify-between text-xs mb-1">
            <span class="text-gray-500">Robot Utilization</span>
            <span class="text-gray-300">{{ formatPercent(metrics.robot_utilization) }}</span>
          </div>
          <div class="h-2 bg-warehouse-bg rounded-full overflow-hidden">
            <div
              class="h-full rounded-full transition-all duration-300"
              :class="{
                'bg-green-500': metrics.robot_utilization < 0.7,
                'bg-yellow-500': metrics.robot_utilization >= 0.7 && metrics.robot_utilization < 0.9,
                'bg-red-500': metrics.robot_utilization >= 0.9,
              }"
              :style="{ width: formatPercent(metrics.robot_utilization) }"
            />
          </div>
        </div>

        <!-- Station Utilization -->
        <div>
          <div class="flex justify-between text-xs mb-1">
            <span class="text-gray-500">Station Utilization</span>
            <span class="text-gray-300">{{ formatPercent(metrics.station_utilization) }}</span>
          </div>
          <div class="h-2 bg-warehouse-bg rounded-full overflow-hidden">
            <div
              class="h-full rounded-full transition-all duration-300"
              :class="{
                'bg-blue-500': metrics.station_utilization < 0.7,
                'bg-yellow-500': metrics.station_utilization >= 0.7 && metrics.station_utilization < 0.9,
                'bg-red-500': metrics.station_utilization >= 0.9,
              }"
              :style="{ width: formatPercent(metrics.station_utilization) }"
            />
          </div>
        </div>
      </div>

      <!-- Avg Cycle Time -->
      <div class="card">
        <div class="flex justify-between items-center">
          <span class="text-xs text-gray-500">Avg Cycle Time</span>
          <span class="text-sm font-mono text-gray-300">
            {{ formatTime(metrics.avg_cycle_time_s) }}
          </span>
        </div>
      </div>

      <!-- Pending Orders -->
      <div class="card">
        <div class="flex justify-between items-center">
          <span class="text-xs text-gray-500">Pending Orders</span>
          <span class="text-sm font-mono text-yellow-400">{{ metrics.orders_pending }}</span>
        </div>
      </div>

      <!-- Stations -->
      <div>
        <h3 class="text-xs text-gray-500 uppercase tracking-wider mb-2">Stations</h3>
        <div class="space-y-1.5">
          <div
            v-for="s in store.stations"
            :key="s.id"
            class="flex items-center justify-between text-xs px-2 py-1 rounded bg-warehouse-bg"
          >
            <span class="text-gray-300">{{ s.name }}</span>
            <div class="flex gap-2 text-gray-500">
              <span>Q:{{ s.queue_length }}</span>
              <span>S:{{ s.serving_count }}/{{ s.concurrency }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-center text-gray-600 text-sm mt-8">
      No simulation running
    </div>
  </div>
</template>
