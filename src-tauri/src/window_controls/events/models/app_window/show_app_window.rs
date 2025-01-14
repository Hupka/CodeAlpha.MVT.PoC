use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    platform::macos::{CodeDocumentFrameProperties, ViewportProperties},
    utils::geometry::{LogicalFrame, LogicalPosition},
    window_controls::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct ShowAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
    pub viewport: ViewportProperties,
    pub code_document: CodeDocumentFrameProperties,
    pub monitor: LogicalFrame,
    pub editor_textarea: LogicalFrame,
    pub widget_position: Option<LogicalPosition>,
    pub explain_window_anchor: Option<LogicalFrame>,
}
