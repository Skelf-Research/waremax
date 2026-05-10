<script setup lang="ts">
import { useSimulationStore } from '../stores/simulation'

const store = useSimulationStore()

const speeds = [0.5, 1, 2, 5, 10, 50]

function togglePlayPause() {
  if (store.isRunning) {
    store.pause()
  } else if (store.isPaused || store.appStatus === 'idle') {
    if (store.appStatus === 'idle' || store.appStatus === 'paused') {
      store.start()
    }
  }
}
</script>

<template>
  <div class="flex items-center justify-between px-6 py-3 bg-warehouse-surface border-t border-warehouse-border h-14">
    <!-- Show controls when connected -->
    <template v-if="store.isConnected">
    <!-- Playback Controls -->
    <div class="flex items-center gap-2">
      <button
        class="btn-primary btn-sm flex items-center gap-1.5"
        :disabled="store.isFinished"
        @click="togglePlayPause"
      >
        <span v-if="store.isRunning">&#9208; Pause</span>
        <span v-else>&#9654; Play</span>
      </button>

      <button
        class="btn-secondary btn-sm"
        :disabled="store.isRunning || store.isFinished"
        @click="store.step()"
      >
        &#9197; Step
      </button>

      <button
        class="btn-secondary btn-sm"
        @click="store.reset()"
      >
        &#8634; Reset
      </button>
    </div>

    <!-- Speed Controls -->
    <div class="flex items-center gap-1">
      <span class="text-xs text-gray-500 mr-2">Speed:</span>
      <button
        v-for="s in speeds"
        :key="s"
        class="speed-btn"
        :class="{ active: store.speed === s }"
        @click="store.setSpeed(s)"
      >
        {{ s }}x
      </button>
    </div>

    <!-- Quick Actions -->
    <div class="flex items-center gap-2">
      <button
        class="btn-secondary btn-sm"
        :disabled="store.isFinished"
        @click="store.addRobot()"
      >
        + Robot
      </button>
    </div>
    </template>

    <!-- Show placeholder when not connected -->
    <div v-else class="flex-1 flex items-center justify-center text-gray-500 text-sm">
      Create a simulation to see controls
    </div>
  </div>
</template>
