use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowStyleMask};
use objc2_foundation::{NSRect, NSString};

use crate::ui::view::View;

pub struct Window {
    pub window: Retained<NSWindow>,
}

impl Window {
    pub fn new(
        mtm: MainThreadMarker,
        frame: NSRect,
        style: NSWindowStyleMask,
        backing_store_type: NSBackingStoreType,
        flag: bool,
    ) -> Self {
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                frame,
                style,
                backing_store_type,
                flag,
            )
        };
        Self { window }
    }

    pub fn set_title(&self, string: &str) {
        self.window.setTitle(&NSString::from_str(string));
    }

    pub fn set_content_view(&self, view: View) {
        self.window.setContentView(Some(&view.view))
    }

    pub fn center(&self) {
        self.window.center();
    }
}
