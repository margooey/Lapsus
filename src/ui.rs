pub mod app;
pub mod button;
pub mod menu;
pub mod menu_item;
pub mod status_item;
pub mod view;
pub mod status_bar_button;
pub mod window;
pub mod window_controller;

use objc2::sel;
use objc2_app_kit::{NSBackingStoreType, NSWindowStyleMask};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

use crate::{
    ui::{
        app::App, button::Button, menu::Menu, menu_item::MenuItem, status_item::StatusItem,
        view::View, window::Window, window_controller::WindowController,
    },
    utils::new_nsrect,
};

pub struct UI {
    pub app: App,
    _window_controller: WindowController,
    pub status_item: StatusItem,
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
        let status_item = StatusItem::new();

        // Settings tab
        let settings_status_item_button = status_item
            .status_item
            .button(mtm)
            .expect("status bar item should have a button");
        let title = NSString::from_str("⬤");
        settings_status_item_button.setTitle(&title);

        // Menu
        let menu = Menu::new(mtm);

        // Quit tab
        let quit_title = NSString::from_str("Quit Lapsus");
        let quit_key_equivalent = NSString::from_str("q");
        let quit_item = menu.add_item_with_title_action_key_equivalent(
            &quit_title,
            Some(sel!(terminate:)),
            &quit_key_equivalent,
        );
        quit_item.set_target(Some(&app.app));

        // Divider
        let divider = MenuItem::separator_item(mtm);
        menu.add_item(divider);

        // Settings item
        let settings_title = NSString::from_str("Settings");
        let settings_key_equivalent = NSString::from_str(",");

        let settings_item = menu.add_item_with_title_action_key_equivalent(
            &settings_title,
            Some(sel!(showWindow:)),
            &settings_key_equivalent,
        );
        settings_item.set_target(Some(&window_controller.window_controller));

        // Initialize status item
        status_item.set_menu(menu);
        return Self {
            app,
            _window_controller: window_controller,
            status_item,
            _buttons: vec![test_button, test_button2],
        };
    }
}
