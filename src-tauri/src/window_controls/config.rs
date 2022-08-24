use serde::{Deserialize, Serialize};
use strum::EnumIter;

use ts_rs::TS;

#[derive(EnumIter, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, TS)]
#[ts(export)]
pub enum AppWindow {
    Overlay,
    Analytics,
    Widget,
    Content,
    Repair,
}

impl std::fmt::Display for AppWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AppWindow {
    pub fn hidden_on_core_engine_inactive() -> Vec<AppWindow> {
        vec![AppWindow::Overlay, AppWindow::Repair]
    }

    pub fn hidden_on_focus_lost() -> Vec<AppWindow> {
        vec![AppWindow::Widget, AppWindow::Overlay, AppWindow::Repair]
    }

    pub fn hidden_on_scroll_event() -> Vec<AppWindow> {
        vec![AppWindow::Overlay, AppWindow::Repair]
    }

    pub fn shown_on_focus_gained() -> Vec<AppWindow> {
        vec![AppWindow::Widget, AppWindow::Overlay]
    }

    pub fn shown_on_core_engine_activated() -> Vec<AppWindow> {
        vec![AppWindow::Widget, AppWindow::Overlay]
    }
}

pub mod default_properties {
    use super::*;

    pub fn url(window: &AppWindow) -> String {
        match window {
            AppWindow::Overlay => {
                format!("{}{}", r"\", AppWindow::Overlay.to_string().to_lowercase())
            }

            AppWindow::Analytics => format!(
                "{}{}",
                r"\",
                AppWindow::Analytics.to_string().to_lowercase()
            ),
            AppWindow::Widget => {
                format!("{}{}", r"\", AppWindow::Widget.to_string().to_lowercase())
            }

            AppWindow::Content => {
                format!("{}{}", r"\", AppWindow::Content.to_string().to_lowercase())
            }
            AppWindow::Repair => {
                format!("{}{}", r"\", AppWindow::Repair.to_string().to_lowercase())
            }
        }
    }

    pub fn title(window: &AppWindow) -> String {
        // ACHTUNG! Before changing the titles, check if these are used elsewhere ... at least in
        // src-tauri/src/ax_interaction/app/callbacks I am using hard coded title strings because I
        // I could not get it to work to match with the strings here.
        match window {
            AppWindow::Overlay => "CodeAlpha - Overlay".to_string(),
            AppWindow::Analytics => "CodeAlpha - Analytics".to_string(),
            AppWindow::Widget => "CodeAlpha - Widget".to_string(),
            AppWindow::Content => "CodeAlpha - Guide".to_string(),
            AppWindow::Repair => "CodeAlpha - Explanation".to_string(),
        }
    }

    pub fn size(window: &AppWindow) -> (f64, f64) {
        match window {
            AppWindow::Overlay => (800.0, 600.0),
            AppWindow::Analytics => (1280.0, 786.0),
            AppWindow::Widget => (48.0, 48.0),
            AppWindow::Content => (322.0, 398.0),
            AppWindow::Repair => (513.0, 500.0),
        }
    }

    pub fn is_resizable(window: &AppWindow) -> bool {
        match window {
            AppWindow::Overlay => false,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::Repair => false,
        }
    }
    pub fn is_transparent(window: &AppWindow) -> bool {
        match window {
            AppWindow::Overlay => true,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::Repair => true,
        }
    }

    pub fn has_decorations(window: &AppWindow) -> bool {
        match window {
            AppWindow::Overlay => false,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::Repair => false,
        }
    }

    pub fn is_visible(window: &AppWindow) -> bool {
        match window {
            AppWindow::Overlay => false,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::Repair => true,
        }
    }

    pub fn is_always_on_top(window: &AppWindow) -> bool {
        match window {
            AppWindow::Overlay => true,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::Repair => true,
        }
    }

    pub fn skip_taskbar(window: &AppWindow) -> bool {
        match window {
            AppWindow::Overlay => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::Repair => true,
        }
    }
}
