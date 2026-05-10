<script setup lang="ts">
import { useSimulationStore } from '../stores/simulation'

const store = useSimulationStore()
</script>

<template>
  <div class="p-4 border-t border-warehouse-border flex-shrink-0" style="max-height: 250px;">
    <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Event Log</h2>
    <div class="overflow-y-auto space-y-0.5" style="max-height: 200px;">
      <div
        v-for="(entry, i) in store.eventLog"
        :key="i"
        class="text-xs font-mono text-gray-500 leading-relaxed"
        :class="{ 'text-red-400': entry.includes('Error'), 'text-yellow-400': entry.includes('LATE') }"
      >
        {{ entry }}
      </div>
      <div v-if="store.eventLog.length === 0" class="text-xs text-gray-600">
        No events yet
      </div>
    </div>
  </div>
</template>
