<script setup lang="ts">
import { open } from '@tauri-apps/api/shell'
import { openPowershellWithCommand } from '@/services/sshService'

const props = defineProps<{
  windowsHelpText: string
  isWindows: boolean
}>()

const emit = defineEmits<{
  error: [message: string]
}>()

async function openHelpLink() {
  if (props.isWindows) {
    await open(
      'https://docs.microsoft.com/en-us/windows-server/administration/openssh/openssh_install_firstuse',
    )
  }
}

async function installPoshSSH() {
  if (props.isWindows) {
    try {
      await openPowershellWithCommand('Install-Module -Name Posh-SSH -Force -Scope CurrentUser')
    } catch (error) {
      if (!error) return
      emit(
        'error',
        'Не удалось открыть PowerShell. Запустите вручную от имени администратора и выполните:\n' +
          'Install-Module -Name Posh-SSH -Force',
      )
    }
  }
}
</script>

<template>
  <div v-if="isWindows" class="windows-info">
    <h3>Подключение на Windows</h3>
    <p>{{ windowsHelpText }}</p>
    <div class="windows-help-buttons">
      <button @click="openHelpLink" class="help-button">Узнать больше об OpenSSH</button>
      <button @click="installPoshSSH" class="help-button">Установить Posh-SSH</button>
    </div>
  </div>
</template>

<style scoped>
.windows-info {
  margin-bottom: 20px;
  padding: 15px;
  background-color: #e8eaf6;
  border-radius: 8px;
  border-left: 4px solid #3f51b5;
}

.windows-help-buttons {
  display: flex;
  gap: 10px;
  margin-top: 10px;
}

.help-button {
  padding: 8px 12px;
  background-color: #3f51b5;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.help-button:hover {
  background-color: #303f9f;
}
</style>
