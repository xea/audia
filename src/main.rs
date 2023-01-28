use cpal::{HostId};
use fast_log::Config;
use fast_log::filter::ModuleFilter;
use iced::{Alignment, Application, Command, Element, Error, executor, Settings, Theme};
use iced::widget::{button, Column, pick_list};
use log::LevelFilter;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum UIMessage {
    ButtonPressed,
    HostChanged(String),
    InputDeviceChanged(String)
}

pub trait Engine {
    fn use_engine(&mut self, host_id: &str);
    fn get_current_engine(&self) -> Option<&str>;
    fn get_available_hosts(&self) -> Vec<String>;

    fn get_input_devices(&self) -> Vec<String>;
}

#[derive(Default)]
pub struct CpalEngine {
    current_host: Option<HostId>
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
                self.current_host = Some(host)
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
            vec![]
        } else {
            vec![]
        }
    }
}

pub struct AudiaParams {
    engine: Box<dyn Engine>,
}

pub struct Audia {
    params: AudiaParams
}

impl Audia {
}

impl Application for Audia {
    type Executor = executor::Default;
    type Message = UIMessage;
    type Theme = Theme;
    type Flags = AudiaParams;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { params: flags }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Audia")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            UIMessage::HostChanged(new_host) => {
                self.params.engine.use_engine(new_host.as_str());
            }
            _ => {}
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
            .push(pick_list(self.params.engine.get_input_devices(), None, UIMessage::InputDeviceChanged).placeholder("Choose an input device"))
            .push(button("Use host").on_press(UIMessage::ButtonPressed))
            .padding(20)
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
    }

}

fn main() -> Result<(), Error> {
    let log_config = Config::new()
        .console()
        .level(LevelFilter::Info)
        .filter(ModuleFilter::new_include(vec![ String::from("audia") ]))
        .chan_len(Some(65536));

    fast_log::init(log_config).expect("Could not initialize logger");

    log::info!("Initializing Audia");

    let engine = CpalEngine::new();

    let flags = AudiaParams {
        engine: Box::new(engine)
    };

    let settings = Settings::with_flags(flags);

    Audia::run(settings)
}
