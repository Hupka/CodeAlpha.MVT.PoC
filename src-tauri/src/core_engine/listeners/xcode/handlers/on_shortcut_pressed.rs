use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorShortcutPressedMessage, core_engine::CoreEngine,
};

pub fn on_editor_shortcut_pressed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    msg: &EditorShortcutPressedMessage,
) {
    let core_engine = &mut core_engine_arc.lock();

    // Checking if the engine is active. If not, don't continue.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = core_engine.code_documents().lock();

    if let Some(code_doc) = code_documents.get_mut(&msg.ui_elem_hash) {
        code_doc.on_save(&msg);
    }
}
