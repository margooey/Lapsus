/*
   I am really in over my head with this UI solution. Some LLM assistance was requested
   to help with the auto layout shenanigans, mostly everything else was rolled by me.
   I specifically referenced https://marioaguzman.github.io/design/layoutguidelines/
   to help me figure out what exactly I wanted the settings window to look like.

   Eventually, I plan to break out all of the work I've done on these objc2 wrappers into a separate
   crate as something like "Lapui" (pronounced LAPOOEY, short for Lapsus UI). Maybe someone else could
   use it.
*/
pub mod app;
pub mod button;
pub mod checkbox;
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

use objc2::sel;
use objc2_app_kit::{
    NSBackingStoreType, NSControlStateValueOff, NSControlStateValueOn, NSFont, NSLayoutConstraint,
    NSTextAlignment, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSPoint, NSRect, NSSize, NSString};

const WINDOW_RECT: NSRect = new_nsrect!(0.0, 0.0, 420.0, 140.0);
const TOP_MARGIN: f64 = 14.0;
const SIDE_MARGIN: f64 = 20.0;
const LABEL_CONTROL_GAP: f64 = 6.0;
const LABEL_COLUMN_WIDTH: f64 = 92.0;

use crate::{
    config,
    ui::{
        app::App, checkbox::Checkbox, menu::Menu, menu_item::MenuItem,
        status_bar_button::StatusBarButton, status_item::StatusItem, text_field::TextField,
        window::Window, window_controller::WindowController,
    },
    utils::new_nsrect,
};

pub struct UI {
    _window_controller: WindowController,
    pub status_item: StatusItem,
    _momentum_checkbox: Checkbox,
}

impl UI {
    fn set_momentum_enabled(is_enabled: bool) {
        let mut config = config();
        let enabled_min_dt = env!("MIN_DT").parse::<f64>().unwrap();
        let disabled_min_dt = 1.0;

        config.min_dt = if is_enabled {
            enabled_min_dt
        } else {
            disabled_min_dt
        };
    }

    fn momentum_is_enabled() -> bool {
        let enabled_min_dt = env!("MIN_DT").parse::<f64>().unwrap();
        config().min_dt == enabled_min_dt
    }

    pub fn initialize() -> Self {
        let mtm = MainThreadMarker::new().expect("must be on the main thread");
        let app = App::new(mtm);
        let label_font = NSFont::systemFontOfSize(13.0);
        app.activate();

        let window = Window::new(
            mtm,
            WINDOW_RECT,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        );
        window.set_title("Settings");
        window.center();

        let window_controller = WindowController::new(mtm, window);

        // Enable/Disable Momentum
        let mut momentum_checkbox = Checkbox::init_with_title(mtm, "Enable momentum");
        momentum_checkbox.set_action(mtm, |sender| {
            Self::set_momentum_enabled(sender.state() == NSControlStateValueOn);
        });
        let content_view = window_controller
            .window
            .window
            .contentView()
            .expect("window should have a content view");

        let general_label = TextField::label(mtm, "General:");
        general_label.set_font(label_font);
        general_label.set_alignment(NSTextAlignment::Right);

        momentum_checkbox.size_to_fit();
        momentum_checkbox.set_state(if Self::momentum_is_enabled() {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });

        general_label
            .text_field
            .setTranslatesAutoresizingMaskIntoConstraints(false);
        momentum_checkbox
            .button
            .setTranslatesAutoresizingMaskIntoConstraints(false);

        content_view.addSubview(&general_label.text_field);
        content_view.addSubview(&momentum_checkbox.button);

        // Auto Layout
        let constraints = NSArray::from_retained_slice(&[
            general_label
                .text_field
                .leadingAnchor()
                .constraintEqualToAnchor_constant(&content_view.leadingAnchor(), SIDE_MARGIN),
            general_label
                .text_field
                .widthAnchor()
                .constraintEqualToConstant(LABEL_COLUMN_WIDTH),
            momentum_checkbox
                .button
                .leadingAnchor()
                .constraintEqualToAnchor_constant(
                    &general_label.text_field.trailingAnchor(),
                    LABEL_CONTROL_GAP,
                ),
            momentum_checkbox
                .button
                .topAnchor()
                .constraintEqualToAnchor_constant(&content_view.topAnchor(), TOP_MARGIN),
            momentum_checkbox
                .button
                .trailingAnchor()
                .constraintLessThanOrEqualToAnchor_constant(
                    &content_view.trailingAnchor(),
                    -SIDE_MARGIN,
                ),
            general_label
                .text_field
                .firstBaselineAnchor()
                .constraintEqualToAnchor(&momentum_checkbox.button.firstBaselineAnchor()),
        ]);
        NSLayoutConstraint::activateConstraints(&constraints);

        let status_item = StatusItem::new();
        let status_bar_button = StatusBarButton::new(mtm, &status_item);
        status_bar_button.set_title("⬤");

        let menu = Menu::new(mtm);

        let quit_item = menu.add_item_with_title_action_key_equivalent(
            &NSString::from_str("Quit Lapsus"),
            Some(sel!(terminate:)),
            &NSString::from_str("q"),
        );
        quit_item.set_target(Some(&app.app));

        let divider = MenuItem::separator_item(mtm);
        menu.add_item(divider);

        let settings_item = menu.add_item_with_title_action_key_equivalent(
            &NSString::from_str("Settings"),
            Some(sel!(showWindow:)),
            &NSString::from_str(","),
        );
        settings_item.set_target(Some(&window_controller.window_controller));

        status_item.set_menu(menu);
        Self {
            _window_controller: window_controller,
            status_item,
            _momentum_checkbox: momentum_checkbox,
        }
    }
}
