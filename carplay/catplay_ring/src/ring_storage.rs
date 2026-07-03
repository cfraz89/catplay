use log::warn;
use std::{
    mem,
    ptr::{self, NonNull},
    slice,
};

pub(super) enum RingStorage<T: Copy + Default> {
    MirroredVec(Vec<T>),
    #[cfg(unix)]
    MirroredMmap(MirroredMmap<T>),
}

impl<T: Copy + Default> RingStorage<T> {
    pub fn new(capacity: usize) -> Self {
        #[cfg(unix)]
        if let Some(storage) = MirroredMmap::new(capacity) {
            return Self::MirroredMmap(storage);
        } else {
            warn!("Mirrored mmap backing (memfd/shm) unavailable, RingBuffer will operate with CPU and RAM overhead")
        }

        Self::MirroredVec(vec![T::default(); capacity * 2])
    }

    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::MirroredVec(ring) => ring.as_slice(),
            #[cfg(unix)]
            Self::MirroredMmap(ring) => ring.as_slice(),
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Self::MirroredVec(ring) => ring.as_mut_slice(),
            #[cfg(unix)]
            Self::MirroredMmap(ring) => ring.as_mut_slice(),
        }
    }

    pub fn sync_mirror_range(&mut self, cap: usize, head: usize, written_phys: usize) {
        match self {
            Self::MirroredVec(ring) => {
                let (base, mirror) = ring.split_at_mut(cap);

                let start = head;
                let end = head + written_phys;

                let b_start = start.min(cap);
                let b_end = end.min(cap);
                if b_end > b_start {
                    mirror[b_start..b_end].copy_from_slice(&base[b_start..b_end]);
                }

                let m_start = start.max(cap);
                let m_end = end.min(2 * cap);
                if m_end > m_start {
                    let src = &mirror[m_start - cap..m_end - cap];
                    let dst = &mut base[m_start - cap..m_end - cap];
                    dst.copy_from_slice(src);
                }
            }
            #[cfg(unix)]
            Self::MirroredMmap(_) => {}
        }
    }
}

#[cfg(unix)]
pub(super) fn mmap_aligned_capacity<T>(capacity: usize) -> Option<usize> {
    let elem_size = mem::size_of::<T>();
    if capacity == 0 || elem_size == 0 {
        return None;
    }

    let page_size = page_size()?;
    let elems_per_page = page_size / gcd_usize(elem_size, page_size);
    Some(capacity.max(elems_per_page))
}

#[cfg(unix)]
const fn gcd_usize(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

#[cfg(unix)]
pub(super) struct MirroredMmap<T: Copy + Default> {
    ptr: NonNull<T>,
    len: usize,
    map_len_bytes: usize,
}

#[cfg(unix)]
unsafe impl<T: Copy + Default + Send> Send for MirroredMmap<T> {}

#[cfg(unix)]
impl<T: Copy + Default> MirroredMmap<T> {
    fn new(capacity: usize) -> Option<Self> {
        let elem_size = mem::size_of::<T>();
        if capacity == 0 || elem_size == 0 {
            return None;
        }

        let page_size = page_size()?;
        let map_len_bytes = capacity.checked_mul(elem_size)?;
        if map_len_bytes == 0 || map_len_bytes % page_size != 0 {
            return None;
        }

        let reservation_len = map_len_bytes.checked_mul(2)?;
        let fd = create_backing_fd()?;

        unsafe {
            if libc::ftruncate(fd, map_len_bytes as libc::off_t) != 0 {
                libc::close(fd);
                return None;
            }

            let reservation = libc::mmap(
                ptr::null_mut(),
                reservation_len,
                libc::PROT_NONE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );
            if reservation == libc::MAP_FAILED {
                libc::close(fd);
                return None;
            }

            let first = libc::mmap(
                reservation,
                map_len_bytes,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_FIXED,
                fd,
                0,
            );
            if first == libc::MAP_FAILED {
                libc::munmap(reservation, reservation_len);
                libc::close(fd);
                return None;
            }

            let second_addr = (reservation as *mut u8).add(map_len_bytes) as *mut libc::c_void;
            let second = libc::mmap(
                second_addr,
                map_len_bytes,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_FIXED,
                fd,
                0,
            );
            if second == libc::MAP_FAILED {
                libc::munmap(reservation, reservation_len);
                libc::close(fd);
                return None;
            }

            libc::close(fd);
            let mut storage = Self {
                ptr: NonNull::new_unchecked(reservation.cast::<T>()),
                len: capacity * 2,
                map_len_bytes,
            };

            // Touch every backing page eagerly so low-memory failures happen during setup,
            // not a few seconds later on the audio write path.
            storage.as_mut_slice()[..capacity].fill(T::default());

            Some(storage)
        }
    }

    fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

#[cfg(unix)]
impl<T: Copy + Default> Drop for MirroredMmap<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = libc::munmap(self.ptr.as_ptr().cast::<libc::c_void>(), self.map_len_bytes * 2);
        }
    }
}

#[cfg(unix)]
fn page_size() -> Option<usize> {
    let value = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    (value > 0).then_some(value as usize)
}

#[cfg(unix)]
fn create_backing_fd() -> Option<libc::c_int> {
    if let Some(fd) = shm_open_unique() {
        return Some(fd);
    }

    #[cfg(target_os = "linux")]
    if let Some(fd) = memfd_create_unique() {
        return Some(fd);
    }

    None
}

#[cfg(unix)]
fn shm_open_unique() -> Option<libc::c_int> {
    use std::ffi::CString;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    for _ in 0..32 {
        let name = format!(
            "/catplay-ring-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, AtomicOrdering::Relaxed)
        );
        let name = CString::new(name).ok()?;

        let fd = unsafe { libc::shm_open(name.as_ptr(), libc::O_RDWR | libc::O_CREAT | libc::O_EXCL, 0o600) };
        if fd >= 0 {
            unsafe {
                libc::shm_unlink(name.as_ptr());
            }
            return Some(fd);
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn memfd_create_unique() -> Option<libc::c_int> {
    use std::ffi::CString;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    for _ in 0..32 {
        let name = format!(
            "catplay-ring-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, AtomicOrdering::Relaxed)
        );
        let name = CString::new(name).ok()?;

        let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
        if fd >= 0 {
            return Some(fd);
        }
    }

    None
}
