use iced::{Alignment, Application, Command, Element, executor, Length, Theme};
use iced::widget::{button, Column, pick_list};
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

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
    DummyMessage
}

pub struct AudiaParams {
    pub engine: Box<dyn crate::engine::Engine>,
    pub frequency_spectrum: Vec<(f32, f32)>
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
