<template>
  <div class="file-system-view">
    <div class="view-header">
      <h3>{{ title || 'Файловая система' }}</h3>
      <div class="view-controls">
        <button class="refresh-button" @click="refreshView" :disabled="loading">Обновить</button>
        <button class="close-button" @click="$emit('close')">Закрыть</button>
      </div>
    </div>

    <div class="file-list-container">
      <table class="file-list">
        <thead>
          <tr>
            <th class="column-name">Имя</th>
            <th class="column-permissions">Права</th>
            <th class="column-size">Размер</th>
            <th class="column-date">Дата</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(entry, index) in fileEntries"
            :key="index"
            @click="handleItemClick(entry)"
            :class="{
              'is-directory': entry.type === 'directory',
              'is-file': entry.type === 'file',
              'is-symlink': entry.type === 'symlink',
            }"
          >
            <td class="column-name">
              <span class="file-icon">{{ getFileIcon(entry.type, entry.name) }}</span>
              <span class="file-name">{{ entry.name }}</span>
            </td>
            <td class="column-permissions">{{ entry.permissions || '-' }}</td>
            <td class="column-size">{{ entry.size || '-' }}</td>
            <td class="column-date">{{ entry.date || '-' }}</td>
          </tr>
        </tbody>
      </table>

      <div v-if="fileEntries.length === 0" class="empty-directory">
        <p v-if="loading">Загрузка...</p>
        <p v-else>Директория пуста</p>
      </div>
    </div>

    <div class="current-path">
      <span>Текущий путь: {{ currentPath }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getFileIcon } from '@/utils/fileUtils'
import { listRemoteDirectories } from '@/services/sshService'

// Интерфейс для файлов
interface FileEntry {
  name: string
  type: string
  size?: string
  permissions?: string
  date?: string
}

const props = defineProps<{
  connectionString: string
  password: string
  initialPath: string
  title?: string
  viewId: number
}>()

const emit = defineEmits<{
  close: []
  error: [message: string]
  fileSelected: [path: string, entry: FileEntry]
  directorySelected: [path: string, entry: FileEntry]
  refresh: [viewId: number]
}>()

const currentPath = ref(props.initialPath || '/')
const fileEntries = ref<FileEntry[]>([])
const loading = ref(false)

// Загрузка содержимого директории при монтировании
onMounted(async () => {
  await loadDirectoryContents()
})

// Загрузка содержимого текущей директории
async function loadDirectoryContents() {
  loading.value = true

  try {
    fileEntries.value = await listRemoteDirectories({
      connectionString: props.connectionString,
      password: props.password,
      path: currentPath.value,
    })
  } catch (err) {
    emit('error', `Ошибка при загрузке директории: ${err}`)
  } finally {
    loading.value = false
  }
}

// Обработчик клика по элементу
async function handleItemClick(entry: FileEntry) {
  if (entry.type === 'directory') {
    // Формируем путь, куда хотим перейти
    let newPath = currentPath.value
    if (newPath.endsWith('/')) {
      newPath += entry.name
    } else {
      newPath += '/' + entry.name
    }

    currentPath.value = newPath
    await loadDirectoryContents()
    emit('directorySelected', newPath, entry)
  } else if (entry.type === 'file') {
    // Сообщаем о выборе файла
    const filePath = `${currentPath.value}${currentPath.value.endsWith('/') ? '' : '/'}${entry.name}`
    emit('fileSelected', filePath, entry)
  }
}

// Обновление текущего вида
async function refreshView() {
  await loadDirectoryContents()
  emit('refresh', props.viewId)
}
</script>

<style scoped>
.file-system-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #ffffff;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 15px;
  background-color: #f5f5f5;
  border-bottom: 1px solid #e0e0e0;
}

.view-header h3 {
  margin: 0;
  font-size: 16px;
}

.view-controls {
  display: flex;
  gap: 8px;
}

.view-controls button {
  padding: 5px 10px;
  border: none;
  border-radius: 3px;
  cursor: pointer;
  font-size: 13px;
}

.refresh-button {
  background-color: #4caf50;
  color: white;
}

.refresh-button:hover {
  background-color: #45a049;
}

.close-button {
  background-color: #f44336;
  color: white;
}

.close-button:hover {
  background-color: #d32f2f;
}

.file-list-container {
  flex: 1;
  overflow-y: auto;
  min-height: 200px;
}

.file-list {
  width: 100%;
  border-collapse: collapse;
}

.file-list th {
  position: sticky;
  top: 0;
  background-color: #f0f0f0;
  padding: 8px 10px;
  text-align: left;
  font-weight: bold;
  border-bottom: 2px solid #ddd;
}

.file-list td {
  padding: 6px 10px;
  border-bottom: 1px solid #eee;
}

.file-list tr:hover {
  background-color: #f8f8f8;
}

.file-list tr.is-directory:hover {
  background-color: #e3f2fd;
  cursor: pointer;
}

.file-list tr.is-file:hover {
  background-color: #e8f5e9;
  cursor: pointer;
}

.file-list tr.is-symlink:hover {
  background-color: #fff8e1;
  cursor: pointer;
}

.column-name {
  min-width: 180px;
}

.column-permissions {
  width: 100px;
  font-family: monospace;
}

.column-size {
  width: 80px;
  text-align: right;
}

.column-date {
  width: 120px;
}

.file-icon {
  margin-right: 5px;
}

.empty-directory {
  padding: 20px;
  text-align: center;
  color: #757575;
  font-style: italic;
}

.current-path {
  padding: 8px 10px;
  background-color: #f5f5f5;
  border-top: 1px solid #e0e0e0;
  font-size: 13px;
  color: #555;
}
</style>
