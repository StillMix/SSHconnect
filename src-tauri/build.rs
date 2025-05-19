fn main() {
    // Стандартный скрипт сборки Tauri
    tauri_build::build();
    
    // Гарантируем, что OUT_DIR существует для rust-analyzer
    let _ = include_str!("src/build_hack.rs");
    
    // Функция ensure_out_dir
    use std::env;
    use std::fs;
    use std::path::Path;
    
    // Создаем OUT_DIR, если его нет
    if env::var("OUT_DIR").is_err() {
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
    
    // Сообщаем cargo, чтобы перекомпилировать при изменении этих файлов
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/build_hack.rs");
}