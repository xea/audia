use std::str::FromStr;

use fast_log::Config;
use fast_log::consts::LogSize;
use fast_log::filter::ModuleFilter;
use iced::{Application, Error, Settings};
use log::LevelFilter;
use ringbuf::HeapRb;
use crate::engine::{AudioSettings, AudioSystem};
use crate::ui::UIParams;

mod engine;
mod ui;

pub const APP_NAME: &str = "audia";

struct AppConfig {}

/// Provide configuration for the global logger, such as log levels, log file name, etc.
struct LogConfig<'a> {
    log_file_name: &'a str,
    max_log_size_mb: usize,
    log_level: &'a str,
}

impl<'a> Default for LogConfig<'a> {
    fn default() -> Self {
        Self {
            // It's important to define the log file name as a path
            log_file_name: "./audia.log",
            max_log_size_mb: 1,
            log_level: "info"
        }
    }
}

fn init_logger(config: &LogConfig) {
    let log_config = Config::new()
        .file_loop(config.log_file_name, LogSize::MB(config.max_log_size_mb))
        .console()
        .level(LevelFilter::from_str(config.log_level).unwrap_or(LevelFilter::Info))
        .filter(ModuleFilter::new_include(vec![ String::from(APP_NAME) ]))
        // Providing a channel length will make the log queue bounded
        .chan_len(Some(65536));

    fast_log::init(log_config).expect("Could not initialize logger");
}

fn main() -> Result<(), Error> {
    // initialise logger
    init_logger(&LogConfig::default());

    log::info!("Initializing application");

    let audio_system = AudioSystem::new(AudioSettings::default());

    let ui_params = UIParams::new(audio_system);

    // Note: the UI must run on the main thread
    ui::Audia::run(Settings::with_flags(ui_params))
}
