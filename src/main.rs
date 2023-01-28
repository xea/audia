use cpal::{HostId};
use iced::{Alignment, Application, Command, Element, Error, executor, Renderer, Settings, Theme};
use iced::widget::{button, checkbox, Column, pick_list, PickList, Text};

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum UIMessage {
    ButtonPressed,
    HostChanged(String)
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

pub struct DummyEngine;

impl Engine for DummyEngine {
    fn get_available_hosts(&self) -> Vec<String> {
        vec![ String::from("Dummy") ]
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

pub struct AudiaParams {
    engine: Box<dyn Engine>,
}

pub struct Audia {
    params: AudiaParams
}

impl Audia {

    fn view_a(&self) -> Element<UIMessage> {
        Column::new()
            .push(pick_list(self.params.engine.get_available_hosts(), None, UIMessage::HostChanged).placeholder("Choose an audio host"))
            .push(button("Use host").on_press(UIMessage::ButtonPressed))
            .padding(20)
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
    }
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

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.view_a()
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
