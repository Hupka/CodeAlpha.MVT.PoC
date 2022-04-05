use tauri::Manager;

use crate::window_controls::{
    close_window, get_window_label, open_window, resize_window, toggle_window, AppWindow,
};

#[tauri::command]
pub fn cmd_is_window_visible(handle: tauri::AppHandle, window_type: AppWindow) -> bool {
    if window_type == AppWindow::None {
        return false;
    }

    let app_window = handle.get_window(&get_window_label(window_type));

    if let Some(app_window) = app_window {
        return app_window.is_visible().unwrap();
    } else {
        return false;
    }
}

#[tauri::command]
pub fn cmd_open_window(handle: tauri::AppHandle, window_label: AppWindow) {
    open_window(handle.clone(), window_label);
}

#[tauri::command]
pub fn cmd_close_window(handle: tauri::AppHandle, window_label: AppWindow) {
    close_window(handle.clone(), window_label);
}

#[tauri::command]
pub fn cmd_toggle_window(handle: tauri::AppHandle, window_label: AppWindow) {
    toggle_window(handle.clone(), window_label);
}

#[tauri::command]
pub fn cmd_resize_window(
    handle: tauri::AppHandle,
    window_label: AppWindow,
    size_x: u32,
    size_y: u32,
) {
    resize_window(
        handle.clone(),
        window_label,
        tauri::LogicalSize {
            width: size_x as f64,
            height: size_y as f64,
        },
    );
}
