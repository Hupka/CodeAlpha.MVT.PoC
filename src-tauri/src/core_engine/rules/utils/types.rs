use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::rules::swift_linter::LintLevel,
    core_engine::rules::RuleMatch,
    utils::geometry::{LogicalPosition, LogicalSize},
};

use super::text_types::TextRange;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub enum RuleName {
    BracketHighlight,
    SearchAndReplace,
    SwiftLinter,
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub struct RuleResults {
    pub rule: RuleName,
    pub results: Vec<RuleMatch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct MatchRectangle {
    pub origin: LogicalPosition,
    pub size: LogicalSize,
}

impl MatchRectangle {
    pub fn contains_point(&self, mouse_x: f64, mouse_y: f64) -> bool {
        // Check if mouse_x and mouse_y are within the bounds of the rectangle.
        let x_in_bounds = mouse_x >= self.origin.x && mouse_x <= self.origin.x + self.size.width;
        let y_in_bounds = mouse_y >= self.origin.y && mouse_y <= self.origin.y + self.size.height;
        x_in_bounds && y_in_bounds
    }
}

pub type LineMatch = (MatchRange, Vec<MatchRectangle>);

#[cfg(test)]
mod tests_MatchRectangle {

    use super::MatchRectangle;
    use crate::utils::geometry::{LogicalPosition, LogicalSize};

    #[test]
    fn test_contains_point() {
        let rectangle = MatchRectangle {
            origin: LogicalPosition { x: 0.0, y: 0.0 },
            size: LogicalSize {
                width: 100.0,
                height: 100.0,
            },
        };

        assert!(rectangle.contains_point(50., 50.));
        assert!(rectangle.contains_point(0., 0.));
        assert!(rectangle.contains_point(100., 100.));
        assert!(!rectangle.contains_point(101., 100.));
        assert!(!rectangle.contains_point(100., 101.));
        assert!(!rectangle.contains_point(150., 150.));
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct MatchRange {
    pub string: Vec<u16>,
    pub range: TextRange,
}

impl MatchRange {
    pub fn from_text_and_range(text: &Vec<u16>, range: TextRange) -> Option<Self> {
        if text.len() < range.index + range.length {
            return None;
        }
        Some(Self {
            string: text[(range.index)..(range.index + range.length)].to_vec(),
            range,
        })
    }
}

#[cfg(test)]
mod tests_MatchRange {

    use crate::core_engine::rules::TextRange;

    use super::MatchRange;

    #[test]
    fn test_from_text_and_range() {
        let s = &"0123456789".encode_utf16().collect();

        let match_range = MatchRange::from_text_and_range(
            s,
            TextRange {
                index: 2,
                length: 5,
            },
        );

        assert_eq!(
            match_range,
            Some(MatchRange {
                string: "23456".encode_utf16().collect(),
                range: TextRange {
                    index: 2,
                    length: 5,
                },
            })
        );

        let match_range_out_of_range = MatchRange::from_text_and_range(
            s,
            TextRange {
                index: 10,
                length: 5,
            },
        );
        assert_eq!(match_range_out_of_range, None);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub enum RuleMatchCategory {
    Error,
    Warning,
    BracketHighlightLineFirst,
    BracketHighlightLineLast,
    None,
}

impl RuleMatchCategory {
    pub fn from_lint_level(lint_level: LintLevel) -> RuleMatchCategory {
        match lint_level {
            LintLevel::Error => RuleMatchCategory::Error,
            LintLevel::Warning => RuleMatchCategory::Warning,
        }
    }
}
