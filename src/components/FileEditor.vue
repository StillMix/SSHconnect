<template>
  <div class="file-editor">
    <div class="editor-header">
      <h3>{{ fileName }}</h3>
      <div class="editor-buttons">
        <button class="save-button" @click="saveFile" :disabled="loading || !hasChanges">
          {{ loading ? 'Сохранение...' : 'Сохранить' }}
        </button>
        <button class="cancel-button" @click="$emit('close')">Закрыть</button>
      </div>
    </div>

    <div v-if="loading" class="loading-overlay">
      <div class="loading-spinner"></div>
    </div>

    <textarea
      v-model="fileContent"
      class="editor-content"
      spellcheck="false"
      @input="hasChanges = true"
    ></textarea>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { saveRemoteFile } from '@/services/sshService'

const props = defineProps<{
  fileName: string
  content: string
  connectionString: string
  password: string
  path: string
}>()

const emit = defineEmits<{
  close: []
  saved: [fileName: string]
  error: [message: string]
}>()

const fileContent = ref(props.content)
const loading = ref(false)
const hasChanges = ref(false)

const fullPath = computed(() => {
  if (props.path.endsWith('/')) {
    return props.path + props.fileName
  }
  return props.path + '/' + props.fileName
})

async function saveFile() {
  if (!hasChanges.value) return

  loading.value = true

  try {
    await saveRemoteFile({
      connectionString: props.connectionString,
      password: props.password,
      path: fullPath.value,
      content: fileContent.value,
    })

    hasChanges.value = false
    emit('saved', props.fileName)
  } catch (err) {
    emit('error', `Ошибка при сохранении файла: ${err}`)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.file-editor {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #f5f5f5;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background-color: #e0e0e0;
  border-bottom: 1px solid #ccc;
}

.editor-buttons {
  display: flex;
  gap: 10px;
}

.save-button {
  padding: 8px 16px;
  background-color: #4caf50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.save-button:hover:not(:disabled) {
  background-color: #45a049;
}

.save-button:disabled {
  background-color: #a5d6a7;
  cursor: not-allowed;
}

.cancel-button {
  padding: 8px 16px;
  background-color: #757575;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.cancel-button:hover {
  background-color: #616161;
}

.editor-content {
  flex: 1;
  padding: 16px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  border: none;
  resize: none;
  width: 100%;
  min-height: 300px;
  background-color: #fff;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}
</style>
