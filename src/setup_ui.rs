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
    app::App, button::Button, checkbox::Checkbox, grid_view::GridView, menu::Menu,
    menu_item::MenuItem, slider::Slider, status_bar_button::StatusBarButton,
    status_item::StatusItem, text_field::TextField, window::Window,
};

use objc2::{rc::Retained, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSButton, NSControlStateValueOff, NSControlStateValueOn, NSFont,
    NSGridCellPlacement, NSGridRowAlignment, NSLayoutConstraint, NSSlider, NSTextAlignment, NSView,
    NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSArray, NSPoint, NSRect, NSSize, NSString};
use objc2_service_management::{SMAppService, SMAppServiceStatus};

const WINDOW_RECT: NSRect = new_nsrect!(0.0, 0.0, 420.0, 228.0);
const TOP_MARGIN: f64 = 14.0;
const BOTTOM_MARGIN: f64 = 20.0;
const SIDE_MARGIN: f64 = 20.0;
const LABEL_CONTROL_GAP: f64 = 6.0;
const ROW_GAP: f64 = 8.0;
const SECTION_GAP: f64 = ROW_GAP * 3.0;
const LABEL_COLUMN_WIDTH: f64 = 92.0;
const SLIDER_WIDTH: f64 = 200.0;
const STATUS_ICON: &[u8] = include_bytes!("../assets/cursoroutline_center.png");

use crate::{
    config::config,
    utils::{env_f64, new_nsrect},
};

/// One control in the settings window.
enum Row {
    Checkbox {
        title: &'static str,
        get: fn() -> bool,
        set: fn(bool),
    },
    /// Snaps to `tick_marks` evenly spaced values between `min` and `max`.
    /// `label` names this slider in the label column.
    Slider {
        label: Option<&'static str>,
        min: f64,
        max: f64,
        tick_marks: isize,
        get: fn() -> f64,
        set: fn(f64),
    },
}

/// A group of rows, e.g. "General:". The section label is shown next to the
/// first row unless that row has a label of its own.
struct Section {
    label: Option<&'static str>,
    rows: &'static [Row],
}

/// Everything shown in the settings window. To add a setting, add a row here.
const SECTIONS: &[Section] = &[
    Section {
        label: Some("General:"),
        rows: &[
            Row::Checkbox {
                title: "Enable momentum",
                get: UI::momentum_is_enabled,
                set: UI::set_momentum_enabled,
            },
            Row::Checkbox {
                title: "Launch on login",
                get: UI::logon_item_is_enabled,
                set: UI::set_logon_item_enabled,
            },
            Row::Checkbox {
                title: "Enable palm rejection (Experimental)",
                get: UI::palm_rejection_is_enabled,
                set: UI::set_palm_rejection_enabled,
            },
            Row::Checkbox {
                title: "Check for updates on launch",
                get: UI::update_check_is_enabled,
                set: UI::set_update_check_enabled,
            },
        ],
    },
    Section {
        label: Some("Speed:"),
        // 0.5x to 3x the default speed, in 0.25 steps.
        rows: &[Row::Slider {
            label: None,
            min: 0.5,
            max: 3.0,
            tick_marks: 11,
            get: UI::speed_multiplier,
            set: UI::set_speed_multiplier,
        }],
    },
    Section {
        label: None,
        rows: &[
            // 0.5x to 2x the default glide length, in 0.25 steps.
            Row::Slider {
                label: Some("Glide length:"),
                min: 0.5,
                max: 2.0,
                tick_marks: 7,
                get: UI::glide_length_multiplier,
                set: UI::set_glide_length_multiplier,
            },
            // 0.25x to 2x the default sensitivity, in 0.25 steps.
            Row::Slider {
                label: Some("Flick sensitivity:"),
                min: 0.25,
                max: 2.0,
                tick_marks: 8,
                get: UI::flick_sensitivity_multiplier,
                set: UI::set_flick_sensitivity_multiplier,
            },
        ],
    },
    Section {
        label: Some("Fun:"),
        rows: &[
            Row::Checkbox {
                title: "Air hockey mode",
                get: UI::air_hockey_mode_is_enabled,
                set: UI::set_air_hockey_mode_enabled,
            },
            Row::Checkbox {
                title: "Wrap around screen edges",
                get: UI::wrap_around_is_enabled,
                set: UI::set_wrap_around_enabled,
            },
        ],
    },
];

/// Controls kept alive so their action closures keep working.
type Controls = Vec<Box<dyn std::any::Any>>;

