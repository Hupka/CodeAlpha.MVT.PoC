use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorWindowCreatedMessage,
    window_controls_two::{windows::EditorWindow, WindowManager},
};

pub fn on_created_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    created_msg: &EditorWindowCreatedMessage,
) {
    let window_manager = window_manager.lock();
    let editor_window_list = &mut window_manager.editor_windows().lock();

    // Add the new editor window to the list of editor windows.
    if editor_window_list.get(&created_msg.ui_elem_hash).is_none() {
        editor_window_list.insert(created_msg.ui_elem_hash, EditorWindow::new(created_msg));
    }
}