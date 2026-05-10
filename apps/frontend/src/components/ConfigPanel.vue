<script setup lang="ts">
import { ref } from 'vue'
import { useSimulationStore } from '../stores/simulation'

const store = useSimulationStore()

const selectedPreset = ref('standard')
const robotCount = ref(15)
const orderRate = ref(60)
const durationMinutes = ref(60)

function onPresetChange() {
  const preset = store.presets.find(p => p.name === selectedPreset.value)
  if (preset) {
    robotCount.value = preset.robots
    orderRate.value = preset.order_rate
    durationMinutes.value = preset.duration_minutes
  }
}

async function startSimulation() {
  await store.createSession({
    preset: selectedPreset.value,
    robot_count: robotCount.value,
    order_rate: orderRate.value,
    duration_minutes: durationMinutes.value,
  })
}
</script>

<template>
  <div class="p-4 space-y-5">
    <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider">Configuration</h2>

    <!-- Preset Selection -->
    <div>
      <label class="block text-xs text-gray-500 mb-1.5">Preset</label>
      <select
        v-model="selectedPreset"
        class="w-full bg-warehouse-bg border border-warehouse-border rounded-lg px-3 py-2
               text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
        :disabled="store.isConnected"
        @change="onPresetChange"
      >
        <option v-for="p in store.presets" :key="p.name" :value="p.name">
          {{ p.name }} - {{ p.description }}
        </option>
      </select>
    </div>

    <!-- Robot Count -->
    <div>
      <div class="flex justify-between items-center mb-1.5">
        <label class="text-xs text-gray-500">Robots</label>
        <span class="text-sm font-mono text-indigo-400">{{ robotCount }}</span>
      </div>
      <input
        v-model.number="robotCount"
        type="range"
        :min="1"
        :max="50"
        :step="1"
        class="w-full accent-indigo-500"
        :disabled="store.isConnected"
      />
      <div class="flex justify-between text-xs text-gray-600 mt-0.5">
        <span>1</span>
        <span>50</span>
      </div>
    </div>

    <!-- Order Rate -->
    <div>
      <div class="flex justify-between items-center mb-1.5">
        <label class="text-xs text-gray-500">Orders/hour</label>
        <span class="text-sm font-mono text-indigo-400">{{ orderRate }}</span>
      </div>
      <input
        v-model.number="orderRate"
        type="range"
        :min="10"
        :max="300"
        :step="10"
        class="w-full accent-indigo-500"
        :disabled="store.isConnected"
      />
      <div class="flex justify-between text-xs text-gray-600 mt-0.5">
        <span>10</span>
        <span>300</span>
      </div>
    </div>

    <!-- Duration -->
    <div>
      <div class="flex justify-between items-center mb-1.5">
        <label class="text-xs text-gray-500">Duration (min)</label>
        <span class="text-sm font-mono text-indigo-400">{{ durationMinutes }}</span>
      </div>
      <input
        v-model.number="durationMinutes"
        type="range"
        :min="10"
        :max="480"
        :step="10"
        class="w-full accent-indigo-500"
        :disabled="store.isConnected"
      />
      <div class="flex justify-between text-xs text-gray-600 mt-0.5">
        <span>10m</span>
        <span>8h</span>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="pt-2">
      <button
        v-if="!store.isConnected"
        class="btn-primary w-full"
        :disabled="store.appStatus === 'connecting'"
        @click="startSimulation"
      >
        {{ store.appStatus === 'connecting' ? 'Creating...' : 'Create Simulation' }}
      </button>
      <button
        v-else
        class="btn-danger w-full"
        @click="store.reset()"
      >
        Reset & Reconfigure
      </button>
    </div>

    <!-- Robot Legend -->
    <div v-if="store.isConnected" class="pt-4 border-t border-warehouse-border">
      <h3 class="text-xs text-gray-500 uppercase tracking-wider mb-3">Robot States</h3>
      <div class="space-y-2">
        <div
          v-for="(count, state) in store.robotCounts"
          :key="state"
          class="flex items-center justify-between"
        >
          <div class="flex items-center gap-2">
            <span
              class="w-3 h-3 rounded-full"
              :class="{
                'bg-gray-400': state === 'Idle',
                'bg-blue-500': state === 'Moving',
                'bg-green-500': state === 'Servicing',
                'bg-yellow-500': state === 'WaitingNode' || state === 'WaitingEdge',
                'bg-purple-500': state === 'Charging',
                'bg-red-500': state === 'Failed',
                'bg-orange-500': state === 'Maintenance',
              }"
            />
            <span class="text-xs text-gray-300">{{ state }}</span>
          </div>
          <span class="text-xs font-mono text-gray-500">{{ count }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
