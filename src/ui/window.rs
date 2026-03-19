use objc2::{
    MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_app_kit::{NSApplication, NSBackingStoreType, NSWindow, NSWindowStyleMask};
use objc2_foundation::{
    MainThreadMarker as FMTMarker, NSObject, NSObjectProtocol, NSRect, NSString,
};

use crate::ui::view::View;

define_class!(
    #[unsafe(super(NSWindow))]
    #[name = "LapsusWindow"]
    struct LapsusWindow;

    // Force the app to the foreground when either opening the settings menu for the first time
    // or clicking it a second time to bring it back into focus if it is behind other windows.
    impl LapsusWindow {
        #[unsafe(method(makeKeyAndOrderFront:))]
        fn make_key_and_order_front(&self, sender: Option<&NSObject>) {
            let _: () = unsafe { msg_send![super(self), makeKeyAndOrderFront: sender] };
            let mtm = FMTMarker::new().expect("must be on the main thread");
            NSApplication::sharedApplication(mtm).activate();
        }
    }

    unsafe impl NSObjectProtocol for LapsusWindow {}
);

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
        let alloc: Allocated<LapsusWindow> = LapsusWindow::alloc(mtm);
        let window: Retained<LapsusWindow> = unsafe {
            msg_send![
                alloc,
                initWithContentRect: frame,
                styleMask: style,
                backing: backing_store_type,
                defer: flag,
            ]
        };
        Self {
            window: window.into_super(),
        }
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
