use std::time::Duration;
use crossbeam_channel::Receiver;
use iced::{Alignment, Application, Command, Element, executor, Length, Subscription, Theme};
use iced::time as iced_time;
use iced::widget::{button, Column, pick_list, text};
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};
use ringbuf::{HeapRb, Rb};

pub struct FreqLog {
    pub log: [u16; 4096],
    pub idx: usize
}

impl FreqLog {
    pub fn view(&self) -> Element<UIMessage> {
        ChartWidget::new(self)
            .width(Length::Units(1000))
            .height(Length::Units(200))
            .into()
    }
}

impl Chart<UIMessage> for FreqLog {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .set_all_label_area_size(40)
            .build_cartesian_2d(0..4096, 0..65535)
            .expect("Failed to build chart");

        let series = LineSeries::new(self.log.iter().enumerate().map(|(x, y)| (x as i32, *y as i32)), &BLACK);
        //let series = LineSeries::new(self.log.iter().map(|val| (*freq as i32, *amp as i32)), &BLACK);

        chart.configure_mesh().draw().expect("Failed to draw mesh");

        chart.draw_series(series)
            .expect("Failed to draw series");
    }
}

pub struct FreqAnalysis {
    pub spectrum: Vec<(f32, f32)>
}

impl FreqAnalysis {
    pub fn view(&self) -> Element<UIMessage> {
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
    RecordingStopped,
    Tick,
    DummyMessage
}

pub struct AudiaParams {
    pub engine: Box<dyn crate::engine::Engine>,
    pub frequency_spectrum: Vec<(f32, f32)>,
    pub rx: Receiver<Vec<f32>>,
}

pub struct Audia {
    params: AudiaParams,
    freq_anal: FreqAnalysis,
    freq_log: FreqLog,
    playing: bool,
    tick_received: u32
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

        (Self {
            params: flags,
            freq_anal: FreqAnalysis { spectrum },
            freq_log: FreqLog { log: [0; 4096], idx: 0 },
            playing: false,
            tick_received: 0
        }, Command::none())
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
                self.playing = true;
                self.params.engine.start_recording();
            }
            UIMessage::RecordingStopped => {
                self.playing = false;
                self.params.engine.stop_recording();
            }
            UIMessage::Tick => {
                self.tick_received += 1;
                {
                    if let Ok(data) = self.params.rx.recv() {
                        self.freq_log.log[self.freq_log.idx % 4096] = data.len().min(65535) as u16;
                        self.freq_log.idx += 1;
                    }
                }
            }
            _ => {
                log::info!("Unknown event: {:?}", message);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let button = if self.playing {
            button("Stop").on_press(UIMessage::RecordingStopped)
        } else {
            button("Record").on_press(UIMessage::RecordingStarted)
        };
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
            .push(button)
            //.push(button("Record").on_press(UIMessage::RecordingStarted))
            .push(self.freq_log.view())
            .push(self.freq_anal.view())
            //.push(FreqAnalysis { spectrum: self.params.frequency_spectrum.clone() }.view())
            .push(text(format!("Received ticks: {}", self.tick_received)))
            .padding(20)
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self.playing {
            true => iced_time::every(Duration::from_millis(10)).map(|_| UIMessage::Tick),
            false => Subscription::none()
        }
    }
}
