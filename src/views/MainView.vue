<template>
  <div class="container">
    <h1>SSH Подключение</h1>

    <!-- Сообщение об ошибке - показывается всегда, если есть ошибка -->
    <div
      v-if="error"
      class="error-message"
      :class="{ 'success-message': error.includes('успешно сохранен') }"
    >
      <p v-for="(line, index) in error.split('\n')" :key="index">{{ line }}</p>
    </div>

    <!-- Форма подключения - показывается только когда нет подключения -->
    <div v-if="!connected">
      <ConnectionForm
        :loading="loading"
        :connected="connected"
        :error="error"
        :connection-status="connectionStatus"
        :is-windows="isWindows"
        @connect="connectAndListDirectories"
      />

      <WindowsHelp
        :is-windows="isWindows"
        :windows-help-text="windowsHelpText"
        @error="handleError"
      />
    </div>

    <!-- Просмотр директорий - показывается только при подключении -->
    <template v-if="connected">
      <div class="view-controls">
        <button class="multi-view-toggle" @click="toggleMultiView">
          {{ showMultiView ? 'Скрыть дополнительные виды' : 'Показать дополнительные виды' }}
        </button>
      </div>

      <DirectoryViewer
        :file-entries="fileEntries"
        :connection-string="connectionString"
        :password="password"
        :current-path="currentPath"
        :path-history="pathHistory"
        :loading="loading"
        @disconnect="disconnect"
        @change-path="changePath"
        @navigate-back="navigateBack"
        @error="handleError"
      />

      <!-- Дополнительные виды -->
      <MultiView
        v-if="showMultiView"
        :connection-string="connectionString"
        :password="password"
        :main-path="currentPath"
        @error="handleError"
        @file-selected="handleFileSelectedFromView"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

import { platform } from '@tauri-apps/api/os'
import ConnectionForm from '@/components/ConnectionForm.vue'
import WindowsHelp from '@/components/WindowsHelp.vue'
import DirectoryViewer from '@/components/DirectoryViewer.vue'
import MultiView from '@/components/MultiView.vue'
import { FileEntry } from '@/utils/fileUtils'
import { listRemoteDirectories } from '@/services/sshService'

// состояния
const connectionString = ref('')
const password = ref('')
const fileEntries = ref<FileEntry[]>([])
const error = ref('')
const loading = ref(false)
const connected = ref(false)
const currentPath = ref('/')
const pathHistory = ref<string[]>(['/'])
const connectionStatus = ref('Не подключено')
const isWindows = ref(false)
const windowsHelpText = ref('')
const showMultiView = ref(false)

// при монтировании
onMounted(async () => {
  const currentPlatform = await platform()
  isWindows.value = currentPlatform === 'win32'
  if (isWindows.value) {
    windowsHelpText.value =
      'Для работы на Windows требуется один из следующих методов:\n' +
      '1. OpenSSH клиент (встроен в Windows 10+)\n' +
      '2. PowerShell модуль Posh-SSH (Install-Module -Name Posh-SSH -Force)'
    connectionStatus.value = 'Подготовка Windows для SSH-подключения'
  }
})

// подключение и получение списка корневой директории
async function connectAndListDirectories(connString: string, pass: string) {
  if (!connString) {
    error.value = 'Введите username@serverip'
    return
  }

  connectionString.value = connString
  password.value = pass
  error.value = ''
  loading.value = true
  connectionStatus.value = 'Подключение...'
  fileEntries.value = []

  try {
    // Сбрасываем путь и историю при новом подключении
    currentPath.value = '/'
    pathHistory.value = ['/']

    // Вызываем обновленный метод для получения файлов
    fileEntries.value = await listRemoteDirectories({
      connectionString: connectionString.value,
      password: password.value,
      path: currentPath.value,
    })

    connectionStatus.value = 'Успешное подключение'
    connected.value = true
  } catch (err) {
    connectionStatus.value = 'Ошибка подключения'
    error.value = String(err)
    connected.value = false
  } finally {
    loading.value = false
  }
}

// Функция для изменения текущего пути и загрузки содержимого
async function changePath(newPath: string) {
  if (newPath === currentPath.value) return

  loading.value = true
  error.value = ''

  try {
    // Получаем содержимое новой директории
    const entries = await listRemoteDirectories({
      connectionString: connectionString.value,
      password: password.value,
      path: newPath,
    })

    // Обновляем текущий путь и добавляем его в историю
    currentPath.value = newPath

    // Если перешли в новый путь, добавляем его в историю
    if (!pathHistory.value.includes(newPath)) {
      pathHistory.value.push(newPath)
    }
    // Если вернулись назад, обрезаем историю до текущего пути
    else {
      const index = pathHistory.value.indexOf(newPath)
      pathHistory.value = pathHistory.value.slice(0, index + 1)
    }

    fileEntries.value = entries
  } catch (err) {
    error.value = `Ошибка при переходе в директорию ${newPath}: ${err}`
  } finally {
    loading.value = false
  }
}

// Навигация назад
function navigateBack() {
  if (pathHistory.value.length > 1) {
    // Возвращаемся к предыдущему пути
    const previousIndex = pathHistory.value.indexOf(currentPath.value) - 1
    if (previousIndex >= 0) {
      const previousPath = pathHistory.value[previousIndex]
      changePath(previousPath)
    }
  }
}

// отключение и возврат к экрану подключения
function disconnect() {
  connected.value = false
  connectionStatus.value = 'Не подключено'
  fileEntries.value = []
  currentPath.value = '/'
  pathHistory.value = ['/']
  showMultiView.value = false
}

// обработчик ошибок
function handleError(errorMessage: string) {
  error.value = errorMessage

  // Автоматически скрываем сообщение об успешном сохранении через 3 секунды
  if (errorMessage.includes('успешно сохранен')) {
    setTimeout(() => {
      if (error.value === errorMessage) {
        error.value = ''
      }
    }, 3000)
  }
}

// Переключение отображения MultiView
function toggleMultiView() {
  showMultiView.value = !showMultiView.value
}

// Обработчик выбора файла из дополнительного вида
function handleFileSelectedFromView(path: string, entry: any, viewId: number) {
  // Здесь можно обработать выбор файла из дополнительного вида
  // Например, открыть его для редактирования
  error.value = `Выбран файл ${entry.name} из вида ${viewId}`
}
</script>

<style scoped>
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

h1 {
  margin-bottom: 10px;
  color: #333;
}

.error-message {
  color: #d32f2f;
  background-color: #ffebee;
  padding: 10px;
  border-radius: 4px;
  border-left: 4px solid #d32f2f;
  margin-bottom: 15px;
}

.success-message {
  color: #2e7d32;
  background-color: #e8f5e9;
  border-left: 4px solid #4caf50;
}

.view-controls {
  margin-bottom: 15px;
  display: flex;
  justify-content: flex-end;
}

.multi-view-toggle {
  padding: 8px 16px;
  background-color: #3f51b5;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.3s;
}

.multi-view-toggle:hover {
  background-color: #303f9f;
}
</style>
