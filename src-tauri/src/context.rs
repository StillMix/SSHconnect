//src-tauri/src/context.rs
pub fn get_context() -> tauri::Context<tauri::utils::assets::EmbeddedAssets> {
    tauri::generate_context!()
}