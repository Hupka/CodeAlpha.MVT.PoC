#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ax_interaction::{setup_observers, utils::TauriState};
use tauri::{command, window, AppHandle, Manager, StateManager, WindowUrl};
use utils::window_state_machine::WindowStateMachine;

use crate::commands::window_control_commands;

mod ax_events;
mod ax_interaction;
mod commands;
mod utils;
mod window_controls;

#[command]
// fn create_child_window(id: String, app: AppHandle) {
//     let main = app.get_window("main").unwrap();

//     let child = window::WindowBuilder::new(&app, id, WindowUrl::default())
//         .title("Child")
//         .inner_size(400.0, 300.0);

//     #[cfg(target_os = "macos")]
//     let child = child.parent_window(main.ns_window().unwrap());
//     #[cfg(target_os = "windows")]
//     let child = child.parent_window(main.hwnd().unwrap());

//     child.build();
// }

fn main() {
    let app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            window_control_commands::cmd_open_window,
            window_control_commands::cmd_toggle_window,
            window_control_commands::cmd_close_window,
            window_control_commands::cmd_resize_window,
            window_control_commands::cmd_is_window_visible,
            utils::window_positioning::cmd_update_widget_position,
            utils::window_positioning::cmd_start_dragging_widget,
            utils::window_positioning::cmd_update_content_position
        ])
        .setup(|app| {
            setup_observers(TauriState {
                handle: app.handle().clone(),
            });

            let mut window_state_machine = WindowStateMachine::new(app.handle().clone());
            window_state_machine.setup();
            app.manage(window_state_machine);

            let mut window_state = window_controls::WindowStateManager::new(app.handle().clone());
            window_state.launch_startup_windows();
            Ok(())
        })
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}