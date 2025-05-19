#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command};
use tauri::{command, Window};
use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;

use std::io::{ Write};

use tempfile::NamedTempFile;
use std::path::Path;


#[command]
fn open_powershell_with_command(_window: Window, command: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Сначала создаём строку с аргументом
        let runas_arg = format!(
            "Start-Process PowerShell -Verb RunAs -ArgumentList '-NoExit -Command \"{}\"'",
            command
        );

        // А потом уже передаём ссылки на статичные &str и на &runas_arg
        let args = vec![
            "-NoExit",
            "-Command",
            &runas_arg,
        ];

        Command::new("powershell")
            .args(&args)
            .spawn()
            .map_err(|e| format!("Не удалось запустить PowerShell: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Функция доступна только на Windows".to_string())
    }
}
fn connect_ssh(connection_string: &str, password: &str) -> Result<Session, String> {
    let mut parts = connection_string.splitn(2, '@');
    let user = parts.next().ok_or("Неверный формат подключения")?;
    let host = parts.next().ok_or("Неверный формат подключения")?;
    let addr = format!("{}:22", host);

    // подключаемся по TCP
    let tcp = TcpStream::connect(&addr)
        .map_err(|e| format!("Не удалось подключиться к {}: {}", addr, e))?;
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    sess.userauth_password(user, &password)
        .map_err(|e| format!("Аутентификация не удалась: {}", e))?;
    
    Ok(sess)
}
#[command]
fn execute_ssh_command(_window: Window,
                        connection_string: String,
                        password: String,
                        command: String) -> Result<Vec<String>, String> {
    let sess = connect_ssh(&connection_string, &password)?;

    // открываем сессию и выполняем команду
    let mut channel = sess.channel_session()
        .map_err(|e| format!("Не удалось открыть канал: {}", e))?;
    channel.exec(&command)
        .map_err(|e| format!("Не удалось выполнить команду: {}", e))?;
    let mut output = String::new();
    channel.read_to_string(&mut output)
        .map_err(|e| format!("Не удалось прочитать вывод: {}", e))?;
    channel.wait_close().ok();

    // парсим строки
    let lines = output
        .lines()
        .map(String::from)
        .collect::<Vec<_>>();
    
    Ok(lines)
}

#[cfg(not(target_os = "windows"))]
fn try_sshpass(connection_string: &str, password: &str) -> Result<Vec<String>, String> {
    // Проверяем наличие sshpass
    if Command::new("which").arg("sshpass").output().is_err() {
        return Err("sshpass не найден в системе".to_string());
    }
    
    // sshpass доступен, используем его
    let output = Command::new("sshpass")
        .args([
            "-p", password,
            "ssh", 
            "-o", "StrictHostKeyChecking=no",
            connection_string,
            "ls", "-la"
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Ошибка выполнения sshpass: {}", e))?;
            
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Ошибка SSH через sshpass: {}", error));
    }
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let directories: Vec<String> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
        
    Ok(directories)
}


#[command]
fn list_remote_directories(_window: Window,
                           connection_string: String,
                           password: String) -> Result<Vec<String>, String> {
    // разбиваем user@host
    let mut parts = connection_string.splitn(2, '@');
    let user = parts.next().ok_or("Неверный формат подключения")?;
    let host = parts.next().ok_or("Неверный формат подключения")?;
    let addr = format!("{}:22", host);

    // подключаемся по TCP
    let tcp = TcpStream::connect(&addr)
        .map_err(|e| format!("Не удалось подключиться к {}: {}", addr, e))?;
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    sess.userauth_password(user, &password)
        .map_err(|e| format!("Аутентификация не удалась: {}", e))?;

    // открываем сессию и выполняем ls
    let mut channel = sess.channel_session()
        .map_err(|e| format!("Не удалось открыть канал: {}", e))?;
    channel.exec("ls -la")
        .map_err(|e| format!("Не удалось выполнить команду: {}", e))?;
    let mut output = String::new();
    channel.read_to_string(&mut output)
        .map_err(|e| format!("Не удалось прочитать вывод: {}", e))?;
    channel.wait_close().ok();

    // парсим строки
    let dirs = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();
    if dirs.is_empty() {
        Err("Список директорий пуст".into())
    } else {
        Ok(dirs)
    }
}

#[command]
fn save_remote_file(_window: Window,
                    connection_string: String,
                    password: String,
                    path: String,
                    content: String) -> Result<(), String> {
    // Создаем временный файл для хранения содержимого
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Не удалось создать временный файл: {}", e))?;
    
    // Записываем содержимое во временный файл
    temp_file.write_all(content.as_bytes())
        .map_err(|e| format!("Не удалось записать содержимое во временный файл: {}", e))?;
    
    // Получаем путь к временному файлу
    let temp_path = temp_file.path().to_string_lossy().to_string();
    
    // Устанавливаем SSH соединение
    let sess = connect_ssh(&connection_string, &password)?;
    
    // Разбираем путь на директорию и имя файла
    let remote_path = Path::new(&path);
    let file_name = remote_path.file_name()
        .ok_or("Неверный путь к файлу")?
        .to_string_lossy();
    
    // Открываем канал SCP для записи файла
    let mut remote_file = sess.scp_send(
        remote_path,
        0o644,  // Права доступа (rw-r--r--)
        content.len() as u64,
        None
    ).map_err(|e| format!("Ошибка при создании SCP сессии: {}", e))?;
    
    // Отправляем содержимое файла
    remote_file.write_all(content.as_bytes())
        .map_err(|e| format!("Ошибка при записи файла: {}", e))?;
    
    // Закрываем соединение
    remote_file.send_eof()
        .map_err(|e| format!("Ошибка при закрытии файла: {}", e))?;
    remote_file.wait_eof()
        .map_err(|e| format!("Ошибка при ожидании EOF: {}", e))?;
    remote_file.close()
        .map_err(|e| format!("Ошибка при закрытии канала: {}", e))?;
    remote_file.wait_close()
        .map_err(|e| format!("Ошибка при ожидании закрытия канала: {}", e))?;
    
    Ok(())
}

fn main() {
    // Инициализируем Tauri приложение
    // Инициализируем Tauri приложение
    let context = tauri::generate_context!();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            execute_ssh_command,
            open_powershell_with_command,
            save_remote_file
        ])
        .run(context)
        .expect("Ошибка при запуске Tauri приложения");

}