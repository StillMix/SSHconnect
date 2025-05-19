//src-tauri/src/build_hack.rs
use std::env;
use std::fs;
use std::path::Path;

pub fn ensure_out_dir() {
    // Этот код будет выполняться при компиляции
    // и создавать директорию OUT_DIR, чтобы rust-analyzer не выдавал ошибку
    
    if env::var("OUT_DIR").is_err() {
        // Если переменная OUT_DIR не установлена, создадим директорию
        let out_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("target")
            .join("dummy_out");
            
        fs::create_dir_all(&out_dir).unwrap();
        
        // Создаем файл с конфигурацией для tauri
        let config_file = out_dir.join("tauri_config.json");
        fs::write(config_file, "{}").unwrap();
        
        // Устанавливаем OUT_DIR для текущего процесса
        env::set_var("OUT_DIR", out_dir);
    }
}