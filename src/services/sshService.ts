import { invoke } from '@tauri-apps/api/tauri'
import { parseDirectoryListing } from '@/utils/fileUtils'
import type { FileEntry } from '@/utils/fileUtils'

interface ConnectionOptions {
  connectionString: string
  password: string
  path?: string
}

interface FileOptions extends ConnectionOptions {
  path: string
  content?: string
}

export async function listRemoteDirectories(options: ConnectionOptions): Promise<FileEntry[]> {
  try {
    // Добавляем команду ls с указанным путем
    const command = options.path ? `ls -la "${options.path}"` : 'ls -la'

    const result = (await invoke('execute_ssh_command', {
      connectionString: options.connectionString,
      password: options.password,
      command: command,
    })) as string[]

    return parseDirectoryListing(result)
  } catch (error) {
    throw error
  }
}

export async function readRemoteFile(options: FileOptions): Promise<string> {
  try {
    const command = `cat "${options.path}"`

    const result = (await invoke('execute_ssh_command', {
      connectionString: options.connectionString,
      password: options.password,
      command: command,
    })) as string[]

    // Объединяем строки результата в одну строку с переносами строк
    return result.join('\n')
  } catch (error) {
    throw error
  }
}

export async function saveRemoteFile(options: FileOptions): Promise<void> {
  if (!options.content) {
    throw new Error('Содержимое файла не может быть пустым')
  }

  try {
    // Создаем временный файл с контентом, а затем копируем его через SCP
    const result = await invoke('save_remote_file', {
      connectionString: options.connectionString,
      password: options.password,
      path: options.path,
      content: options.content,
    })

    return result as void
  } catch (error) {
    throw error
  }
}

export async function openPowershellWithCommand(command: string): Promise<void> {
  try {
    await invoke('open_powershell_with_command', { command })
  } catch (error) {
    throw error
  }
}
