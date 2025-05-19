import { invoke } from '@tauri-apps/api/tauri'
import { parseDirectoryListing } from '@/utils/fileUtils'
import type { FileEntry } from '@/utils/fileUtils'

interface ConnectionOptions {
  connectionString: string
  password: string
  path?: string
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

export async function openPowershellWithCommand(command: string): Promise<void> {
  try {
    await invoke('open_powershell_with_command', { command })
  } catch (error) {
    throw error
  }
}
