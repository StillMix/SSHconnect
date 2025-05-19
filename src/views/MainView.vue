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

onMounted(async () => {
  // Проверяем операционную систему
  const currentPlatform = await platform()

  if (currentPlatform === 'win32') {
    // На Windows показываем сообщение о необходимости установки PuTTY
    error.value =
      'Для работы на Windows рекомендуется установить PuTTY (plink.exe должен быть доступен в PATH)'
  }
})

async function connectAndListDirectories() {
  if (!connectionString.value) {
    error.value = 'Пожалуйста, введите строку подключения в формате username@serverip'
    return
  }

  error.value = ''
  loading.value = true

  try {
    directories.value = await invoke('list_remote_directories', {
      connectionString: connectionString.value,
      password: password.value,
    })
  } catch (err) {
    error.value = `${err}`
    directories.value = []
  } finally {
    loading.value = false
  }
}

function togglePasswordVisibility() {
  showPassword.value = !showPassword.value
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
        {{ loading ? 'Подключение...' : 'Подключиться' }}
      </button>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <div v-if="directories.length > 0" class="directory-list">
      <h2>Список файлов и папок:</h2>
      <ul>
        <li v-for="(dir, index) in directories" :key="index">{{ dir }}</li>
      </ul>
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
}

.connection-form {
  margin-bottom: 20px;
  display: flex;
  flex-direction: column;
  gap: 15px;
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
  padding: 10px;
  background-color: #6c757d;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.connect-button {
  padding: 12px 20px;
  background-color: #4caf50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  align-self: flex-start;
}

button:disabled {
  background-color: #cccccc;
}

.error-message {
  color: red;
  margin-top: 5px;
}

.directory-list {
  background-color: #f5f5f5;
  border-radius: 4px;
  padding: 15px;
  margin-top: 20px;
}

ul {
  list-style-type: none;
  padding: 0;
  margin: 0;
}

li {
  padding: 8px 0;
  border-bottom: 1px solid #ddd;
}

li:last-child {
  border-bottom: none;
}
</style>
