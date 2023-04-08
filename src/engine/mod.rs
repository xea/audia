use cpal::{Device, HostId, Stream, StreamError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, RecvError, TryRecvError};
use ringbuf::HeapRb;

pub type AudioHostName = String;
pub type InputDeviceName = String;
pub type SampleType = f32;
pub type PacketType = Vec<SampleType>;

pub struct AudiaError {
    message: String
}

impl From<String> for AudiaError {
    fn from(message: String) -> Self {
        AudiaError { message }
    }
}

impl From<&str> for AudiaError {
    fn from(value: &str) -> Self {
        AudiaError { message: value.into() }
    }
}

/// Provides an abstraction of the underlying audio systems.
pub trait Engine {
    // Host operations.
    fn get_available_hosts(&self) -> Vec<AudioHostName>;
    fn get_current_host(&self) -> Option<AudioHostName>;
    fn use_host(&mut self, host_name: AudioHostName);

    // Input device operations
    fn get_input_devices(&self) -> Vec<InputDeviceName>;
    fn get_current_input_device(&self) -> Option<InputDeviceName>;
    fn use_input_device(&mut self, device_name: InputDeviceName);

    // Recording operations
    fn start_recording(&mut self) -> Result<AudioStream, AudiaError>;
    fn stop_recording(&mut self);
}

/// CPAL-based audio engine
pub struct CpalEngine {
    current_host: Option<HostId>,
    current_input_device: Option<Device>,
    current_stream: Option<Stream>
}

impl CpalEngine {
}

impl CpalEngine {
    pub fn new() -> Self {
        Self {
            current_host: None,
            current_input_device: None,
            current_stream: None
        }
    }

    fn run_stream(&mut self, stream: Stream, rx: Receiver<PacketType>) -> Result<AudioStream, AudiaError> {
        if let Err(error) = stream.play() {
            log::error!("Failed to run stream: {error:?}");
            Err(AudiaError::from(format!("Failed to run stream: {error:?}")))
        } else {
            self.current_stream = Some(stream);
            log::info!("Running stream");
            Ok(AudioStream::new(rx))
        }
    }
}

impl Default for CpalEngine {
    fn default() -> Self {
        log::info!("Using CPAL engine with default settings");

        Self {
            current_host: Some(cpal::default_host().id()),
            current_input_device: cpal::default_host().default_input_device(),
            current_stream: None
        }
    }
}

impl Engine for CpalEngine {
    fn get_available_hosts(&self) -> Vec<AudioHostName> {
        cpal::available_hosts()
            .iter()
            .map(|host_id| host_id.name())
            .map(|name| name.into())
            .collect()
    }

    fn get_current_host(&self) -> Option<AudioHostName> {
        self.current_host.map(|h| h.name()).map(String::from)
    }

    fn use_host(&mut self, host_id: AudioHostName) {
        for host in cpal::available_hosts() {
            if host.name().eq(host_id.as_str()) {
                self.current_host = Some(host);
                log::info!("Switched to audio host {}", host_id);
            }
        }
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

    fn start_recording(&mut self) -> Result<AudioStream, AudiaError> {
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

                let err_fn = move |err: StreamError| {
                    log::error!("An error occurred during reading from the stream: {:?}", err);
                };

                let (tx, rx) = crossbeam_channel::unbounded::<PacketType>();
                let mut counter = 0;

                let stream_result = device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[SampleType], _info| {
                            counter += 1;

                            if counter % 100 == 0 {
                                println!("Current queue size: {}", tx.len());
                            }

                            if let Err(error) = tx.send(data.into()) {
                                log::error!("Failed to send stream data: {error:?}");
                            }
                        },
                        err_fn, None);

                stream_result
                    .map_err(|error| AudiaError::from(format!("Failed to create audio stream: {error:?}")))
                    .and_then(|stream| self.run_stream(stream, rx))
            } else {
                Err(AudiaError::from("Could not find default input config"))
            }
        } else {
            Err(AudiaError::from("No input device is selected"))
        }
    }

    fn stop_recording(&mut self) {
        let mut maybe_stream = None;
        std::mem::swap(&mut maybe_stream, &mut self.current_stream);

        if let Some(stream) = maybe_stream {
            drop(stream);

            log::info!("Streaming stopped");
        }
    }
}

/// Collection of configuration settings required by the audio system
pub struct AudioSettings {

}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {}
    }
}

pub struct AudioSystem {
    pub engine: Box<dyn Engine>,
    pub stream: Vec<AudioStream>
}

impl AudioSystem {
    pub fn new(_settings: AudioSettings) -> Self {
        AudioSystem {
            engine: Box::new(CpalEngine::default()),
            stream: vec![]
        }
    }
}

/// `AudioStream` represents a live recording session from an input device.
pub struct AudioStream {
    rx: Receiver<PacketType>
}

impl AudioStream {

    pub fn new(rx: Receiver<Vec<f32>>) -> Self {
        Self {
            rx
        }
    }

    pub fn receive(&self) -> Result<PacketType, TryRecvError> {
        self.rx.try_recv()
    }
}
