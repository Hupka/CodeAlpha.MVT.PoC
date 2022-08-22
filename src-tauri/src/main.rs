#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};

use ax_interaction::setup_observers;
use core_engine::CoreEngine;
use tauri::{Menu, MenuEntry, MenuItem, Submenu, SystemTrayEvent};
use window_controls::WindowManager;

use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};

mod ax_interaction;
mod core_engine;
mod utils;
mod window_controls;

use lazy_static::lazy_static;

use crate::window_controls::cmd_toggle_app_activation;

lazy_static! {
    static ref APP_HANDLE: parking_lot::Mutex<Option<tauri::AppHandle>> =
        parking_lot::Mutex::new(None);
}

pub static CORE_ENGINE_ACTIVE_AT_STARTUP: bool = true;
pub static DEV_MODE: bool = true;

fn set_static_app_handle(app_handle: &tauri::AppHandle) {
    APP_HANDLE.lock().replace(app_handle.clone());
}

pub fn app_handle() -> tauri::AppHandle {
    let app_handle = APP_HANDLE.lock().clone();

    app_handle.as_ref().unwrap().clone()
}

fn main() {
    // Configure system tray
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    let mut app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cmd_toggle_app_activation])
        .setup(|app| {
            // Set the app handle for the static APP_HANDLE variable
            set_static_app_handle(&app.handle());

            // Setup the observers for AX interactions and mouse events
            setup_observers();

            let core_engine_arc = Arc::new(Mutex::new(CoreEngine::new()));
            CoreEngine::start_core_engine_listeners(&core_engine_arc);

            // Start the window manager instance
            let window_manager = Arc::new(parking_lot::Mutex::new(WindowManager::new()?));
            WindowManager::start_event_listeners(&window_manager);

            // Continuously check if the accessibility APIs are enabled, show popup if not
            let handle_move_copy = app.handle().clone();
            let ax_apis_enabled_at_start = ax_interaction::application_is_trusted();
            tauri::async_runtime::spawn(async move {
                loop {
                    if ax_interaction::application_is_trusted_with_prompt() {
                        // In case AX apis were not enabled at program start, restart the app to
                        // ensure the AX observers are properly registered.
                        if !ax_apis_enabled_at_start {
                            handle_move_copy.restart();
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            });

            // Spin up a thread to detect potential Mutex deadlocks.
            deadlock_detection();

            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|_app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .menu(Menu::with_items([
            #[cfg(target_os = "macos")]
            MenuEntry::Submenu(Submenu::new(
                "dummy-menu-for-shortcuts-to-work-on-input-fields-see-github-issue-#-1055",
                Menu::with_items([
                    MenuItem::Undo.into(),
                    MenuItem::Redo.into(),
                    MenuItem::Cut.into(),
                    MenuItem::Copy.into(),
                    MenuItem::Paste.into(),
                    MenuItem::SelectAll.into(),
                ]),
            )),
        ]))
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}

fn deadlock_detection() {
    use parking_lot::deadlock;
    use std::thread;
    use std::time::Duration;

    // Create a background thread which checks for deadlocks every 2s
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        let deadlocks = deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        println!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            println!("Deadlock #{}", i);
            for t in threads {
                println!("Thread Id {:#?}", t.thread_id());
                println!("{:#?}", t.backtrace());
            }
        }
    });
}
