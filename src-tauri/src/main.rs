#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ax_interaction::setup_observers;
use commands::search_and_replace_commands;
use core_engine::CoreEngine;
use tauri::{Menu, MenuEntry, MenuItem, Submenu, SystemTrayEvent};
use window_controls::{EditorWindow, WidgetWindow, WindowControls};

use crate::window_controls::{
    cmd_toggle_app_activation, content_window::cmd_resize_content_window,
};
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};

mod ax_interaction;
mod commands;
mod core_engine;
mod utils;
mod window_controls;

use lazy_static::lazy_static;

lazy_static! {
    static ref APP_HANDLE: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
}

fn set_static_app_handle(app_handle: &tauri::AppHandle) {
    APP_HANDLE.lock().unwrap().replace(app_handle.clone());
}

pub fn app_handle() -> tauri::AppHandle {
    let app_handle = &*(match APP_HANDLE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    app_handle.as_ref().unwrap().clone()
}

fn main() {
    // Configure system tray
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    let mut app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_and_replace_commands::cmd_search_and_replace,
            cmd_resize_content_window,
            cmd_toggle_app_activation
        ])
        .setup(|app| {
            setup_observers(&app.handle());

            let handle = app.handle();

            // Create vector of editor windows
            let editor_windows_arc: Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>> =
                Arc::new(Mutex::new(HashMap::new()));

            let core_engine_arc = Arc::new(Mutex::new(CoreEngine::new(&handle)));
            CoreEngine::start_core_engine_listeners(&handle, &core_engine_arc);

            // Create instance of widget window; panics if creation fails
            let widget_window_arc =
                Arc::new(Mutex::new(WidgetWindow::new(&handle, &editor_windows_arc)));
            WidgetWindow::setup_widget_listeners(&handle, &widget_window_arc);

            let _window_controls = WindowControls::new(&handle, editor_windows_arc.clone());

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
