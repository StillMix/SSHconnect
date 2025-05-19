<template>
  <div class="multi-view-container">
    <div class="views-toolbar">
      <h3>Дополнительные представления</h3>
      <div class="view-buttons">
        <button
          v-for="i in 4"
          :key="i"
          @click="toggleView(i)"
          :class="{ active: activeViews.includes(i) }"
        >
          Вид {{ i }}
        </button>
      </div>
    </div>

    <div class="views-grid" :class="`views-count-${activeViews.length}`">
      <div v-for="viewId in activeViews" :key="viewId" class="view-panel">
        <FileSystemView
          :connection-string="connectionString"
          :password="password"
          :initial-path="viewPaths[viewId - 1]"
          :title="`Вид ${viewId}`"
          :view-id="viewId"
          @close="closeView(viewId)"
          @error="$emit('error', $event)"
          @file-selected="handleFileSelected($event, $event2, viewId)"
          @directory-selected="handleDirectorySelected($event, $event2, viewId)"
          @refresh="handleViewRefresh"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import FileSystemView from '@/components/FileSystemView.vue'

const props = defineProps<{
  connectionString: string
  password: string
  mainPath: string
}>()

const emit = defineEmits<{
  error: [message: string]
  fileSelected: [path: string, entry: any, viewId: number]
}>()

// Состояние активных видов
const activeViews = ref<number[]>([])
const viewPaths = ref<string[]>(['/', '/', '/', '/'])

// Отслеживание изменения основного пути
watch(
  () => props.mainPath,
  (newPath) => {
    // При изменении основного пути, обновляем пути неактивных видов
    viewPaths.value = viewPaths.value.map((path, index) => {
      // Только если вид не активен, обновляем его путь
      if (!activeViews.value.includes(index + 1)) {
        return newPath
      }
      return path
    })
  },
)

// Включить/выключить вид
function toggleView(viewId: number) {
  if (activeViews.value.includes(viewId)) {
    // Выключаем вид
    activeViews.value = activeViews.value.filter((id) => id !== viewId)
  } else {
    // Включаем вид с текущим путем
    activeViews.value.push(viewId)
    // Устанавливаем путь из основного вида
    viewPaths.value[viewId - 1] = props.mainPath
  }
}

// Закрыть вид
function closeView(viewId: number) {
  activeViews.value = activeViews.value.filter((id) => id !== viewId)
}

// Обработчик выбора файла
function handleFileSelected(path: string, entry: any, viewId: number) {
  emit('fileSelected', path, entry, viewId)
}

// Обработчик выбора директории
function handleDirectorySelected(path: string, entry: any, viewId: number) {
  // Сохраняем новый путь для этого вида
  viewPaths.value[viewId - 1] = path
}

// Обработчик обновления вида
function handleViewRefresh(viewId: number) {
  // Здесь можно добавить логику при обновлении вида
  console.log(`Вид ${viewId} обновлен`)
}
</script>

<style scoped>
.multi-view-container {
  margin-top: 20px;
  background-color: #f8f8f8;
  border-radius: 8px;
  padding: 15px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.views-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid #ddd;
}

.views-toolbar h3 {
  margin: 0;
  font-size: 16px;
  color: #333;
}

.view-buttons {
  display: flex;
  gap: 10px;
}

.view-buttons button {
  padding: 8px 12px;
  background-color: #e0e0e0;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.view-buttons button:hover {
  background-color: #d5d5d5;
}

.view-buttons button.active {
  background-color: #2196f3;
  color: white;
}

.views-grid {
  display: grid;
  gap: 15px;
  grid-template-columns: 1fr;
  grid-auto-rows: minmax(300px, auto);
}

.views-grid.views-count-2 {
  grid-template-columns: repeat(2, 1fr);
}

.views-grid.views-count-3,
.views-grid.views-count-4 {
  grid-template-columns: repeat(2, 1fr);
}

.view-panel {
  background-color: white;
  border-radius: 4px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  height: 350px;
}

@media (max-width: 768px) {
  .views-grid.views-count-2,
  .views-grid.views-count-3,
  .views-grid.views-count-4 {
    grid-template-columns: 1fr;
  }
}
</style>
