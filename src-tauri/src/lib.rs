use std::sync::Arc;

use brownie::Brownie;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
};

pub mod brownie;

type AppState = Arc<Brownie>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let brownie = Arc::new(Brownie::new());

            let close = MenuItemBuilder::with_id("close", "Close").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&close]).build()?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "close" => {
                        app.exit(0);
                    }
                    _ => (),
                })
                .on_tray_icon_event(move |tray, event| {
                    let brownie = Arc::clone(&brownie);
                    match event {
                        TrayIconEvent::Click {
                            id: _,
                            position: _,
                            rect: _,
                            button: _,
                            button_state: _,
                        } => {
                            println!("playing");
                            if brownie.is_playing() {
                                brownie.pause();
                            } else {
                                brownie.play();
                            }
                        }
                        _ => todo!(),
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
