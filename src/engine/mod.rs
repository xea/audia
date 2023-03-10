use std::sync::Arc;
use cpal::{Device, HostId, Stream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::Sender;

pub trait Engine {
    fn use_engine(&mut self, host_id: &str);
    fn get_current_engine(&self) -> Option<&str>;
    fn get_available_hosts(&self) -> Vec<String>;

    fn get_input_devices(&self) -> Vec<String>;
    fn get_current_input_device(&self) -> Option<String>;
    fn use_input_device(&mut self, device_name: String);

    fn start_recording(&mut self);
    fn stop_recording(&mut self);
}

pub struct CpalEngine {
    current_host: Option<HostId>,
    current_input_device: Option<Device>,
    tx: Arc<Sender<Vec<f32>>>,
    current_stream: Option<Stream>
}

impl CpalEngine {
    pub fn new(tx: Sender<Vec<f32>>) -> Self {
        Self {
            current_host: None,
            current_input_device: None,
            tx: Arc::new(tx),
            current_stream: None
        }
    }
}

impl Engine for CpalEngine {
    fn use_engine(&mut self, host_id: &str) {
        for host in cpal::available_hosts() {
            if host.name().eq(host_id) {
                self.current_host = Some(host);
                log::info!("Switched to audio host {}", host_id);
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

    fn start_recording(&mut self) {
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

                let err_fn = move |err| {
                    log::error!("An error occurred during reading from the stream: {:?}", err);
                };

                let tx = self.tx.clone();

                let stream_result = device.build_input_stream(&config.into(), move |data: &[f32], _info| {
                    if let Err(error) = tx.send(data.into()) {
                        log::error!("Failed to send stream data: {error:?}");
                    }
                }, err_fn, None);

                match stream_result {
                    Ok(stream) => {
                        if let Err(error) = stream.play() {
                            log::error!("Failed to play stream: {:?}", error);
                        } else {
                            self.current_stream = Some(stream);
                            log::info!("Playing");


                            log::info!("Playing completed, read values");
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to create stream: {:?}", err);
                    }
                }
            }
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
