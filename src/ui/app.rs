use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

pub struct App {
    pub app: Retained<NSApplication>,
}

impl App {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let app = NSApplication::init(NSApplication::alloc(mtm));
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        Self { app }
    }
}
