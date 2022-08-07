use std::thread;

use accessibility::AXUIElement;

use super::xcode::register_observer_xcode;

static LOOP_TIME_IN_MS: u64 = 150;

// This is the entry point for the Observer registrations. We register observers
// for the following notifications:
// - notifications from accessibility api for Xcode
// - subscribe to mouse(/keyboard) events
// It is called from the main thread at program startup
pub fn setup_observers(app_handle: &tauri::AppHandle) {
    let app_handle_move_copy = app_handle.clone();
    // Other apps than our own might only be restarted at a later point
    // This thread periodically checks if the app is running and registers the observers
    thread::spawn(move || {
        let mut xcode_app: Option<AXUIElement> = None;
        let mut _known_replit_editors: Vec<(String, AXUIElement)> = Vec::new();

        loop {
            // Register XCode observer (also take care of registering the mouse and keyboard observers due to the macOS runloop behavior)
            // =======================
            let _ = register_observer_xcode(&mut xcode_app, &app_handle_move_copy);

            // Register Replit observer
            // =======================
            // let _ = register_observer_replit(&mut known_replit_editors, &app_handle_move_copy);

            thread::sleep(std::time::Duration::from_millis(LOOP_TIME_IN_MS));
        }
    });
}
