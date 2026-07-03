/// Part of `RingBuffer` used to represents gaps in the timeline.
pub struct Bitmap {
    gap_bitmap: Vec<u64>,
    capacity: usize,
    empty: bool,
}

#[allow(clippy::needless_range_loop)]
impl Bitmap {
    pub fn new(capacity: usize) -> Self {
        let words = capacity.div_ceil(64);
        let gap_bitmap = vec![0u64; words];

        Self {
            gap_bitmap,
            capacity,
            empty: true,
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.empty
    }

    #[cfg(test)]
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn mark_region(&mut self, start: usize, len: usize) {
        if len == 0 {
            return;
        }

        debug_assert!(start + len <= self.capacity);

        let end = start + len;

        let bm = &mut self.gap_bitmap;

        let first_word = start >> 6;
        let last_word = (end - 1) >> 6;

        let start_bit = start & 63;
        let first_mask = if first_word == last_word {
            let end_bit = end & 63;
            if end_bit == 0 {
                !((1u64 << start_bit) - 1)
            } else {
                ((1u64 << end_bit) - 1) & !((1u64 << start_bit) - 1)
            }
        } else {
            !((1u64 << start_bit) - 1)
        };
        bm[first_word] |= first_mask;

        for w in (first_word + 1)..last_word {
            bm[w] = u64::MAX;
        }

        if last_word != first_word {
            let end_bit = end & 63;
            let last_mask = if end_bit == 0 { u64::MAX } else { (1u64 << end_bit) - 1 };
            bm[last_word] |= last_mask;
        }

        self.empty = false;
    }

    pub fn unmark_region(&mut self, start: usize, len: usize) {
        if len == 0 || self.is_empty() {
            return;
        }

        debug_assert!(start + len <= self.capacity);

        let end = start + len;
        let mut still_non_empty = false;

        let bm = &mut self.gap_bitmap;

        let first_word = start >> 6;
        let last_word = (end - 1) >> 6;

        let start_bit = start & 63;
        let first_mask = if first_word == last_word {
            let end_bit = end & 63;
            if end_bit == 0 {
                !((1u64 << start_bit) - 1)
            } else {
                ((1u64 << end_bit) - 1) & !((1u64 << start_bit) - 1)
            }
        } else {
            !((1u64 << start_bit) - 1)
        };
        bm[first_word] &= !first_mask;

        for w in (first_word + 1)..last_word {
            bm[w] = 0;
        }

        if last_word != first_word {
            let end_bit = end & 63;
            let last_mask = if end_bit == 0 { u64::MAX } else { (1u64 << end_bit) - 1 };
            bm[last_word] &= !last_mask;
        }

        for &w in bm.iter() {
            if w != 0 {
                still_non_empty = true;
                break;
            }
        }

        if !still_non_empty {
            self.empty = true;
        }
    }

    pub fn search_next_gap(&mut self, start: usize, len: usize) -> Option<(usize, usize)> {
        if len == 0 || self.is_empty() {
            return None;
        }

        debug_assert!(start + len <= self.capacity);

        let end = start + len;

        let bm = &mut self.gap_bitmap;

        let first_word = start >> 6;
        let last_word = (end - 1) >> 6;

        let get_masked_word = |wi: usize| -> u64 {
            let mut w = bm[wi];

            if wi == first_word {
                let lo = start & 63;
                if lo != 0 {
                    w &= !((1u64 << lo) - 1);
                }
            }
            if wi == last_word {
                let hi = end & 63;
                if hi != 0 {
                    w &= (1u64 << hi) - 1;
                }
            }

            w
        };

        let mut wi = first_word;
        let mut w = get_masked_word(wi);

        while w == 0 {
            if wi == last_word {
                return None;
            }
            wi += 1;
            w = get_masked_word(wi);
        }

        let tz = w.trailing_zeros() as usize;
        let gap_start = (wi << 6) + tz;

        let mut run = 0usize;

        let w_tail = w >> tz;
        let ones_here = (!w_tail).trailing_zeros() as usize;
        let bits_available_in_word = 64 - tz;
        let mut take = ones_here.min(bits_available_in_word);

        let max_total = end - gap_start;
        if take > max_total {
            take = max_total;
        }
        run += take;

        if run == max_total {
            return Some((gap_start, run));
        }

        if take < bits_available_in_word {
            return Some((gap_start, run));
        }

        let mut next_wi = wi + 1;
        while gap_start + run < end && next_wi <= last_word {
            let ww = get_masked_word(next_wi);

            if ww == u64::MAX {
                let remaining = end - (gap_start + run);
                let add = remaining.min(64);
                run += add;
                if run == max_total {
                    return Some((gap_start, run));
                }
                next_wi += 1;
                continue;
            }

            if ww == 0 {
                break;
            }

            let ones = (!ww).trailing_zeros() as usize;
            let remaining = end - (gap_start + run);
            let add = ones.min(remaining).min(64);
            run += add;
            break;
        }

        Some((gap_start, run))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_new_is_empty_and_exposes_capacity() {
        let bm = Bitmap::new(128);
        assert!(bm.is_empty());
        assert!(bm.capacity() == 128);
    }

    #[test]
    fn mark_single_region() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(10, 5);

        assert!(!bm.is_empty());
        assert_eq!(bm.search_next_gap(0, 128), Some((10, 5)));
    }

    #[test]
    fn unmark_single_region() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(10, 5);
        bm.unmark_region(10, 5);

        assert!(bm.is_empty());
        assert_eq!(bm.search_next_gap(0, 128), None);
    }

