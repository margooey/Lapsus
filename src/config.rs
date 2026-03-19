use std::{
    env, fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard, OnceLock},
};

use serde::{Deserialize, Serialize};

use crate::utils::env_f64;

const ERR_POISON: &str = "config lock poisoned";
const APP_SUPPORT_PATH: &str = "Library/Application Support/Lapsus";
const SETTINGS_FILE_NAME: &str = "settings.toml";

fn default_maximum_momentum_speed() -> f64 { env_f64!("MAXIMUM_MOMENTUM_SPEED") }
fn default_trackpad_velocity_gain() -> f64 { env_f64!("TRACKPAD_VELOCITY_GAIN") }
fn default_glide_decay_per_second() -> f64 { env_f64!("GLIDE_DECAY_PER_SECOND") }
fn default_minimum_glide_velocity() -> f64 { env_f64!("MINIMUM_GLIDE_VELOCITY") }
fn default_glide_stop_speed_factor() -> f64 { env_f64!("GLIDE_STOP_SPEED_FACTOR") }
fn default_velocity_smoothing() -> f64 { env_f64!("VELOCITY_SMOOTHING") }
fn default_min_dt() -> f64 { env_f64!("MIN_DT") }
fn default_multi_finger_suppression_deadline() -> f64 { env_f64!("MULTI_FINGER_SUPPRESSION_DEADLINE") }
fn default_logon_item_enabled() -> bool { true }

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_maximum_momentum_speed")]
    pub maximum_momentum_speed: f64,
    #[serde(default = "default_trackpad_velocity_gain")]
    pub trackpad_velocity_gain: f64,
    #[serde(default = "default_glide_decay_per_second")]
    pub glide_decay_per_second: f64,
    #[serde(default = "default_minimum_glide_velocity")]
    pub minimum_glide_velocity: f64,
    #[serde(default = "default_glide_stop_speed_factor")]
    pub glide_stop_speed_factor: f64,
    #[serde(default = "default_velocity_smoothing")]
    pub velocity_smoothing: f64,
    #[serde(default = "default_min_dt")]
    pub min_dt: f64,
    #[serde(default = "default_multi_finger_suppression_deadline")]
    pub multi_finger_suppression_deadline: f64,
    #[serde(default = "default_logon_item_enabled")]
    pub logon_item_enabled: bool,
}

impl Config {
    pub fn init() -> Self {
        let Some(settings_path) = settings_file_path() else {
            return Self::defaults();
        };
        let Ok(contents) = fs::read_to_string(&settings_path) else {
            return Self::defaults();
        };

        toml::from_str(&contents).unwrap_or_else(|err| {
            log::warn!("failed to parse settings: {err}");
            Self::defaults()
        })
    }

    fn defaults() -> Self {
        Self {
            maximum_momentum_speed: default_maximum_momentum_speed(),
            trackpad_velocity_gain: default_trackpad_velocity_gain(),
            glide_decay_per_second: default_glide_decay_per_second(),
            minimum_glide_velocity: default_minimum_glide_velocity(),
            glide_stop_speed_factor: default_glide_stop_speed_factor(),
            velocity_smoothing: default_velocity_smoothing(),
            min_dt: default_min_dt(),
            multi_finger_suppression_deadline: default_multi_finger_suppression_deadline(),
            logon_item_enabled: default_logon_item_enabled(),
        }
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
        match toml::to_string(&*config) {
            Ok(text) => text,
            Err(err) => {
                log::warn!("failed to serialize settings: {err}");
                return;
            }
        }
    };

    if let Err(error) = fs::write(&settings_path, settings_text) {
        log::warn!(
            "failed to persist settings: unable to write {:?}: {}",
            settings_path,
            error
        );
    }
}
