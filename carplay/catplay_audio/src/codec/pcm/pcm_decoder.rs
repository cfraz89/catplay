use log::debug;

use crate::{
    AudioStreamBasicDescription,
    codec::{AudioDecoder, AudioDecoderFactory},
};

pub struct PcmDecoder {
    sample_rate: u32,
    channels: usize,
}

impl PcmDecoder {
    #[inline(always)]
    pub(crate) fn copy_pcm16_be_to_samples(bytes: &[u8], output: &mut [i16]) {
        let sample_count = output.len();
        debug_assert_eq!(bytes.len(), sample_count * 2);

        if !cfg!(target_endian = "little") {
            let out_bytes = unsafe { std::slice::from_raw_parts_mut(output.as_mut_ptr() as *mut u8, bytes.len()) };
            out_bytes.copy_from_slice(bytes);
            return;
        }

        let (prefix, src_i16, suffix) = unsafe { bytes.align_to::<i16>() };

        if prefix.is_empty() && suffix.is_empty() && src_i16.len() >= sample_count {
            for (o, &v) in output.iter_mut().zip(&src_i16[..sample_count]) {
                *o = v.swap_bytes();
            }
            return;
        }

        debug!("PCM decode slow-path (unaligned BE -> LE)");

        for (o, chunk) in output.iter_mut().zip(bytes.chunks_exact(2)) {
            *o = i16::from_be_bytes([chunk[0], chunk[1]]);
        }
    }

    #[inline(always)]
    pub(crate) fn copy_samples_to_pcm16_be(samples: &[i16], output: &mut [u8]) {
        let sample_count = samples.len();
        debug_assert_eq!(output.len(), sample_count * 2);

        if !cfg!(target_endian = "little") {
            let src = unsafe { std::slice::from_raw_parts(samples.as_ptr() as *const u8, output.len()) };
            output.copy_from_slice(src);
            return;
        }

        let (prefix, out_i16, suffix) = unsafe { output.align_to_mut::<i16>() };

        if prefix.is_empty() && suffix.is_empty() && out_i16.len() >= sample_count {
            for (o, &s) in out_i16[..sample_count].iter_mut().zip(samples) {
                *o = s.swap_bytes();
            }
            return;
        }

        debug!("PCM encode slow-path (unaligned LE -> BE)");

        for (chunk, &s) in output.chunks_exact_mut(2).zip(samples) {
            let b = s.to_be_bytes();
            chunk[0] = b[0];
            chunk[1] = b[1];
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum PcmError {}

impl AudioDecoderFactory for PcmDecoder {
    type Error = PcmError;

    fn new(asbd: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let channels = asbd.channels();
        assert!(channels > 0);

        Ok(Self {
            channels: channels as _,
            sample_rate: asbd.sample_rate,
        })
    }
}

impl AudioDecoder for PcmDecoder {
    type Error = PcmError;
    type Sample = i16;

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        let available_samples = data.len() / 2;
        let writable_samples = available_samples.min(output.len());

        if writable_samples == 0 {
            return Ok((0, 0));
        }

        let bytes = &data[..writable_samples * 2];

        PcmDecoder::copy_pcm16_be_to_samples(bytes, &mut output[..writable_samples]);

        Ok((writable_samples, writable_samples * 2))
    }

    fn output_type(&self) -> AudioStreamBasicDescription {
        AudioStreamBasicDescription::fill_pcm(self.sample_rate, 16, 16, self.channels as _, false)
    }

    // fn input_type(&self) -> AudioStreamBasicDescription {
    //     AudioStreamBasicDescription::fill_pcm(self.sample_rate, 16, 16, self.channels as _, false)
    // }
}
