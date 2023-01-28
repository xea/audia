use cpal::{HostId};
use iced::{Alignment, Application, Command, Element, Error, executor, Settings, Theme};
use iced::widget::{button, Column, Text};

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum UIMessage {
    ButtonPressed
}

pub trait Engine {
    fn get_available_hosts(&self) -> Vec<String>;
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
    fn get_available_hosts(&self) -> Vec<String> {
        cpal::available_hosts()
            .iter()
            .map(|host_id| host_id.name())
            .map(|name| name.into())
            .collect()
    }
}

pub struct Audia {
    params: AudiaParams
}

pub struct AudiaParams {
    engine: Box<dyn Engine>,
}

impl Application for Audia {
    type Executor = executor::Default;
    type Message = ();
    type Theme = Theme;
    type Flags = AudiaParams;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { params: flags }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Audia")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let hosts = self.params.engine.get_available_hosts();

        let mut column = Column::new();

        for host in hosts {
            column = column.push(Text::new(host));
        }

        column
            .push(button("Use host").on_press(()))
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}

fn main() -> Result<(), Error> {
    let engine = CpalEngine::new();

    let flags = AudiaParams {
        engine: Box::new(engine)
    };

    let settings = Settings::with_flags(flags);

    Audia::run(settings)
}
