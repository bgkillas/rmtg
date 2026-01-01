use audioadapter_buffers::direct::InterleavedSlice;
use bevy::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opus::{Application, Channels, Decoder, Encoder};
use rubato::{Fft, FixedSync, Resampler};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, mpsc};
use std::thread;
use std::time::Duration;
pub const SAMPLE_RATE: usize = 48000;
pub const FRAME_SIZE: usize = 960;
pub const CHANNELS: Channels = Channels::Mono;
#[derive(Resource)]
pub struct AudioResource(Mutex<AudioManager>);
impl AudioResource {
    pub fn new(audio: &AudioSettings) -> Self {
        Self(AudioManager::new(audio).into())
    }
    pub fn recv_audio<F>(&self, f: F)
    where
        F: FnMut(&[f32]),
    {
        self.0.lock().unwrap().recv_audio(f)
    }
}
pub struct AudioManager {
    rx: Receiver<Vec<u8>>,
    decoder: Decoder,
}
#[derive(Default)]
pub struct AudioSettings {
    input_device: Option<String>,
    disabled: bool,
}
impl AudioManager {
    pub fn new(audio: &AudioSettings) -> Self {
        #[cfg(target_os = "linux")]
        let host = cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .and_then(|id| cpal::host_from_id(id).ok())
            .unwrap_or(cpal::default_host());
        #[cfg(not(target_os = "linux"))]
        let host = cpal::default_host();
        let device = {
            let input = audio.input_device.clone();
            if audio.disabled {
                None
            } else if input.is_none() {
                host.default_input_device()
            } else if let Some(d) = host
                .input_devices()
                .map(|mut d| {
                    d.find(|d| {
                        d.description()
                            .ok()
                            .and_then(|a| input.as_ref().map(|i| i == a.name()))
                            .unwrap_or(false)
                    })
                })
                .ok()
                .flatten()
            {
                Some(d)
            } else {
                host.default_input_device()
            }
        };
        let decoder = Decoder::new(SAMPLE_RATE as u32, CHANNELS).unwrap();
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        thread::spawn(move || {
            if let Some(device) = device {
                if let Ok(cfg) = device.default_input_config() {
                    let sample = cfg.sample_rate();
                    let channels = cfg.channels();
                    let config = cpal::SupportedStreamConfig::new(
                        if channels <= 2 { cfg.channels() } else { 2 },
                        sample,
                        *cfg.buffer_size(),
                        cpal::SampleFormat::F32,
                    );
                    if let Ok(mut resamp) = Fft::<f32>::new(
                        sample as usize,
                        SAMPLE_RATE,
                        FRAME_SIZE,
                        8,
                        1,
                        FixedSync::Output,
                    ) {
                        let mut encoder =
                            Encoder::new(SAMPLE_RATE as u32, CHANNELS, Application::Audio).unwrap();
                        let mut extra = Vec::new();
                        match device.build_input_stream(
                            &config.into(),
                            move |data: &[f32], _| {
                                if channels == 1 {
                                    extra.extend(data);
                                } else {
                                    extra.extend(
                                        data.chunks(2)
                                            .map(|a| (a[0] + a[1]) * 0.5)
                                            .collect::<Vec<f32>>(),
                                    )
                                }
                                let mut v = Vec::new();
                                let mut compressed = [0u8; 1024];
                                let mut buffer = [0f32; 1024];
                                while extra.len() >= FRAME_SIZE {
                                    let input =
                                        InterleavedSlice::new(&extra[..FRAME_SIZE], 1, FRAME_SIZE)
                                            .unwrap();
                                    let mut output =
                                        InterleavedSlice::new_mut(&mut buffer, 1, FRAME_SIZE)
                                            .unwrap();
                                    resamp
                                        .process_into_buffer(&input, &mut output, None)
                                        .unwrap();
                                    if let Ok(len) = encoder.encode_float(&buffer, &mut compressed)
                                        && len != 0
                                    {
                                        v.push(compressed[..len].to_vec())
                                    }
                                    extra.drain(..FRAME_SIZE);
                                }
                                for v in v {
                                    let _ = tx.send(v);
                                }
                            },
                            |err| error!("Stream error: {}", err),
                            Some(Duration::from_millis(10)),
                        ) {
                            Ok(stream) => {
                                if let Ok(_s) = stream.play() {
                                    loop {
                                        thread::sleep(Duration::from_millis(10))
                                    }
                                } else {
                                    error!("failed to play stream")
                                }
                            }
                            Err(s) => {
                                error!(
                                    "no stream {}, {}, {}, {}",
                                    s,
                                    cfg.channels(),
                                    cfg.sample_rate(),
                                    cfg.sample_format()
                                )
                            }
                        }
                    } else {
                        warn!("resamp not found")
                    }
                } else {
                    warn!("input config not found")
                }
            } else {
                warn!("input device not found")
            }
        });
        Self { rx, decoder }
    }
    pub fn recv_audio<F>(&mut self, mut f: F)
    where
        F: FnMut(&[f32]),
    {
        let mut out = [0f32; FRAME_SIZE];
        while let Ok(data) = self.rx.try_recv() {
            if let Ok(len) = self.decoder.decode_float(&data, &mut out, false)
                && len != 0
            {
                f(&out[..len])
            }
        }
    }
}
