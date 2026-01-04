use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use std::sync::{Arc, Mutex};

pub struct AudioOutput {
    _stream: Stream,
}

impl AudioOutput {
    pub fn new(
        buffer: Arc<Mutex<Vec<(f32, f32)>>>,
        sample_rate: u32,
    ) -> Result<(Self, u32), String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no output device available")?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| format!("no default config: {}", e))?;

        let actual_sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels() as usize;

        if actual_sample_rate != sample_rate {
            web_sys::console::warn_1(
                &format!(
                    "sample rate mismatch: requested {}, got {}",
                    sample_rate, actual_sample_rate
                )
                .into(),
            );
        }

        let stream = device
            .build_output_stream(
                &supported_config.into(),
                move |out_buf: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut in_buf = match buffer.lock() {
                        Ok(buf) => buf,
                        Err(_) => return,
                    };

                    // Calculate how many stereo frames we need
                    let frames_needed = out_buf.len() / channels;
                    let frames_available = in_buf.len();
                    let frames_to_drain = frames_needed.min(frames_available);

                    // Write audio data
                    for (frame_idx, (l, r)) in in_buf.drain(..frames_to_drain).enumerate() {
                        let base = frame_idx * channels;
                        if channels >= 2 {
                            out_buf[base] = l;
                            out_buf[base + 1] = r;
                            // Fill any extra channels with silence
                            for ch in 2..channels {
                                out_buf[base + ch] = 0.0;
                            }
                        } else if channels == 1 {
                            // Mono: mix left and right
                            out_buf[base] = (l + r) * 0.5;
                        }
                    }

                    // Fill remaining frames with silence
                    for frame_idx in frames_to_drain..frames_needed {
                        let base = frame_idx * channels;
                        for ch in 0..channels {
                            out_buf[base + ch] = 0.0;
                        }
                    }
                },
                |err| {
                    web_sys::console::error_1(&format!("audio stream error: {}", err).into());
                },
                None,
            )
            .map_err(|e| format!("failed to build stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("failed to play: {}", e))?;

        Ok((Self { _stream: stream }, actual_sample_rate))
    }

    pub fn default_sample_rate() -> u32 {
        let host = cpal::default_host();
        host.default_output_device()
            .and_then(|d| d.default_output_config().ok())
            .map(|c| c.sample_rate().0)
            .unwrap_or(48000)
    }
}
