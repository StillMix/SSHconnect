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

    let username = parts[0];
    let server = parts[1];
    
    // Сначала проверим, является ли система Windows
    #[cfg(target_os = "windows")]
    {
        // Используем plink в Windows, если он доступен
        let plink_result = try_plink(username, server, &password);
        if let Ok(directories) = plink_result {
            return Ok(directories);
        }
    }
    
    // Для Linux/Mac, сначала попробуем sshpass
    #[cfg(not(target_os = "windows"))]
    {
        let sshpass_result = try_sshpass(&connection_string, &password);
        if let Ok(directories) = sshpass_result {
            return Ok(directories);
        }
    }
    
    // Если предыдущие методы не сработали, используем прямой SSH
    try_direct_ssh(&connection_string, &password)
}

#[cfg(target_os = "windows")]
fn try_plink(username: &str, server: &str, password: &str) -> Result<Vec<String>, String> {
    // Проверяем наличие plink
    let plink_check = Command::new("where")
        .arg("plink")
        .output();
        
    if let Ok(output) = plink_check {
        if output.status.success() {
            // Используем plink для подключения с передачей пароля
            let mut cmd = Command::new("plink");
            cmd.args([
                "-batch",
                "-ssh",
                &format!("{}@{}", username, server),
                "-pw", password,
                "ls", "-la"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
               
            let output = cmd.output()
                .map_err(|e| format!("Ошибка выполнения plink: {}", e))?;
                
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Ошибка SSH: {}", error));
            }
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let directories: Vec<String> = stdout
                .lines()
                .filter(|line| !line.is_empty())
                .map(String::from)
                .collect();
            
            return Ok(directories);
        }
    }
    
    Err("plink не найден в системе".to_string())
}

#[cfg(not(target_os = "windows"))]
fn try_sshpass(connection_string: &str, password: &str) -> Result<Vec<String>, String> {
    let sshpass_check = Command::new("which")
        .arg("sshpass")
        .output();
        
    if let Ok(output) = sshpass_check {
        if output.status.success() {
            // sshpass доступен, используем его
            let mut cmd = Command::new("sshpass");
            cmd.args([
                "-p", password,
                "ssh", 
                "-o", "StrictHostKeyChecking=no",
                connection_string,
                "ls", "-la"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
            let output = cmd.output()
                .map_err(|e| format!("Ошибка выполнения sshpass: {}", e))?;
                
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Ошибка SSH: {}", error));
            }
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let directories: Vec<String> = stdout
                .lines()
                .filter(|line| !line.is_empty())
                .map(String::from)
                .collect();
            
            return Ok(directories);
        }
    }
    
    Err("sshpass не найден в системе".to_string())
}

fn try_direct_ssh(connection_string: &str, password: &str) -> Result<Vec<String>, String> {
    // Создаем процесс ssh
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

    // Проверяем stdin и stderr
    if child.stdin.is_some() && child.stderr.is_some() {
        let stdin = child.stdin.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
        // Создаем читателя для stderr
        let stderr_reader = BufReader::new(stderr);
        let password_clone = password.to_string();
        let mut stdin_clone = stdin;

        // Запускаем отдельный поток для мониторинга stderr
        thread::spawn(move || {
            // Читаем строки из stderr
            for line in stderr_reader.lines() {
                if let Ok(line) = line {
                    // Если видим запрос пароля, вводим его
                    if line.contains("password") {
                        // Небольшая задержка перед вводом пароля
                        thread::sleep(Duration::from_millis(500));
                        
                        // Вводим пароль и перевод строки
                        let _ = stdin_clone.write_all(format!("{}\n", password_clone).as_bytes());
                        let _ = stdin_clone.flush();
                        break;
                    }
                }
            }
        });
    }

    // Ждем завершения процесса и получаем вывод
    let output = child.wait_with_output()
        .map_err(|e| format!("Ошибка получения вывода SSH: {}", e))?;

    // Проверяем успешность выполнения
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        
        // Если ошибка содержит "Permission denied", значит пароль неверный
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
        return Err("Не удалось получить список файлов. Директория может быть пустой или возникла ошибка.".to_string());
    }
    
    Ok(directories)
}

fn main() {
    // Инициализируем Tauri приложение
    let context = tauri::generate_context!();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_remote_directories])
        .run(context)
        .expect("Ошибка при запуске Tauri приложения");
}