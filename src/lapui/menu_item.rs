use objc2::{MainThreadMarker, rc::Retained, runtime::AnyObject};
use objc2_app_kit::NSMenuItem;

pub struct MenuItem {
    pub menu_item: Retained<NSMenuItem>,
}

impl MenuItem {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let menu_item = NSMenuItem::new(mtm);
        Self { menu_item }
    }

    pub fn separator_item(mtm: MainThreadMarker) -> Self {
        Self {
            menu_item: NSMenuItem::separatorItem(mtm),
        }
    }

    pub fn set_target(&self, target: Option<&AnyObject>) {
        unsafe { self.menu_item.setTarget(target) };
    }
}
