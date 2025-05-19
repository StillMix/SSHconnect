<script setup lang="ts">
import { getFileIcon } from '@/utils/fileUtils'

// Интерфейс для файлов
interface FileEntry {
  name: string
  type: string
  size?: string
  permissions?: string
  date?: string
}

defineProps<{
  fileEntries: FileEntry[]
  connectionString: string
  currentDirectory: string
  loading: boolean
}>()

const emit = defineEmits<{
  disconnect: []
}>()
</script>

<template>
  <div class="directory-view">
    <div class="directory-header">
      <div class="directory-path">
        <h3>{{ connectionString }} [{{ currentDirectory }}]</h3>
      </div>
      <button class="disconnect-button" @click="emit('disconnect')">Отключиться</button>
    </div>

    <div class="directory-list">
      <div class="file-list-header">
        <span class="file-icon"></span>
        <span class="file-name">Имя</span>
        <span class="file-permissions">Права</span>
        <span class="file-size">Размер</span>
        <span class="file-date">Дата</span>
      </div>

      <div v-for="(entry, index) in fileEntries" :key="index" class="file-entry">
        <span class="file-icon">{{ getFileIcon(entry.type) }}</span>
        <span class="file-name">{{ entry.name }}</span>
        <span class="file-permissions">{{ entry.permissions || '-' }}</span>
        <span class="file-size">{{ entry.size || '-' }}</span>
        <span class="file-date">{{ entry.date || '-' }}</span>
      </div>

      <div v-if="fileEntries.length === 0" class="empty-directory">
        <p v-if="loading">Загрузка...</p>
        <p v-else>Директория пуста</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.directory-view {
  background-color: #f5f5f5;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.directory-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid #ddd;
}

.directory-path {
  flex: 1;
}

.disconnect-button {
  padding: 8px 16px;
  background-color: #f44336;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.3s;
}

.disconnect-button:hover {
  background-color: #d32f2f;
}

.directory-list {
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
  background-color: white;
}

.file-list-header {
  display: flex;
  background-color: #f0f0f0;
  padding: 10px;
  font-weight: bold;
  border-bottom: 2px solid #ddd;
}

.file-entry {
  display: flex;
  padding: 8px 10px;
  border-bottom: 1px solid #eee;
  transition: background-color 0.2s;
}

.file-entry:hover {
  background-color: #f8f8f8;
}

.file-entry:last-child {
  border-bottom: none;
}

.file-icon {
  width: 40px;
  text-align: center;
}

.file-name {
  flex: 1;
  min-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-permissions {
  width: 120px;
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  width: 80px;
  text-align: right;
}

.file-date {
  width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-directory {
  padding: 20px;
  text-align: center;
  color: #757575;
  font-style: italic;
}

@media (max-width: 600px) {
  .file-permissions {
    display: none;
  }

  .file-date {
    display: none;
  }
}
</style>
