use ::digest::{FixedOutput, HashMarker, Update, crypto_common};

fn array64_from_vec(data: &[u8]) -> Option<[u8; 64]> {
    <[u8; 64]>::try_from(data).ok()
}

use crypto_common::{Output, OutputSizeUser, typenum::U64};
use ring::digest;

/// Adapter ring::SHA512 -> Digest API
#[derive(Clone)]
pub struct RingSha512 {
    ctx: digest::Context,
}

impl OutputSizeUser for RingSha512 {
    type OutputSize = U64;
}

impl Default for RingSha512 {
    fn default() -> Self {
        Self {
            ctx: digest::Context::new(&digest::SHA512),
        }
    }
}

impl Update for RingSha512 {
    fn update(&mut self, data: &[u8]) {
        self.ctx.update(data);
    }
}

impl FixedOutput for RingSha512 {
    fn finalize_into(self, out: &mut Output<Self>) {
        let data = array64_from_vec(self.ctx.finish().as_ref()).unwrap();
        out.copy_from_slice(&data);
    }
}

impl HashMarker for RingSha512 {}
