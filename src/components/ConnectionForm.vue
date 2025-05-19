<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  loading: boolean
  connected: boolean
  error: string
  connectionStatus: string
  isWindows: boolean
}>()

const emit = defineEmits<{
  connect: [connectionString: string, password: string]
}>()

const connectionString = ref('')
const password = ref('')
const showPassword = ref(false)

function connectAndListDirectories() {
  if (!connectionString.value) {
    emit('connect', '', '')
    return
  }

  emit('connect', connectionString.value, password.value)
}

function togglePasswordVisibility() {
  showPassword.value = !showPassword.value
}
</script>

<template>
  <div class="status-indicator" :class="{ connected, error, connecting: loading }">
    {{ connectionStatus }}
  </div>

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
      <p v-for="(line, index) in error.split('\n')" :key="index">{{ line }}</p>
    </div>
  </div>
</template>

<style scoped>
.status-indicator {
  padding: 8px 12px;
  margin-bottom: 15px;
  background-color: #f0f0f0;
  border-radius: 4px;
  border-left: 4px solid #999;
  font-weight: bold;
}

.status-indicator.connected {
  background-color: #e8f5e9;
  border-left: 4px solid #4caf50;
  color: #2e7d32;
}

.status-indicator.error {
  background-color: #ffebee;
  border-left: 4px solid #d32f2f;
  color: #c62828;
}

.status-indicator.connecting {
  background-color: #e3f2fd;
  border-left: 4px solid #2196f3;
  color: #1565c0;
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
</style>
