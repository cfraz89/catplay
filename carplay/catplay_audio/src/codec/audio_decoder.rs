use std::error::Error;

use crate::{AudioStreamBasicDescription, Zeroable};

pub trait AudioDecoder: Send + 'static {
    type Error: Error;
    type Sample: Zeroable;

    // // Input type for undecoded data.
    // fn input_type(&self) -> AudioStreamBasicDescription;

    // PCM output type for decoded data.
    fn output_type(&self) -> AudioStreamBasicDescription;

    /// Decode input data into `output`.
    ///
    /// Returns:
    /// - number of PCM samples written to `output` (interleaved, total across all channels)
    /// - number of input bytes consumed
    ///
    /// Notes:
    /// - `written_samples == 0` means no output was produced (e.g. not enough input data)
    /// - caller MUST NOT advance audio timeline unless `written_samples > 0`
    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error>;

    /// Conceal lost packet with data better than pure silence, if possible.
    #[allow(unused)]
    fn conceal_lost_packet(&mut self, output: &mut [Self::Sample]) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

pub trait AudioDecoderFactory: Sized {
    type Error: Error;

    fn new(asbd: AudioStreamBasicDescription) -> Result<Self, Self::Error>;
}
