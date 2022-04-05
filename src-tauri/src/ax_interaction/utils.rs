use accessibility::{AXAttribute, AXUIElement, Error};
use accessibility_sys::pid_t;

static EDITOR_NAME: &str = "Xcode";

// Method to get the focused AXUIElement's top-level window
pub fn currently_focused_app() -> Result<AXUIElement, Error> {
    let system_wide_element = AXUIElement::system_wide();
    let focused_ui_element = system_wide_element.attribute(&AXAttribute::focused_uielement())?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    focused_window.attribute(&AXAttribute::parent())
}

pub fn is_uielement_editor(pid: pid_t) -> Result<bool, Error> {
    let application = AXUIElement::application(pid);
    let focused_ui_element = application.attribute(&AXAttribute::focused_uielement())?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    let parent = focused_window.attribute(&AXAttribute::parent())?;
    let title = parent.attribute(&AXAttribute::title())?;

    let role = focused_ui_element.attribute(&AXAttribute::role())?;

    if role == "AXTextArea" && title == EDITOR_NAME {
        Ok(true)
    } else {
        Ok(false)
    }
}

// A bit WIP - tiny struct to help move context info into callback functions of the observers
pub struct TauriState {
    pub handle: tauri::AppHandle,
}