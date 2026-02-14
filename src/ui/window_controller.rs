use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::NSWindowController;

use crate::ui::{view::View, window::Window};

pub struct WindowController {
    pub window_controller: Retained<NSWindowController>,
    pub window: Window,
}

impl WindowController {
    pub fn new(mtm: MainThreadMarker, window: Window) -> Self {
        let window_controller = NSWindowController::initWithWindow(
            NSWindowController::alloc(mtm),
            Some(&window.window),
        );
        Self {
            window_controller,
            window,
        }
    }

    pub fn set_title(&self, string: &str) {
        self.window.set_title(string);
    }

    pub fn set_content_view(&self, view: View) {
        self.window.set_content_view(view);
    }
}
