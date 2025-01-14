use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use cocoa::appkit::CGPoint;
use core_foundation::base::{CFEqual, TCFType};
use core_graphics_types::geometry::CGSize;

use crate::{
    platform::macos::{
        get_minimal_viewport_properties, get_viewport_frame,
        models::editor::EditorWindowResizedMessage, xcode::XCodeObserverState, AXEventXcode,
        GetVia,
    },
    utils::{assert_or_error_trace, geometry::LogicalSize},
};

/// Notify Tauri that an editor window has been resized
/// Method requires AXUIElement of type "AXScrollBar". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_resized(
    ui_element: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let window_element = if let Ok(window_elem) = ui_element.window() {
        window_elem
    } else {
        let ui_elem_role = ui_element.role()?;

        if ui_elem_role.to_string() != "AXWindow" {
            return Ok(());
        }

        ui_element.clone()
    };

    // Find window_element in xcode_observer_state.window_list to get id
    let mut known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = &mut known_window {
        // Get updated window position and size
        let pos_ax_value = window_element.position()?;
        let size_ax_value = window_element.size()?;

        let origin = pos_ax_value.get_value::<CGPoint>()?;
        let size = size_ax_value.get_value::<CGSize>()?;

        // Set editor window dimensions
        let (viewport_props, code_doc_props) = get_minimal_viewport_properties(&GetVia::Current)
            .map_err(|_| accessibility::Error::NotFound)?;

        let mut resize_msg = EditorWindowResizedMessage {
            window_uid: window.0,
            window_position: tauri::LogicalPosition {
                x: origin.x,
                y: origin.y,
            },
            window_size: tauri::LogicalSize {
                width: size.width,
                height: size.height,
            },
            textarea_position: None,
            textarea_size: None,
            window_origin_delta: LogicalSize {
                width: origin.x - window.2.x,
                height: origin.y - window.2.y,
            },
            viewport: viewport_props,
            code_document: code_doc_props,
        };

        if "AXScrollBar" == ui_element.role()? {
            // Determine editor textarea dimensions
            // For now at least, ignore errors and still continue with control flow.
            let _ = derive_resize_parameters_from_scrollbar(&mut resize_msg, ui_element);

            // Avoid spam by checking if the editor textarea dimensions have changed
            if let (Some(old_size), Some(new_size)) = (window.3, resize_msg.textarea_size) {
                if old_size.width as i32 == new_size.width as i32
                    && old_size.height as i32 == new_size.height as i32
                {
                    // Don't publish new event because nothing has changed --> the event was likely emited by a scroll event rather than resize
                    return Ok(());
                }
            } else {
                // Don't publish new event because textarea dimensions are not available
                return Ok(());
            }

            let new_tuple = (
                window.0,
                window.1.clone(),
                resize_msg.window_position,
                resize_msg.textarea_size.clone(),
            );

            // Update item window_list
            xcode_observer_state
                .window_list
                .retain(|vec_elem| vec_elem.0 != new_tuple.0);

            xcode_observer_state.window_list.push(new_tuple);
        }

        AXEventXcode::EditorWindowResized(resize_msg)
            .publish_to_tauri(&xcode_observer_state.app_handle);
    }

    Ok(())
}

/// The editor field (TextArea) is children to a AXScrollArea. This AXScrollArea has four children, only one of
/// which is the AXTextArea containing the source code. In order to determine the exact dimensions
/// of AXTextArea, we need to substract the width of all other children of AXScrollArea
/// From left to right, the children are:
///   - Identifier: "Source Editor Change Gutter", role: AXGroup
///   - Identifier: "Source Editor Gutter", role: "AXGroup"
///   - Identifier: -, role "AXTextArea" <-- This is the AXTextArea containing the source code
///   - Identifier: "Source Editor Minimap", role: "AXGroup"
fn derive_resize_parameters_from_scrollbar(
    resize_msg: &mut EditorWindowResizedMessage,
    scrollbar_element: &AXUIElement,
) -> Result<(), Error> {
    let role = scrollbar_element.role()?;
    assert_or_error_trace(
        role.to_string() == "AXScrollBar",
        &format!(
            "derive_resize_parameters_from_scrollbar() called with scrollbar_element of type {}; expected AXScrollBar",
            role.to_string()
        ),
    );

    if let Ok(code_section_frame) = get_viewport_frame(&GetVia::Current) {
        // Update EditorWindowResizedMessage
        resize_msg.textarea_position = Some(code_section_frame.origin.as_tauri_LogicalPosition());
        resize_msg.textarea_size = Some(code_section_frame.size.as_tauri_LogicalSize());
    }

    Ok(())
}
