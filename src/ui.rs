use objc2::{MainThreadOnly, rc::Retained, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSMenu, NSMenuItem,
    NSStatusBar, NSStatusItem, NSVariableStatusItemLength, NSWindow, NSWindowController,
    NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};
pub struct UI {
    pub app: Retained<NSApplication>,
    _window_controller: Retained<NSWindowController>,
    pub status_item: Retained<NSStatusItem>,
}

impl UI {
    pub fn initialize() -> Self {
        // Generic AppKit setup
        let mtm = MainThreadMarker::new().expect("must be on the main thread");
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        // Window setup (settings)
        let window = NSWindow::alloc(mtm);
        let window_controller = NSWindowController::alloc(mtm);
        let _window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                window,
                NSRect::new(NSPoint::new(100.0, 100.0), NSSize::new(800.0, 600.0)),
                NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        let window_controller =
            NSWindowController::initWithWindow(window_controller, Some(&_window));
        let _window_controller = window_controller;

        // Status item setup
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);
        let button = status_item
            .button(mtm)
            .expect("status bar item should have a button");
        let title = NSString::from_str("⬤");
        button.setTitle(&title);
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
        unsafe { quit_item.setTarget(Some(&app)) };

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
        unsafe { settings_item.setTarget(Some(&_window_controller)) };

        // Initialize status item
        status_item.setMenu(Some(&menu));
        return Self {
            app,
            _window_controller,
            status_item,
        };
    }
}
