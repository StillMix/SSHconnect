<script setup lang="ts">
import { ref, onMounted } from 'vue'

import { platform } from '@tauri-apps/api/os'
import ConnectionForm from '@/components/ConnectionForm.vue'
import WindowsHelp from '@/components/WindowsHelp.vue'
import DirectoryViewer from '@/components/DirectoryViewer.vue'
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
}

// обработчик ошибок из WindowsHelp
function handleWindowsError(errorMessage: string) {
  error.value = errorMessage
}
</script>

<template>
  <div class="container">
    <h1>SSH Подключение</h1>

    <!-- Сообщение об ошибке - показывается всегда, если есть ошибка -->
    <div v-if="error" class="error-message">
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
        @error="handleWindowsError"
      />
    </div>

    <!-- Просмотр директорий - показывается только при подключении -->
    <DirectoryViewer
      v-if="connected"
      :file-entries="fileEntries"
      :connection-string="connectionString"
      :current-path="currentPath"
      :path-history="pathHistory"
      :loading="loading"
      @disconnect="disconnect"
      @change-path="changePath"
      @navigate-back="navigateBack"
    />
  </div>
</template>

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
</style>