pub struct UI {
    _window: Window,
    pub status_item: StatusItem,
    // Held so the controls' action closures stay alive.
    _controls: Controls,
    _restore_defaults_button: Button,
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

    /// Speed relative to the default gain (1.0 = default).
    fn set_speed_multiplier(multiplier: f64) {
        config().trackpad_velocity_gain = env_f64!("TRACKPAD_VELOCITY_GAIN") * multiplier;
        crate::config::persist_config();
    }

    fn speed_multiplier() -> f64 {
        config().trackpad_velocity_gain / env_f64!("TRACKPAD_VELOCITY_GAIN")
    }

    /// Glide length relative to the default (2.0 = glides twice as long).
    /// Longer glides mean less decay, so this divides the decay rate.
    fn set_glide_length_multiplier(multiplier: f64) {
        config().glide_decay_per_second = env_f64!("GLIDE_DECAY_PER_SECOND") / multiplier;
        crate::config::persist_config();
    }

    fn glide_length_multiplier() -> f64 {
        env_f64!("GLIDE_DECAY_PER_SECOND") / config().glide_decay_per_second
    }

    /// Flick sensitivity relative to the default (2.0 = half the speed needed to glide).
    /// Higher sensitivity means a lower minimum glide speed, so this divides it.
    fn set_flick_sensitivity_multiplier(multiplier: f64) {
        config().minimum_glide_velocity = env_f64!("MINIMUM_GLIDE_VELOCITY") / multiplier;
        crate::config::persist_config();
    }

