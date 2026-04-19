// warning: a lot of this is llm code, but some attempted optimizations have been made by me to make it a little better

use crate::{config::config, engine::ZERO_VECTOR};
use cidre::cg::{Float, Point, Vector};
use macos_multitouch::{self, MultitouchDevice};
use std::collections::HashSet;
use std::mem;
use std::sync::{Arc, Mutex};

pub const ZERO_POINT: Point = Point { x: 0.0, y: 0.0 };
const ERR_POISON: &str = "trackpad state lock poisoned";

pub struct TouchMetrics {
    pub centroid: Option<Point>,
    pub normalized_velocity: Vector,
    pub is_touching: bool,
}

struct TrackpadState {
    is_touching: bool,
    latest_positions: Vec<Point>,
    latest_centroid: Option<Point>,
    previous_centroid: Option<Point>,
    last_sample_timestamp: f64,
    normalized_velocity: Vector,
    suppress_glide_deadline: f64,
    previous_real_ids: HashSet<i32>,
    tainted_palm_ids: HashSet<i32>,
}

pub struct TrackpadMonitor {
    devices: Vec<MultitouchDevice>,
    state: Arc<Mutex<TrackpadState>>,
    listener_started: bool,
}

impl TrackpadMonitor {
    pub fn init() -> Self {
        Self {
            devices: Vec::new(),
            state: Arc::new(Mutex::new(TrackpadState {
                is_touching: false,
                latest_positions: Vec::new(),
                latest_centroid: None,
                previous_centroid: None,
                last_sample_timestamp: 0.0,
                normalized_velocity: Vector { dx: 0.0, dy: 0.0 },
                suppress_glide_deadline: 0.0,
                previous_real_ids: HashSet::new(),
                tainted_palm_ids: HashSet::new(),
            })),
            listener_started: false,
        }
    }

    pub fn start(&mut self) {
        if self.listener_started {
            return;
        }
        self.listener_started = true;

        let state = self.state.clone();
        let mut devices = macos_multitouch::get_multitouch_devices();
        log::debug!("trackpad devices: {}", devices.len());
        if devices.is_empty() {
            log::warn!("no multitouch devices detected");
        }

        // Iterate over each multitouch device's data
        for device in devices.iter_mut() {
            let state = state.clone();
            let _ = device.register_contact_frame_callback(
                move |_device, data: &[macos_multitouch::Finger], timestamp, _frame| {
                    let mut state = state.lock().expect("trackpad state lock poisoned");

                    let palm_rejection_enabled = config().palm_rejection_enabled;

                    if palm_rejection_enabled {
                        // Mark any finger as tainted if it was previously a "real finger" and is now registered by macOS as a palm
                        // Ignore the first frame, since fingers can briefly appear as palm_rejection == 0
                        for touch in data.iter() {
                            if touch.palm_rejection == 0
                                && state.previous_real_ids.contains(&touch.identifier)
                                && state.tainted_palm_ids.insert(touch.identifier)
                            {
                                log::debug!(
                                    "finger {} tainted as palm (was real finger)",
                                    touch.identifier
                                );
                            }
                        }
                        // Remove identifiers no longer present
                        let current_ids: HashSet<i32> = data.iter().map(|f| f.identifier).collect();
                        state.tainted_palm_ids.retain(|id| current_ids.contains(id));
                        state.previous_real_ids = data
                            .iter()
                            .filter(|f| {
                                f.palm_rejection != 0 && !state.tainted_palm_ids.contains(&f.identifier)
                            })
                            .map(|f| f.identifier)
                            .collect();

                        log::debug!(
                            "frame: {} contacts, {} real, {} tainted — {:?}",
                            data.len(),
                            data.iter()
                                .filter(|f| f.palm_rejection != 0
                                    && !state.tainted_palm_ids.contains(&f.identifier))
                                .count(),
                            state.tainted_palm_ids.len(),
                            data.iter()
                                .map(|f| (f.identifier, f.palm_rejection))
                                .collect::<Vec<_>>()
                        );
                    }

                    // Reuse the existing positions buffer
                    let mut positions = mem::take(&mut state.latest_positions);
                    positions.clear();
                    positions.reserve(data.len());

                    // Get the position of each finger and update the touch metrics
                    for finger in data.iter() {
                        // Filter out palms and tainted identifiers when palm rejection is enabled
                        if palm_rejection_enabled
                            && (finger.palm_rejection == 0
                                || state.tainted_palm_ids.contains(&finger.identifier))
                        {
                            continue;
                        }
                        positions.push(Point {
                            x: finger.normalized.pos.x as Float,
                            y: finger.normalized.pos.y as Float,
                        });
                    }

                    update_touch_metrics(&mut state, &positions, timestamp);
                    state.latest_positions = positions;
                },
            );
        }

        self.devices = devices;
    }

