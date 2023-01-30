use std::f32::consts::PI;

use fast_log::Config;
use fast_log::filter::ModuleFilter;
use iced::{Application, Error, Settings};
use log::LevelFilter;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;

mod engine;
mod ui;

fn generate_sine_wave(frequencies: Vec<f32>, sample_rate: f32) -> [f32; 4096] {
    let mut samples = [0.0; 4096];

    let increments: Vec<f32> = frequencies.iter().map(|frequency| (frequency * 2.0 * PI) / sample_rate).collect();

    for (i, sample) in samples.iter_mut().enumerate() {
        for increment in &increments {
            *sample += (increment * i as f32).sin();
        }
    }

    samples
}

fn main() -> Result<(), Error> {
    // initialise logger
    let log_config = Config::new()
        .console()
        .level(LevelFilter::Info)
        .filter(ModuleFilter::new_include(vec![ String::from("audia") ]))
        .chan_len(Some(65536));

    fast_log::init(log_config).expect("Could not initialize logger");

    log::info!("Initializing Audia");

    // generate some wave functions
    let samples = generate_sine_wave(vec![ 440.0, 80.0, 2000.0 ], 44100.0);
    let hann_window = hann_window(&samples);
    let spectrum_result = samples_fft_to_spectrum(&hann_window, 44100, FrequencyLimit::Max(4000.0), Some(&divide_by_N));

    if let Err(error) = &spectrum_result {
        log::error!("Failed to get frequency spectrum: {error:?}");
    }

    let frequency_spectrum = spectrum_result.expect("Failed to get frequency spectrum");

    // initialise channels
    let (tx, rx) = std::sync::mpsc::channel();

    // initialise engine
    let engine = engine::CpalEngine::new(tx);

    // run UI
    let flags = ui::AudiaParams {
        engine: Box::new(engine),
        frequency_spectrum: frequency_spectrum.data().iter().map(|(freq, amp)| (freq.val(), amp.val() * 100.0)).collect(),
        rx
    };

    let settings = Settings::with_flags(flags);

    // Note: the UI must run on the main thread
    ui::Audia::run(settings)
}
