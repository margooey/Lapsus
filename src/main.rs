pub mod controller;
pub mod engine;
pub mod tests;
pub mod trackpad;
pub mod ui;
pub mod utils;

use objc2::rc::autoreleasepool;
use objc2_app_kit::NSEventMask;
use objc2_foundation::{NSDate, NSDefaultRunLoopMode};
use std::{env, sync::OnceLock};

use crate::ui::UI;

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
fn main() {
    // Disabling logging for now until MacOS bundle support is complete
    /*
    use std::fs::File;
    use std::io::Write;

    let target = Box::new(File::create("lapsus_log.txt").expect("Can't create file"));

    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(target))
        .filter(None, LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();*/

    // UI setup
    let ui = UI::initialize();

    // Controller setup
    let mut controller = controller::Controller::new();
    controller.start();

    loop {
        // Update the cursor state and drain the autoreleasepool on every tick (default MIN_DT = 200Hz)
        autoreleasepool(|_pool| {
            let _ = &ui.status_item;
            let expiration = NSDate::dateWithTimeIntervalSinceNow(config().min_dt);
            if let Some(event) = ui.app.app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(&expiration),
                unsafe { NSDefaultRunLoopMode },
                true,
            ) {
                ui.app.app.sendEvent(&event);
            }
            ui.app.app.updateWindows();
            utils::disable_local_event_suppression();
            controller.update_state();
        });
    }
}
