use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::NSView;
use objc2_foundation::NSRect;

pub struct View {
    pub view: Retained<NSView>,
}

impl View {
    pub fn new(mtm: MainThreadMarker, frame: NSRect) -> Self {
        let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
        Self { view }
    }
    pub fn add_subview(&self, view: &NSView) {
        self.view.addSubview(view);
    }
}
