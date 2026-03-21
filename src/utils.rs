use cidre::cg::{EventSrc, EventSrcStateId, Point, Rect, Size};
use std::ffi::CString;

type CFStringRef = *const std::ffi::c_void;
type CFBooleanRef = *const std::ffi::c_void;
type CGSConnectionID = u32;
type CFStringEncoding = u32;

const K_CF_STRING_ENCODING_MAC_ROMAN: CFStringEncoding = 0;
const K_CG_DIRECT_MAIN_DISPLAY: u32 = 0;

unsafe extern "C" {
    unsafe fn CFStringCreateWithCString(
        alloc: *const std::ffi::c_void,
        c_str: *const std::ffi::c_char,
        encoding: CFStringEncoding,
    ) -> CFStringRef;
    unsafe fn CFRelease(cf: *const std::ffi::c_void);

    unsafe static kCFBooleanTrue: CFBooleanRef;

    unsafe fn _CGSDefaultConnection() -> CGSConnectionID;
    unsafe fn CGSSetConnectionProperty(
        cid: CGSConnectionID,
        target_cid: CGSConnectionID,
        key: CFStringRef,
        value: *const std::ffi::c_void,
    );

    unsafe fn CGDisplayHideCursor(display: u32) -> i32;
}

pub fn min(a: f64, b: f64) -> f64 {
    if a > b { b } else { a }
}

pub fn max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

pub fn union_rect(a: &Rect, b: &Rect) -> Rect {
    if *a == Rect::null() {
        return *b;
    }
    if *b == Rect::null() {
        return *a;
    }
    let min_x = min(a.origin.x, b.origin.x);
    let min_y = min(a.origin.y, b.origin.y);
    let max_x = max(a.origin.x + a.size.width, b.origin.x + b.size.width);
    let max_y = max(a.origin.y + a.size.height, b.origin.y + b.size.height);

    Rect {
        origin: Point { x: min_x, y: min_y },
        size: Size {
            width: max_x - min_x,
            height: max_y - min_y,
        },
    }
}

pub fn disable_local_event_suppression() {
    let state_id = EventSrcStateId::CombinedSession;
    let mut event_source_ref = EventSrc::with_state(state_id);
    if let Some(ref mut retained) = event_source_ref {
        EventSrc::set_local_events_suppression_interval(retained, 0.0);
    }
}

/*
    Original cursor hiding solution by Nick Bolton, rust variant by myself.
    In the (far) future, I may include a "Magnes" mode that hides the cursor and displays
    my custom iPadOS cursor rendered on top of it. Mostly just included here as a poc though
*/
pub fn hide_cursor() {
    unsafe {
        let property = CString::new("SetsCursorInBackground").unwrap();
        let property_string = CFStringCreateWithCString(
            std::ptr::null(),
            property.as_ptr(),
            K_CF_STRING_ENCODING_MAC_ROMAN,
        );

        let connection = _CGSDefaultConnection();
        CGSSetConnectionProperty(
            connection,
            connection,
            property_string,
            kCFBooleanTrue as *const _,
        );
        CFRelease(property_string);

        let error = CGDisplayHideCursor(K_CG_DIRECT_MAIN_DISPLAY);
        if error != 0 {
            eprintln!("[Error] CGDisplayHideCursor failed (error = {})", error);
        }
    }
}

// My first rust macro. I got tired of writing "NSRect::new(NSPoint::new..."
macro_rules! new_nsrect {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        NSRect::new(NSPoint::new($a, $b), NSSize::new($c, $d))
    };
}
pub(crate) use new_nsrect;

macro_rules! env_f64 {
    ($name:literal) => {
        env!($name).parse::<f64>().unwrap()
    };
}
pub(crate) use env_f64;
