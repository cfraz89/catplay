use std::error::Error;

use crate::{AudioStreamBasicDescription, Zeroable};

pub trait AudioEncoder: Send + 'static {
    type Error: Error;
    type Sample: Zeroable;

    // Input type - which PCM variant.
    fn input_type(&self) -> AudioStreamBasicDescription;

    // Output type - describes encoded data.
    fn output_type(&self) -> AudioStreamBasicDescription;

    /// Encodes input `samples` into `output`.
    ///
    /// Returns:
    /// - number of bytes written to `output`
    /// - number of input samples consumed
    fn encode(&mut self, samples: &[Self::Sample], output: &mut [u8]) -> Result<(usize, usize), Self::Error>;
}

pub trait AudioEncoderFactory: Sized {
    type Error: Error;

    fn new(input: AudioStreamBasicDescription, output: AudioStreamBasicDescription) -> Result<Self, Self::Error>;
}
