use std::f32::consts::PI;
use cpal::{Device, HostId};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use fast_log::Config;
use fast_log::filter::ModuleFilter;
use iced::{Alignment, Application, Command, Element, Error, executor, Length, Settings, Theme};
use iced::widget::{button, Column, pick_list};
use log::LevelFilter;
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};
use spectrum_analyzer::{FrequencyLimit, FrequencySpectrum, samples_fft_to_spectrum};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;

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
            .build_cartesian_2d(0..2000, 0..100)
            .expect("Failed to build chart");

        //let series = LineSeries::new((0..100).map(|x| (x, 100 - x)), &BLACK);
        let series = LineSeries::new(self.spectrum.iter().map(|(freq, amp)| (*freq as i32, *amp as i32)), &BLACK);

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

pub trait Engine {
    fn use_engine(&mut self, host_id: &str);
    fn get_current_engine(&self) -> Option<&str>;
    fn get_available_hosts(&self) -> Vec<String>;

    fn get_input_devices(&self) -> Vec<String>;
    fn get_current_input_device(&self) -> Option<String>;
    fn use_input_device(&mut self, device_name: String);

    fn start_recording(&mut self);
}

#[derive(Default)]
pub struct CpalEngine {
    current_host: Option<HostId>,
    current_input_device: Option<Device>
}

impl CpalEngine {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Engine for CpalEngine {
    fn use_engine(&mut self, host_id: &str) {
        for host in cpal::available_hosts() {
            if host.name().eq(host_id) {
                self.current_host = Some(host);
                log::info!("Switched to audio host {}", host_id);
            }
        }
    }

    fn get_current_engine(&self) -> Option<&str> {
        self.current_host.map(|h| h.name())
    }

    fn get_available_hosts(&self) -> Vec<String> {
        cpal::available_hosts()
            .iter()
            .map(|host_id| host_id.name())
            .map(|name| name.into())
            .collect()
    }

    fn get_input_devices(&self) -> Vec<String> {
        if let Some(host_id) = self.current_host {
            let host = cpal::host_from_id(host_id).expect("Could not open audio host");
            let devices = host.input_devices().expect("Could not find input devices on host");

            devices.into_iter()
                .map(|d| d.name().unwrap_or(String::from("No device name")))
                .collect()
        } else {
            vec![]
        }
    }

    fn get_current_input_device(&self) -> Option<String> {
        self.current_input_device.as_ref()
            .map(|input_device| input_device.name()
                .unwrap_or(String::from("No device name found")))
    }

    fn use_input_device(&mut self, device_name: String) {
        if let Some(host_id) = self.current_host {
            let host = cpal::host_from_id(host_id).expect("Could not open audio host");
            for input_device in host.input_devices().expect("Could not open input devices on host") {
                if input_device.name().map(|name| name.eq(device_name.as_str())).unwrap_or(false) {
                    self.current_input_device = Some(input_device);
                    log::info!("Using input device {}", device_name);
                }
            }
        }
    }

    fn start_recording(&mut self) {
        log::info!("Recording started using {}", self.get_current_input_device().unwrap_or(String::from("No input device name")));

        if let Some(device) = &self.current_input_device {
            if let Ok(configs) = device.supported_input_configs() {
                log::info!("Supported input configurations: ");
                for config in configs {
                    log::info!("  {:?}", config);
                }
            }

            if let Ok(config) = device.default_input_config() {
                log::info!("Default input config: {:?}", config);

                let err_fn = move |err| {
                    log::error!("An error occurred during reading from the stream: {:?}", err);
                };

                let mut raw_data: Vec<f32> = vec![];

                let stream_result = device.build_input_stream(&config.into(), move |_data: &[f32], _info| {
                    //log::info!("Read data: {:?}", data);
                }, err_fn, None);

                match stream_result {
                    Ok(stream) => {
                        if let Err(error) = stream.play() {
                            log::error!("Failed to play stream: {:?}", error);
                        } else {
                            log::info!("Playing");

                            std::thread::sleep(std::time::Duration::from_secs(3));
                            drop(stream);

                            log::info!("Playing completed, read {} values", &raw_data.len());
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to create stream: {:?}", err);
                    }
                }
            }
        }
    }
}

pub struct AudiaParams {
    engine: Box<dyn Engine>,
    frequency_spectrum: Vec<(f32, f32)>
}

pub struct Audia {
    params: AudiaParams,
    freq_anal: FreqAnalysis
}

impl Audia {
}

impl Application for Audia {
    type Executor = executor::Default;
    type Message = UIMessage;
    type Theme = Theme;
    type Flags = AudiaParams;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let spectrum = flags.frequency_spectrum.clone();

        (Self { params: flags, freq_anal: FreqAnalysis { spectrum } }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Audia")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            UIMessage::HostChanged(new_host) => {
                self.params.engine.use_engine(new_host.as_str());
            }
            UIMessage::InputDeviceChanged(device_name) => {
                self.params.engine.use_input_device(device_name);
            }
            UIMessage::RecordingStarted => {
                self.params.engine.start_recording();
            }
            _ => {
                log::info!("Unknown event: {:?}", message);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        Column::new()
            .push(
                pick_list(
                    self.params.engine.get_available_hosts(),
                    self.params.engine.get_current_engine().map(|e| e.into()),
                    UIMessage::HostChanged)
                    .placeholder("Choose an audio host"))
            .push(
                pick_list(
                    self.params.engine.get_input_devices(),
                    self.params.engine.get_current_input_device(),
                    UIMessage::InputDeviceChanged)
                    .placeholder("Choose an input device"))
            .push(button("Record").on_press(UIMessage::RecordingStarted))
            .push(self.freq_anal.view())
            //.push(FreqAnalysis { spectrum: self.params.frequency_spectrum.clone() }.view())
            .padding(20)
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
    }

}

fn main() -> Result<(), Error> {
    let mut samples = [0.0; 4096];

    let sample_rate: f32 = 44100.0;
    let frequency: f32 = 880.0;
    let increment: f32 = sample_rate / (frequency * 2.0 * PI);

    println!("Increment: {increment}");

    for i in 0..4096 {
        samples[i] = (increment * i as f32).sin();
    }

    let hann_window = hann_window(&samples);

    let spectrum_result = samples_fft_to_spectrum(&hann_window, 44100, FrequencyLimit::Max(2000.0), Some(&divide_by_N));

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

    let engine = CpalEngine::new();

    let flags = AudiaParams {
        engine: Box::new(engine),
        frequency_spectrum: frequency_spectrum.data().iter().map(|(freq, amp)| (freq.val(), amp.val())).collect()
    };

    let settings = Settings::with_flags(flags);

    Audia::run(settings)
}
