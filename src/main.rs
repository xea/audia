use std::f32::consts::PI;

use fast_log::Config;
use fast_log::filter::ModuleFilter;
use iced::{Application, Element, Error, Length, Settings};
use log::LevelFilter;
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;

mod engine;
mod ui;

struct FreqAnalysis {
   spectrum: Vec<(f32, f32)>
}

impl FreqAnalysis {
    fn view(&self) -> Element<UIMessage> {
        ChartWidget::new(self)
            .width(Length::Units(1000))
            .height(Length::Units(400))
            .into()
    }
}

impl Chart<UIMessage> for FreqAnalysis {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .set_all_label_area_size(40)
            .build_cartesian_2d(0..4000, 0..100)
            .expect("Failed to build chart");

        //let series = LineSeries::new((0..100).map(|x| (x, 100 - x)), &BLACK);
        let series = LineSeries::new(self.spectrum.iter().map(|(freq, amp)| (*freq as i32, *amp as i32)), &BLACK);

        println!("{:?}", self.spectrum);

        chart.configure_mesh().draw().expect("Failed to draw mesh");

        chart.draw_series(series)
                .expect("Failed to draw series");
    }
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum UIMessage {
    HostChanged(String),
    InputDeviceChanged(String),
    RecordingStarted,
    DummyMessage
}

fn main() -> Result<(), Error> {
    let mut samples = [0.0; 4096];


    let sample_rate: f32 = 44100.0;
    let frequency1: f32 = 440.0;
    let frequency2: f32 = 2000.0;
    let increment1: f32 = (frequency1 * 2.0 * PI) / sample_rate;
    let increment2: f32 = (frequency2 * 2.0 * PI) / sample_rate;

    println!("Increment: {increment1}");

    for i in 0..4096 {
        samples[i] = (increment1 * i as f32).sin() + (increment2 * i as f32).sin();
    }

    let hann_window = hann_window(&samples);

    let spectrum_result = samples_fft_to_spectrum(&hann_window, 44100, FrequencyLimit::Max(4000.0), Some(&divide_by_N));

    if let Err(error) = &spectrum_result {
        log::error!("Failed to get frequency spectrum: {error:?}");
    }

    let frequency_spectrum = spectrum_result.expect("Failed to get frequency spectrum");

    /*
            for (frequency, amplitude) in spectrum.data().iter() {
                println!("Frequency: {frequency} Hz - {amplitude}");
     */

    //

    let log_config = Config::new()
        .console()
        .level(LevelFilter::Info)
        .filter(ModuleFilter::new_include(vec![ String::from("audia") ]))
        .chan_len(Some(65536));

    fast_log::init(log_config).expect("Could not initialize logger");

    log::info!("Initializing Audia");

    let engine = engine::CpalEngine::new();

    let flags = ui::AudiaParams {
        engine: Box::new(engine),
        frequency_spectrum: frequency_spectrum.data().iter().map(|(freq, amp)| (freq.val(), amp.val() * 100.0)).collect()
    };

    let settings = Settings::with_flags(flags);

    ui::Audia::run(settings)
}
