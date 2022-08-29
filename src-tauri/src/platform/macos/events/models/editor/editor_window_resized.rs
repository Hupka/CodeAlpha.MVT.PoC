use serde::{Deserialize, Serialize};

use crate::utils::geometry::LogicalSize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorWindowResizedMessage {
    pub window_uid: usize,
    pub window_position: tauri::LogicalPosition<f64>,
    pub window_size: tauri::LogicalSize<f64>,
    pub window_origin_delta: LogicalSize, // How much the window's origin has changed from the previous position
    pub textarea_position: Option<tauri::LogicalPosition<f64>>,
    pub textarea_size: Option<tauri::LogicalSize<f64>>,
}