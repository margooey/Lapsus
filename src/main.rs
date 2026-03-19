pub mod config;
pub mod controller;
pub mod engine;
pub mod tests;
pub mod trackpad;
pub mod ui;
pub mod utils;

use std::cell::RefCell;

use objc2::{
    ClassType, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
    sel,
};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, NSTimer};

use crate::{config::init_config, controller::Controller, ui::UI, utils::env_f64};

// https://docs.rs/objc2/latest/objc2/topics/run_loop/index.html#graphical-applications
struct AppState {
    ui: RefCell<Option<UI>>,
    controller: RefCell<Option<Controller>>,
    timer: RefCell<Option<Retained<NSTimer>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppState]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(AppState {
                ui: RefCell::new(None),
                controller: RefCell::new(None),
                timer: RefCell::new(None),
            });
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) {
            utils::disable_local_event_suppression();

            if let Some(controller) = self.ivars().controller.borrow_mut().as_mut() {
                controller.update_state();
            }
        }
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn application_did_finish_launching(&self, _notification: &NSNotification)  {
            let _mtm = MainThreadMarker::new().expect("must be on the main thread");
            let ui = UI::initialize();
            let mut controller = Controller::new();
            controller.start();
            let tick_interval = env_f64!("MIN_DT");

            let timer = unsafe {
                NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                    tick_interval,
                    self,
                    sel!(tick:),
                    None,
                    true,
                )
            };

            *self.ivars().ui.borrow_mut() = Some(ui);
            *self.ivars().controller.borrow_mut() = Some(controller);
            *self.ivars().timer.borrow_mut() = Some(timer);
        }

        #[unsafe(method(applicationWillTerminate:))]
        fn application_will_terminate(&self, _notification: &NSNotification) {
            if let Some(timer) = self.ivars().timer.borrow_mut().take() {
                timer.invalidate();
            }

            if let Some(controller) = self.ivars().controller.borrow_mut().as_mut() {
                controller.stop();
            }
        }
    }
);

fn main() {
    env_logger::init();
    init_config();

    let mtm = MainThreadMarker::new().expect("must be on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let delegate: Retained<AppDelegate> = unsafe { msg_send![AppDelegate::class(), new] };
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    app.run();
}
