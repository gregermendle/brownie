use std::sync::Arc;

use brownie::Brownie;
use tauri::{AppHandle, Manager};

pub mod brownie;

type AppState = Arc<Brownie>;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(app: AppHandle) {
    let store = app.state::<AppState>();
    let brownie = Arc::clone(&store);
    if brownie.is_playing() {
        brownie.pause()
    } else {
        brownie.play();
    }
    ()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let brownie = Arc::new(Brownie::new());
            app.manage::<AppState>(brownie);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
