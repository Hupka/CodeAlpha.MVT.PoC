use tauri::Manager;

use crate::window_controls::{config::AppWindow, get_window_label};

use super::create::create_window;

pub fn open_window(handle: tauri::AppHandle, window_type: AppWindow) {
    if window_type == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&get_window_label(window_type));

    if let Some(app_window) = app_window {
        let _ = app_window.show();
    } else {
        create_window(handle, window_type);
    }
}
