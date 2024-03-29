use std::time::Duration;

use iced::{Alignment, Application, Command, Element, executor, Subscription, Theme};
use iced::time as iced_time;
use iced::widget::{button, Column, pick_list, Row, text};
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
use spectrum_analyzer::scaling::{divide_by_N, divide_by_N_sqrt};
use spectrum_analyzer::windows::hann_window;

use crate::engine::{AudioHostName, AudioStream, AudioSystem, InputDeviceName, PacketType};
use crate::ui::spectrogram::Spectrogram;

mod spectrogram;

// this needs to be a power of two
const RECEIVE_PACKET_SIZE: usize = 256;

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
    HostChanged(String),
    InputDeviceChanged(String),
    OutputDeviceChanged(String),
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
                self.update_state(&mut packet);
            } else {
                // There was no audio data in the stream, ignore
            }
        } else {
            log::info!("Stream update request but no stream :(");
        }
    }

    fn update_state(&mut self, packet: &mut PacketType) {
        self.spectrogram.current_buf.append(packet);

        while self.spectrogram.current_buf.len() >= RECEIVE_PACKET_SIZE {
            let current_packet: PacketType = self.spectrogram.current_buf.drain(0..RECEIVE_PACKET_SIZE).collect();

            self.spectrogram.user_data += RECEIVE_PACKET_SIZE;

            self.spectrogram.freq_data.clear();

            let hann_window = hann_window(current_packet.as_slice());
            let spectrum = samples_fft_to_spectrum(
                &hann_window,

                48000,
                FrequencyLimit::Max(2200.0),
                Some(&divide_by_N_sqrt))
                .expect("Could not extract frequency spectrum");

            let points: Vec<(i32, f32)> = spectrum.data()
                .iter()
                .map(|(freq, amp)| {
                    (freq.val() as i32, amp.val() * 2048.0)
                }).collect();


            self.spectrogram.peak_freq = points.iter().fold((0, 0.0), |a, b| {
                if a.1 >= b.1 {
                    a
                } else {
                    *b
                }
            }).0 as f32;
            //self.spectrogram.peak_freq = points.iter().fold(0.0, |a, b| a.max(b.0 as f32));
            self.spectrogram.freq_data = points;
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
            UIMessage::HostChanged(new_host) => self.audio_system.engine.use_host(AudioHostName::from(new_host.as_str())),
            UIMessage::InputDeviceChanged(new_device) => self.audio_system.engine.use_input_device(InputDeviceName::from(new_device.as_str())),
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
            .push(
                Row::new()
                .push(text("Audio host: "))
                    .push(
                        pick_list(
                            self.audio_system.engine.get_available_hosts(),
                            self.audio_system.engine.get_current_host().map(|e| e.into()),
                            UIMessage::HostChanged)
                            .placeholder("Choose an audio host")))
            .push(
                Row::new()
                    .spacing(5)
                    .push(text("Input device"))
                    .push(
                        pick_list(
                            self.audio_system.engine.get_input_devices(),
                            self.audio_system.engine.get_current_input_device(),
                            UIMessage::InputDeviceChanged)
                            .placeholder("Choose an input device")))
            .push(
                Row::new()
                    .spacing(5)
                    .push(text("Output device"))
                    .push(
                        pick_list(
                            self.audio_system.engine.get_output_devices(),
                            self.audio_system.engine.get_current_output_device(),
                            UIMessage::OutputDeviceChanged)
                            .placeholder("Choose an output device")))
            .push(stream_button)
            .push(self.spectrogram.view())
            .push(text(format!("{:3.2}Hz {}", self.spectrogram.peak_freq, self.spectrogram.user_data)))
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

