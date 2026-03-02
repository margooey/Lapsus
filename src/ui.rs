pub mod app;
pub mod button;
pub mod grid_view;
pub mod menu;
pub mod menu_item;
pub mod slider;
pub mod status_bar_button;
pub mod status_item;
pub mod text_field;
pub mod view;
pub mod window;
pub mod window_controller;

use objc2::{ClassType, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSGridCellPlacement, NSLayoutConstraint, NSView, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSPoint, NSRect, NSSize, NSString};

const ZERO_RECT: NSRect = new_nsrect!(0.0, 0.0, 0.0, 0.0);
const BUTTON_OFFSET: f64 = 10.0;
const GRID_SPACING: f64 = 10.0;
const WINDOW_RECT: NSRect = new_nsrect!(0.0, 0.0, 600.0, 400.0);

use crate::{
    config,
    ui::{
        app::App, button::Button, grid_view::GridView, menu::Menu, menu_item::MenuItem,
        status_bar_button::StatusBarButton, status_item::StatusItem, text_field::TextField,
        window::Window, window_controller::WindowController,
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
        let mut views: Vec<&NSView> = vec![];
        app.activate();

        // Window
        let window = Window::new(
            mtm,
            WINDOW_RECT,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        );
        window.set_title("Settings");
        window.center();

        // Window Controller
        let window_controller = WindowController::new(mtm, window);

        // Buttons
        let mut test_button = Button::init(mtm);
        let mut test_button2 = Button::init(mtm);
        test_button.set_title("Stop");
        test_button.set_action(mtm, |_| {
            config().min_dt = 1.0; // Disable
            print!("set min_dt to 1.0")
        });
        test_button2.set_title("Start");
        test_button2.set_action(mtm, |_| {
            config().min_dt = 0.005; // Enable
            println!("set min_dt to 0.005");
        });
        views.push(&test_button.button);
        views.push(&test_button2.button);

        // Label
        let label = TextField::init(mtm);
        label.set_string_value("Test Label");
        label.set_editable(false);
        label.set_bordered(false);
        views.push(&label.text_field);

        // View
        let content_view = window_controller
            .window
            .window
            .contentView()
            .expect("window should have a content view");

        // Grid View
        let grid_view = GridView::new(mtm, ZERO_RECT);
        // TODO: Allow passing in arbitrary objects that have an NSView superclass (.as_view())
        grid_view.add_row_with_views(&views);

        grid_view.set_x_placeholder(NSGridCellPlacement::Leading);
        grid_view.set_translates_autoresizing_mask_into_constraints(false);

        grid_view.set_column_spacing(GRID_SPACING);
        grid_view.set_row_spacing(GRID_SPACING);
        content_view.addSubview(grid_view.grid_view.as_super());

        // Auto Layout
        let constraints = NSArray::from_retained_slice(&[
            grid_view
                .grid_view
                .leadingAnchor()
                .constraintEqualToAnchor_constant(&content_view.leadingAnchor(), BUTTON_OFFSET),
            grid_view
                .grid_view
                .topAnchor()
                .constraintEqualToAnchor_constant(&content_view.topAnchor(), BUTTON_OFFSET),
        ]);
        NSLayoutConstraint::activateConstraints(&constraints);

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
            _buttons: vec![test_button, test_button2],
        };
    }
}
