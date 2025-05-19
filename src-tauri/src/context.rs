#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::io::Write;
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
    
    // В Windows будем использовать команду plink из PuTTY
    #[cfg(target_os = "windows")]
    {
        // Проверим наличие plink
        let plink_check = Command::new("where")
            .arg("plink")
            .output();
            
        if let Ok(output) = plink_check {
            if output.status.success() {
                // Используем plink для подключения
                let mut cmd = Command::new("plink");
                cmd.args(["-batch", "-ssh", &format!("{}@{}", username, server), "-pw", &password, "ls"])
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
    }
    
    // Если plink недоступен или мы на другой системе, используем обычный SSH
    let mut cmd = Command::new("ssh");
    cmd.args([&connection_string, "ls"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Ошибка выполнения SSH: {}", e))?;

    // Если пароль предоставлен, попробуем его передать
    if let Some(mut stdin) = child.stdin.take() {
        if !password.is_empty() {
            // Даем немного времени на запрос пароля
            thread::sleep(Duration::from_millis(500));
            
            stdin.write_all(password.as_bytes())
                .map_err(|e| format!("Ошибка передачи пароля: {}", e))?;
            stdin.write_all(b"\n")
                .map_err(|e| format!("Ошибка передачи пароля: {}", e))?;
        }
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Ошибка получения вывода SSH: {}", e))?;

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