    #[test]
    fn mark_multiple_regions() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(10, 5);
        bm.mark_region(30, 3);

        assert_eq!(bm.search_next_gap(0, 128), Some((10, 5)));

        bm.unmark_region(10, 5);
        assert_eq!(bm.search_next_gap(0, 128), Some((30, 3)));

        bm.unmark_region(30, 3);
        assert!(bm.is_empty());
    }

    #[test]
    fn partial_unmark_keeps_gap() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(20, 10);
        bm.unmark_region(20, 5);

        assert!(!bm.is_empty());
        assert_eq!(bm.search_next_gap(0, 128), Some((25, 5)));
    }

    #[test]
    fn search_respects_start_offset() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(10, 5);
        bm.mark_region(30, 5);

        assert_eq!(bm.search_next_gap(0, 128), Some((10, 5)));
        assert_eq!(bm.search_next_gap(15, 128 - 15), Some((30, 5)));
    }

    #[test]
    fn search_limited_range() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(10, 5);
        bm.mark_region(30, 5);

        assert_eq!(bm.search_next_gap(0, 20), Some((10, 5)));
        assert_eq!(bm.search_next_gap(0, 10), None);
        assert_eq!(bm.search_next_gap(20, 10), None);
        assert_eq!(bm.search_next_gap(20, 20), Some((30, 5)));
    }

    #[test]
    fn gap_at_zero() {
        let mut bm = Bitmap::new(64);

        bm.mark_region(0, 8);

        assert_eq!(bm.search_next_gap(0, 64), Some((0, 8)));
    }

    #[test]
    fn gap_at_end() {
        let mut bm = Bitmap::new(64);

        bm.mark_region(60, 4);

        assert_eq!(bm.search_next_gap(0, 64), Some((60, 4)));
        assert_eq!(bm.search_next_gap(0, 60), None);
    }

    #[test]
    fn multiple_adjacent_marks_merge_logically() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(10, 5);
        bm.mark_region(15, 5);

        assert_eq!(bm.search_next_gap(0, 128), Some((10, 10)));

        bm.unmark_region(10, 10);
        assert!(bm.is_empty());
    }

    #[test]
    fn unmark_middle_splits_gap() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(10, 10);
        bm.unmark_region(14, 2);

        assert_eq!(bm.search_next_gap(0, 128), Some((10, 4)));

        bm.unmark_region(10, 4);
        assert_eq!(bm.search_next_gap(0, 128), Some((16, 4)));
    }

    #[test]
    fn repeated_mark_same_region_is_idempotent() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(20, 5);
        bm.mark_region(20, 5);

        assert_eq!(bm.search_next_gap(0, 128), Some((20, 5)));
    }

    #[test]
    fn repeated_unmark_same_region_is_idempotent() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(20, 5);
        bm.unmark_region(20, 5);
        bm.unmark_region(20, 5);

        assert!(bm.is_empty());
    }

    #[test]
    fn empty_flag_consistency() {
        let mut bm = Bitmap::new(128);

        assert!(bm.is_empty());

        bm.mark_region(1, 1);
        assert!(!bm.is_empty());

        bm.unmark_region(1, 1);
        assert!(bm.is_empty());
    }

    #[test]
    fn search_does_not_overflow_range() {
        let mut bm = Bitmap::new(128);

        bm.mark_region(100, 10);

        assert_eq!(bm.search_next_gap(0, 50), None);
        assert_eq!(bm.search_next_gap(90, 10), None);
        assert_eq!(bm.search_next_gap(90, 20), Some((100, 10)));
    }

    #[test]
    fn gap_crosses_word_boundary() {
        let mut bm = Bitmap::new(256);

        bm.mark_region(60, 10);

        assert_eq!(bm.search_next_gap(0, 256), Some((60, 10)));
    }

    #[test]
    fn gap_spans_multiple_full_words() {
        let mut bm = Bitmap::new(512);

        bm.mark_region(32, 160);

        assert_eq!(bm.search_next_gap(0, 512), Some((32, 160)));
    }

    #[test]
    fn unmark_splits_multiword_gap() {
        let mut bm = Bitmap::new(256);

        bm.mark_region(20, 200);
        bm.unmark_region(100, 20);

        assert_eq!(bm.search_next_gap(0, 256), Some((20, 80)));

        bm.unmark_region(20, 80);
        assert_eq!(bm.search_next_gap(0, 256), Some((120, 100)));
    }

    #[test]
    fn search_crosses_words_with_offset() {
        let mut bm = Bitmap::new(256);

        bm.mark_region(10, 100);

        assert_eq!(bm.search_next_gap(50, 40), Some((50, 40)));
    }
}
