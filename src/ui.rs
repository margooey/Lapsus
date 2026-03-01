pub mod app;
pub mod button;
pub mod grid_view;
pub mod menu;
pub mod menu_item;
pub mod slider;
pub mod status_bar_button;
pub mod status_item;
pub mod view;
pub mod window;
pub mod window_controller;

use objc2::{ClassType, sel};
use objc2_app_kit::{NSBackingStoreType, NSWindowStyleMask};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

use crate::{
    ui::{
        app::App, button::Button, grid_view::GridView, menu::Menu, menu_item::MenuItem,
        status_bar_button::StatusBarButton, status_item::StatusItem, window::Window,
        window_controller::WindowController,
    },
    utils::new_nsrect,
};

pub struct UI {
    _window_controller: WindowController,
    pub status_item: StatusItem,
    _buttons: Vec<Button>,
}

impl UI {
    pub fn initialize() -> Self {
        let mtm = MainThreadMarker::new().expect("must be on the main thread");
        // App
        let app = App::new(mtm);
        app.activate();

        // Window
        let window = Window::new(
            mtm,
            new_nsrect!(0.0, 0.0, 600.0, 400.0),
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        );
        window.set_title("Settings");
        window.center();

        // Window Controller
        let window_controller = WindowController::new(mtm, window);

        // Buttons
        let mut test_button = Button::new(mtm, new_nsrect!(40.0, 40.0, 100.0, 100.0));
        let mut test_button2 = Button::new(mtm, new_nsrect!(180.0, 40.0, 100.0, 100.0));
        test_button.set_title("Test");
        test_button.set_action(mtm, |_| {
            println!("clicked");
        });
        test_button2.set_title("Test 2");
        test_button2.set_action(mtm, |_| {
            println!("wow");
        });

        let buttons = vec![test_button, test_button2];

        // View
        let content_view = window_controller.window.window.contentView().expect("…");

        // Grid View
        let grid_view = GridView::new(mtm, content_view.bounds());
        // TODO: Allow passing in arbitrary objects that have an NSView superclass (.as_view())
        grid_view.add_row_with_views(&[buttons[0].button.as_super(), buttons[1].button.as_super()]);

        grid_view.set_column_spacing(10.0);
        grid_view.set_row_spacing(10.0);

        // Set content view after adding all subviews
        window_controller
            .window
            .set_content_view(GridView::as_view(&grid_view));

        // Status item
        let status_item = StatusItem::new();

        // Settings bar button
        let status_bar_button = StatusBarButton::new(mtm, &status_item);
        status_bar_button.set_title("⬤");

        // Menu
        let menu = Menu::new(mtm);

        // Quit tab
        let quit_item = menu.add_item_with_title_action_key_equivalent(
            &NSString::from_str("Quit Lapsus"),
            Some(sel!(terminate:)),
            &NSString::from_str("q"),
        );
        quit_item.set_target(Some(&app.app));

        // Divider
        let divider = MenuItem::separator_item(mtm);
        menu.add_item(divider);

        // Settings item
        let settings_item = menu.add_item_with_title_action_key_equivalent(
            &NSString::from_str("Settings"),
            Some(sel!(showWindow:)),
            &NSString::from_str(","),
        );
        settings_item.set_target(Some(&window_controller.window_controller));

        // Initialize status item
        status_item.set_menu(menu);
        return Self {
            _window_controller: window_controller,
            status_item,
            _buttons: buttons,
        };
    }
}
