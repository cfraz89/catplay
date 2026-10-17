mod cbc;
mod ctr;
mod ctr_kernel;
#[cfg(feature = "openssl")]
mod ctr_openssl;
mod ctr_soft;

pub use cbc::Aes128Cbc;
pub use ctr::Aes128Ctr;
pub use ctr_kernel::{Aes128CtrKernelStream, AfAlgError};
#[cfg(feature = "openssl")]
pub use ctr_openssl::Aes128CtrOpenSsl;
pub use ctr_soft::Aes128CtrSoft;

// Android's SELinux policy denies apps AF_ALG sockets.
#[cfg(all(feature = "mips_prefers_af_alg", not(target_os = "android")))]
pub(crate) use ctr_kernel::AfAlgCtrAes128 as Aes128CtrBackend;
#[cfg(all(any(not(feature = "mips_prefers_af_alg"), target_os = "android"), feature = "openssl"))]
pub(crate) use ctr_openssl::Aes128CtrOpenSsl as Aes128CtrBackend;
#[cfg(all(any(not(feature = "mips_prefers_af_alg"), target_os = "android"), not(feature = "openssl")))]
pub(crate) use ctr_soft::Aes128CtrSoft as Aes128CtrBackend;
