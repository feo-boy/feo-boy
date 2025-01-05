//! Audio playback functionality.
//!
//! Plays the audio based on the state of the sound hardware.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, SampleFormat, SampleRate, Stream};
use derivative::Derivative;
use log::*;

use crate::cpu;

use super::SampleBuffer;

/// Audio sample rate. 44.1K Hz is CD-quality audio.
const SAMPLE_RATE: SampleRate = SampleRate(44100);

/// Outputs PCM audio generated by the sound controller.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Output {
    /// The audio output stream.
    #[derivative(Debug = "ignore")]
    stream: Stream,

    /// The CPU produces audio samples at the CPU clock rate. This is a much higher rate than PC
    /// hardware typically supports. Therefore, we must downsample the raw signal to be playable
    /// by audio hardware.
    ///
    /// The simplest way to accomplish this is "decimation": keeping only every nth sample. This
    /// factor is computed by dividing the Game Boy CPU frequency by the audio hardware's sampling
    /// rate.
    pub decimation_factor: u32,

    /// Queued raw emulated PCM audio samples.
    pub sample_buffer: SampleBuffer,
}

impl Output {
    pub fn new() -> Result<Self> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| anyhow!("no audio output devices found"))?;

        let sample_buffer = SampleBuffer::default();

        let config = device
            .supported_output_configs()?
            .find(|config| config.channels() == 1 && config.sample_format() == SampleFormat::F32)
            .map(|config| config.with_sample_rate(SAMPLE_RATE))
            .ok_or_else(|| anyhow!("no supported audio output configuration found"))?
            .config();

        info!("initializing audio playback with {:?}", config);

        let decimation_factor = cpu::FREQUENCY / config.sample_rate.0;

        let stream_buffer = Arc::clone(&sample_buffer);
        let stream = device.build_output_stream(
            &config,
            move |dst: &mut [f32], _: &OutputCallbackInfo| {
                let mut src = stream_buffer.lock().unwrap();

                for sample in dst.iter_mut() {
                    *sample = src.pop_front().unwrap_or(0.0);
                }
            },
            |err| panic!("{}", err),
            None,
        )?;

        stream.play()?;

        Ok(Output {
            stream,
            sample_buffer,
            decimation_factor,
        })
    }
}
