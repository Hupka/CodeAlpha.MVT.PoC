use accessibility::{AXAttribute, AXUIElement, Error};
use accessibility_sys::kAXErrorInvalidUIElement;
use core_foundation::base::{CFEqual, TCFType};

use crate::{
    platform::macos::{
        models::editor::EditorWindowDestroyedMessage, xcode::XCodeObserverState, AXEventXcode,
    },
    utils::assert_or_error_trace,
};

/// Notify Tauri that an editor window has been destroyed
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_destroyed(
    ui_element: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let mut app_element = ui_element.clone();
    if "AXWindow" == app_element.attribute(&AXAttribute::role())? {
        app_element = app_element.attribute(&AXAttribute::parent())?;
    } else if "AXApplication" != app_element.attribute(&AXAttribute::role())? {
        return Err(Error::Ax(kAXErrorInvalidUIElement));
    }

    let role = app_element.attribute(&AXAttribute::role())?;
    assert_or_error_trace(
        role.to_string() == "AXApplication",
        &format!(
            "notify_window_destroyed() called with app_element of type {}; expected AXApplication",
            role.to_string()
        ),
    );

    let windows = app_element.attribute(&AXAttribute::children())?;

    let _ = &xcode_observer_state.window_list.retain(|known_window| {
        let mut still_exists = false;
        for window in &windows {
            unsafe {
                if CFEqual(window.as_CFTypeRef(), known_window.1.as_CFTypeRef()) != 0 {
                    still_exists = true;
                    break;
                }
            }
        }

        if !still_exists {
            AXEventXcode::EditorWindowDestroyed(EditorWindowDestroyedMessage {
                window_uid: known_window.0,
            })
            .publish_to_tauri(&xcode_observer_state.app_handle);
        }

        // returning false in Vec::retain() will remove the element from the vector
        still_exists
    });

    Ok(())
}
