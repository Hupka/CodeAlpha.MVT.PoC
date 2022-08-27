use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::{TextPosition, TextRange},
    utils::geometry::{LogicalFrame, LogicalPosition},
};
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightElbow {
    origin: Option<LogicalPosition>,
    origin_x_left_most: bool,
    bottom_line_top: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightBracket {
    pub text_range: TextRange,
    pub text_position: TextPosition,
    pub rectangle: LogicalFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightBracketPair {
    pub first: Option<BracketHighlightBracket>,
    pub last: Option<BracketHighlightBracket>,
}

impl BracketHighlightBracketPair {
    pub fn new(
        first_range: TextRange,
        first_rectangle: Option<LogicalFrame>,
        first_text_position: TextPosition,
        last_range: TextRange,
        last_rectangle: Option<LogicalFrame>,
        last_text_position: TextPosition,
    ) -> Self {
        let mut first = None;
        if let Some(first_rectangle) = first_rectangle {
            first = Some(BracketHighlightBracket {
                text_range: first_range,
                text_position: first_text_position,
                rectangle: LogicalFrame {
                    origin: first_rectangle.origin,
                    size: first_rectangle.size,
                },
            });
        }

        let mut last = None;
        if let Some(last_rectangle) = last_rectangle {
            last = Some(BracketHighlightBracket {
                text_range: last_range,
                text_position: last_text_position,
                rectangle: LogicalFrame {
                    origin: last_rectangle.origin,
                    size: last_rectangle.size,
                },
            });
        }

        Self { first, last }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightResults {
    lines: BracketHighlightBracketPair,
    elbow: Option<BracketHighlightElbow>,
    boxes: BracketHighlightBracketPair,
}
