pub mod app;
pub mod button;
pub mod view;
pub mod window;
pub mod window_controller;

use objc2::{rc::Retained, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength, 
    NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

use crate::{
    ui::{app::App, button::Button, view::View, window::Window, window_controller::WindowController},
    utils::new_nsrect,
};

pub struct UI {
    pub app: App,
    _window_controller: WindowController,
    pub status_item: Retained<NSStatusItem>,
    _buttons: Vec<Button>,
}

impl UI {
    pub fn initialize() -> Self {
        let mtm = MainThreadMarker::new().expect("must be on the main thread");
        // App
        let app = App::new(mtm);
        let frame_rect = new_nsrect!(100.0, 100.0, 480.0, 480.0);

        // Window
        let window = Window::new(
            mtm,
            frame_rect,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        );
        window.set_title("Settings");

        // Window Controller
        let window_controller = WindowController::new(mtm, window);

        // View
        let view = View::new(mtm, frame_rect);

        // Buttons
        let mut test_button = Button::new(mtm, new_nsrect!(100.0, 100.0, 100.0, 100.0));
        let mut test_button2 = Button::new(mtm, new_nsrect!(200.0, 200.0, 100.0, 100.0));
        test_button.set_title("Test");
        test_button.set_action(mtm, |_sender| {
            println!("clicked");
        });
        test_button2.set_title("Test 2");
        test_button2.set_action(mtm, |_sender| {
            println!("wow");
        });
        view.add_subview(&test_button.button);
        view.add_subview(&test_button2.button);

        // Set content view after adding all subviews
        window_controller.set_content_view(view);

        // Status item
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);
        let settings_status_item_button = status_item
            .button(mtm)
            .expect("status bar item should have a button");
        let title = NSString::from_str("⬤");
        settings_status_item_button.setTitle(&title);
        let menu = NSMenu::new(mtm);

        // Quit item
        let quit_title = NSString::from_str("Quit Lapsus");
        let quit_key_equivalent = NSString::from_str("q");
        let quit_item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                &quit_title,
                Some(sel!(terminate:)),
                &quit_key_equivalent,
            )
        };
        unsafe { quit_item.setTarget(Some(&app.app)) };

        // Divider item
        let divider = NSMenuItem::separatorItem(mtm);
        menu.addItem(&divider);

        // Settings item
        let settings_title = NSString::from_str("Settings");
        let settings_key_equivalent = NSString::from_str(",");
        let settings_item = unsafe {
            menu.addItemWithTitle_action_keyEquivalent(
                &settings_title,
                Some(sel!(showWindow:)),
                &settings_key_equivalent,
            )
        };
        unsafe { settings_item.setTarget(Some(&window_controller.window_controller)) };

        // Initialize status item
        status_item.setMenu(Some(&menu));
        return Self {
            app,
            _window_controller: window_controller,
            status_item,
            _buttons: vec![test_button, test_button2],
        };
    }
}
