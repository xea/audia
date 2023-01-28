use cpal::{available_hosts, default_host};
use iced::{Alignment, Application, Command, Element, Error, executor, Renderer, Sandbox, Settings, Theme};
use iced::widget::{button, column};

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum Message {
    ButtonPressed
}

pub struct Audia {
}

#[derive(Default)]
pub struct AudiaParams {
    available_hosts: Vec<String>
}

impl Application for Audia {
    type Executor = executor::Default;
    type Theme = Theme;
    type Message = ();
    type Flags = AudiaParams;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("Audia")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        "Hello".into()
    }
}

pub struct MainWindow {

}

impl Sandbox for MainWindow {
    type Message = Message;

    fn new() -> Self {
        Self {}
    }

    fn title(&self) -> String {
        "Audia Sandbox".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::ButtonPressed => {}
        }
    }

    fn view(&self) -> Element<Self::Message> {
        column![
            button("+").on_press(Message::ButtonPressed)
        ]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}

fn main() -> Result<(), Error> {
    let default_host = default_host();
    let hosts = available_hosts();

    let flags = AudiaParams {
        available_hosts: hosts.iter().map(|host| host.name().into()).collect()
    };

    let mut settings = Settings::default();

    settings.flags = flags;

    Audia::run(settings)
}
