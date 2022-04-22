use std::sync::{Arc, Mutex};

use crate::ax_interaction::{
    is_currently_focused_app_our_app,
    models::editor::{
        EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppDeactivatedMessage,
        EditorUIElementFocusedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage,
        FocusedUIElement,
    },
};

use super::{
    widget_window::{
        hide_widget_routine, show_widget_routine, temporary_hide_check_routine, XCODE_EDITOR_NAME,
    },
    WidgetWindow,
};

pub fn on_resize_editor_window(
    app_handle: &tauri::AppHandle,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    resize_msg: &EditorWindowResizedMessage,
) {
    {
        let widget_props = &mut *(widget_arc.lock().unwrap());
        let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

        if let Some(editor_window) = editor_list_locked
            .iter_mut()
            .find(|window| window.id == resize_msg.id)
        {
            editor_window.update_window_dimensions(
                resize_msg.window_position,
                resize_msg.window_size,
                resize_msg.textarea_position,
                resize_msg.textarea_size,
            );
        } else {
            return;
        }
    }

    temporary_hide_check_routine(&app_handle, widget_arc);
}

pub fn on_move_editor_window(
    app_handle: &tauri::AppHandle,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    moved_msg: &EditorWindowMovedMessage,
) {
    {
        let widget_props = &mut *(widget_arc.lock().unwrap());
        let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

        if let Some(editor_window) = editor_list_locked
            .iter_mut()
            .find(|window| window.id == moved_msg.id)
        {
            editor_window.update_window_dimensions(
                moved_msg.window_position,
                moved_msg.window_size,
                None,
                None,
            );
        } else {
            return;
        }
    }

    temporary_hide_check_routine(&app_handle, widget_arc);
}

/// Update EditorWindow to which of it's ui elements is currently in focus. Furthermore, also update
/// which of all open editor windows is currently in focus.
pub fn on_editor_ui_element_focus_change(
    app_handle: &tauri::AppHandle,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    focus_msg: &EditorUIElementFocusedMessage,
) {
    // "Hack" - introduce this boolean to conveniently wrap subsequent logic in own block to have
    // mutex drop at the end.
    let mut need_temporary_hide = false;

    {
        let widget_props = &mut *(widget_arc.lock().unwrap());
        let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

        // Update the focused ui element on the corresponding editor window instance.
        if let Some(editor_window) = editor_list_locked
            .iter_mut()
            .find(|window| window.id == focus_msg.window_id)
        {
            editor_window.update_focused_ui_element(
                &focus_msg.focused_ui_element,
                focus_msg.textarea_position,
                focus_msg.textarea_size,
            );
        } else {
            return;
        }

        if let Some(previously_focused_window_id) = widget_props.currently_focused_editor_window {
            if previously_focused_window_id != focus_msg.window_id {
                if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                    need_temporary_hide = true;
                } else {
                    hide_widget_routine(
                        &widget_props.app_handle,
                        widget_props,
                        &mut editor_list_locked,
                    )
                }
            } else {
                if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                    show_widget_routine(
                        &widget_props.app_handle,
                        widget_props,
                        &mut editor_list_locked,
                    )
                } else {
                    hide_widget_routine(
                        &widget_props.app_handle,
                        widget_props,
                        &mut editor_list_locked,
                    )
                }
            }
        }

        // Set which editor window is currently focused
        widget_props.currently_focused_editor_window = Some(focus_msg.window_id);
        widget_props.is_xcode_focused = true;
    }

    if need_temporary_hide {
        temporary_hide_check_routine(&app_handle, widget_arc);
    }
}

pub fn on_deactivate_editor_app(
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    deactivated_msg: &EditorAppDeactivatedMessage,
) {
    let widget_window = &mut *(widget_arc.lock().unwrap());

    if deactivated_msg.editor_name == XCODE_EDITOR_NAME {
        widget_window.is_xcode_focused = false;
    }

    if let Some(is_focused_app_our_app) = is_currently_focused_app_our_app() {
        if !is_focused_app_our_app {
            let editor_windows = &mut *(widget_window.editor_windows.lock().unwrap());
            hide_widget_routine(&widget_window.app_handle, &widget_window, editor_windows);
        }
    }
}

pub fn on_close_editor_app(
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    closed_msg: &EditorAppClosedMessage,
) {
    let widget_window = &mut *(widget_arc.lock().unwrap());

    if closed_msg.editor_name == XCODE_EDITOR_NAME {
        widget_window.is_xcode_focused = false;

        let editor_windows = &mut *(widget_window.editor_windows.lock().unwrap());
        hide_widget_routine(&widget_window.app_handle, &widget_window, editor_windows);
    }
}

pub fn on_activate_editor_app(
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    activated_msg: &EditorAppActivatedMessage,
) {
    let widget_props = &mut *(widget_arc.lock().unwrap());
    let editor_list_locked = widget_props.editor_windows.lock().unwrap();

    // Check if focused ui element of the currently focused editor window is textarea.
    if let Some(currently_focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_list_locked
            .iter()
            .find(|window| window.id == currently_focused_editor_window_id)
        {
            if let Some(focused_ui_element) = &editor_window.focused_ui_element {
                if *focused_ui_element == FocusedUIElement::Textarea {
                    show_widget_routine(&widget_props.app_handle, widget_props, &editor_list_locked)
                }
            }
        }
    }

    if activated_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = true;
    }
}
