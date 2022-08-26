// Scrolling has high performance requirements, so we fast-track the usual event architecture here.
// This only handles mousewheel scrolling events, since they come much faster than the native AX scroll event.
// We additionally use the native AX scroll event to handle other scrolling cases like scrollbar click-and-drag.

use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use parking_lot::Mutex;

use crate::{
    app_handle,
    window_controls::{
        models::editor_window::CodeOverlayDimensionsUpdateMessage, EventWindowControls,
    },
};

use super::{
    generate_axui_element_hash, get_textarea_uielement,
    internal::{get_focused_uielement, get_uielement_frame},
    GetVia,
};
lazy_static! {
    static ref CORRECTION_EVENT_PUBLISHING_TIME: Mutex<Option<Instant>> = Mutex::new(None);
}

pub fn fast_track_handle_text_editor_mousewheel_scroll(text_editor_hash: usize) -> Option<()> {
    execute_publishing_event(text_editor_hash);

    let mut publishing_time_mutex = CORRECTION_EVENT_PUBLISHING_TIME.lock();
    if publishing_time_mutex.is_none() {
        // Case: This is the first scrolling event. It will be responsible for the final execution, after the last scrolling event has happened.
        // So we wait until the last scrolling event was more than 50 millis ago.

        // Refresh the time to publish final, correction event, since we are still observing scrolling.
        publishing_time_mutex.replace(Instant::now() + std::time::Duration::from_millis(50));

        tauri::async_runtime::spawn(async move {
            loop {
                let mut publishing_moment_reached = false;
                let mut sleep_duration: Duration = Duration::from_millis(3);
                if let Some(mut locked_correction_event_timestamp) =
                    CORRECTION_EVENT_PUBLISHING_TIME.try_lock()
                {
                    if let Some(hide_until) = locked_correction_event_timestamp.as_ref() {
                        // Is zero when hide_until is older than Instant::now()
                        let duration = hide_until.duration_since(Instant::now());

                        if duration.is_zero() {
                            // Scrolling has finished. Publish correction event.
                            *locked_correction_event_timestamp = None;
                            publishing_moment_reached = true;
                        } else {
                            sleep_duration = duration;
                        }
                    } else {
                        println!("scroll_correction_event_publishing_time_locked is None -- this should not happen");
                        break;
                    }
                }

                if publishing_moment_reached {
                    // Sometimes, XCode handles the scroll event quickly, but sometimes it takes longer.
                    // Send multiple correction events at different delays for optimal handling.
                    execute_publishing_event(text_editor_hash);
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    execute_publishing_event(text_editor_hash);
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    execute_publishing_event(text_editor_hash);

                    break;
                } else {
                    tokio::time::sleep(sleep_duration).await;
                }
            }
        });
    } else {
        // Refresh the time to publish final, correction event, since we are stil observing scrolling.
        publishing_time_mutex.replace(Instant::now() + std::time::Duration::from_millis(50));
    }

    Some(())
}

fn execute_publishing_event(text_editor_hash: usize) -> Option<()> {
    // Check if text editor is still focused
    let current_hash = generate_axui_element_hash(&get_focused_uielement(&GetVia::Current).ok()?);

    if text_editor_hash == current_hash {
        // This is the window that is currently focused.
        // We do not need to publish the correction event.
        let code_document_uielement = get_textarea_uielement(&GetVia::Current).ok()?;
        let code_document_frame = get_uielement_frame(&code_document_uielement).ok()?;

        // Fast-track method instead of calling editor_windows.update_code_document_frame()
        EventWindowControls::CodeOverlayDimensionsUpdate(CodeOverlayDimensionsUpdateMessage {
            code_viewport_rect: None,
            code_document_rect: code_document_frame,
        })
        .publish_to_tauri(&app_handle());
        return Some(());
    }

    Some(())
}
