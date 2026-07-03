use bytes::BytesMut;

pub struct BytesMutUtil(());

impl BytesMutUtil {
    /// Ensure capacity for at least `n` bytes in front of the read head,
    /// including bytes already written, but not yet read.
    pub fn ensure_writable(b: &mut BytesMut, n: usize) {
        let missing = n.saturating_sub(b.len());
        if missing > 0 {
            b.reserve(missing);
        }
    }

    pub fn adjacent_slice_mut<'a>(buffers: &mut [&'a mut BytesMut]) -> Option<&'a mut [u8]> {
        let mut start_ptr = core::ptr::null_mut();
        let mut previous_end = None;
        let mut total_len = 0usize;

        for buf in buffers.iter_mut() {
            let len = buf.len();
            if len == 0 {
                continue;
            }

            let ptr = buf.as_mut_ptr();
            if let Some(previous_end) = previous_end {
                if previous_end != ptr {
                    return None;
                }
            } else {
                start_ptr = ptr;
            }

            previous_end = Some(unsafe { ptr.add(len) });
            total_len = total_len.checked_add(len)?;
        }

        if total_len == 0 {
            let ptr = buffers.first_mut()?.as_mut_ptr();
            return Some(unsafe { core::slice::from_raw_parts_mut(ptr, 0) });
        }

        // SAFETY:
        // - `start_ptr` starts at the first non-empty segment.
        // - We verified every non-empty segment is physically contiguous with the previous one.
        // - Empty segments are ignored and do not contribute to the returned range.
        // - `total_len` is checked and spans exactly the non-empty contiguous segments.
        Some(unsafe { core::slice::from_raw_parts_mut(start_ptr, total_len) })
    }
}
