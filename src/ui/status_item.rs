use objc2::rc::Retained;
use objc2_app_kit::{NSStatusBar, NSStatusItem, NSVariableStatusItemLength};

use crate::ui::menu::Menu;

pub struct StatusItem {
    pub status_item: Retained<NSStatusItem>,
}

impl StatusItem {
    pub fn new() -> Self {
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);
        Self { status_item }
    }

    pub fn set_menu(&self, menu: Menu) {
        self.status_item.setMenu(Some(&menu.menu));
    }
}
