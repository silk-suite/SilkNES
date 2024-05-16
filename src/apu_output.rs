use std::collections::VecDeque;
use std::f32::consts::PI;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use rodio::source::Source;

/// An infinite source that produces a sine.
///
/// Always has a rate of 48kHz and one channel.
pub struct APUOutput {
  freq: f32,
  num_sample: usize,
  apu_messenger: Receiver<Vec<f32>>,
  buffer: VecDeque<f32>,
}

impl APUOutput {
  /// The frequency of the square wave.
  #[inline]
  pub fn new(freq: f32, apu_messenger: Receiver<Vec<f32>>) -> APUOutput {
    APUOutput {
      freq,
      num_sample: 0,
      apu_messenger,
      buffer: vec![].into(),
    }
  }
}

impl Iterator for APUOutput {
  type Item = f32;

  #[inline]
  fn next(&mut self) -> Option<f32> {
    match self.apu_messenger.try_recv() {
      Ok(buffer) => {
        self.buffer.extend(buffer)
      },
      Err(_) => {},
    }

    let value = self.buffer.pop_front().unwrap_or(0.0);
    Some(value)
  }
}

impl Source for APUOutput {
  #[inline]
  fn current_frame_len(&self) -> Option<usize> {
    None
  }

  #[inline]
  fn channels(&self) -> u16 {
    1
  }

  #[inline]
  fn sample_rate(&self) -> u32 {
    48000
  }

  #[inline]
  fn total_duration(&self) -> Option<Duration> {
    None
  }
}