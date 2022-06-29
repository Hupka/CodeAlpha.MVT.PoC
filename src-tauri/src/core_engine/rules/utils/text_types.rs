#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A position in a multi-line text document, in terms of rows and columns.
/// Rows and columns are zero-based.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct TextPosition {
    pub row: usize,
    pub column: usize,
}

impl TextPosition {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }

    pub fn from_TSPoint(tree_sitter_point: &tree_sitter::Point) -> Self {
        Self {
            row: tree_sitter_point.row,
            column: tree_sitter_point.column,
        }
    }

    /// > Given a string and an index, return the row number and column number of the character at that
    /// index. Different from TextPosition, index does include the newline character.
    /// In case the index references a new line character in the text, we return the position of the
    /// next valid character.
    ///
    /// Arguments:
    ///
    /// * `text`: The text to search through.
    /// * `index`: The index of the character in the text.
    ///
    /// Returns:
    ///
    /// A TextPosition struct
    pub fn from_TextIndex(text: &String, index: usize) -> Option<TextPosition> {
        let mut position: Option<TextPosition> = None;

        let mut char_count = 0;
        let mut line_number = 0;
        let mut lines = text.lines();
        'outer: while let Some(line) = lines.next() {
            let mut char_indices = line.char_indices();
            '_inner: while let Some((col, _)) = char_indices.next() {
                if index == char_count {
                    position = Some(TextPosition {
                        row: line_number,
                        column: col,
                    });

                    break 'outer;
                }
                char_count += 1;
            }
            char_count += 1;
            line_number += 1;
        }

        position
    }

    pub fn as_TSPoint(&self) -> tree_sitter::Point {
        tree_sitter::Point {
            row: self.row,
            column: self.column,
        }
    }

    pub fn as_TextIndex(&self, text: &String) -> Option<usize> {
        let mut index: Option<usize> = None;

        let mut char_count = 0;
        let mut line_number = 0;
        let mut lines = text.lines();
        'outer: while let Some(line) = lines.next() {
            let mut char_indices = line.char_indices();
            '_inner: while let Some((col, _)) = char_indices.next() {
                if self.column == col && self.row == line_number {
                    index = Some(char_count);

                    break 'outer;
                }
                char_count += 1;
            }
            line_number += 1;
        }

        index
    }
}

#[cfg(test)]
mod tests_TextPosition {
    use crate::core_engine::rules::utils::text_types::TextPosition;

    #[test]
    fn test_TextPosition_from_TextIndex_respects_new_line_character() {
        let text = "\nHello, World!";
        let index = 0;
        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 0);
        assert_eq!(position.column, 0);
    }

    #[test]
    fn test_TextPosition_from_TextIndex_one_line() {
        let text = "Hello, World!";
        let index = 5;
        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 0);
        assert_eq!(position.column, 5);
    }

    #[test]
    fn test_TextPosition_from_TextIndex_two_lines() {
        let text = "Hello, World!\nGoodbye, World!";
        let index = 20;
        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 1);
        assert_eq!(position.column, 7);
    }

    #[test]
    fn test_TextPosition_from_TextIndex_too_far() {
        let text = "Hello, World!";
        let index = 100;
        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_none(), true);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex() {
        let text = "Hello, World!";
        let position = TextPosition::new(0, 5);
        let index_option = position.as_TextIndex(&text.to_string());

        assert_eq!(index_option.is_some(), true);

        let index = index_option.unwrap();

        assert_eq!(index, 5);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_multi_line() {
        let text = "Hello,\n World\n!";
        let position = TextPosition::new(2, 0);
        let index_option = position.as_TextIndex(&text.to_string());

        assert_eq!(index_option.is_some(), true);

        let index = index_option.unwrap();

        assert_eq!(index, 12);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far() {
        let text = "Hello, World!";
        let position = TextPosition::new(0, 100);
        let index_option = position.as_TextIndex(&text.to_string());

        assert_eq!(index_option.is_none(), true);
    }
}

/// A position in a multi-line text document, in terms of index and length.
/// index is zero-based.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct TextRange {
    pub index: usize,
    pub length: usize,
}

impl TextRange {
    pub fn new(index: usize, length: usize) -> Self {
        Self { index, length }
    }

    pub fn from_StartEndIndex(start_index: usize, end_index: usize) -> TextRange {
        TextRange {
            index: start_index,
            length: end_index - start_index,
        }
    }

