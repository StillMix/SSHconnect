<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { platform } from '@tauri-apps/api/os'

const connectionString = ref('')
const password = ref('')
const showPassword = ref(false)
const directories = ref<string[]>([])
const error = ref('')
const loading = ref(false)
const connected = ref(false)
const currentDirectory = ref('')

// Интерфейс для записи файловой системы
interface FileEntry {
  name: string
  type: string
  size?: string
  permissions?: string
  date?: string
}

const fileEntries = ref<FileEntry[]>([])

onMounted(async () => {
  // Проверяем операционную систему
  const currentPlatform = await platform()

  if (currentPlatform === 'win32') {
    // На Windows показываем сообщение о необходимости установки PuTTY
    error.value = 'Для работы на Windows требуется PuTTY (plink.exe должен быть доступен в PATH)'
  }
})

async function connectAndListDirectories() {
  if (!connectionString.value) {
    error.value = 'Пожалуйста, введите строку подключения в формате username@serverip'
    return
  }

  error.value = ''
  loading.value = true
  fileEntries.value = []

  try {
    const result = (await invoke('list_remote_directories', {
      connectionString: connectionString.value,
      password: password.value,
    })) as string[]

    directories.value = result
    connected.value = true
    currentDirectory.value = '/'

    // Преобразуем результаты в структурированные записи
    fileEntries.value = parseDirectoryListing(result)
  } catch (err) {
    error.value = `${err}`
    directories.value = []
    connected.value = false
  } finally {
    loading.value = false
  }
}

function parseDirectoryListing(listing: string[]): FileEntry[] {
  return listing
    .filter((line) => !line.startsWith('total')) // Фильтруем строку 'total X'
    .map((line) => {
      const parts = line.trim().split(/\s+/)

      // Типичный формат вывода ls -la:
      // [0]    [1]  [2]     [3]      [4]         [5] [6] [7]    [8+]
      // drwxr-xr-x 2 user   group    4096        Jun 15 10:17   dirname
      // -rw-r--r-- 1 user   group    2048        May 23 14:22   filename.txt

      if (parts.length < 8) {
        // Недостаточно частей для полного разбора, возвращаем базовую запись
        return {
          name: line,
          type: 'unknown',
        }
      }

      // Определяем тип на основе первого символа прав доступа
      // d - директория, l - символическая ссылка, - - обычный файл
      const permissions = parts[0]
      let type = 'file'

      if (permissions.startsWith('d')) {
        type = 'directory'
      } else if (permissions.startsWith('l')) {
        type = 'symlink'
      }

      // Извлекаем размер
      const size = parts[4]

      // Определяем дату (части 5-7)
      const date = `${parts[5]} ${parts[6]} ${parts[7]}`

      // Имя файла - все остальные части (с учетом пробелов в именах)
      const name = parts.slice(8).join(' ')

      return {
        name,
        type,
        size,
        permissions,
        date,
      }
    })
    .filter((entry) => entry.name && entry.name !== '.' && entry.name !== '..') // Фильтруем . и ..
}

function togglePasswordVisibility() {
  showPassword.value = !showPassword.value
}

function getFileIcon(type: string): string {
  switch (type) {
    case 'directory':
      return '📁'
    case 'symlink':
      return '🔗'
    case 'file':
      // Определение иконки по расширению файла
      return '📄'
    default:
      return '❓'
  }
}
</script>

<template>
  <div class="container">
    <h1>SSH Подключение</h1>

    <div class="connection-form">
      <div class="input-group">
        <input
          v-model="connectionString"
          placeholder="username@serverip"
          @keyup.enter="connectAndListDirectories"
        />
      </div>

      <div class="password-group">
        <input
          :type="showPassword ? 'text' : 'password'"
          v-model="password"
          placeholder="Пароль"
          @keyup.enter="connectAndListDirectories"
        />
        <button class="password-toggle" @click="togglePasswordVisibility">
          {{ showPassword ? 'Скрыть' : 'Показать' }}
        </button>
      </div>

      <button class="connect-button" @click="connectAndListDirectories" :disabled="loading">
        {{ loading ? 'Подключение...' : connected ? 'Переподключиться' : 'Подключиться' }}
      </button>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <div v-if="connected" class="directory-view">
      <div class="directory-path">
        <h3>Текущая директория: {{ currentDirectory }}</h3>
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
  </div>
</template>

<style scoped>
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

h1 {
  margin-bottom: 20px;
  color: #333;
}

.connection-form {
  margin-bottom: 20px;
  display: flex;
  flex-direction: column;
  gap: 15px;
  background-color: #f5f5f5;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.input-group,
.password-group {
  display: flex;
  gap: 10px;
  width: 100%;
}

input {
  flex: 1;
  padding: 10px;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 16px;
}

.password-toggle {
  padding: 10px 15px;
  background-color: #6c757d;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.2s;
}

.password-toggle:hover {
  background-color: #5a6268;
}

.connect-button {
  padding: 12px 20px;
  background-color: #4caf50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  transition: background-color 0.3s;
  align-self: flex-start;
}

.connect-button:hover {
  background-color: #45a049;
}

button:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}

.error-message {
  color: #d32f2f;
  background-color: #ffebee;
  padding: 10px;
  border-radius: 4px;
  border-left: 4px solid #d32f2f;
}

.directory-view {
  background-color: #f5f5f5;
  border-radius: 8px;
  padding: 20px;
  margin-top: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.directory-path {
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid #ddd;
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
