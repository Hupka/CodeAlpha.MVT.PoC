use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{core_engine::EditorWindowUid, utils::geometry::LogicalFrame};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/node_annotation/")]
pub struct UpdateNodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
    pub annotation_icon: Option<LogicalFrame>,
    pub annotation_codeblock: Option<LogicalFrame>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/node_annotation/")]
pub struct RemoveNodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct NodeAnnotationClickedMessage {
    pub annotation_id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
}
