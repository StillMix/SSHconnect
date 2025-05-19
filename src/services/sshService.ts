import { invoke } from '@tauri-apps/api/tauri'
import { parseDirectoryListing } from '@/utils/fileUtils'
import type { FileEntry } from '@/utils/fileUtils'

interface ConnectionOptions {
  connectionString: string
  password: string
}

export async function listRemoteDirectories(options: ConnectionOptions): Promise<FileEntry[]> {
  try {
    const result = (await invoke('list_remote_directories', {
      connectionString: options.connectionString,
      password: options.password,
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
