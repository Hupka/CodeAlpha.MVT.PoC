use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Point, Tree};
use ts_rs::TS;

use crate::{
    ax_interaction::get_textarea_uielement,
    core_engine::{
        ax_utils::get_bounds_of_TextRange,
        rules::{get_index_of_next_row, TextPosition, TextRange},
        types::MatchRectangle,
    },
};

use super::utils::{
    get_code_block_parent, get_left_most_column_in_rows,
    get_match_range_of_first_and_last_char_in_node, length_to_code_block_body_start,
    only_whitespace_on_line_until_position, rectanges_of_wrapped_line, rectangles_from_match_range,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightElbow {
    origin_x: Option<f64>,
    origin_x_left_most: bool,
    bottom_line_top: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightBracket {
    text_range: TextRange,
    text_position: TextPosition,
    rectangle: MatchRectangle,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightBracketPair {
    first: Option<BracketHighlightBracket>,
    last: Option<BracketHighlightBracket>,
}

impl BracketHighlightBracketPair {
    pub fn new(
        first_range: TextRange,
        first_rectangle: Option<MatchRectangle>,
        first_text_position: TextPosition,
        last_range: TextRange,
        last_rectangle: Option<MatchRectangle>,
        last_text_position: TextPosition,
    ) -> Self {
        let mut first = None;
        if let Some(first_rectangle) = first_rectangle {
            first = Some(BracketHighlightBracket {
                text_range: first_range,
                text_position: first_text_position,
                rectangle: first_rectangle,
            });
        }

        let mut last = None;
        if let Some(last_rectangle) = last_rectangle {
            last = Some(BracketHighlightBracket {
                text_range: last_range,
                text_position: last_text_position,
                rectangle: last_rectangle,
            });
        }

        Self { first, last }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightResults {
    lines: BracketHighlightBracketPair,
    elbow: Option<BracketHighlightElbow>,
    boxes: BracketHighlightBracketPair,
}

pub struct BracketHighlight {
    results: Option<BracketHighlightResults>,
    selected_text_range: Option<TextRange>,
    swift_syntax_tree: Option<Tree>,
    text_content: Option<String>,
    window_pid: i32,
}

impl BracketHighlight {
    pub fn new(window_pid: i32) -> Self {
        Self {
            results: None,
            selected_text_range: None,
            swift_syntax_tree: None,
            text_content: None,
            window_pid,
        }
    }

    pub fn update_content(
        &mut self,
        swift_syntax_tree: Option<Tree>,
        text_content: Option<String>,
    ) {
        self.swift_syntax_tree = swift_syntax_tree;
        self.text_content = text_content;
    }

    pub fn update_selected_text_range(&mut self, selected_text_range: Option<TextRange>) {
        self.selected_text_range = selected_text_range;
    }

    pub fn get_results(&self) -> Option<BracketHighlightResults> {
        self.results.clone()
    }

    pub fn generate_results(&mut self) {
        let (selected_node, selected_text_range, text_content, textarea_ui_element) = if let (
            Some(node),
            Some(selected_text_range),
            Some(text_content),
            Some(textarea_ui_element),
        ) = (
            self.get_selected_code_node(),
            self.selected_text_range.clone(),
            self.text_content.clone(),
            get_textarea_uielement(self.window_pid),
        ) {
            (node, selected_text_range, text_content, textarea_ui_element)
        } else {
            // Failed to get selected_node, selected_text_range, text_content, or ui_element
            self.results = None;
            return;
        };

        let mut code_block_node =
            if let Some(code_block_node) = get_code_block_parent(selected_node, false) {
                code_block_node
            } else {
                self.results = None;
                return;
            };

        let mut line_brackets_match_range = if let Some(line_brackets_match_range) =
            get_match_range_of_first_and_last_char_in_node(
                &code_block_node,
                &text_content,
                selected_text_range.index,
            ) {
            line_brackets_match_range
        } else {
            self.results = None;
            return;
        };

        let length_to_bad_code_block_start = length_to_code_block_body_start(
            &code_block_node,
            &text_content,
            selected_text_range.index,
        );
        // If selected block is in bad code block declaration, then get parent
        if length_to_bad_code_block_start.is_some() && length_to_bad_code_block_start.unwrap().1 {
            code_block_node =
                if let Some(code_block_node) = get_code_block_parent(code_block_node, true) {
                    code_block_node
                } else {
                    self.results = None;
                    return;
                };

            line_brackets_match_range = if let Some(line_brackets_match_range) =
                get_match_range_of_first_and_last_char_in_node(
                    &code_block_node,
                    &text_content,
                    selected_text_range.index,
                ) {
                line_brackets_match_range
            } else {
                self.results = None;
                return;
            };
        }

        let mut line_positions = (
            TextPosition::from_TSPoint(&code_block_node.start_position()),
            TextPosition::from_TSPoint(&code_block_node.end_position()),
        );
        let box_brackets_match_range = line_brackets_match_range.clone();
        let box_positions = line_positions.clone();

        let is_touching_left_first_char =
            selected_text_range.index == line_brackets_match_range.0.range.index;

        if is_touching_left_first_char {
            if let Some(parent_node) = code_block_node.clone().parent() {
                if let Some(code_block_parent_node) = get_code_block_parent(parent_node, true) {
                    if let Some(parent_line_brackets) =
                        get_match_range_of_first_and_last_char_in_node(
                            &code_block_parent_node,
                            &text_content,
                            selected_text_range.index,
                        )
                    {
                        line_brackets_match_range = parent_line_brackets;
                        line_positions = (
                            TextPosition::from_TSPoint(&code_block_parent_node.start_position()),
                            TextPosition::from_TSPoint(&code_block_parent_node.end_position()),
                        );
                    }
                }
            }
        }

        // Get rectangles from the match ranges
        let (first_line_rectangle, last_line_rectangle, first_box_rectangle, last_box_rectangle) = (
            rectangles_from_match_range(&line_brackets_match_range.0, &textarea_ui_element),
            rectangles_from_match_range(&line_brackets_match_range.1, &textarea_ui_element),
            rectangles_from_match_range(&box_brackets_match_range.0, &textarea_ui_element),
            rectangles_from_match_range(&box_brackets_match_range.1, &textarea_ui_element),
        );

        let line_pair = BracketHighlightBracketPair::new(
            line_brackets_match_range.0.range,
            first_line_rectangle,
            line_positions.0,
            line_brackets_match_range.1.range,
            last_line_rectangle,
            line_positions.1,
        );

        let box_pair = BracketHighlightBracketPair::new(
            box_brackets_match_range.0.range,
            first_box_rectangle,
            box_positions.0,
            box_brackets_match_range.1.range,
            last_box_rectangle,
            box_positions.1,
        );

        // Check if elbow is needed
        let mut elbow_origin_x = None;
        let mut elbow_origin_x_left_most = false;
        let mut elbow = None;

        // Elbow needed because the open and closing bracket are on different lines
        let is_line_on_same_row = line_positions.0.row == line_positions.1.row;
        if !is_line_on_same_row {
            let first_line_bracket_range = line_brackets_match_range.0.range.clone();
            if let Some(next_row_index) =
                get_index_of_next_row(first_line_bracket_range.index, &text_content)
            {
                if let Some(left_most_column) = get_left_most_column_in_rows(
                    TextRange {
                        index: next_row_index,
                        length: line_brackets_match_range.1.range.index - next_row_index + 1,
                    },
                    &text_content,
                ) {
                    if let (Some(elbow_match_rectangle), Some(line_pair_last)) = (
                        get_bounds_of_TextRange(
                            &TextRange {
                                index: left_most_column.index,
                                length: 1,
                            },
                            &textarea_ui_element,
                        ),
                        line_pair.last,
                    ) {
                        if line_pair_last.rectangle.origin.x > elbow_match_rectangle.origin.x {
                            // Closing bracket is further to the right than the elbow point
                            elbow_origin_x = Some(elbow_match_rectangle.origin.x);
                        }
                        if let Some(first_line_rectangle) = first_line_rectangle {
                            if first_line_rectangle.origin.x < elbow_match_rectangle.origin.x {
                                // Opening bracket is further to the left than the elbow point
                                elbow_origin_x = Some(first_line_rectangle.origin.x);
                            }
                        }
                    }
                }
            }
        }

        let first_line_wrapped_rectangles =
            rectanges_of_wrapped_line(line_positions.0.row, &text_content, textarea_ui_element);
        if first_line_wrapped_rectangles.len() > 1 {
            if let (
                Some(last_wrapped_line_rectangle),
                Some(first_line_rectangle),
                Some(last_line_rectangle),
            ) = (
                first_line_wrapped_rectangles.last(),
                first_line_rectangle,
                last_line_rectangle,
            ) {
                if last_wrapped_line_rectangle.origin.y != first_line_rectangle.origin.y
                    && last_line_rectangle.origin.y != first_line_rectangle.origin.y
                {
                    // Elbow most to the right because open bracket is not at the end of a wrapped line
                    elbow_origin_x_left_most = true;
                }
            }
        }

        // Check if bottom line should be to the top or bottom of last line rectangle
        let elbow_bottom_line_top = only_whitespace_on_line_until_position(
            TextPosition {
                row: line_positions.1.row,
                column: if line_positions.1.column == 0 {
                    0
                } else {
                    line_positions.1.column - 1
                },
            },
            &text_content,
        );

        if elbow_origin_x_left_most {
            elbow = Some(BracketHighlightElbow {
                origin_x: None,
                bottom_line_top: elbow_bottom_line_top,
                origin_x_left_most: true,
            });
        } else if let Some(elbow_origin_x) = elbow_origin_x {
            elbow = Some(BracketHighlightElbow {
                origin_x: Some(elbow_origin_x),
                bottom_line_top: elbow_bottom_line_top,
                origin_x_left_most: false,
            });
        }

        self.results = Some(BracketHighlightResults {
            lines: line_pair,
            elbow,
            boxes: box_pair,
        });
    }

    fn get_selected_code_node(&self) -> Option<Node> {
        if let (Some(selected_text_range), Some(syntax_tree), Some(text_content)) = (
            self.selected_text_range.clone(),
            &self.swift_syntax_tree,
            &self.text_content,
        ) {
            if let Some((start_position, _)) =
                selected_text_range.as_StartEndTextPosition(text_content)
            {
                let node = syntax_tree.root_node().named_descendant_for_point_range(
                    Point {
                        row: start_position.row,
                        column: start_position.column,
                    },
                    Point {
                        row: start_position.row,
                        column: start_position.column,
                    },
                );

                return node;
            }
        }
        None
    }
}