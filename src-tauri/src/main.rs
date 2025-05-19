#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::io::{Write, BufReader, BufRead};
use std::thread;
use std::time::Duration;
use tauri::{command, Window};

#[command]
fn list_remote_directories(_window: Window, connection_string: String, password: String) -> Result<Vec<String>, String> {
    let parts: Vec<&str> = connection_string.split('@').collect();
    if parts.len() != 2 {
        return Err("Неверный формат строки подключения. Используйте формат username@serverip".to_string());
    }

    let _username = parts[0];
    let _server = parts[1];
    
    // Сначала проверим, является ли система Windows
    #[cfg(target_os = "windows")]
    {
        // Пробуем использовать команду cmd для перенаправления пароля в ssh
        match try_windows_ssh(connection_string.as_str(), password.as_str()) {
            Ok(directories) => return Ok(directories),
            Err(e) => {
                eprintln!("Windows SSH метод не удался: {}", e);
                // Если метод Windows SSH не сработал, пробуем прямой SSH
                return try_direct_ssh(&connection_string, &password);
            }
        }
    }
    
    // Для Linux/Mac
    #[cfg(not(target_os = "windows"))]
    {
        // Пробуем сначала sshpass
        match try_sshpass(&connection_string, &password) {
            Ok(directories) => return Ok(directories),
            Err(e) => {
                eprintln!("sshpass метод не удался: {}", e);
                // Пробуем прямой SSH
                return try_direct_ssh(&connection_string, &password);
            }
        }
    }
}

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


#[cfg(target_os = "windows")]
fn try_windows_ssh(connection_string: &str, password: &str) -> Result<Vec<String>, String> {
    // 1. Создаём временный файл и пишем в него пароль
    use tempfile::NamedTempFile;
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Не удалось создать временный файл: {}", e))?;
    temp_file.write_all(password.as_bytes())
        .map_err(|e| format!("Не удалось записать пароль: {}", e))?;
    let password_file = temp_file.path().to_string_lossy().into_owned();

    // 2. Формируем команду заранее и хранем её в String
    let ssh_cmd = format!(
        "type \"{}\" | ssh -o StrictHostKeyChecking=no {} ls -la",
        password_file, connection_string
    );

    // 3. Вызываем cmd, прокидывая ссылку на живой ssh_cmd
    let output = Command::new("cmd")
        .args(&["/C", &ssh_cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Ошибка выполнения SSH: {}", e))?;

    // 4. Обработка результата как раньше
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("Permission denied") {
            return Err("Неверный пароль или имя пользователя".into());
        }
        return Err(format!("SSH вернул ошибку: {}", err));
    }

    let out = String::from_utf8_lossy(&output.stdout);
    let dirs = out.lines().map(String::from).filter(|l| !l.is_empty()).collect::<Vec<_>>();
    if dirs.is_empty() {
        return Err("Не удалось получить список директорий".into());
    }
    Ok(dirs)
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

fn try_direct_ssh(connection_string: &str, password: &str) -> Result<Vec<String>, String> {
    // Создаем процесс ssh с поддержкой интерактивного ввода пароля
    let mut cmd = Command::new("ssh");
    cmd.args([
        "-o", "StrictHostKeyChecking=no",
        connection_string,
        "ls", "-la"
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    // Запускаем процесс
    let mut child = cmd.spawn()
        .map_err(|e| format!("Ошибка запуска SSH: {}", e))?;

    // Получаем stdin и stderr
    if let (Some(mut stdin), Some(stderr)) = (child.stdin.take(), child.stderr.take()) {
        // Создаем читателя для stderr
        let stderr_reader = BufReader::new(stderr);
        
        // Клонируем пароль для использования в потоке
        let password = password.to_string();
        
        // Запускаем поток для мониторинга stderr и ввода пароля при необходимости
        let handle = thread::spawn(move || {
            for line in stderr_reader.lines() {
                if let Ok(line) = line {
                    // Если видим запрос пароля, вводим его
                    if line.contains("password:") || line.contains("Password:") {
                        thread::sleep(Duration::from_millis(500));
                        
                        if let Err(e) = stdin.write_all(format!("{}\n", password).as_bytes()) {
                            eprintln!("Ошибка при вводе пароля: {}", e);
                            return false;
                        }
                        
                        if let Err(e) = stdin.flush() {
                            eprintln!("Ошибка при отправке пароля: {}", e);
                            return false;
                        }
                        
                        return true; // Пароль введен успешно
                    } else if line.contains("Permission denied") {
                        // Ошибка аутентификации
                        return false;
                    }
                }
            }
            
            false // Пароль не запрашивался или произошла ошибка
        });
        
        // Ждем завершения ввода пароля (максимум 10 секунд)
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if handle.is_finished() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    // Ждем завершения процесса и получаем вывод
    let result = child.wait_with_output();
    
    // Обрабатываем результат
    let output = match result {
        Ok(output) => output,
        Err(e) => {
            return Err(format!("Ошибка получения вывода SSH: {}", e));
        }
    };

    // Проверяем успешность выполнения
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        
        if error.contains("Permission denied") {
            return Err("Неверный пароль или имя пользователя".to_string());
        }
        
        return Err(format!("Ошибка SSH: {}", error));
    }

    // Обрабатываем stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    let directories: Vec<String> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    if directories.is_empty() {
        return Err("Не удалось получить список файлов или директория пуста".to_string());
    }
    
    Ok(directories)
}

fn main() {
    // Инициализируем Tauri приложение
    let context = tauri::generate_context!();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_remote_directories,
            open_powershell_with_command
        ])
        .run(context)
        .expect("Ошибка при запуске Tauri приложения");
}