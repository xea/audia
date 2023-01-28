use iced::{Alignment, Element, Error, Sandbox, Settings};
use iced::widget::{button, column};

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum Message {
    ButtonPressed
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
    MainWindow::run(Settings::default())
}
