<script setup lang="ts">
import { ref } from 'vue'
import { getFileIcon, isTextFile } from '@/utils/fileUtils'
import FileEditor from '@/components/FileEditor.vue'
import { readRemoteFile } from '@/services/sshService'

// Интерфейс для файлов
interface FileEntry {
  name: string
  type: string
  size?: string
  permissions?: string
  date?: string
}

const props = defineProps<{
  fileEntries: FileEntry[]
  connectionString: string
  password: string
  currentPath: string
  loading: boolean
  pathHistory: string[]
}>()

const emit = defineEmits<{
  disconnect: []
  changePath: [path: string]
  navigateBack: []
  error: [message: string]
}>()

// Состояние для открытого файла
const isEditorOpen = ref(false)
const openedFileName = ref('')
const fileContent = ref('')
const loadingFile = ref(false)

// Обработчик клика по элементу
async function handleItemClick(entry: FileEntry) {
  if (entry.type === 'directory') {
    // Формируем путь, куда хотим перейти
    let newPath = props.currentPath
    if (newPath.endsWith('/')) {
      newPath += entry.name
    } else {
      newPath += '/' + entry.name
    }
    emit('changePath', newPath)
  } else if (entry.type === 'file') {
    // Открываем файл для редактирования, если это текстовый файл
    if (isTextFile(entry.name)) {
      await openFileForEditing(entry.name)
    } else {
      emit('error', `Файл ${entry.name} не поддерживается для редактирования`)
    }
  }
}

// Открыть файл для редактирования
async function openFileForEditing(fileName: string) {
  loadingFile.value = true

  try {
    // Получаем содержимое файла
    const content = await readRemoteFile({
      connectionString: props.connectionString,
      password: props.password,
      path: `${props.currentPath}${props.currentPath.endsWith('/') ? '' : '/'}${fileName}`,
    })

    // Открываем редактор с полученным содержимым
    fileContent.value = content
    openedFileName.value = fileName
    isEditorOpen.value = true
  } catch (err) {
    emit('error', `Ошибка при открытии файла: ${err}`)
  } finally {
    loadingFile.value = false
  }
}

// Закрытие редактора
function closeEditor() {
  isEditorOpen.value = false
  openedFileName.value = ''
  fileContent.value = ''
}

// Обработчик успешного сохранения файла
function handleFileSaved(fileName: string) {
  closeEditor()
  emit('error', `Файл ${fileName} успешно сохранен`)
}

// Обработчик для хлебных крошек
function navigateToBreadcrumb(index: number) {
  if (index < props.pathHistory.length) {
    emit('changePath', props.pathHistory[index])
  }
}

// Функция для создания хлебных крошек из пути
function getBreadcrumbs() {
  if (!props.currentPath) return ['/']

  const parts = props.currentPath.split('/').filter((p) => p !== '')
  const breadcrumbs = ['/']

  for (let i = 0; i < parts.length; i++) {
    const path = '/' + parts.slice(0, i + 1).join('/')
    breadcrumbs.push(path)
  }

  return breadcrumbs
}
</script>

<template>
  <div class="directory-view">
    <!-- Редактор файлов (показывается при открытии файла) -->
    <div v-if="isEditorOpen" class="editor-container">
      <FileEditor
        :file-name="openedFileName"
        :content="fileContent"
        :connection-string="connectionString"
        :password="password"
        :path="currentPath"
        @close="closeEditor"
        @saved="handleFileSaved"
        @error="emit('error', $event)"
      />
    </div>

    <!-- Просмотр директории (показывается, когда редактор скрыт) -->
    <div v-else>
      <div class="directory-header">
        <div class="directory-path">
          <h3>{{ connectionString }}</h3>

          <!-- Хлебные крошки -->
          <div class="breadcrumbs">
            <span
              v-for="(path, index) in getBreadcrumbs()"
              :key="index"
              class="breadcrumb"
              @click="navigateToBreadcrumb(index)"
            >
              {{ index === 0 ? '/' : path.split('/').pop() }}
              <span v-if="index < getBreadcrumbs().length - 1" class="separator">/</span>
            </span>
          </div>
        </div>

        <div class="navigation-buttons">
          <button
            class="back-button"
            @click="emit('navigateBack')"
            :disabled="pathHistory.length <= 1"
          >
            Назад
          </button>
          <button class="disconnect-button" @click="emit('disconnect')">Отключиться</button>
        </div>
      </div>

      <div class="directory-list" :class="{ 'loading-overlay': loadingFile }">
        <div class="file-list-header">
          <span class="file-icon"></span>
          <span class="file-name">Имя</span>
          <span class="file-permissions">Права</span>
          <span class="file-size">Размер</span>
          <span class="file-date">Дата</span>
        </div>

        <div
          v-for="(entry, index) in fileEntries"
          :key="index"
          class="file-entry"
          @click="handleItemClick(entry)"
          :class="{
            'is-directory': entry.type === 'directory',
            'is-file': entry.type === 'file',
          }"
        >
          <span class="file-icon">{{ getFileIcon(entry.type, entry.name) }}</span>
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
  </div>
</template>

<style scoped>
.directory-view {
  background-color: #f5f5f5;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.editor-container {
  height: 70vh;
  min-height: 400px;
}

.directory-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid #ddd;
}

.directory-path {
  flex: 1;
}

.breadcrumbs {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  margin-top: 8px;
  font-size: 14px;
  background-color: #fff;
  padding: 8px 12px;
  border-radius: 4px;
  border: 1px solid #e0e0e0;
}

.breadcrumb {
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 3px;
}

.breadcrumb:hover {
  background-color: #e0e0e0;
}

.separator {
  margin: 0 4px;
  color: #888;
}

.navigation-buttons {
  display: flex;
  gap: 10px;
}

.back-button {
  padding: 8px 16px;
  background-color: #2196f3;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.3s;
}

.back-button:hover {
  background-color: #1976d2;
}

.back-button:disabled {
  background-color: #b3e5fc;
  cursor: not-allowed;
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
  position: relative;
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

.file-entry.is-directory {
  cursor: pointer;
}

.file-entry.is-directory:hover {
  background-color: #e3f2fd;
}

.file-entry.is-file {
  cursor: pointer;
}

.file-entry.is-file:hover {
  background-color: #e8f5e9;
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

.loading-overlay {
  position: relative;
}

.loading-overlay::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.7);
  z-index: 2;
  display: flex;
  justify-content: center;
  align-items: center;
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
