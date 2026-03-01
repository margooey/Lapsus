use objc2::{MainThreadMarker, rc::Retained, runtime::Sel};
use objc2_app_kit::NSMenu;
use objc2_foundation::NSString;

use crate::ui::menu_item::MenuItem;

pub struct Menu {
    pub menu: Retained<NSMenu>,
}

impl Menu {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let menu = NSMenu::new(mtm);
        Self { menu }
    }

    pub fn add_item(&self, new_item: MenuItem) {
        self.menu.addItem(&new_item.menu_item)
    }

    pub fn add_item_with_title_action_key_equivalent(
        &self,
        string: &NSString,
        selector: Option<Sel>,
        char_code: &NSString,
    ) -> MenuItem {
        let menu_item = unsafe {
            self.menu
                .addItemWithTitle_action_keyEquivalent(string, selector, char_code)
        };
        MenuItem { menu_item }
    }
}
