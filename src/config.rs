use std::sync::{Mutex, MutexGuard, OnceLock};

const ERR_POISON: &str = "config lock poisoned";

pub struct Config {
    pub maximum_momentum_speed: f64,
    pub trackpad_velocity_gain: f64,
    pub glide_decay_per_second: f64,
    pub minimum_glide_velocity: f64,
    pub glide_stop_speed_factor: f64,
    pub velocity_smoothing: f64,
    pub min_dt: f64,
    pub multi_finger_suppression_deadline: f64,
}

impl Config {
    pub fn init() -> Self {
        Self {
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
        }
    }
}

static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();

pub fn init_config() {
    let _ = CONFIG.get_or_init(|| Mutex::new(Config::init()));
}

pub fn config() -> MutexGuard<'static, Config> {
    CONFIG
        .get()
        .expect("config must be initialized before use")
        .lock()
        .expect(ERR_POISON)
}

pub fn config_mutex() -> &'static Mutex<Config> {
    CONFIG.get().expect("config must be initialized before use")
}
