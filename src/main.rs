pub mod controller;
pub mod engine;
pub mod tests;
pub mod trackpad;
pub mod ui;
pub mod utils;

use std::{cell::RefCell, env, sync::OnceLock};

use objc2::{
    ClassType, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
    sel,
};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, NSTimer};

use crate::{controller::Controller, ui::UI};

pub struct Config {
    maximum_momentum_speed: f64,
    trackpad_velocity_gain: f64,
    glide_decay_per_second: f64,
    minimum_glide_velocity: f64,
    glide_stop_speed_factor: f64,
    velocity_smoothing: f64,
    min_dt: f64,
    multi_finger_suppression_deadline: f64,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config {
        maximum_momentum_speed: env!("MAXIMUM_MOMENTUM_SPEED").parse::<f64>().unwrap(),
        trackpad_velocity_gain: env!("TRACKPAD_VELOCITY_GAIN").parse::<f64>().unwrap(),
        glide_decay_per_second: env!("GLIDE_DECAY_PER_SECOND").parse::<f64>().unwrap(),
        minimum_glide_velocity: env!("MINIMUM_GLIDE_VELOCITY").parse::<f64>().unwrap(),
        glide_stop_speed_factor: env!("GLIDE_STOP_SPEED_FACTOR").parse::<f64>().unwrap(),
        velocity_smoothing: env!("VELOCITY_SMOOTHING").parse::<f64>().unwrap(),
        min_dt: env!("MIN_DT").parse::<f64>().unwrap(),
        multi_finger_suppression_deadline: env!("MULTI_FINGER_SUPPRESSION_DEADLINE")
            .parse::<f64>()
            .unwrap(),
    })
}

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
        fn application_did_finish_launching(&self, _notification: &NSNotification) {
            let _mtm = MainThreadMarker::new().expect("must be on the main thread");
            let ui = UI::initialize();
            let mut controller = Controller::new();
            controller.start();

            let timer = unsafe {
                NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                    config().min_dt,
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
    let mtm = MainThreadMarker::new().expect("must be on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let delegate: Retained<AppDelegate> = unsafe { msg_send![AppDelegate::class(), new] };
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    app.run();
}
