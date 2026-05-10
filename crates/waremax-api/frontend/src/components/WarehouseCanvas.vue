<script setup lang="ts">
import { ref, toRef } from 'vue'
import { storeToRefs } from 'pinia'
import { useSimulationStore } from '../stores/simulation'
import { useWarehouseCanvas } from '../composables/useWarehouseCanvas'

const store = useSimulationStore()
const { mapData, robots, stations } = storeToRefs(store)
const canvasRef = ref<HTMLCanvasElement | null>(null)

useWarehouseCanvas(canvasRef, mapData, robots, stations)
</script>

<template>
  <canvas
    ref="canvasRef"
    class="w-full h-full block"
  />
</template>
