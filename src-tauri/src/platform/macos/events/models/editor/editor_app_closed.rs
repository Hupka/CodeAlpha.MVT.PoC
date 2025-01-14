use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorAppClosedMessage {
    pub editor_name: String,
    pub pid: u32,
    pub browser: Option<String>,
}
