use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppDeactivatedMessage {
    pub app_name: String,
    pub pid: u32,
}