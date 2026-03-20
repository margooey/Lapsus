use objc2::{AnyThread, MainThreadMarker, rc::Retained};
use objc2_app_kit::{NSImage, NSStatusBarButton};
use objc2_foundation::{NSData, NSString};

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

    pub fn set_image(&self, bytes: &[u8]) {
        let data = NSData::with_bytes(bytes);
        let image = NSImage::initWithData(NSImage::alloc(), &data);
        if let Some(ref img) = image {
            img.setSize(objc2_core_foundation::CGSize::new(18.0, 18.0));
        }
        self.status_bar_button.setImage(image.as_deref());
    }
}
