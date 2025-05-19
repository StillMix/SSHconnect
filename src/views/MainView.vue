import { ref, onMounted }
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { platform } from '@tauri-apps/api/os'
import ConnectionForm from '@/components/ConnectionForm.vue'
import WindowsHelp from '@/components/WindowsHelp.vue'
import DirectoryViewer from '@/components/DirectoryViewer.vue'
import { FileEntry, parseDirectoryListing } from '@/utils/fileUtils'

// состояния
const connectionString = ref('')
const password = ref('')
const fileEntries = ref<FileEntry[]>([])
const error = ref('')
const loading = ref(false)
const connected = ref(false)
const currentDirectory = ref('')
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

// подключение и получение списка
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
    const result = (await invoke('list_remote_directories', {
      connectionString: connectionString.value,
      password: password.value,
    })) as string[]

    connectionStatus.value = 'Успешное подключение'
    connected.value = true
    currentDirectory.value = '/'
    fileEntries.value = parseDirectoryListing(result)
  } catch (err) {
    connectionStatus.value = 'Ошибка подключения'
    error.value = String(err)
    connected.value = false
  } finally {
    loading.value = false
  }
}

// парсер ls -la
function parseDirectoryListing(listing: string[]): FileEntry[] {
  return listing
    .filter((l) => !l.startsWith('total'))
    .map((line) => {
      const parts = line.trim().split(/\s+/)
      if (parts.length < 8) return { name: line, type: 'unknown' }

      const permissions = parts[0]
      const type = permissions.startsWith('d')
        ? 'directory'
        : permissions.startsWith('l')
          ? 'symlink'
          : 'file'

      const size = parts[4]
      const date = `${parts[5]} ${parts[6]} ${parts[7]}`
      const name = parts.slice(8).join(' ')
      return { name, type, size, permissions, date }
    })
    .filter((e) => e.name !== '.' && e.name !== '..')
}

// отключение и возврат к экрану подключения
function disconnect() {
  connected.value = false
  connectionStatus.value = 'Не подключено'
  fileEntries.value = []
}

// обработчик ошибок из WindowsHelp
function handleWindowsError(errorMessage: string) {
  error.value = errorMessage
}
</script>

<template>
  <div class="container">
    <h1>SSH Подключение</h1>

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
      :current-directory="currentDirectory"
      :loading="loading"
      @disconnect="disconnect"
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
</style>
