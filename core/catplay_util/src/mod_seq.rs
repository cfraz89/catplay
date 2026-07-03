use core::fmt;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ModSeq<T: Ord + Sized>(pub T);
pub type ModSeq8 = ModSeq<u8>;
pub type ModSeq16 = ModSeq<u16>;
pub type ModSeq32 = ModSeq<u32>;

impl<T: Ord + Sized> ModSeq<T> {
    pub fn value(&self) -> T
    where
        T: Copy,
    {
        self.0
    }
}

impl<T: fmt::Display + Ord + Sized> fmt::Display for ModSeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModSeq({})", self.0)
    }
}

impl<T: fmt::Display + Ord + Sized> fmt::Debug for ModSeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModSeq({})", self.0)
    }
}

macro_rules! impl_modseq {
    ($uint:ty, $int:ty) => {
        impl ModSeq<$uint> {
            const HALF: $uint = 1 << (<$uint>::BITS - 1);
        }

        impl From<ModSeq<$uint>> for $uint {
            #[inline]
            fn from(val: ModSeq<$uint>) -> Self {
                val.0
            }
        }

        impl PartialOrd for ModSeq<$uint> {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                match self.0.wrapping_sub(other.0) {
                    0 => Some(std::cmp::Ordering::Equal),
                    d if d == Self::HALF => None,
                    d if d < Self::HALF => Some(std::cmp::Ordering::Greater),
                    _ => Some(std::cmp::Ordering::Less),
                }
            }
        }

        impl std::ops::Add<$uint> for ModSeq<$uint> {
            type Output = ModSeq<$uint>;
            #[inline]
            fn add(self, rhs: $uint) -> Self::Output {
                ModSeq(self.0.wrapping_add(rhs))
            }
        }

        impl std::ops::Sub<$uint> for ModSeq<$uint> {
            type Output = ModSeq<$uint>;
            #[inline]
            fn sub(self, rhs: $uint) -> Self::Output {
                ModSeq(self.0.wrapping_sub(rhs))
            }
        }

        impl std::ops::Add<ModSeq<$uint>> for ModSeq<$uint> {
            type Output = ModSeq<$uint>;
            #[inline]
            fn add(self, rhs: ModSeq<$uint>) -> Self::Output {
                self + rhs.0
            }
        }

        impl std::ops::Sub<ModSeq<$uint>> for ModSeq<$uint> {
            type Output = ModSeq<$uint>;
            #[inline]
            fn sub(self, rhs: ModSeq<$uint>) -> Self::Output {
                self - rhs.0
            }
        }

        impl ModSeq<$uint> {
            #[inline]
            pub fn distance(self, other: Self) -> $int {
                use std::cmp::Ordering::*;

                match self.partial_cmp(&other) {
                    Some(Equal) => 0 as $int,
                    Some(Greater) => self.0.wrapping_sub(other.0) as $int,
                    Some(Less) => -((other.0.wrapping_sub(self.0)) as $int),
                    None => 0 as $int,
                }
            }
        }
    };
}

impl_modseq!(u8, i8);
impl_modseq!(u16, i16);
impl_modseq!(u32, i32);
impl_modseq!(u64, i64);
impl_modseq!(usize, isize);

#[cfg(test)]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
mod tests {
    use super::ModSeq;
    use core::cmp::Ordering;
    use paste::paste;

