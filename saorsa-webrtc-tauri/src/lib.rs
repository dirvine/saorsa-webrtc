//! Tauri plugin for desktop integration

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[tauri::command]
async fn initialize(identity: String) -> Result<(), String> {
    // TODO: Implement
    Ok(())
}

#[tauri::command]
async fn call(peer: String) -> Result<String, String> {
    // TODO: Implement
    Ok("call-id".to_string())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("saorsa-webrtc")
        .invoke_handler(tauri::generate_handler![
            initialize,
            call,
        ])
        .build()
}
