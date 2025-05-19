//src-tauri/build.rs
fn main() {
    // Стандартный скрипт сборки Tauri
    tauri_build::build();
    
    // Гарантируем, что OUT_DIR существует для rust-analyzer
    include!("src/build_hack.rs");
    ensure_out_dir();
    
    // Сообщаем cargo, чтобы перекомпилировать при изменении этих файлов
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/build_hack.rs");
}