use std::thread;

use tauri::{AppHandle, Manager};

pub mod brownie;

#[derive(Default)]
struct Store {
    playing: bool,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(app: AppHandle) {
    let store = app.state::<Store>();
    // store.playing = true;
    ()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    thread::spawn(move || {
        let _ = brownie::brownie();
    });
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Store { playing: false });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