    fn flick_sensitivity_multiplier() -> f64 {
        env_f64!("MINIMUM_GLIDE_VELOCITY") / config().minimum_glide_velocity
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

    fn set_palm_rejection_enabled(is_enabled: bool) {
        config().palm_rejection_enabled = is_enabled;
        crate::config::persist_config();
    }

    fn palm_rejection_is_enabled() -> bool {
        config().palm_rejection_enabled
    }

    fn set_air_hockey_mode_enabled(is_enabled: bool) {
        config().air_hockey_mode_enabled = is_enabled;
        crate::config::persist_config();
    }

    fn air_hockey_mode_is_enabled() -> bool {
        config().air_hockey_mode_enabled
    }

    fn set_update_check_enabled(is_enabled: bool) {
        config().update_check_enabled = is_enabled;
        crate::config::persist_config();
    }

    fn update_check_is_enabled() -> bool {
        config().update_check_enabled
    }

    fn set_wrap_around_enabled(is_enabled: bool) {
        config().wrap_around_enabled = is_enabled;
        crate::config::persist_config();
    }

    fn wrap_around_is_enabled() -> bool {
        config().wrap_around_enabled
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

    /// Builds the label/control grid from `SECTIONS`. Also returns a function that
    /// re-reads every setting into its control.
    fn build_settings_grid(mtm: MainThreadMarker) -> (GridView, Controls, Box<dyn Fn()>) {
        let grid = GridView::new(mtm, NSRect::ZERO);
        grid.set_row_spacing(ROW_GAP);
        grid.set_column_spacing(LABEL_CONTROL_GAP);
        grid.set_row_alignment(NSGridRowAlignment::FirstBaseline);

        let label_font = NSFont::systemFontOfSize(13.0);
        let mut controls: Controls = Vec::new();
        let mut section_start_rows = Vec::new();
        let mut slider_rows = Vec::new();
        let mut refreshers: Vec<Box<dyn Fn()>> = Vec::new();
        let mut row_count = 0;

        for section in SECTIONS {
            section_start_rows.push(row_count);
            for (i, row) in section.rows.iter().enumerate() {
                let row_label = match *row {
                    Row::Slider { label, .. } => label,
                    Row::Checkbox { .. } => None,
                };
                let label_text = row_label.or(if i == 0 { section.label } else { None });
                let label: Retained<NSView> = match label_text {
                    Some(text) => {
                        let label = TextField::label(mtm, text);
                        label.set_font(label_font.clone());
                        label.set_alignment(NSTextAlignment::Right);
                        // Keeps the label column at least this wide; longer labels widen it.
                        label
                            .width_anchor()
                            .constraintGreaterThanOrEqualToConstant(LABEL_COLUMN_WIDTH)
                            .setActive(true);
                        label.text_field.into_super().into_super()
                    }
                    None => GridView::empty_cell(mtm),
                };

                match *row {
                    Row::Checkbox { title, get, set } => {
                        let mut checkbox = Checkbox::init_with_title(mtm, title);
                        checkbox.set_action(mtm, move |sender: &NSButton| {
                            set(sender.state() == NSControlStateValueOn);
                        });
                        checkbox.set_state(control_state(get()));
                        let button = checkbox.button.clone();
                        refreshers.push(Box::new(move || button.setState(control_state(get()))));
                        grid.add_row_with_views(&[&label, &checkbox.button]);
                        controls.push(Box::new(checkbox));
                    }
                    Row::Slider {
                        min,
                        max,
                        tick_marks,
                        get,
                        set,
                        ..
                    } => {
                        let mut slider = Slider::new(mtm, get().clamp(min, max), min, max);
                        slider.set_number_of_tick_marks(tick_marks);
                        slider.set_allows_tick_mark_values_only(true);
                        // Save once the knob is released, not on every drag step.
                        slider.set_continuous(false);
                        slider.set_action(mtm, move |sender: &NSSlider| {
                            set(sender.doubleValue());
                        });
                        slider
                            .slider
                            .widthAnchor()
                            .constraintEqualToConstant(SLIDER_WIDTH)
                            .setActive(true);
                        grid.add_row_with_views(&[&label, &slider.slider]);
                        let view = slider.slider.clone();
                        refreshers
                            .push(Box::new(move || view.setDoubleValue(get().clamp(min, max))));
                        controls.push(Box::new(slider));
                        slider_rows.push(row_count);
                    }
                }
                row_count += 1;
            }
        }

        let label_column = grid.column_at_index(0);
        label_column.setXPlacement(NSGridCellPlacement::Trailing);

        // Sliders have no text baseline, so center their label vertically instead.
        for &row in &slider_rows {
            let row = grid.row_at_index(row);
            row.setRowAlignment(NSGridRowAlignment::None);
            row.setYPlacement(NSGridCellPlacement::Center);
        }

        // Extra space above every section but the first.
        for &row in section_start_rows.iter().skip(1) {
            grid.row_at_index(row).setTopPadding(SECTION_GAP - ROW_GAP);
        }

        let refresh = Box::new(move || refreshers.iter().for_each(|refresh| refresh()));
        (grid, controls, refresh)
    }

    pub fn initialize() -> Self {
        let mtm = MainThreadMarker::new().expect("must be on the main thread");
        let app = App::new(mtm);
        app.activate();

        let window = Window::new(
            mtm,
            WINDOW_RECT,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        );
        window.set_title("Settings");

        let content_view = window
            .window
            .contentView()
            .expect("window should have a content view");

        let (grid, controls, refresh_controls) = Self::build_settings_grid(mtm);

        let mut restore_defaults_button = Button::with_title(mtm, "Restore Defaults");
        restore_defaults_button.set_action(mtm, move |_| {
            crate::config::restore_defaults();
            refresh_controls();
        });
        let restore_view = &restore_defaults_button.button;
        restore_view.setTranslatesAutoresizingMaskIntoConstraints(false);
        content_view.addSubview(restore_view);

        grid.set_translates_autoresizing_mask_into_constraints(false);
        let grid_view = &grid.grid_view;
        content_view.addSubview(grid_view);
        NSLayoutConstraint::activateConstraints(&NSArray::from_retained_slice(&[
            grid_view
                .topAnchor()
                .constraintEqualToAnchor_constant(&content_view.topAnchor(), TOP_MARGIN),
            grid_view
                .leadingAnchor()
                .constraintEqualToAnchor_constant(&content_view.leadingAnchor(), SIDE_MARGIN),
            grid_view
                .trailingAnchor()
                .constraintLessThanOrEqualToAnchor_constant(
                    &content_view.trailingAnchor(),
                    -SIDE_MARGIN,
                ),
            restore_view
                .topAnchor()
                .constraintEqualToAnchor_constant(&grid_view.bottomAnchor(), SECTION_GAP),
            restore_view
                .trailingAnchor()
                .constraintEqualToAnchor_constant(&content_view.trailingAnchor(), -SIDE_MARGIN),
            restore_view
                .bottomAnchor()
                .constraintEqualToAnchor_constant(&content_view.bottomAnchor(), -BOTTOM_MARGIN),
        ]));

        // Size the window to fit the grid, keeping at least the default width.
        let fitting = content_view.fittingSize();
        window.window.setContentSize(NSSize::new(
            fitting.width.max(WINDOW_RECT.size.width),
            fitting.height,
        ));
        window.center();

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
            _controls: controls,
            _restore_defaults_button: restore_defaults_button,
        }
    }
}
