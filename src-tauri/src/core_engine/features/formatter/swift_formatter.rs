use std::path::Path;
use tauri::{
    api::process::{Command, CommandEvent},
    async_runtime::block_on,
};

use crate::{
    ax_interaction::{
        derive_xcode_textarea_dimensions, get_textarea_uielement, send_event_mouse_wheel,
        set_selected_text_range, update_xcode_editor_content,
    },
    core_engine::{
        events::EventRuleExecutionState, rules::get_bounds_of_first_char_in_range,
        utils::XcodeText, TextPosition, TextRange,
    },
};

pub fn format_swift(
    file_path: &String,
    old_text: &XcodeText,
    selected_text_range: &TextRange,
    pid: i32,
    app_handle: &tauri::AppHandle,
) -> Option<XcodeText> {
    let formatted_content = if let Some(formatted_content) = format_swift_file(file_path) {
        formatted_content
    } else {
        return None;
    };

    if formatted_content == *old_text {
        return None;
    }

    // Get position of selected text
    let mut scroll_delta = None;
    if let Some(editor_textarea_ui_element) = get_textarea_uielement(pid) {
        // Get the dimensions of the textarea viewport
        if let Ok(textarea_viewport) = derive_xcode_textarea_dimensions(&editor_textarea_ui_element)
        {
            if let Some(bounds_of_selected_text) =
                get_bounds_of_first_char_in_range(&selected_text_range, &editor_textarea_ui_element)
            {
                scroll_delta = Some(tauri::LogicalSize {
                    width: textarea_viewport.0.x - bounds_of_selected_text.origin.x,
                    height: bounds_of_selected_text.origin.y - textarea_viewport.0.y,
                });
            }
        }
    }

    // Update content
    let formatted_content_string =
        if let Ok(formatted_content_string) = String::from_utf16(&formatted_content) {
            formatted_content_string
        } else {
            return None;
        };
    if let Ok(_) = update_xcode_editor_content(pid, &formatted_content_string) {
    } else {
        return None;
    };

    // Restore cursor position
    // At this point we only place the curser a the exact same ROW | COL as before the formatting.
    if let Ok(_) = set_selected_text_range(
        pid,
        get_new_cursor_index(&old_text, &formatted_content, selected_text_range.index),
        selected_text_range.length,
    ) {}

    // Scroll to the same position as before the formatting
    let pid_move_copy = pid;
    if let Some(scroll_delta) = scroll_delta {
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            if let Ok(true) = send_event_mouse_wheel(pid_move_copy, scroll_delta) {}
        });
    }

    // Notifiy the frontend that the file has been formatted successfully
    EventRuleExecutionState::SwiftFormatFinished().publish_to_tauri(&app_handle);

    Some(formatted_content)
}

pub fn format_swift_file(file_path: &String) -> Option<XcodeText> {
    if !Path::new(file_path).exists() {
        println!("File does not exist: {}", file_path);
        return None;
    }
    let handle = format_file(file_path);
    let formatted_file = block_on(handle);

    if let Some(content) = formatted_file {
        Some(XcodeText::from_str(&content))
    } else {
        None
    }
}

async fn format_file(file_path: &String) -> Option<String> {
    let (mut rx, _) = Command::new_sidecar("swiftformat")
        .expect("failed to create `my-sidecar` binary command")
        .args([
            file_path.to_string(),
            "--output".to_string(),
            "stdout".to_string(),
            "--quiet".to_string(),
        ])
        .spawn()
        .expect("Failed to spawn sidecar");
    let mut text_content = "".to_string();
    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(line) = event {
            text_content.push_str(&(line + "\n"));
        }
    }

    if !text_content.is_empty() {
        return Some(text_content);
    }
    None
}

fn get_new_cursor_index(
    old_content: &XcodeText,
    formatted_content: &XcodeText,
    index: usize,
) -> usize {
    let mut new_index = formatted_content.len();
    if let Some(text_position) = TextPosition::from_TextIndex(old_content, index) {
        if let Some(text_index) = text_position.as_TextIndex_stay_on_line(formatted_content, true) {
            new_index = text_index;
        }
    }

    new_index
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command as StdCommand;

    use rand::Rng;

    use super::*;

    struct FileSystemSetup {
        pub test_file_not_existing_str: String,
        pub test_folder_dir: PathBuf,
    }

    impl FileSystemSetup {
        pub fn new() -> Self {
            let mut rng = rand::thread_rng();
            let random_number: u32 = rng.gen::<u32>();
            let test_folder_dir =
                std::env::temp_dir().join(format!("test_format_swift_file-{}", random_number));
            let test_file_path = test_folder_dir.join("test_file.txt");
            let test_file_not_existing_str = test_folder_dir
                .join("test_file_not_existing.txt")
                .to_str()
                .unwrap()
                .to_string();

            // create an empty folder temp folder
            let _ = StdCommand::new("mkdir")
                .arg(test_folder_dir.clone())
                .output()
                .expect("failed to execute process");

            assert!(test_folder_dir.exists());

            // create a file in the test_folder
            let _ = StdCommand::new("touch")
                .arg("-a")
                .arg(test_file_path.clone())
                .output()
                .expect("failed to execute process");

            Self {
                test_file_not_existing_str,
                test_folder_dir,
            }
        }
    }

    impl Drop for FileSystemSetup {
        fn drop(&mut self) {
            // remove the test folder
            let _ = StdCommand::new("rm")
                .arg("-rf")
                .arg(self.test_folder_dir.clone())
                .output()
                .expect("failed to execute process");

            assert!(!self.test_folder_dir.exists());
        }
    }

    #[test]
    fn test_format_swift_file() {
        let test_resources = FileSystemSetup::new();

        // Format non-existing file
        let formatted_content =
            format_swift_file(&test_resources.test_file_not_existing_str.clone());
        assert!(formatted_content.is_none());
    }
}
