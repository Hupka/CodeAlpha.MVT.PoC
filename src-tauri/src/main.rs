#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ax_interaction::{setup_observers, utils::TauriState};
use plugins::xcode_state_plugin;
use tauri::Manager;
use utils::{window_controls, window_state_machine::WindowStateMachine};

mod ax_events;
mod ax_interaction;
mod plugins;
mod utils;

fn main() {
    let app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            window_controls::cmd_open_window,
            window_controls::cmd_toggle_window,
            window_controls::cmd_close_window,
            window_controls::cmd_resize_window,
            window_controls::cmd_is_window_visible,
            utils::window_positioning::cmd_update_widget_position,
            utils::window_positioning::cmd_start_dragging_widget,
            utils::window_positioning::cmd_update_content_position
        ])
        .plugin(xcode_state_plugin::init())
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    setup_observers(TauriState {
        handle: app.handle().clone(),
    });

    let mut window_state_machine = WindowStateMachine::new(app.handle().clone());
    window_state_machine.setup();
    app.manage(window_state_machine);

    // Load default windows
    window_controls::startup_windows(app.handle().clone());

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
