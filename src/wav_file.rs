//! This module contains the `WavWriter` for writing morse code to a wav file
//!
//! # Example use
//! ```
//! let file = io::BufWriter::new(std::fs::File::create("out.wav")?);
//! let transport = wav_file::WavWriter::new(file, 600.0)?;
//! ```

use crate::{Keyable, Symbol};
use hound;
use std::io::{self, Seek, Write};
use std::time::Duration;

pub struct WavWriter<W: Write + Seek> {
    wav_writer: hound::WavWriter<W>,
    tone: f32,
}

impl<W> WavWriter<W>
where
    W: Write + Seek,
{
    pub fn new(writer: W, tone: f32) -> Result<Self, io::Error> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let wav_writer = match hound::WavWriter::new(writer, spec) {
            Ok(w) => w,
            Err(hound::Error::IoError(e)) => return Err(e),
            Err(e) => panic!("{}", e),
        };

        Ok(WavWriter { wav_writer, tone })
    }

    fn write_tone(&mut self, tone: f32, length: Duration) -> Result<(), hound::Error> {
        let fs = self.wav_writer.spec().sample_rate as f32;
        let num_samples = fs * length.as_secs_f32();

        for t in (0..num_samples as usize).map(|x| x as f32 / fs) {
            let sample = (t * tone * std::f32::consts::TAU).sin();
            let amplitude = i16::MAX as f32;
            self.wav_writer.write_sample((sample * amplitude) as i16)?;
        }

        Ok(())
    }

    fn write_silence(&mut self, length: Duration) -> Result<(), hound::Error> {
        let fs = self.wav_writer.spec().sample_rate as f32;
        let num_samples = fs * length.as_secs_f32();

        for _ in 0..num_samples as usize {
            self.wav_writer.write_sample(0)?;
        }

        Ok(())
    }
}

impl<W: Write + Seek> Keyable for WavWriter<W> {
    type Error = io::Error;

    async fn play(
        &mut self,
        on: Duration,
        off: Duration,
        _symbol: Symbol,
    ) -> Result<(), io::Error> {
        match self.write_tone(self.tone, on) {
            Err(hound::Error::IoError(e)) => return Err(e),
            Err(e) => panic!("{}", e),
            Ok(()) => {}
        }

        match self.write_silence(off) {
            Err(hound::Error::IoError(e)) => return Err(e),
            Err(e) => panic!("{}", e),
            Ok(()) => {}
        }

        Ok(())
    }
}
