pub mod app;
pub mod button;
pub mod grid_view;
pub mod menu;
pub mod menu_item;
pub mod status_bar_button;
pub mod status_item;
pub mod switch;
pub mod text_field;
pub mod view;
pub mod window;
pub mod window_controller;

use objc2::{ClassType, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSControlStateValueOn, NSFont, NSGridCellPlacement, NSLayoutConstraint,
    NSView, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSPoint, NSRect, NSSize, NSString};

const ZERO_RECT: NSRect = new_nsrect!(0.0, 0.0, 0.0, 0.0);
const CONTENT_PADDING: f64 = 10.0;
const GRID_SPACING: f64 = 10.0;
const WINDOW_RECT: NSRect = new_nsrect!(0.0, 0.0, 600.0, 400.0);

use crate::{
    config,
    ui::{
        app::App, grid_view::GridView, menu::Menu, menu_item::MenuItem,
        status_bar_button::StatusBarButton, status_item::StatusItem, switch::Switch,
        text_field::TextField, window::Window, window_controller::WindowController,
    },
    utils::new_nsrect,
};

pub struct UI {
    _window_controller: WindowController,
    pub status_item: StatusItem,
    _switch: Switch,
}

impl UI {
    fn toggle_momentum() {
        let mut config = config();
        let enabled_min_dt = env!("MIN_DT").parse::<f64>().unwrap();
        let disabled_min_dt = 1.0;

        config.min_dt = if config.min_dt == enabled_min_dt {
            disabled_min_dt
        } else {
            enabled_min_dt
        };
    }
    pub fn initialize() -> Self {
        let mtm = MainThreadMarker::new().expect("must be on the main thread");
        // App
        let app = App::new(mtm);
        let mut views: Vec<&NSView> = vec![];
        let header_font = NSFont::boldSystemFontOfSize(24.0);
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

        // Enable/Disable Switch
        let mut switch = Switch::init(mtm);
        switch.set_action(mtm, |_| {
            Self::toggle_momentum();
        });
        switch.set_state(NSControlStateValueOn);
        views.push(&switch.switch);

        // Settings Window Header
        let header = TextField::init(mtm);
        header.set_string_value("Lapsus Settings");
        header.set_editable(false);
        header.set_bordered(false);
        header.set_font(header_font);
        header
            .text_field
            .setTranslatesAutoresizingMaskIntoConstraints(false);

        // View
        let content_view = window_controller
            .window
            .window
            .contentView()
            .expect("window should have a content view");

        // Grid View
        let grid_view = GridView::new(mtm, ZERO_RECT);
        grid_view.add_row_with_views(&views);

        grid_view.set_x_placeholder(NSGridCellPlacement::Leading);
        grid_view.set_translates_autoresizing_mask_into_constraints(false);

        grid_view.set_column_spacing(GRID_SPACING);
        grid_view.set_row_spacing(GRID_SPACING);

        // Setup Layout
        content_view.addSubview(&header.text_field);
        content_view.addSubview(grid_view.grid_view.as_super());

        // Auto Layout Constraints
        let constraints = NSArray::from_retained_slice(&[
            header
                .text_field
                .leadingAnchor()
                .constraintEqualToAnchor_constant(&content_view.leadingAnchor(), CONTENT_PADDING),
            header
                .text_field
                .topAnchor()
                .constraintEqualToAnchor_constant(&content_view.topAnchor(), CONTENT_PADDING),
            grid_view
                .grid_view
                .leadingAnchor()
                .constraintEqualToAnchor_constant(&content_view.leadingAnchor(), CONTENT_PADDING),
            grid_view
                .grid_view
                .topAnchor()
                .constraintEqualToAnchor_constant(&header.text_field.bottomAnchor(), GRID_SPACING),
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
            _switch: switch,
        };
    }
}