    macro_rules! modseq_tests {
        ($T:ty, $prefix:ident) => {
            paste! {
                #[test]
                fn [<$prefix _off_by_one>]() {
                    assert!(!(ModSeq(1 as $T) > ModSeq(1 as $T)));
                    assert!(!(ModSeq(1 as $T) < ModSeq(1 as $T)));
                    assert!(ModSeq(1 as $T) >= ModSeq(1 as $T));
                    assert!(ModSeq(1 as $T) <= ModSeq(1 as $T));
                }

                #[test]
                fn [<$prefix _simple_order>]() {
                    assert!(ModSeq(2 as $T) > ModSeq(1 as $T));
                    assert!(ModSeq(1 as $T) < ModSeq(2 as $T));
                }

                #[test]
                fn [<$prefix _wraparound_forward>]() {
                    assert!(ModSeq(0 as $T) > ModSeq(<$T>::MAX));
                    assert!(ModSeq(<$T>::MAX) < ModSeq(0 as $T));
                }

                #[test]
                fn [<$prefix _wraparound_backward>]() {
                    assert!(!(ModSeq(<$T>::MAX) > ModSeq(0 as $T)));
                    assert!(!(ModSeq(0 as $T) < ModSeq(<$T>::MAX)));
                }

                #[test]
                fn [<$prefix _far_ahead_vs_behind>]() {
                    let half = (<$T>::MAX / 2) + 2;
                    assert!(!(ModSeq(half) > ModSeq(0 as $T)));
                    assert!(ModSeq(half) < ModSeq(0 as $T));
                    assert!(ModSeq(0 as $T) > ModSeq(half));
                    assert!(!(ModSeq(0 as $T) < ModSeq(half)));
                }

                #[test]
                fn [<$prefix _add_sub>]() {
                    assert_eq!(ModSeq(10 as $T) + 5, ModSeq(15 as $T));
                    assert_eq!(ModSeq(<$T>::MAX - 5) + 10, ModSeq(4 as $T));
                    assert_eq!(ModSeq(10 as $T) - 5, ModSeq(5 as $T));
                    assert_eq!(ModSeq(0 as $T) - 1, ModSeq(<$T>::MAX));
                }

                #[test]
                fn [<$prefix _equality>]() {
                    assert!(ModSeq(42 as $T) == ModSeq(42 as $T));
                    assert!(ModSeq(42 as $T) != ModSeq(43 as $T));
                }

                #[test]
                fn [<$prefix _invalid_deltas>]() {
                    let half: $T = (<$T>::MAX / 2) as $T;
                    let half_plus_one: $T = half.wrapping_add(1 as $T);

                    assert!(ModSeq(half) > ModSeq(0 as $T));
                    assert!(ModSeq(0 as $T) < ModSeq(half));

                    assert!(ModSeq(half_plus_one).partial_cmp(&ModSeq(0 as $T)).is_none());
                    assert!(ModSeq(0 as $T).partial_cmp(&ModSeq(half_plus_one)).is_none());
                }

                #[test]
                fn [<$prefix _distance_wraparound_forward>]() {
                    let d = ModSeq(0 as $T).distance(ModSeq(<$T>::MAX));
                    assert!(d > 0, "distance should be positive, got {}", d);
                    assert_eq!(d, 1);
                }

                #[test]
                fn [<$prefix _distance_wraparound_backward>]() {
                    let d = ModSeq(<$T>::MAX).distance(ModSeq(0 as $T));
                    assert!(d < 0, "distance should be negative, got {}", d);
                    assert_eq!(d, -1);
                }

                #[test]
                fn [<$prefix _distance_zero>]() {
                    let x = 123 as $T;
                    assert_eq!(ModSeq(x).distance(ModSeq(x)), 0);
                }

                #[test]
                fn [<$prefix _distance_simple_forward>]() {
                    let d = ModSeq(10 as $T).distance(ModSeq(7 as $T));
                    assert_eq!(d, 3);
                }

                #[test]
                fn [<$prefix _distance_simple_backward>]() {
                    let d = ModSeq(7 as $T).distance(ModSeq(10 as $T));
                    assert_eq!(d, -3);
                }

                #[test]
                fn [<$prefix _distance_symmetry>]() {
                    let a = ModSeq(42 as $T);
                    let b = ModSeq(17 as $T);

                    let dab = a.distance(b);
                    let dba = b.distance(a);

                    assert_eq!(dab, -dba);
                }

                #[test]
                fn [<$prefix _distance_half_range>]() {
                    let half = (<$T>::MAX / 2) as $T;

                    let d1 = ModSeq(half).distance(ModSeq(0 as $T));
                    let d2 = ModSeq(0 as $T).distance(ModSeq(half));

                    assert!(d1 > 0 || d2 > 0);
                    assert!(d1 < 0 || d2 < 0);
                }

                #[test]
                fn [<$prefix _distance_half_range_plus_one>]() {
                    let half = (<$T>::MAX / 2) as $T;
                    let half_plus_one: $T = half.wrapping_add(1 as $T);

                    let d1 = ModSeq(half_plus_one).distance(ModSeq(0 as $T));
                    let d2 = ModSeq(0 as $T).distance(ModSeq(half_plus_one));

                    assert_eq!(d1, 0);
                    assert_eq!(d2, 0);
                }

                #[test]
                fn [<$prefix _distance_matches_ordering>]() {
                    let a = ModSeq(5 as $T);
                    let b = ModSeq(250 as $T);

                    match a.partial_cmp(&b).unwrap() {
                        Ordering::Greater => assert!(a.distance(b) > 0),
                        Ordering::Less => assert!(a.distance(b) < 0),
                        Ordering::Equal => assert_eq!(a.distance(b), 0),
                    }
                }
            }
        };
    }

    modseq_tests!(u8, u8);
    modseq_tests!(u16, u16);
    modseq_tests!(u32, u32);
    modseq_tests!(u64, u64);
    modseq_tests!(usize, usize);
}
