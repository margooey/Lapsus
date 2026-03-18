/*
    Some LLM assistance used here for the persisted settings stuff.
*/

use std::{
    env, fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard, OnceLock},
};

use crate::utils::env_f64;

const ERR_POISON: &str = "config lock poisoned";
const APP_SUPPORT_PATH: &str = "Library/Application Support/Lapsus";
const SETTINGS_FILE_NAME: &str = "settings.conf";

pub struct Config {
    pub maximum_momentum_speed: f64,
    pub trackpad_velocity_gain: f64,
    pub glide_decay_per_second: f64,
    pub minimum_glide_velocity: f64,
    pub glide_stop_speed_factor: f64,
    pub velocity_smoothing: f64,
    pub min_dt: f64,
    pub multi_finger_suppression_deadline: f64,
    pub logon_item_enabled: bool,
    pub momentum_requires_key: bool,
    pub momentum_activation_key: Option<u16>,
    pub momentum_activation_modifiers: u64,
}

impl Config {
    pub fn init() -> Self {
        let mut config = Self {
            maximum_momentum_speed: env_f64!("MAXIMUM_MOMENTUM_SPEED"),
            trackpad_velocity_gain: env_f64!("TRACKPAD_VELOCITY_GAIN"),
            glide_decay_per_second: env_f64!("GLIDE_DECAY_PER_SECOND"),
            minimum_glide_velocity: env_f64!("MINIMUM_GLIDE_VELOCITY"),
            glide_stop_speed_factor: env_f64!("GLIDE_STOP_SPEED_FACTOR"),
            velocity_smoothing: env_f64!("VELOCITY_SMOOTHING"),
            min_dt: env_f64!("MIN_DT"),
            multi_finger_suppression_deadline: env_f64!("MULTI_FINGER_SUPPRESSION_DEADLINE"),
            logon_item_enabled: true,
            momentum_requires_key: false,
            momentum_activation_key: None,
            momentum_activation_modifiers: 0,
        };
        config.load_persisted_settings();
        config
    }

    fn load_persisted_settings(&mut self) {
        let Some(settings_path) = settings_file_path() else {
            return;
        };
        let Ok(contents) = fs::read_to_string(&settings_path) else {
            return;
        };

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();

            match key {
                "maximum_momentum_speed" => parse_f64(value, &mut self.maximum_momentum_speed),
                "trackpad_velocity_gain" => parse_f64(value, &mut self.trackpad_velocity_gain),
                "glide_decay_per_second" => parse_f64(value, &mut self.glide_decay_per_second),
                "minimum_glide_velocity" => parse_f64(value, &mut self.minimum_glide_velocity),
                "glide_stop_speed_factor" => parse_f64(value, &mut self.glide_stop_speed_factor),
                "velocity_smoothing" => parse_f64(value, &mut self.velocity_smoothing),
                "min_dt" => parse_f64(value, &mut self.min_dt),
                "multi_finger_suppression_deadline" => {
                    parse_f64(value, &mut self.multi_finger_suppression_deadline)
                }
                "logon_item_enabled" => parse_bool(value, &mut self.logon_item_enabled),
                "momentum_requires_key" => parse_bool(value, &mut self.momentum_requires_key),
                "momentum_activation_key" => {
                    if value == "none" {
                        self.momentum_activation_key = None;
                    } else {
                        parse_u16(value, &mut self.momentum_activation_key);
                    }
                }
                "momentum_activation_modifiers" => {
                    parse_u64(value, &mut self.momentum_activation_modifiers)
                }
                _ => {}
            }
        }
    }

    fn as_persisted_text(&self) -> String {
        format!(
            "\
maximum_momentum_speed={}\n\
trackpad_velocity_gain={}\n\
glide_decay_per_second={}\n\
minimum_glide_velocity={}\n\
glide_stop_speed_factor={}\n\
velocity_smoothing={}\n\
min_dt={}\n\
multi_finger_suppression_deadline={}\n\
logon_item_enabled={}\n\
momentum_requires_key={}\n\
momentum_activation_key={}\n\
momentum_activation_modifiers={}\n",
            self.maximum_momentum_speed,
            self.trackpad_velocity_gain,
            self.glide_decay_per_second,
            self.minimum_glide_velocity,
            self.glide_stop_speed_factor,
            self.velocity_smoothing,
            self.min_dt,
            self.multi_finger_suppression_deadline,
            self.logon_item_enabled,
            self.momentum_requires_key,
            match self.momentum_activation_key {
                Some(key) => key.to_string(),
                None => "none".to_string(),
            },
            self.momentum_activation_modifiers,
        )
    }
}

fn parse_f64(value: &str, target: &mut f64) {
    if let Ok(parsed) = value.parse::<f64>() {
        *target = parsed;
    }
}

fn parse_bool(value: &str, target: &mut bool) {
    if let Ok(parsed) = value.parse::<bool>() {
        *target = parsed;
    }
}

fn parse_u16(value: &str, target: &mut Option<u16>) {
    if let Ok(parsed) = value.parse::<u16>() {
        *target = Some(parsed);
    }
}

fn parse_u64(value: &str, target: &mut u64) {
    if let Ok(parsed) = value.parse::<u64>() {
        *target = parsed;
    }
}

fn settings_file_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(APP_SUPPORT_PATH)
            .join(SETTINGS_FILE_NAME),
    )
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

pub fn persist_config() {
    let Some(settings_path) = settings_file_path() else {
        log::warn!("failed to persist settings: $HOME is not set");
        return;
    };

    let Some(parent) = settings_path.parent() else {
        log::warn!("failed to persist settings: invalid settings path");
        return;
    };

    if let Err(error) = fs::create_dir_all(parent) {
        log::warn!(
            "failed to persist settings: unable to create {:?}: {}",
            parent,
            error
        );
        return;
    }

    let settings_text = {
        let config = config();
        config.as_persisted_text()
    };

    if let Err(error) = fs::write(&settings_path, settings_text) {
        log::warn!(
            "failed to persist settings: unable to write {:?}: {}",
            settings_path,
            error
        );
    }
}
