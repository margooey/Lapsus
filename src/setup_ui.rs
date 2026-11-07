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
    app::App, checkbox::Checkbox, grid_view::GridView, menu::Menu, menu_item::MenuItem,
    status_bar_button::StatusBarButton, status_item::StatusItem, text_field::TextField,
    window::Window,
};

use objc2::{rc::Retained, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSButton, NSControlStateValueOff, NSControlStateValueOn, NSFont,
    NSGridCellPlacement, NSGridRowAlignment, NSLayoutConstraint, NSTextAlignment, NSView,
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
const STATUS_ICON: &[u8] = include_bytes!("../assets/cursoroutline_center.png");

use crate::{
    config::config,
    utils::{env_f64, new_nsrect},
};

/// One checkbox in the settings window.
struct Toggle {
    title: &'static str,
    get: fn() -> bool,
    set: fn(bool),
}

/// A labelled group of checkboxes, e.g. "General:".
struct Section {
    label: &'static str,
    toggles: &'static [Toggle],
}

/// Everything shown in the settings window. To add a setting, add a line here.
const SECTIONS: &[Section] = &[
    Section {
        label: "General:",
        toggles: &[
            Toggle {
                title: "Enable momentum",
                get: UI::momentum_is_enabled,
                set: UI::set_momentum_enabled,
            },
            Toggle {
                title: "High speed",
                get: UI::high_speed_is_enabled,
                set: UI::set_high_speed_enabled,
            },
            Toggle {
                title: "Launch on login",
                get: UI::logon_item_is_enabled,
                set: UI::set_logon_item_enabled,
            },
            Toggle {
                title: "Enable palm rejection (Experimental)",
                get: UI::palm_rejection_is_enabled,
                set: UI::set_palm_rejection_enabled,
            },
        ],
    },
    Section {
        label: "Fun:",
        toggles: &[Toggle {
            title: "Air hockey mode",
            get: UI::air_hockey_mode_is_enabled,
            set: UI::set_air_hockey_mode_enabled,
        }],
    },
];

pub struct UI {
    _window: Window,
    pub status_item: StatusItem,
    // Held so the checkbox action closures stay alive.
    _checkboxes: Vec<Checkbox>,
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

    fn set_logon_item_enabled(is_enabled: bool) {
        Self::update_logon_item_registration(is_enabled);
    }

    fn logon_item_is_enabled() -> bool {
        let app_service = unsafe { SMAppService::mainAppService() };
        let status = unsafe { app_service.status() };
        return matches!(
            status,
            SMAppServiceStatus::Enabled | SMAppServiceStatus::RequiresApproval
        );
    }

    /// Builds the label/checkbox grid from `SECTIONS`.
    fn build_settings_grid(mtm: MainThreadMarker) -> (GridView, Vec<Checkbox>) {
        let grid = GridView::new(mtm, NSRect::ZERO);
        grid.set_row_spacing(ROW_GAP);
        grid.set_column_spacing(LABEL_CONTROL_GAP);
        grid.set_row_alignment(NSGridRowAlignment::FirstBaseline);

        let label_font = NSFont::systemFontOfSize(13.0);
        let mut checkboxes = Vec::new();
        let mut section_start_rows = Vec::new();
        let mut row_count = 0;

        for section in SECTIONS {
            section_start_rows.push(row_count);
            for (i, toggle) in section.toggles.iter().enumerate() {
                let mut checkbox = Checkbox::init_with_title(mtm, toggle.title);
                let set = toggle.set;
                checkbox.set_action(mtm, move |sender: &NSButton| {
                    set(sender.state() == NSControlStateValueOn);
                });
                checkbox.set_state(control_state((toggle.get)()));

                // Only the first row of a section shows the section label.
                let label: Retained<NSView> = if i == 0 {
                    let label = TextField::label(mtm, section.label);
                    label.set_font(label_font.clone());
                    label.set_alignment(NSTextAlignment::Right);
                    label.text_field.into_super().into_super()
                } else {
                    GridView::empty_cell(mtm)
                };

                grid.add_row_with_views(&[&label, &checkbox.button]);
                checkboxes.push(checkbox);
                row_count += 1;
            }
        }

        let label_column = grid.column_at_index(0);
        label_column.setXPlacement(NSGridCellPlacement::Trailing);
        label_column.setWidth(LABEL_COLUMN_WIDTH);

        // Extra space above every section but the first.
        for &row in section_start_rows.iter().skip(1) {
            grid.row_at_index(row).setTopPadding(SECTION_GAP - ROW_GAP);
        }

        (grid, checkboxes)
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

        let (grid, checkboxes) = Self::build_settings_grid(mtm);
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
            grid_view
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
            _checkboxes: checkboxes,
        }
    }
}
