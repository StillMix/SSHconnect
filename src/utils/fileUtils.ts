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

// Получение иконки в зависимости от типа файла и расширения
export function getFileIcon(type: string, name?: string): string {
  // Если это директория или симлинк, иконка не зависит от расширения
  switch (type) {
    case 'directory':
      return '📁'
    case 'symlink':
      return '🔗'
  }

  // Для файлов определяем иконку по расширению
  if (name) {
    // Получаем расширение файла (последняя часть после точки)
    const extension = name.split('.').pop()?.toLowerCase() || ''

    switch (extension) {
      // Текстовые файлы
      case 'txt':
      case 'md':
      case 'log':
        return '📄'

      // Исходный код
      case 'js':
      case 'ts':
      case 'jsx':
      case 'tsx':
      case 'vue':
      case 'py':
      case 'java':
      case 'c':
      case 'cpp':
      case 'h':
      case 'hpp':
      case 'cs':
      case 'php':
      case 'rb':
      case 'go':
      case 'rs':
      case 'swift':
      case 'kt':
        return '📝'

      // Web-файлы
      case 'html':
      case 'htm':
      case 'css':
      case 'scss':
      case 'sass':
      case 'less':
        return '🌐'

      // Конфигурационные файлы
      case 'json':
      case 'xml':
      case 'yaml':
      case 'yml':
      case 'toml':
      case 'ini':
      case 'conf':
      case 'config':
        return '⚙️'

      // Исполняемые файлы
      case 'exe':
      case 'dll':
      case 'so':
      case 'dylib':
      case 'sh':
      case 'bat':
      case 'cmd':
      case 'ps1':
        return '⚡'

      // Архивы
      case 'zip':
      case 'rar':
      case 'tar':
      case 'gz':
      case '7z':
      case 'bz2':
        return '📦'

      // Медиа-файлы
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif':
      case 'bmp':
      case 'svg':
      case 'ico':
        return '🖼️'

      case 'mp3':
      case 'wav':
      case 'ogg':
      case 'flac':
      case 'aac':
        return '🎵'

      case 'mp4':
      case 'avi':
      case 'mkv':
      case 'mov':
      case 'wmv':
      case 'flv':
        return '🎬'

      // Документы
      case 'pdf':
        return '📑'
      case 'doc':
      case 'docx':
        return '📘'
      case 'xls':
      case 'xlsx':
        return '📊'
      case 'ppt':
      case 'pptx':
        return '📽️'

      // По умолчанию для всех остальных файлов
      default:
        return '📄'
    }
  }

  // Если имя файла не передано или это неизвестный тип
  return '📄'
}

// Проверка, является ли файл текстовым (доступным для редактирования)
export function isTextFile(name: string): boolean {
  // Получаем расширение файла
  const extension = name.split('.').pop()?.toLowerCase() || ''

  // Список расширений текстовых файлов, которые можно редактировать
  const textExtensions = [
    // Обычные текстовые файлы
    'txt',
    'md',
    'log',
    'csv',
    'tsv',

    // Исходный код
    'js',
    'ts',
    'jsx',
    'tsx',
    'vue',
    'py',
    'java',
    'c',
    'cpp',
    'h',
    'hpp',
    'cs',
    'php',
    'rb',
    'go',
    'rs',
    'swift',
    'kt',
    'scala',
    'hs',
    'lua',

    // Web-файлы
    'html',
    'htm',
    'css',
    'scss',
    'sass',
    'less',

    // Конфигурационные файлы
    'json',
    'xml',
    'yaml',
    'yml',
    'toml',
    'ini',
    'conf',
    'config',

    // Скрипты
    'sh',
    'bat',
    'cmd',
    'ps1',

    // Системные файлы Linux
    'bashrc',
    'bash_profile',
    'profile',
    'vimrc',
    'gitconfig',
  ]

  // Проверяем, есть ли расширение в списке текстовых файлов
  // Если расширение отсутствует, считаем файл текстовым
  return extension === '' || textExtensions.includes(extension)
}
