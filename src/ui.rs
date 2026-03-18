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

use std::cell::Cell;
use std::ptr::NonNull;
use std::sync::Mutex;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{sel, Message};
use objc2_app_kit::{
    NSBackingStoreType, NSBezelStyle, NSButton, NSControlStateValueOff, NSControlStateValueOn,
    NSEvent, NSEventMask, NSEventType, NSFont, NSLayoutConstraint, NSTextAlignment, NSView,
    NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSPoint, NSRect, NSSize, NSString};
use objc2_service_management::{SMAppService, SMAppServiceStatus};

const WINDOW_RECT: NSRect = new_nsrect!(0.0, 0.0, 420.0, 195.0);
const TOP_MARGIN: f64 = 14.0;
const SIDE_MARGIN: f64 = 20.0;
const LABEL_CONTROL_GAP: f64 = 6.0;
const ROW_GAP: f64 = 8.0;
const SECTION_GAP: f64 = 16.0;
const LABEL_COLUMN_WIDTH: f64 = 92.0;
const KEYBIND_BUTTON_GAP: f64 = 6.0;

use crate::{
    config::config,
    key_monitor,
    ui::{
        app::App, button::Button, checkbox::Checkbox, menu::Menu, menu_item::MenuItem,
        status_bar_button::StatusBarButton, status_item::StatusItem, text_field::TextField,
        window::Window, window_controller::WindowController,
    },
    utils::{env_f64, new_nsrect},
};

/// Wrapper to allow storing `Retained<AnyObject>` in a static.
struct SendRetained(Retained<AnyObject>);
unsafe impl Send for SendRetained {}
unsafe impl Sync for SendRetained {}

impl std::ops::Deref for SendRetained {
    type Target = AnyObject;
    fn deref(&self) -> &AnyObject {
        &self.0
    }
}

// Holds the local event monitor handle during keybind capture so it can be
// removed after the key is captured.
static KEYBIND_MONITOR: Mutex<Option<SendRetained>> = Mutex::new(None);

// Thread-local flag: true while we are waiting for the user to press a key.
thread_local! {
    static CAPTURING_KEYBIND: Cell<bool> = const { Cell::new(false) };
}

pub struct UI {
    _window_controller: WindowController,
    pub status_item: StatusItem,
    _momentum_checkbox: Checkbox,
    _high_speed_checkbox: Checkbox,
    _logon_item_checkbox: Checkbox,
    _require_key_checkbox: Checkbox,
    _keybind_button: Button,
}

impl UI {
    fn set_momentum_enabled(is_enabled: bool) {
        {
            let mut config = config();
            let enabled_min_dt = env_f64!("MIN_DT");
            let disabled_min_dt = 1.0;

            config.min_dt = if is_enabled {
                enabled_min_dt
            } else {
                disabled_min_dt
            };
        }
        crate::config::persist_config();
    }

    fn momentum_is_enabled() -> bool {
        let enabled_min_dt = env_f64!("MIN_DT");
        config().min_dt == enabled_min_dt
    }

    fn set_high_speed_enabled(is_enabled: bool) {
        {
            let mut config = config();
            let default_gain = env_f64!("TRACKPAD_VELOCITY_GAIN");

            config.trackpad_velocity_gain = if is_enabled {
                default_gain * 2.0
            } else {
                default_gain
            };
        }
        crate::config::persist_config();
    }

    fn high_speed_is_enabled() -> bool {
        let default_gain = env_f64!("TRACKPAD_VELOCITY_GAIN");
        let high_speed_gain = default_gain * 2.0;
        config().trackpad_velocity_gain == high_speed_gain
    }

    fn update_logon_item_registration(is_enabled: bool) {
        let app_service = unsafe { SMAppService::mainAppService() };
        let status = unsafe { app_service.status() };

        if is_enabled
            && (status == SMAppServiceStatus::Enabled
                || status == SMAppServiceStatus::RequiresApproval)
        {
            return;
        }
        if !is_enabled && status == SMAppServiceStatus::NotRegistered {
            return;
        }

        let result = if is_enabled {
            unsafe { app_service.registerAndReturnError() }
        } else {
            unsafe { app_service.unregisterAndReturnError() }
        };

        if let Err(error) = result {
            log::warn!("failed to update logon item state: {:?}", error);
        }
    }

