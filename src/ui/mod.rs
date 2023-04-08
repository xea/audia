use std::time::Duration;

use iced::{Alignment, Application, Command, Element, executor, Subscription, Theme};
use iced::time as iced_time;
use iced::widget::{button, Column, text};
use crate::engine::{AudioStream, AudioSystem};

use crate::ui::spectrogram::Spectrogram;

mod spectrogram;

pub struct UIParams {
    pub audio_system: AudioSystem
}

impl UIParams {
    pub fn new(audio_system: AudioSystem) -> Self {
        Self { audio_system }
    }
}

#[derive(Debug, Clone)]
pub enum UIMessage {
    HostChanged,
    InputDeviceChanged,
    StartStreaming,
    StopStreaming,
    StreamTick,
    DebugEvent
}

pub struct Audia {
    spectrogram: Spectrogram,
    audio_system: AudioSystem,
    current_stream: Option<AudioStream>
}

impl Audia {
    fn start_streaming(&mut self) {
        log::info!("Start streaming");

        if self.current_stream.is_none() {
            match self.audio_system.engine.start_recording() {
                Ok(stream) => {
                    self.current_stream = Some(stream);
                },
                Err(error) => {
                    log::error!("Error jaj");
                }
            };

        } else {
            log::info!("Stream is already running");
        }
    }

    fn stop_streaming(&mut self) {
        log::info!("Stop streaming");

        if self.current_stream.is_some() {
            self.current_stream = None;
        } else {
            log::info!("Stream has not been stopped");
        }
    }

    fn stream_update(&mut self) {
        if let Some(stream) = &self.current_stream {
            if let Ok(mut packet) = stream.receive() {
                self.spectrogram.user_data += packet.len();
                self.spectrogram.current_buf.clear();
                self.spectrogram.current_buf.append(&mut packet);
            } else {
                log::error!("Failed to receive packet");
            }
        } else {
            log::info!("Stream update request but no stream :(");
        }
    }
}

impl Application for Audia {
    type Executor = executor::Default;
    type Message = UIMessage;
    type Theme = Theme;
    type Flags = UIParams;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let audio_system = flags.audio_system;

        (Self {
            spectrogram: Spectrogram::new(),
            current_stream: None,
            audio_system
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Audia")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            UIMessage::StartStreaming => self.start_streaming(),
            UIMessage::StopStreaming => self.stop_streaming(),
            UIMessage::StreamTick => self.stream_update(),
            _ => {
                log::info!("Unknown event: {:?}", message);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let stream_button = if self.current_stream.is_none() {
            button("Start streaming").on_press(UIMessage::StartStreaming)
        } else {
            button("Stop streaming").on_press(UIMessage::StopStreaming)
        };

        Column::new()
            .push(stream_button)
            .push(self.spectrogram.view())
            .push(text(format!("{}", self.spectrogram.user_data)))
            /*
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

             */
            .padding(20)
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.current_stream.is_some() {
            let duration = Duration::from_millis(5);
            iced_time::every(duration).map(|_instant| UIMessage::StreamTick)
        } else {
            Subscription::none()
        }
    }

}

