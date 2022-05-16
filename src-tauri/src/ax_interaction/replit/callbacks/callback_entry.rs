#![allow(non_upper_case_globals)]

use std::{ffi::c_void, mem};

use accessibility::{AXAttribute, AXObserver, AXUIElement};
use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXValueChangedNotification, kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification, kAXWindowMovedNotification, kAXWindowResizedNotification,
    AXObserverRef, AXUIElementRef,
};
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};

use crate::ax_interaction::{
    replit::callbacks::{notify_window_created, notify_window_destroyed},
    ReplitObserverState,
};

use super::{
    notifiy_app_activated, notifiy_app_deactivated, notify_uielement_focused, notify_window_moved,
    notify_window_resized,
};

// This file contains the callback function that is registered with the AXObserver
// that listens to notifications on the Replit AXUIElement.
//
// Adjacent files contain the control logic for the different notifications received
pub unsafe extern "C" fn callback_replit_notifications(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    context: *mut c_void,
) {
    let _observer: AXObserver = TCFType::wrap_under_get_rule(observer);
    let element: AXUIElement = TCFType::wrap_under_get_rule(element);
    let notification = CFString::wrap_under_get_rule(notification);
    let context: *mut ReplitObserverState = mem::transmute(context);

    match notification.to_string().as_str() {
        kAXFocusedUIElementChangedNotification => {
            let _ = notify_uielement_focused(&element, &mut (*context));
        }
        kAXValueChangedNotification => {
            // Check, weather the ui element changed is the scroll bar of text area
            if let Ok(role) = element.attribute(&AXAttribute::role()) {
                if role.to_string() == "AXTextArea" {
                    let _ = notify_window_resized(&element, &mut (*context));
                }
            }
        }
        kAXMainWindowChangedNotification => {
            let _ = notify_window_created(&element, &mut (*context));
            let _ = notify_window_destroyed(&element, &mut (*context));
        }
        kAXWindowCreatedNotification => {
            let _ = notify_window_created(&element, &mut (*context));
        }
        kAXApplicationActivatedNotification => {
            let _ = notify_window_created(&element, &mut (*context));
            let _ = notify_window_destroyed(&element, &mut (*context));
            let _ = notifiy_app_activated(&element, &mut (*context));
        }
        kAXApplicationDeactivatedNotification => {
            let _ = notifiy_app_deactivated(&element, &mut (*context));
            let _ = notify_window_destroyed(&element, &mut (*context));
        }
        kAXApplicationHiddenNotification => {
            let _ = notifiy_app_deactivated(&element, &mut (*context));
        }
        kAXApplicationShownNotification => {
            let _ = notifiy_app_activated(&element, &mut (*context));
        }
        kAXWindowMovedNotification => {
            let _ = notify_window_moved(&element, &mut (*context));
        }
        kAXWindowResizedNotification => {
            let _ = notify_window_resized(&element, &mut (*context));
        }
        kAXWindowMiniaturizedNotification => {
            // Here we do nothing, because this behavior would be duplicated with kAXFocusedUIElementChangedNotification
        }
        kAXWindowDeminiaturizedNotification => {
            // Here we do nothing, because this behavior would be duplicated with kAXFocusedUIElementChangedNotification
        }
        _other => {
            println!("Forgotten notification: {:?}", _other)
        }
    }
}