use brownie::Brownie;
use std::{
    sync::{mpsc, Arc},
    thread,
};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub mod brownie;
pub mod lowpass;

enum Command {
    Toggle,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let brownie = Arc::new(Brownie::new());

            let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let muted = CheckMenuItem::with_id(app, "muted", "Muted", true, false, None::<String>)?;

            let menu = MenuBuilder::new(app)
                .items(&[&quit, &separator, &muted])
                .build()?;

            let on_icon_path = app.path().resource_dir().unwrap().join("icons/icon.ico");

            let off_icon_path = app
                .path()
                .resource_dir()
                .unwrap()
                .join("icons-off/icon.ico");

            let on_icon = Image::from_path(on_icon_path).unwrap();
            let off_icon = Image::from_path(off_icon_path).unwrap();
            
            let (sender, receiver) = mpsc::channel::<Command>();
            let sender1 = sender.clone();
            let sender2 = sender.clone();

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Brownie (press to mute / unmute)")
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "muted" => {
                        sender1.send(Command::Toggle).unwrap();
                    }
                    _ => {}
                })
                .on_tray_icon_event(move |_, event| match event {
                    TrayIconEvent::Click {
                        button,
                        button_state,
                        ..
                    } => match button {
                        MouseButton::Left => match button_state {
                            MouseButtonState::Down => {
                                sender2.send(Command::Toggle).unwrap();
                            }
                            _ => (),
                        },
                        _ => (),
                    },
                    _ => (),
                })
                .build(app)?;

            thread::spawn(move || {
                while let Ok(command) = receiver.recv() {
                    match command {
                        Command::Toggle => {
                            if brownie.is_muted() {
                                tray.set_icon(Some(on_icon.clone())).unwrap();
                                muted.set_checked(false).unwrap();
                                brownie.unmute();
                            } else {
                                tray.set_icon(Some(off_icon.clone())).unwrap();
                                muted.set_checked(true).unwrap();
                                brownie.mute();
                            }
                        }
                    }
                }
            });

            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
