use std::sync::Arc;

use cocoa::{base::id, foundation::NSInteger};
use objc::{class, msg_send, sel, sel_impl};

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls::{
        config::{AppWindow, WindowLevel},
        utils::create_default_window_builder,
        windows::utils::{register_tracking_area, update_tracking_area},
        EventTrackingArea, TrackingArea,
    },
};

use super::listeners::window_control_events_listener;

#[derive(Clone, Debug)]
pub struct CodeOverlayWindow {
    app_handle: tauri::AppHandle,

    // The window's tracking area
    tracking_area: TrackingArea,
}

impl CodeOverlayWindow {
    pub fn new() -> Result<Self, tauri::Error> {
        let app_handle = app_handle();

        // Create CodeOverlay Window at startup.
        // If the window is already created, don't open it again.
        if app_handle
            .get_window(&AppWindow::CodeOverlay.to_string())
            .is_none()
        {
            let window_builder =
                create_default_window_builder(&app_handle, AppWindow::CodeOverlay)?;
            let window = window_builder.build()?;

            set_shadow(&window, false).expect("Unsupported platform!");

            #[cfg(debug_assertions)] // only include this code on debug builds
            window.open_devtools();
        }

        Ok(Self {
            app_handle,
            tracking_area: Self::register_tracking_area(),
        })
    }

    pub fn set_macos_properties(&self) -> Option<()> {
        let ns_window_ptr_overlay = self
            .app_handle
            .get_window(&AppWindow::CodeOverlay.to_string())?
            .ns_window();

        if let Ok(ns_window_ptr_overlay) = ns_window_ptr_overlay {
            unsafe {
                // Setting the mouse events to be ignored for the overlay window.
                if !msg_send![ns_window_ptr_overlay as id, ignoresMouseEvents] {
                    let _: () = msg_send![ns_window_ptr_overlay as id, setIgnoresMouseEvents: true];
                }

                // Set the code overlay's window level
                let _: () = msg_send![
                    ns_window_ptr_overlay as id,
                    setLevel: WindowLevel::CodeOverlay as NSInteger
                ];
            }
        }

        Some(())
    }

    pub fn start_event_listeners(code_overlay_window: &Arc<Mutex<CodeOverlayWindow>>) {
        window_control_events_listener(code_overlay_window);
    }

    pub fn show(&self, position: &LogicalPosition, size: &LogicalSize) -> Option<()> {
        let tauri_window = self
            .app_handle
            .get_window(&AppWindow::CodeOverlay.to_string())?;

        tauri_window
            .set_position(position.as_tauri_LogicalPosition())
            .ok()?;

        // Need to set the size with all of these calls, don't know why. See question here https://discord.com/channels/616186924390023171/1032276378617380946
        tauri_window.set_size(size.as_tauri_LogicalSize()).ok()?;
        tauri_window
            .set_min_size(Some(size.as_tauri_LogicalSize()))
            .ok()?;
        tauri_window
            .set_max_size(Some(size.as_tauri_LogicalSize()))
            .ok()?;

        tauri_window.show().ok()?;

        self.update_tracking_area(true);

        Some(())
    }

    pub fn hide(&self) -> Option<()> {
        _ = self
            .app_handle
            .get_window(&AppWindow::CodeOverlay.to_string())?
            .hide();

        self.update_tracking_area(false);

        Some(())
    }

    fn update_tracking_area(&self, is_visible: bool) {
        update_tracking_area(
            AppWindow::CodeOverlay,
            self.tracking_area.clone(),
            is_visible,
        )
    }

    fn register_tracking_area() -> TrackingArea {
        register_tracking_area(AppWindow::CodeOverlay)
    }

    fn _is_main_thread() -> Option<bool> {
        unsafe { Some(msg_send![class!(NSThread), isMainThread]) }
    }
}

impl Drop for CodeOverlayWindow {
    fn drop(&mut self) {
        EventTrackingArea::Remove(vec![self.tracking_area.id]).publish_to_tauri(&app_handle());
    }
}
