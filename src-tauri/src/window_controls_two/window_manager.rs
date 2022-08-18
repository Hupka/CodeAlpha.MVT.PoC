use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{
    app_handle, utils::geometry::LogicalFrame, window_controls::code_overlay::TrackingAreasManager,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::{
    config::AppWindow,
    events::{
        models::app_window::{HideAppWindowMessage, ShowAppWindowMessage},
        EventWindowControls,
    },
    listeners::{app_listener, user_interaction_listener, xcode_listener},
    windows::{CodeOverlayWindow, EditorWindow, WidgetWindow},
};

pub type Uuid = usize;

#[derive(Clone, Debug)]
pub struct WindowManager {
    app_handle: tauri::AppHandle,

    /// HashMap of open editor windows.
    editor_windows: Arc<Mutex<HashMap<Uuid, EditorWindow>>>,

    // WidgetWindow
    _widget_window: Arc<Mutex<WidgetWindow>>,

    // CodeOverlayWindow
    _code_overlay_window: Arc<Mutex<CodeOverlayWindow>>,

    // TrackingAreasManager
    _tracking_areas_manager: Arc<Mutex<TrackingAreasManager>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    focused_editor_window: Arc<Mutex<Option<Uuid>>>,

    /// Boolean saying if the currently focused application is an editor window.
    is_editor_focused: bool,

    /// Identitfier of the currently focused app window. Is None until the first window was focused.
    focused_app_window: Option<AppWindow>,

    /// Boolean saying if the currently focused application is our app.
    is_app_focused: bool,

    /// Boolean stating if the the core engine is active.
    is_core_engine_active: bool,
}

impl WindowManager {
    pub fn new() -> Result<Self, tauri::Error> {
        // Instantiate app windows. If this fails, the app will not work.
        let widget_window = Arc::new(Mutex::new(WidgetWindow::new()?));
        WidgetWindow::start_event_listeners(&widget_window);

        let code_overlay_window = Arc::new(Mutex::new(CodeOverlayWindow::new()?));
        CodeOverlayWindow::start_event_listeners(&code_overlay_window);

        let tracking_areas_manager = Arc::new(Mutex::new(TrackingAreasManager::new()));
        TrackingAreasManager::start_event_listeners(&tracking_areas_manager);

        Ok(Self {
            app_handle: app_handle(),
            editor_windows: Arc::new(Mutex::new(HashMap::new())),
            focused_editor_window: Arc::new(Mutex::new(None)),
            is_app_focused: false,
            is_editor_focused: false,
            focused_app_window: None,
            is_core_engine_active: CORE_ENGINE_ACTIVE_AT_STARTUP,
            _widget_window: widget_window,
            _code_overlay_window: code_overlay_window,
            _tracking_areas_manager: tracking_areas_manager,
        })
    }

    pub fn editor_windows(&self) -> &Arc<Mutex<HashMap<Uuid, EditorWindow>>> {
        &self.editor_windows
    }

    pub fn clear_editor_windows(&mut self, editor_name: &String) {
        let mut editor_windows = self.editor_windows.lock();
        editor_windows.retain(|_, editor_window| {
            if editor_window.editor_name() != editor_name {
                true
            } else {
                self.focused_editor_window.lock().take();
                self.is_editor_focused = false;
                false
            }
        });
    }

    pub fn focused_editor_window(&self) -> Option<Uuid> {
        self.focused_editor_window.lock().clone()
    }

    pub fn set_is_editor_focused(&mut self, is_editor_focused: bool) {
        self.is_editor_focused = is_editor_focused;
    }

    pub fn set_is_app_focused(&mut self, is_app_focused: bool) {
        self.is_app_focused = is_app_focused;
    }

    pub fn is_core_engine_active(&self) -> bool {
        self.is_core_engine_active
    }

    pub fn set_is_core_engine_active(&mut self, is_core_engine_active: bool) {
        self.is_core_engine_active = is_core_engine_active;
    }

    pub fn set_focused_editor_window(&mut self, editor_window_hash: Uuid) {
        self.focused_editor_window
            .lock()
            .replace(editor_window_hash);
    }

    pub fn set_focused_app_window(&mut self, app_window: AppWindow) {
        self.focused_app_window = Some(app_window);
    }

    pub fn hide_app_windows(&self, app_windows: Vec<AppWindow>) {
        EventWindowControls::AppWindowHide(HideAppWindowMessage { app_windows })
            .publish_to_tauri(&app_handle());
    }

    pub fn show_app_windows(
        &self,
        app_windows: Vec<AppWindow>,
        editor_id: Option<Uuid>,
    ) -> Option<()> {
        // If no editor id is given, the app windows are being shown in relation to the currently
        // focused editor window.
        let editor_id = if let Some(id) = editor_id {
            id
        } else {
            self.focused_editor_window()?
        };

        let editor_windows = self.editor_windows.lock();
        let editor_window = editor_windows.get(&editor_id)?;

        let textarea_position = editor_window.textarea_position(true)?;
        let textarea_size = editor_window.textarea_size()?;

        // double check that no windows are included that should not be shown
        // when the CoreEngine is not running
        let mut corrected_app_window_list = app_windows.clone();
        if !self.is_core_engine_active() {
            corrected_app_window_list.retain(|app_window| {
                if AppWindow::hidden_on_core_engine_inactive().contains(&app_window) {
                    false
                } else {
                    true
                }
            });
        }

        EventWindowControls::AppWindowShow(ShowAppWindowMessage {
            app_windows: corrected_app_window_list,
            editor_textarea: LogicalFrame {
                origin: textarea_position,
                size: textarea_size,
            },
            widget_position: editor_window.widget_position(true),
            monitor: editor_window.get_monitor(&self.app_handle)?,
        })
        .publish_to_tauri(&app_handle());

        Some(())
    }

    pub fn temporarily_hide_app_windows(&self, app_windows: Vec<AppWindow>) {
        self.hide_app_windows(app_windows);

        let editor_windows_move_copy = self.editor_windows.clone();
        let focused_editor_window_move_copy = self.focused_editor_window.clone();
        tauri::async_runtime::spawn(async move {
            Self::async_show_app_windows(editor_windows_move_copy, focused_editor_window_move_copy)
        });
    }

    pub async fn async_show_app_windows(
        editor_windows_arc: Arc<Mutex<HashMap<Uuid, EditorWindow>>>,
        focused_editor_window_arc: Arc<Mutex<Option<Uuid>>>,
    ) -> Option<()> {
        let editor_windows = editor_windows_arc.lock();
        let focused_editor_window = focused_editor_window_arc.lock();

        let editor_uuid = focused_editor_window.as_ref()?;
        let editor_window = editor_windows.get(editor_uuid)?;

        let textarea_position = editor_window.textarea_position(true)?;
        let textarea_size = editor_window.textarea_size()?;

        EventWindowControls::AppWindowShow(ShowAppWindowMessage {
            app_windows: AppWindow::shown_on_focus_gained(),
            editor_textarea: LogicalFrame {
                origin: textarea_position,
                size: textarea_size,
            },
            widget_position: editor_window.widget_position(true),
            monitor: editor_window.get_monitor(&app_handle())?,
        })
        .publish_to_tauri(&app_handle());

        Some(())
    }

    pub fn start_event_listeners(window_manager: &Arc<Mutex<WindowManager>>) {
        app_listener(window_manager);
        user_interaction_listener(window_manager);
        xcode_listener(window_manager);
    }
}
