use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::NSStatusBarButton;
use objc2_foundation::NSString;

use crate::ui::status_item::StatusItem;

pub struct StatusBarButton {
    pub status_bar_button: Retained<NSStatusBarButton>,
}

impl StatusBarButton {
    pub fn new(mtm: MainThreadMarker, status_item: &StatusItem) -> Self {
        let status_bar_button = status_item.status_item.button(mtm).unwrap();
        Self { status_bar_button }
    }

    pub fn set_title(&self, title: &str) {
        self.status_bar_button.setTitle(&NSString::from_str(title));
    }
}
