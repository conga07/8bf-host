//! Lets a command-line process show Cocoa filter dialogs on macOS.
//!
//! Filters open Cocoa windows for their dialogs. That only works once the
//! process has an `NSApplication` that may come to the front, which a
//! command-line program does not set up by itself.

/// Creates the shared `NSApplication`; activates it only when requested. Safe to call twice.
///
/// Returns false on other platforms, or when AppKit is unavailable.
#[cfg(target_os = "macos")]
pub fn ensure_application(activate: bool) -> bool {
    use std::ffi::{c_char, c_void};

    const ACTIVATION_POLICY_REGULAR: isize = 0;
    const ACTIVATION_POLICY_ACCESSORY: isize = 1;

    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn objc_msgSend();
    }
    #[link(name = "AppKit", kind = "framework")]
    extern "C" {}

    type SendId = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
    type SendLong = unsafe extern "C" fn(*mut c_void, *mut c_void, isize) -> bool;
    type SendBool = unsafe extern "C" fn(*mut c_void, *mut c_void, bool);

    // SAFETY: objc_msgSend is called through function pointer types that
    // match each method's real signature, as the Objective-C ABI requires.
    unsafe {
        let class = objc_getClass(c"NSApplication".as_ptr());
        if class.is_null() {
            return false;
        }
        let send_id: SendId = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let send_long: SendLong = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let send_bool: SendBool = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let app = send_id(class, sel_registerName(c"sharedApplication".as_ptr()));
        if app.is_null() {
            return false;
        }
        send_long(
            app,
            sel_registerName(c"setActivationPolicy:".as_ptr()),
            if activate {
                ACTIVATION_POLICY_REGULAR
            } else {
                ACTIVATION_POLICY_ACCESSORY
            },
        );
        if activate {
            send_bool(
                app,
                sel_registerName(c"activateIgnoringOtherApps:".as_ptr()),
                true,
            );
        }
    }
    true
}

/// Creates the shared `NSApplication`; activates it only when requested. Safe to call twice.
///
/// Returns false on other platforms, or when AppKit is unavailable.
#[cfg(not(target_os = "macos"))]
pub fn ensure_application(_activate: bool) -> bool {
    false
}

/// Let ordinary Cocoa callbacks finish before a synchronous filter starts.
#[cfg(target_os = "macos")]
pub fn service_events(milliseconds: u64) {
    use std::ffi::c_void;
    use std::time::{Duration, Instant};
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFRunLoopDefaultMode: *const c_void;
        fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, return_after_source: u8) -> i32;
    }
    let deadline = Instant::now() + Duration::from_millis(milliseconds.min(10_000));
    while Instant::now() < deadline {
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, 1);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[cfg(not(target_os = "macos"))]
pub fn service_events(_milliseconds: u64) {}
