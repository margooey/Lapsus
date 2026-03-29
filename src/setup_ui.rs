/*
   I am really in over my head with this UI solution. Some LLM assistance was requested
   to help with the auto layout shenanigans, mostly everything else was rolled by me.
   I specifically referenced https://marioaguzman.github.io/design/layoutguidelines/
   to help me figure out what exactly I wanted the settings window to look like.

   Eventually, I plan to break out all of the work I've done on these objc2 wrappers into a separate
   crate as something like "Lapui" (pronounced LAPOOEY, short for Lapsus UI). Maybe someone else could
   use it.

   Oh look! Lapui is here below :)
*/

use lapui::{
    app::App, checkbox::Checkbox, menu::Menu, menu_item::MenuItem,
    status_bar_button::StatusBarButton, status_item::StatusItem, text_field::TextField,
    window::Window,
};

use objc2::{rc::Retained, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSControlStateValueOff, NSControlStateValueOn, NSFont, NSLayoutConstraint,
    NSTextAlignment, NSView, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSPoint, NSRect, NSSize, NSString};
use objc2_service_management::{SMAppService, SMAppServiceStatus};

const WINDOW_RECT: NSRect = new_nsrect!(0.0, 0.0, 420.0, 140.0);
const TOP_MARGIN: f64 = 14.0;
const SIDE_MARGIN: f64 = 20.0;
const LABEL_CONTROL_GAP: f64 = 6.0;
const ROW_GAP: f64 = 8.0;
const LABEL_COLUMN_WIDTH: f64 = 92.0;
const STATUS_ICON: &[u8] = include_bytes!("../assets/cursoroutline_center.png");

use crate::{
    config::config,
    utils::{env_f64, new_nsrect},
};

pub struct UI {
    _window: Window,
    pub status_item: StatusItem,
    _momentum_checkbox: Checkbox,
    _high_speed_checkbox: Checkbox,
    _logon_item_checkbox: Checkbox,
}

fn control_state(enabled: bool) -> objc2_foundation::NSInteger {
    if enabled {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    }
}

impl UI {
    fn set_momentum_enabled(is_enabled: bool) {
        config().min_dt = if is_enabled { env_f64!("MIN_DT") } else { 1.0 };
        crate::config::persist_config();
    }

    fn momentum_is_enabled() -> bool {
        config().min_dt == env_f64!("MIN_DT")
    }

    fn set_high_speed_enabled(is_enabled: bool) {
        let default_gain = env_f64!("TRACKPAD_VELOCITY_GAIN");
        config().trackpad_velocity_gain = if is_enabled {
            default_gain * 2.0
        } else {
            default_gain
        };
        crate::config::persist_config();
    }

    fn high_speed_is_enabled() -> bool {
        config().trackpad_velocity_gain == env_f64!("TRACKPAD_VELOCITY_GAIN") * 2.0
    }

    fn update_logon_item_registration(is_enabled: bool) {
        let app_service = unsafe { SMAppService::mainAppService() };
        let status = unsafe { app_service.status() };

        let already_set = matches!(
            (is_enabled, status),
            (
                true,
                SMAppServiceStatus::Enabled | SMAppServiceStatus::RequiresApproval
            ) | (false, SMAppServiceStatus::NotRegistered)
        );
        if already_set {
            return;
        }

        let result = unsafe {
            if is_enabled {
                app_service.registerAndReturnError()
            } else {
                app_service.unregisterAndReturnError()
            }
        };

        if let Err(error) = result {
            log::warn!("failed to update logon item state: {:?}", error);
        }
    }

    fn set_logon_item_enabled(is_enabled: bool) {
        Self::update_logon_item_registration(is_enabled);
    }

    fn logon_item_is_enabled() -> bool {
        let app_service = unsafe { SMAppService::mainAppService() };
        let status = unsafe { app_service.status() };
        matches!(
            status,
            SMAppServiceStatus::Enabled | SMAppServiceStatus::RequiresApproval
        )
    }

