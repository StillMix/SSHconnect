// Интерфейс для файловых записей
export interface FileEntry {
  name: string
  type: string
  size?: string
  permissions?: string
  date?: string
}

// Функция для разбора вывода ls -la
export function parseDirectoryListing(listing: string[]): FileEntry[] {
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

// Получение иконки в зависимости от типа файла
export function getFileIcon(type: string): string {
  switch (type) {
    case 'directory':
      return '📁'
    case 'symlink':
      return '🔗'
    default:
      return '📄'
  }
}
