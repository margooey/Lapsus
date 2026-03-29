use crate::{config::config, utils::max};
use cidre::cg::{Float, Point, Rect, Vector};
use core_graphics::display;
use objc2_app_kit::NSScreen;

pub const ZERO_VECTOR: Vector = Vector { dx: 0.0, dy: 0.0 };

pub struct State {
    position: Point,
    previous_position: Point,
    last_input_delta: Vector,
    velocity: Vector,
    pub is_gliding: bool,
}

pub struct Engine {
    pub state: State,
    last_physical_mouse_position: Point,
    desktop_bounds: Rect,
}

impl Engine {
    pub fn init() -> Self {
        Engine {
            state: State {
                position: Point { x: 0.0, y: 0.0 },
                previous_position: Point { x: 0.0, y: 0.0 },
                last_input_delta: Vector { dx: 0.0, dy: 0.0 },
                velocity: Vector { dx: 0.0, dy: 0.0 },
                is_gliding: false,
            },
            last_physical_mouse_position: Point { x: 0.0, y: 0.0 },
            desktop_bounds: Rect::null(),
        }
    }

    pub fn set_gliding(&mut self, value: bool) {
        if self.state.is_gliding != value {
            log::debug!("glide {}", if value { "start" } else { "stop" });
        }
        self.state.is_gliding = value;
    }

    pub fn begin_touch(&mut self, position: Point) {
        self.state.position = position;
        self.state.previous_position = position;
        self.state.last_input_delta = ZERO_VECTOR;
        self.state.velocity = ZERO_VECTOR;
        self.set_gliding(false);
    }

    pub fn handle_touch(
        &mut self,
        physical_position: Point,
        delta_time: Float,
        normalized_trackpad_velocity: Option<Vector>,
    ) {
        let delta_pos = Point {
            x: physical_position.x - self.last_physical_mouse_position.x,
            y: physical_position.y - self.last_physical_mouse_position.y,
        };
        self.last_physical_mouse_position = physical_position;
        self.state.previous_position = self.state.position;

        let pointer_velocity = Vector {
            dx: delta_pos.x / delta_time,
            dy: delta_pos.y / delta_time,
        };

        let mut velocity = pointer_velocity;
        let trackpad_velocity = self.trackpad_velocity_in_pixels(normalized_trackpad_velocity);
        if let Some(trackpad_velocity) = trackpad_velocity
            && Self::magnitude(&trackpad_velocity) > Self::magnitude(&pointer_velocity) {
                velocity = trackpad_velocity;
            }
        self.state.velocity = velocity;
        self.state.position.x += delta_pos.x;
        self.state.position.y += delta_pos.y;
        self.state.last_input_delta = Vector {
            dx: delta_pos.x,
            dy: delta_pos.y,
        };

        if self.state.is_gliding {
            self.set_gliding(false);
        }
    }

    pub fn handle_no_touch(
        &mut self,
        physical_position: Point,
        delta_time: Float,
        suppress_glide: bool,
        touch_ended_recently: bool,
    ) {
        self.last_physical_mouse_position = physical_position;
        if touch_ended_recently {
            if suppress_glide {
                self.set_gliding(false);
            } else {
                self.begin_glide_if_needed();
            }
        }

        if self.state.is_gliding {
            self.apply_momentum(delta_time);
        } else {
            self.state.last_input_delta = ZERO_VECTOR;
        }
    }

    fn begin_glide_if_needed(&mut self) {
        let speed = Self::magnitude(&self.state.velocity);
        let min_speed = config().minimum_glide_velocity;
        if speed < min_speed {
            log::debug!(
                "glide suppressed: speed {:.3} < min {:.3}",
                speed,
                min_speed
            );
            self.set_gliding(false);
            self.state.velocity = ZERO_VECTOR;
        } else {
            log::debug!("glide start: speed {:.3} >= min {:.3}", speed, min_speed);
            self.set_gliding(true);
            self.update_cursor_position_on_screen();
        }
    }

    pub fn apply_momentum(&mut self, delta_time: Float) {
        let glide_decay_per_second = config().glide_decay_per_second;
        let decay_factor = max(0.0, 1.0 - glide_decay_per_second * delta_time);
        self.state.velocity.dx *= decay_factor;
        self.state.velocity.dy *= decay_factor;

        let momentum_delta = Vector {
            dx: self.state.velocity.dx * delta_time,
            dy: self.state.velocity.dy * delta_time,
        };

        self.state.previous_position = self.state.position;
        self.state.position.x += momentum_delta.dx;
        self.state.position.y += momentum_delta.dy;
        self.state.last_input_delta = momentum_delta;

        self.update_cursor_position_on_screen();

        let speed = Self::magnitude(&self.state.velocity);
        let config = config();
        if speed < config.minimum_glide_velocity {
            self.set_gliding(false);
            self.state.velocity = ZERO_VECTOR;
            self.update_cursor_position_on_screen();
        }
    }

    fn magnitude(vector: &Vector) -> Float {
        (vector.dx * vector.dx + vector.dy * vector.dy).sqrt()
    }

    pub fn update_desktop_bounds(&mut self, bounds: Rect) {
        self.desktop_bounds = bounds;
        log::debug!(
            "desktop bounds: origin ({:.1},{:.1}) size ({:.1},{:.1})",
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height
        );
    }

    // Advance the cursor position based on the current momentum
    pub fn update_cursor_position_on_screen(&mut self) {
        let target = self.state.position;
        let mtm = objc2_foundation::MainThreadMarker::new().expect("must be on the main thread");
        if let Some(screen) = NSScreen::mainScreen(mtm) {
            let local_x = target.x - screen.frame().min().x;
            let local_y_from_bottom = target.y - screen.frame().min().y;
            let local_y = screen.frame().size.height - local_y_from_bottom;
            let _error = display::CGDisplay::move_cursor_to_point(
                &display::CGDisplay::main(),
                display::CGPoint {
                    x: local_x,
                    y: local_y,
                },
            );
        } else {
        }
    }

    pub fn update_engine_state(&mut self, physical_position: Point) {
        self.state.position = physical_position;
        self.state.previous_position = physical_position;
        self.state.last_input_delta = ZERO_VECTOR;
        self.last_physical_mouse_position = physical_position;
    }

    fn trackpad_velocity_in_pixels(
        &mut self,
        normalized_velocity: Option<Vector>,
    ) -> Option<Vector> {
        if self.desktop_bounds == Rect::null() {
            return None;
        }
        if let Some(normalized_velocity) = normalized_velocity {
            let config = config();
            let scaled = Vector {
                dx: normalized_velocity.dx
                    * self.desktop_bounds.size.width
                    * config.trackpad_velocity_gain,
                dy: normalized_velocity.dy
                    * self.desktop_bounds.size.height
                    * config.trackpad_velocity_gain,
            };
            Some(Self::clamped_velocity(
                &scaled,
                config.default_maximum_magnitude,
            ))
        } else {
            None
        }
    }

    fn clamped_velocity(vector: &Vector, maximum_magnitude: Float) -> Vector {
        let magnitude = Self::magnitude(vector);
        if magnitude > maximum_magnitude && magnitude > 0.0 {
            let scale = maximum_magnitude / magnitude;
            Vector {
                dx: vector.dx * scale,
                dy: vector.dy * scale,
            }
        } else {
            *vector
        }
    }
}