    pub fn stop(&mut self) {
        for device in self.devices.iter_mut() {
            device.stop();
        }
        self.devices.clear();
        self.listener_started = false;
    }

    pub fn is_touching(&self) -> bool {
        self.state.lock().expect(ERR_POISON).is_touching
    }

    pub fn current_touch_positions(&self) -> Vec<Point> {
        self.state
            .lock()
            .expect(ERR_POISON)
            .latest_positions
            .clone()
    }

    pub fn current_touch_centroid(&self) -> Option<Point> {
        self.state.lock().expect(ERR_POISON).latest_centroid
    }

    pub fn current_normalized_velocity(&self) -> Option<Vector> {
        let state = self.state.lock().expect(ERR_POISON);
        if state.is_touching {
            Some(state.normalized_velocity)
        } else {
            None
        }
    }

    pub fn metrics(&self) -> TouchMetrics {
        let state = self.state.lock().expect(ERR_POISON);
        TouchMetrics {
            centroid: state.latest_centroid,
            normalized_velocity: state.normalized_velocity,
            is_touching: state.is_touching,
        }
    }

    // Flag to prevent erroneous gliding when there is more than one finger touching the trackpad
    pub fn should_suppress_glide(&self) -> bool {
        let deadline = self.state.lock().expect(ERR_POISON).suppress_glide_deadline;
        objc2_core_foundation::CFAbsoluteTimeGetCurrent() < deadline
    }
}

fn update_touch_metrics(state: &mut TrackpadState, positions: &[Point], timestamp: f64) {
    let config = config();
    // if there is more than one real finger, suppress gliding for a short duration since it's likely some gesture
    if positions.len() > 1 {
        let now = objc2_core_foundation::CFAbsoluteTimeGetCurrent();
        state.suppress_glide_deadline = now + config.multi_touch_suppression_deadline;
    }
    let was_touching = state.is_touching;
    state.is_touching = !positions.is_empty();
    if state.is_touching != was_touching {
        log::debug!("touch {}", if state.is_touching { "start" } else { "end" });
    }

    if positions.is_empty() {
        state.latest_centroid = None;
        state.previous_centroid = None;
        state.normalized_velocity = ZERO_VECTOR;
        state.last_sample_timestamp = timestamp;
        return;
    }

    // Find the average position of all the current touch points
    let mut centroid = ZERO_POINT;
    for point in positions {
        centroid.x += point.x;
        centroid.y += point.y;
    }
    let divisor = positions.len() as Float;
    centroid.x /= divisor;
    centroid.y /= divisor;
    state.latest_centroid = Some(centroid);

    // Determine the velocity given the previous average if it exists
    if let Some(previous) = state.previous_centroid {
        if state.last_sample_timestamp > 0.0 {
            let mut delta_time = (timestamp - state.last_sample_timestamp) as Float;
            if delta_time < config.min_dt {
                delta_time = config.min_dt;
            }
            let raw_velocity = Vector {
                dx: (centroid.x - previous.x) / delta_time,
                dy: (centroid.y - previous.y) / delta_time,
            };
            // Apply velocity smoothing
            state.normalized_velocity = Vector {
                dx: state.normalized_velocity.dx * (1.0 - config.velocity_smoothing)
                    + raw_velocity.dx * config.velocity_smoothing,
                dy: state.normalized_velocity.dy * (1.0 - config.velocity_smoothing)
                    + raw_velocity.dy * config.velocity_smoothing,
            };
        } else {
            state.normalized_velocity = ZERO_VECTOR;
        }
    } else {
        state.normalized_velocity = ZERO_VECTOR;
    }

    state.previous_centroid = Some(centroid);
    state.last_sample_timestamp = timestamp;
}
