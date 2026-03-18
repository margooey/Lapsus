use crate::config::config;
use crate::key_monitor;
use crate::utils::{max, union_rect};
use crate::{engine, trackpad};
use cidre::cg::{Float, Point, Rect, Size, Vector};

pub struct Controller {
    pub engine: engine::Engine,
    monitor: trackpad::TrackpadMonitor,
    is_running: bool,
    last_update_timestamp: f64,
    touch_ended_recently: bool,
    pub is_touching: bool,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            engine: engine::Engine::new(),
            monitor: trackpad::TrackpadMonitor::new(),
            is_running: false,
            last_update_timestamp: 0.0,
            touch_ended_recently: false,
            is_touching: false,
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.is_running = true;
            log::debug!("controller start");
            self.monitor.start();
            self.update_desktop_bounds();
            self.update_engine_state();
            self.is_touching = self.monitor.is_touching();
        }
    }

    pub fn begin_touch(&mut self, position: Point) {
        self.is_touching = true;
        self.touch_ended_recently = false;
        self.engine.begin_touch(position);
    }

    pub fn handle_touch(
        &mut self,
        position: Point,
        delta_time: Float,
        normalized_velocity: Vector,
    ) {
        self.engine
            .handle_touch(position, delta_time, Some(normalized_velocity));
    }

    pub fn end_touch(&mut self) {
        self.is_touching = false;
        self.touch_ended_recently = true;
    }

    pub fn handle_no_touch(&mut self, position: Point, delta_time: Float, suppress_glide: bool) {
        self.engine.handle_no_touch(
            position,
            delta_time,
            suppress_glide,
            self.touch_ended_recently,
        );
        self.touch_ended_recently = false;
    }

    pub fn stop(&mut self) {
        if self.is_running {
            self.is_running = false;
            self.monitor.stop();
        }
    }

    pub fn update_state(&mut self) {
        let now = objc2_core_foundation::CFAbsoluteTimeGetCurrent();
        let min_dt = config().min_dt;
        let delta_seconds = max(now - self.last_update_timestamp, min_dt);
        self.last_update_timestamp = now;
        let delta_time = delta_seconds;
        let ns_mouse_location = objc2_app_kit::NSEvent::mouseLocation();
        let physical_position = Point {
            x: ns_mouse_location.x,
            y: ns_mouse_location.y,
        };
        let is_touching = self.monitor.is_touching();

        if is_touching {
            if !self.touch_ended_recently {
                log::debug!("touch begin detected");
                self.engine.begin_touch(physical_position);
            }
            self.engine.handle_touch(
                physical_position,
                delta_time,
                self.monitor.current_normalized_velocity(),
            );
        } else {
            if self.touch_ended_recently {
                log::debug!("touch end detected");
            }
            let mut suppress_glide = self.monitor.should_suppress_glide();

            // When momentum_requires_key is enabled, suppress glide unless the
            // configured activation key is currently held down.
            let activation_key_missing = {
                let cfg = config();
                cfg.momentum_requires_key && match cfg.momentum_activation_key {
                    Some(keycode) => !key_monitor::is_key_held(keycode),
                    None => true,
                }
            };
            if activation_key_missing {
                suppress_glide = true;
                // Also cancel any in-progress glide
                if self.engine.state.is_gliding {
                    self.engine.set_gliding(false);
                }
            }

            self.engine.handle_no_touch(
                physical_position,
                delta_time,
                suppress_glide,
                self.touch_ended_recently,
            );
        }
        self.touch_ended_recently = is_touching;
    }

    fn update_engine_state(&mut self) {
        let current_position = objc2_app_kit::NSEvent::mouseLocation();
        let physical_position = Point {
            x: current_position.x,
            y: current_position.y,
        };
        self.engine.update_engine_state(physical_position);
        self.last_update_timestamp = objc2_core_foundation::CFAbsoluteTimeGetCurrent();
    }

    fn update_desktop_bounds(&mut self) {
        let mut bounds = Rect::null();
        let mtm = objc2_foundation::MainThreadMarker::new().expect("must be on the main thread");
        let screens = objc2_app_kit::NSScreen::screens(mtm);
        for screen in screens {
            let frame = screen.frame();
            let rect = Rect {
                origin: Point {
                    x: frame.origin.x,
                    y: frame.origin.y,
                },
                size: Size {
                    width: frame.size.width,
                    height: frame.size.height,
                },
            };
            bounds = union_rect(&bounds, &rect);
        }
        if bounds == Rect::null() {
            let main_frame = objc2_app_kit::NSScreen::mainScreen(mtm);
            if let Some(screen) = main_frame {
                let frame = screen.frame();
                bounds = Rect {
                    origin: Point {
                        x: frame.origin.x,
                        y: frame.origin.y,
                    },
                    size: Size {
                        width: frame.size.width,
                        height: frame.size.height,
                    },
                }
            }
        }
        self.engine.update_desktop_bounds(bounds);
    }
}
