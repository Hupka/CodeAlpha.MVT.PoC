use std::{
    fs::create_dir_all,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
use tree_sitter::{Node, Parser, Tree};

use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};

use super::swift_codeblock::SwiftCodeBlock;

#[derive(thiserror::Error, Debug)]
pub enum SwiftSyntaxTreeError {
    #[error("No treesitter node could be retreived with the given text range.")]
    NoTreesitterNodeFound,
    #[error("No valid codeblock could be derived from the provided text range.")]
    NoValidCodeblockFound,
    #[error("At this point, no valid tree is available.")]
    NoTreeParsed,
}

pub struct SwiftSyntaxTree {
    tree_sitter_parser: Parser,
    tree_sitter_tree: Option<Tree>,
    content: Option<XcodeText>,
    logging_folder: Option<PathBuf>,
}

impl SwiftSyntaxTree {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_swift::language())
            .expect("Swift Language not found");

        Self {
            tree_sitter_parser: parser,
            tree_sitter_tree: None,
            content: None,
            logging_folder: None,
        }
    }

    pub fn _reset(&mut self) {
        self.tree_sitter_tree = None;
        self.content = None;
    }

    pub fn parse(&mut self, content: &XcodeText) -> bool {
        let updated_tree = self.tree_sitter_parser.parse_utf16(content, None);

        if updated_tree.is_some() {
            self.tree_sitter_tree = updated_tree;
            self.content = Some(content.to_owned());
            return true;
        } else {
            return false;
        }
    }

    pub fn get_selected_code_node(
        &self,
        selected_text_range: &TextRange,
    ) -> Result<Node, SwiftSyntaxTreeError> {
        if let (Some(syntax_tree), Some(text_content)) =
            (self.tree_sitter_tree.as_ref(), self.content.as_ref())
        {
            if let Some((start_position, _)) =
                selected_text_range.as_StartEndTextPosition(text_content)
            {
                if let Some(node) = syntax_tree.root_node().named_descendant_for_point_range(
                    TextPosition {
                        row: start_position.row,
                        column: start_position.column,
                    }
                    .as_TSPoint(),
                    TextPosition {
                        row: start_position.row,
                        column: start_position.column,
                    }
                    .as_TSPoint(),
                ) {
                    return Ok(node);
                } else {
                    return Err(SwiftSyntaxTreeError::NoTreesitterNodeFound);
                }
            }
        }

        Err(SwiftSyntaxTreeError::NoTreeParsed)
    }

    pub fn get_selected_codeblock_node(
        &self,
        selected_text_range: &TextRange,
    ) -> Result<SwiftCodeBlock, SwiftSyntaxTreeError> {
        let mut node = self.get_selected_code_node(selected_text_range)?;
        let content = self
            .content
            .as_ref()
            .ok_or(SwiftSyntaxTreeError::NoTreeParsed)?;

        loop {
            if let Ok(codeblock_node) = SwiftCodeBlock::new(node, content) {
                return Ok(codeblock_node);
            } else {
                if let Some(parent) = node.parent() {
                    node = parent;
                } else {
                    return Err(SwiftSyntaxTreeError::NoValidCodeblockFound);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn tree(&self) -> Option<&Tree> {
        self.tree_sitter_tree.as_ref()
    }

    #[allow(dead_code)]
    pub fn start_logging(&mut self, path: &PathBuf) -> std::io::Result<()> {
        let html_header: &[u8] = b"<!DOCTYPE html>\n<style>svg { width: 100%; }</style>\n\n";

        create_dir_all(&path)?;

        let mut html_file_path = path.to_str().unwrap().to_string();
        html_file_path.push_str("tree.html");

        let mut html_file = std::fs::File::create(html_file_path)?;

        html_file.write(html_header)?;
        let mut dot_process = Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(html_file)
            .spawn()
            .expect("Failed to run Dot");
        let dot_stdin = dot_process
            .stdin
            .take()
            .expect("Failed to open stdin for Dot");

        // let mut dot_file_path = path.to_str().unwrap().to_string();
        // dot_file_path.push_str("tree.dot");

        // let mut dot_file = std::fs::File::create(dot_file_path)?;
        // dot_file.write(html_header)?;
        // let mut dot_process = Command::new("dot")
        //     .arg("-Tsvg")
        //     .stdin(Stdio::piped())
        //     .stdout(dot_file)
        //     .spawn()
        //     .expect("Failed to run Dot");
        // let dot_stdin = dot_process
        //     .stdin
        //     .take()
        //     .expect("Failed to open stdin for Dot");
        self.tree_sitter_parser.print_dot_graphs(&dot_stdin);

        Ok(())
    }
}

impl Drop for SwiftSyntaxTree {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if let Some(logging_folder) = &self.logging_folder {
            println!("{:?}", logging_folder);
        }
        self.tree_sitter_parser.stop_printing_dot_graphs();
    }
}

#[cfg(test)]
mod tests_SwiftSyntaxTree {

    use std::path::PathBuf;

    use crate::core_engine::utils::{TextPosition, XcodeText};

    use super::SwiftSyntaxTree;
    use pretty_assertions::assert_eq;
    use rand::Rng;

    #[test]
    #[ignore]
    fn test_tree_reset() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        let original_string = "let";
        let replace_string = "var";
        let code_original = XcodeText::from_str(&format!(
            "{} apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"",
            original_string
        ));
        let code_updated = XcodeText::from_str(&format!(
            "{} apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"",
            replace_string
        ));

        swift_syntax_tree.parse(&code_original);
        {
            let root = swift_syntax_tree.tree().unwrap().root_node();
            let mut cursor = root.walk();
            println!(
                "Root before - ID {:?}",
                swift_syntax_tree.tree().unwrap().root_node().id()
            );
            for child in root.children(&mut cursor) {
                println!(
                    "Child before - ID {:?}, Kind {:?}, Text {:?}",
                    child.id(),
                    child.kind(),
                    child.utf16_text(&code_original)
                );
            }
        }

        swift_syntax_tree.parse(&code_updated);
        let root = swift_syntax_tree.tree().unwrap().root_node();
        let mut cursor = root.walk();
        println!(
            "Root after - ID {:?}",
            swift_syntax_tree.tree().unwrap().root_node().id()
        );
        for child in root.children(&mut cursor) {
            println!(
                "Child after - ID {:?}, Kind {:?}, Text {:?}",
                child.id(),
                child.kind(),
                child.utf16_text(&code_original)
            );
        }
    }

    #[test]
    #[ignore]
    fn test_tree_sitter_logging() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        _ = swift_syntax_tree.start_logging(&prepare_treesitter_logging());

        let code =
            XcodeText::from_str("var apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"");
        swift_syntax_tree.parse(&code);
        let updated_code = XcodeText::from_str("let apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"\nlet appleSummary2 = \"I have \\(apples) apples.\"");
        swift_syntax_tree.parse(&updated_code);
    }

    fn prepare_treesitter_logging() -> PathBuf {
        let mut rng = rand::thread_rng();
        let n1: u32 = rng.gen::<u32>();

        let content = format!("{}/", n1);
        let mut path_buf = std::env::temp_dir();
        path_buf.push(content);

        println!("{}", path_buf.to_str().unwrap());

        path_buf
    }

    #[test]
    fn test_start_end_point_end_newline_char() {
        let text = XcodeText::from_str("let x = 1; console.log(x);\n");
        //                |------------------------>| <- end column is zero on row 1
        //                                            <- end byte is one past the last byte (27), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new();
        swift_syntax_tree.parse(&text);

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), text.utf16_bytes_count());
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 1, column: 0 }
        );
    }

    #[test]
    fn test_start_end_point_end_no_newline_char() {
        let text = XcodeText::from_str("let x = 1; console.log(x);");
        //                |------------------------>| <- end column is one past the last char (26)
        //                |------------------------>| <- end byte is one past the last byte (26), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new();
        swift_syntax_tree.parse(&text);

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), text.utf16_bytes_count());
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 0, column: 26 }
        );
    }

    #[test]
    fn test_start_end_point_with_UTF16_chars() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        let mut text = XcodeText::from_str("// 😊\n");
        let mut utf8_str = XcodeText::from_str("let x = 1; console.log(x);");
        text.append(&mut utf8_str);

        swift_syntax_tree.parse(&text);

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), text.utf16_bytes_count());
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 1, column: 26 }
        );
    }
}