    fn apply_general_row_constraints(
        content_view: &Retained<NSView>,
        general_label: &TextField,
        momentum_checkbox: &Checkbox,
        high_speed_checkbox: &Checkbox,
        logon_item_checkbox: &Checkbox,
    ) {
        general_label.set_translates_autoresizing_mask_into_constraints(false);
        momentum_checkbox.set_translates_autoresizing_mask_into_constraints(false);
        high_speed_checkbox.set_translates_autoresizing_mask_into_constraints(false);
        logon_item_checkbox.set_translates_autoresizing_mask_into_constraints(false);

        let constraints = NSArray::from_retained_slice(&[
            general_label
                .leading_anchor()
                .constraintEqualToAnchor_constant(&content_view.leadingAnchor(), SIDE_MARGIN),
            general_label
                .width_anchor()
                .constraintEqualToConstant(LABEL_COLUMN_WIDTH),
            momentum_checkbox
                .leading_anchor()
                .constraintEqualToAnchor_constant(
                    &general_label.text_field.trailingAnchor(),
                    LABEL_CONTROL_GAP,
                ),
            momentum_checkbox
                .top_anchor()
                .constraintEqualToAnchor_constant(&content_view.topAnchor(), TOP_MARGIN),
            momentum_checkbox
                .trailing_anchor()
                .constraintLessThanOrEqualToAnchor_constant(
                    &content_view.trailingAnchor(),
                    -SIDE_MARGIN,
                ),
            general_label
                .first_baseline_anchor()
                .constraintEqualToAnchor(&momentum_checkbox.button.firstBaselineAnchor()),
            high_speed_checkbox
                .leading_anchor()
                .constraintEqualToAnchor(&momentum_checkbox.leading_anchor()),
            high_speed_checkbox
                .top_anchor()
                .constraintEqualToAnchor_constant(
                    &momentum_checkbox.button.bottomAnchor(),
                    ROW_GAP,
                ),
            high_speed_checkbox
                .trailing_anchor()
                .constraintLessThanOrEqualToAnchor_constant(
                    &content_view.trailingAnchor(),
                    -SIDE_MARGIN,
                ),
            logon_item_checkbox
                .leading_anchor()
                .constraintEqualToAnchor(&momentum_checkbox.leading_anchor()),
            logon_item_checkbox
                .top_anchor()
                .constraintEqualToAnchor_constant(
                    &high_speed_checkbox.button.bottomAnchor(),
                    ROW_GAP,
                ),
            logon_item_checkbox
                .trailing_anchor()
                .constraintLessThanOrEqualToAnchor_constant(
                    &content_view.trailingAnchor(),
                    -SIDE_MARGIN,
                ),
        ]);
        NSLayoutConstraint::activateConstraints(&constraints);
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

        let mut momentum_checkbox = Checkbox::init_with_title(mtm, "Enable momentum");
        momentum_checkbox.set_action(mtm, |sender| {
            Self::set_momentum_enabled(sender.state() == NSControlStateValueOn);
        });
        let mut high_speed_checkbox = Checkbox::init_with_title(mtm, "High speed");
        high_speed_checkbox.set_action(mtm, |sender| {
            Self::set_high_speed_enabled(sender.state() == NSControlStateValueOn);
        });
        let mut logon_item_checkbox = Checkbox::init_with_title(mtm, "Launch on login");
        logon_item_checkbox.set_action(mtm, |sender| {
            Self::set_logon_item_enabled(sender.state() == NSControlStateValueOn);
        });
        let content_view = window
            .window
            .contentView()
            .expect("window should have a content view");

        let general_label = TextField::label(mtm, "General:");
        general_label.set_font(label_font);
        general_label.set_alignment(NSTextAlignment::Right);

        momentum_checkbox.size_to_fit();
        momentum_checkbox.set_state(control_state(Self::momentum_is_enabled()));
        high_speed_checkbox.size_to_fit();
        high_speed_checkbox.set_state(control_state(Self::high_speed_is_enabled()));
        logon_item_checkbox.size_to_fit();
        logon_item_checkbox.set_state(control_state(Self::logon_item_is_enabled()));

        content_view.addSubview(&general_label.text_field);
        content_view.addSubview(&momentum_checkbox.button);
        content_view.addSubview(&high_speed_checkbox.button);
        content_view.addSubview(&logon_item_checkbox.button);

        Self::apply_general_row_constraints(
            &content_view,
            &general_label,
            &momentum_checkbox,
            &high_speed_checkbox,
            &logon_item_checkbox,
        );

        let status_item = StatusItem::init();
        let status_bar_button = StatusBarButton::new(mtm, &status_item);
        status_bar_button.set_image(STATUS_ICON);

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
            Some(sel!(makeKeyAndOrderFront:)),
            &NSString::from_str(","),
        );
        settings_item.set_target(Some(&window.window));

        status_item.set_menu(menu);
        Self {
            _window: window,
            status_item,
            _momentum_checkbox: momentum_checkbox,
            _high_speed_checkbox: high_speed_checkbox,
            _logon_item_checkbox: logon_item_checkbox,
        }
    }
}