    pub fn from_StartEndTextPosition(
        text: &String,
        start_position: &TextPosition,
        end_position: &TextPosition,
    ) -> Option<TextRange> {
        let mut index: Option<usize> = None;
        let mut length: Option<usize> = None;

        let mut char_count = 0;
        let mut line_number = 0;
        while let Some(line) = text.lines().next() {
            if line_number == start_position.row {
                while let Some((col, _)) = line.char_indices().next() {
                    if start_position.column == col {
                        index = Some(char_count);
                    }
                    char_count += 1;
                }
                line_number += 1;
            }

            if line_number == end_position.row {
                while let Some((col, _)) = line.char_indices().next() {
                    if end_position.column == col {
                        length = Some(char_count - index.unwrap());
                    }
                    char_count += 1;
                }
                line_number += 1;
            }
        }

        if let (Some(index), Some(length)) = (index, length) {
            Some(TextRange { index, length })
        } else {
            None
        }
    }

    pub fn from_StartEndTSPoint(
        text: &String,
        start_position: &tree_sitter::Point,
        end_position: &tree_sitter::Point,
    ) -> Option<TextRange> {
        Self::from_StartEndTextPosition(
            text,
            &TextPosition {
                row: start_position.row,
                column: start_position.column,
            },
            &TextPosition {
                row: end_position.row,
                column: end_position.column,
            },
        )
    }

    pub fn as_StartEndIndex(&self) -> (usize, usize) {
        (self.index, self.index + self.length)
    }

    pub fn as_StartEndTSPoint(
        &self,
        text: &String,
    ) -> Option<(tree_sitter::Point, tree_sitter::Point)> {
        if let Some((start_position, end_position)) =
            StartEndTextPosition_from_TextRange(text, self)
        {
            Some((start_position.as_TSPoint(), end_position.as_TSPoint()))
        } else {
            None
        }
    }

    pub fn as_StartEndTextPosition(&self, text: &String) -> Option<(TextPosition, TextPosition)> {
        if let Some((start_position, end_position)) =
            StartEndTextPosition_from_TextRange(text, self)
        {
            Some((start_position, end_position))
        } else {
            None
        }
    }
}

pub fn StartEndIndex_from_TextRange(char_range: &TextRange) -> (usize, usize) {
    (char_range.index, char_range.index + char_range.length)
}

pub fn StartEndIndex_from_StartEndTextPosition(
    text: &String,
    start_position: &TextPosition,
    end_position: &TextPosition,
) -> Option<(usize, usize)> {
    if let Some(char_range) =
        TextRange::from_StartEndTextPosition(text, start_position, end_position)
    {
        Some(StartEndIndex_from_TextRange(&char_range))
    } else {
        None
    }
}

pub fn StartEndIndex_from_StartEndTSPoint(
    text: &String,
    start_point: &tree_sitter::Point,
    end_point: &tree_sitter::Point,
) -> Option<(usize, usize)> {
    StartEndIndex_from_StartEndTextPosition(
        text,
        &TextPosition {
            row: start_point.row,
            column: start_point.column,
        },
        &TextPosition {
            row: end_point.row,
            column: end_point.column,
        },
    )
}

pub fn StartEndTextPosition_from_TextRange(
    text: &String,
    char_range: &TextRange,
) -> Option<(TextPosition, TextPosition)> {
    let (start_index, end_index) = StartEndIndex_from_TextRange(&char_range);

    if let Some((start_position, end_position)) =
        StartEndTextPosition_from_StartEndIndex(text, start_index, end_index)
    {
        Some((start_position, end_position))
    } else {
        None
    }
}

pub fn StartEndTextPosition_from_StartEndIndex(
    text: &String,
    start_index: usize,
    end_index: usize,
) -> Option<(TextPosition, TextPosition)> {
    let mut start_position: Option<TextPosition> = None;
    let mut end_position: Option<TextPosition> = None;

    let mut char_count = 0;
    let mut line_number = 0;
    while let Some(line) = text.lines().next() {
        while let Some((col, _)) = line.char_indices().next() {
            if start_index == char_count {
                start_position = Some(TextPosition {
                    row: line_number,
                    column: col,
                });
            }

            if end_index == char_count {
                end_position = Some(TextPosition {
                    row: line_number,
                    column: col,
                });
            }
            char_count += 1;
        }
        line_number += 1;
    }

    if let (Some(start_position), Some(end_position)) = (start_position, end_position) {
        Some((start_position, end_position))
    } else {
        None
    }
}

pub fn StartEndTextPosition_from_StartEndTSPoint(
    start_point: &tree_sitter::Point,
    end_point: &tree_sitter::Point,
) -> (TextPosition, TextPosition) {
    (
        TextPosition::from_TSPoint(start_point),
        TextPosition::from_TSPoint(end_point),
    )
}