    fn set_logon_item_enabled(is_enabled: bool) {
        Self::update_logon_item_registration(is_enabled);
        {
            let mut config = config();
            config.logon_item_enabled = is_enabled;
        }
        crate::config::persist_config();
    }

    fn apply_saved_logon_item_setting() {
        Self::update_logon_item_registration(Self::logon_item_is_enabled());
    }

    fn logon_item_is_enabled() -> bool {
        config().logon_item_enabled
    }

    fn set_require_key_enabled(is_enabled: bool) {
        {
            let mut config = config();
            config.momentum_requires_key = is_enabled;
        }
        crate::config::persist_config();
    }

    fn require_key_is_enabled() -> bool {
        config().momentum_requires_key
    }

    fn keybind_display_title() -> String {
        match config().momentum_activation_key {
            Some(keycode) => key_monitor::keycode_name(keycode).to_string(),
            None => "Click to set".to_string(),
        }
    }

    fn begin_keybind_capture(button: &NSButton) {
        if CAPTURING_KEYBIND.get() {
            return;
        }
        CAPTURING_KEYBIND.set(true);
        button.setTitle(&NSString::from_str("Press a key..."));

        let button = button.retain();
        let mask = NSEventMask::KeyDown | NSEventMask::FlagsChanged;

        let block = RcBlock::new(move |event_ptr: NonNull<NSEvent>| -> *mut NSEvent {
            let event = unsafe { event_ptr.as_ref() };
            let event_type = event.r#type();
            let keycode = event.keyCode();

            // For FlagsChanged, only capture if a modifier was pressed (not released)
            if event_type == NSEventType::FlagsChanged {
                let flags = event.modifierFlags();
                if let Some(flag) = modifier_flag_for_keycode(keycode) {
                    if !flags.contains(flag) {
                        // Modifier released, not pressed — ignore
                        return event_ptr.as_ptr();
                    }
                } else {
                    return event_ptr.as_ptr();
                }
            }

            // Capture this key
            {
                let mut cfg = config();
                cfg.momentum_activation_key = Some(keycode);
            }
            crate::config::persist_config();

            button.setTitle(&NSString::from_str(key_monitor::keycode_name(keycode)));
            CAPTURING_KEYBIND.set(false);

            // Remove the local monitor
            if let Ok(mut monitor) = KEYBIND_MONITOR.lock() {
                if let Some(m) = monitor.take() {
                    unsafe { NSEvent::removeMonitor(&m.0) };
                }
            }

            // Swallow the event so it doesn't propagate
            std::ptr::null_mut()
        });

        // Safety: the block handles all event types in the mask and returns
        // either the event pointer or null to swallow it.
        let monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &block)
        };
        if let Ok(mut m) = KEYBIND_MONITOR.lock() {
            *m = monitor.map(SendRetained);
        }
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

    fn apply_activation_key_row_constraints(
        content_view: &Retained<NSView>,
        activation_label: &TextField,
        require_key_checkbox: &Checkbox,
        keybind_button: &Button,
        logon_item_checkbox: &Checkbox,
    ) {
        activation_label.set_translates_autoresizing_mask_into_constraints(false);
        require_key_checkbox.set_translates_autoresizing_mask_into_constraints(false);
        keybind_button
            .button
            .setTranslatesAutoresizingMaskIntoConstraints(false);

        let constraints = NSArray::from_retained_slice(&[
            // Label
            activation_label
                .leading_anchor()
                .constraintEqualToAnchor_constant(&content_view.leadingAnchor(), SIDE_MARGIN),
            activation_label
                .width_anchor()
                .constraintEqualToConstant(LABEL_COLUMN_WIDTH),
            // Checkbox
            require_key_checkbox
                .leading_anchor()
                .constraintEqualToAnchor_constant(
                    &activation_label.text_field.trailingAnchor(),
                    LABEL_CONTROL_GAP,
                ),
            require_key_checkbox
                .top_anchor()
                .constraintEqualToAnchor_constant(
                    &logon_item_checkbox.button.bottomAnchor(),
                    SECTION_GAP,
                ),
            activation_label
                .first_baseline_anchor()
                .constraintEqualToAnchor(&require_key_checkbox.button.firstBaselineAnchor()),
            // Keybind button — to the right of the checkbox
            keybind_button
                .button
                .leadingAnchor()
                .constraintEqualToAnchor_constant(
                    &require_key_checkbox.button.trailingAnchor(),
                    KEYBIND_BUTTON_GAP,
                ),
            keybind_button
                .button
                .firstBaselineAnchor()
                .constraintEqualToAnchor(&require_key_checkbox.button.firstBaselineAnchor()),
            keybind_button
                .button
                .trailingAnchor()
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

        let window_controller = WindowController::new(mtm, window);

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

        // Activation key controls
        let mut require_key_checkbox =
            Checkbox::init_with_title(mtm, "Require key for momentum");
        require_key_checkbox.set_action(mtm, |sender| {
            Self::set_require_key_enabled(sender.state() == NSControlStateValueOn);
        });

        let mut keybind_button = Button::init(mtm);
        keybind_button.set_title(&Self::keybind_display_title());
        keybind_button.button.setBezelStyle(NSBezelStyle::Push);
        keybind_button.set_action(mtm, |sender| {
            Self::begin_keybind_capture(sender);
        });

        let content_view = window_controller
            .window
            .window
            .contentView()
            .expect("window should have a content view");

        let general_label = TextField::label(mtm, "General:");
        general_label.set_font(label_font.clone());
        general_label.set_alignment(NSTextAlignment::Right);

        let activation_label = TextField::label(mtm, "Activation:");
        activation_label.set_font(label_font);
        activation_label.set_alignment(NSTextAlignment::Right);

        momentum_checkbox.size_to_fit();
        momentum_checkbox.set_state(if Self::momentum_is_enabled() {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        high_speed_checkbox.size_to_fit();
        high_speed_checkbox.set_state(if Self::high_speed_is_enabled() {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        Self::apply_saved_logon_item_setting();
        logon_item_checkbox.size_to_fit();
        logon_item_checkbox.set_state(if Self::logon_item_is_enabled() {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        require_key_checkbox.size_to_fit();
        require_key_checkbox.set_state(if Self::require_key_is_enabled() {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });

        content_view.addSubview(&general_label.text_field);
        content_view.addSubview(&momentum_checkbox.button);
        content_view.addSubview(&high_speed_checkbox.button);
        content_view.addSubview(&logon_item_checkbox.button);
        content_view.addSubview(&activation_label.text_field);
        content_view.addSubview(&require_key_checkbox.button);
        content_view.addSubview(&keybind_button.button);

        Self::apply_general_row_constraints(
            &content_view,
            &general_label,
            &momentum_checkbox,
            &high_speed_checkbox,
            &logon_item_checkbox,
        );
        Self::apply_activation_key_row_constraints(
            &content_view,
            &activation_label,
            &require_key_checkbox,
            &keybind_button,
            &logon_item_checkbox,
        );

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
            Some(sel!(makeKeyAndOrderFront:)),
            &NSString::from_str(","),
        );
        settings_item.set_target(Some(&window_controller.window.window));

        status_item.set_menu(menu);
        Self {
            _window_controller: window_controller,
            status_item,
            _momentum_checkbox: momentum_checkbox,
            _high_speed_checkbox: high_speed_checkbox,
            _logon_item_checkbox: logon_item_checkbox,
            _require_key_checkbox: require_key_checkbox,
            _keybind_button: keybind_button,
        }
    }
}

use objc2_app_kit::NSEventModifierFlags;

fn modifier_flag_for_keycode(keycode: u16) -> Option<NSEventModifierFlags> {
    match keycode {
        56 | 60 => Some(NSEventModifierFlags::Shift),
        59 | 62 => Some(NSEventModifierFlags::Control),
        58 | 61 => Some(NSEventModifierFlags::Option),
        55 | 54 => Some(NSEventModifierFlags::Command),
        63 => Some(NSEventModifierFlags::Function),
        _ => None,
    }
}
