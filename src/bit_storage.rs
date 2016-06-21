use std::mem;
use std::ops::{BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Not,Shl,ShlAssign,Shr,ShrAssign};
use num;
use num::{One,Zero,Unsigned,NumCast,Bounded};

pub trait BitStorage: Sized +
    BitAnd<Self, Output = Self> +
    BitAndAssign<Self> +
    BitOr<Self, Output = Self> +
    BitOrAssign<Self> +
    BitXor<Self, Output = Self> +
    BitXorAssign<Self> +
    Not<Output = Self> +
    Shl<Self, Output = Self> +
    ShlAssign<Self> +
    Shr<Self, Output = Self> +
    ShrAssign<Self> +
    Eq + Zero + One + Unsigned + NumCast + Bounded + Copy {
        fn storage_size() -> usize;

        fn set(storage: &mut Self, storage_index: Self, value: bool);

        fn get(storage: &Self, storage_index: Self) -> bool;

        fn compute_data_index(index: usize) -> usize;

        fn compute_remainder(index: usize) -> Self;

        fn compute_data_index_and_remainder(index: usize) -> (usize, Self);
}

impl<S> BitStorage for S where S: Sized +
    BitAnd<S, Output = S> +
    BitAndAssign<S> +
    BitOr<S, Output = S> +
    BitOrAssign<S> +
    BitXor<S, Output = S> +
    BitXorAssign<S> +
    Not<Output = S> +
    Shl<S, Output = S> +
    ShlAssign<S> +
    Shr<S, Output = S> +
    ShrAssign<S> +
    Eq + Zero + One + Unsigned + NumCast + Bounded + Copy {
        #[inline]
        fn storage_size() -> usize {
            mem::size_of::<S>() * 8
        }

        #[inline]
        fn set(storage: &mut S, storage_index: S, value: bool) {
            if value {
                *storage |= S::one() << storage_index;
            }
            else {
                *storage &= !(S::one() << storage_index);
            }
        }

        #[inline]
        fn get(storage: &S, storage_index: S) -> bool {
            (*storage & (S::one() << storage_index)) != S::zero()
        }

        #[inline]
        fn compute_data_index(index: usize) -> usize {
            index / S::storage_size()
        }

        #[inline]
        fn compute_remainder(index: usize) -> S {
            let remainder = index % S::storage_size();
            // we know that remainder is always smaller or equal to the size that S can hold
            // for example if S = u8 then remainder <= 2^8 - 1
            let remainder: S = num::cast(remainder).unwrap();
            remainder
        }

        #[inline]
        fn compute_data_index_and_remainder(index: usize) -> (usize, S) {
            (S::compute_data_index(index), S::compute_remainder(index))
        }
}

#[cfg(test)]
mod tests {
    use super::BitStorage;

    #[test]
    fn test_storage_size() {
        assert_eq!(u8::storage_size(), 8);
    }

    #[test]
    fn test_set() {
        let mut byte = 0b01010101;

        u8::set(&mut byte, 3, true);
        assert_eq!(byte, 0b01011101);

        u8::set(&mut byte, 3, true);
        assert_eq!(byte, 0b01011101);

        u8::set(&mut byte, 6, false);
        assert_eq!(byte, 0b00011101);

        u8::set(&mut byte, 6, false);
        assert_eq!(byte, 0b00011101);
    }

    #[test]
    fn test_get() {
        let byte = 0b01010101;

        assert_eq!(true, u8::get(&byte, 2));
        assert_eq!(false, u8::get(&byte, 5));
    }

    #[test]
    fn test_compute_data_index() {
        assert_eq!(0, u8::compute_data_index(0));
        assert_eq!(0, u8::compute_data_index(4));
        assert_eq!(0, u8::compute_data_index(7));
        assert_eq!(1, u8::compute_data_index(8));
        assert_eq!(1, u8::compute_data_index(15));
        assert_eq!(2, u8::compute_data_index(16));
    }

    #[test]
    fn test_compute_remainder() {
        assert_eq!(0, u8::compute_remainder(0));
        assert_eq!(4, u8::compute_remainder(4));
        assert_eq!(7, u8::compute_remainder(7));
        assert_eq!(0, u8::compute_remainder(8));
        assert_eq!(7, u8::compute_remainder(15));
        assert_eq!(0, u8::compute_remainder(16));
    }

    #[test]
    fn test_compute_data_index_and_remainder() {
        assert_eq!((0, 0), u8::compute_data_index_and_remainder(0));
        assert_eq!((0, 4), u8::compute_data_index_and_remainder(4));
        assert_eq!((0, 7), u8::compute_data_index_and_remainder(7));
        assert_eq!((1, 0), u8::compute_data_index_and_remainder(8));
        assert_eq!((1, 7), u8::compute_data_index_and_remainder(15));
        assert_eq!((2, 0), u8::compute_data_index_and_remainder(16));
    }
}