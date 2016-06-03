extern crate num;

use std::cmp::Eq;
use std::ops::{BitAnd,Index,Not,Shl};
use num::{One,Zero,Unsigned};

pub struct BitVector<S = usize>
    where S: Sized + BitAnd<S, Output = S> + Shl<S, Output = S> + Not + Eq + Zero + One + Unsigned + FromUSize + Copy {
    data: Vec<S>,
    capacity: usize
}

impl<S> BitVector<S>
    where S: Sized + BitAnd<S, Output = S> + Shl<S, Output = S> + Not + Eq + Zero + One + Unsigned + FromUSize + Copy {
    pub fn with_capacity(capacity: usize) -> BitVector<S> {
        let len = (capacity / (std::mem::size_of::<S>() * 8)) + 1;
        BitVector { data: vec![S::zero(); len], capacity: capacity }
    }
}

static TRUE: bool = true;
static FALSE: bool = false;

macro_rules! bool_ref {
    ($cond:expr) => (if $cond { &TRUE } else { &FALSE })
}

impl<S> Index<usize> for BitVector<S>
    where S: Sized + BitAnd<S, Output = S> + Shl<S, Output = S> + Not + Eq + Zero + One + Unsigned + FromUSize + Copy {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        let data_index = index / (std::mem::size_of::<S>() * 8);
        let remainder = index % (std::mem::size_of::<S>() * 8);
        // we know that remainder is always smaller or equal to the size that S can hold
        // for example if S = u8 then remainder <= 2^8 - 1
        let remainder = S::from_usize(remainder);
        bool_ref!((self.data[data_index] & (S::one() << remainder)) != S::zero())
    }
}

pub trait FromUSize {
    fn from_usize(value: usize) -> Self;
}

macro_rules! impl_zero_one {
    ($($ty:ty),*) => {
        $(
            impl FromUSize for $ty {
                fn from_usize(value: usize) -> $ty {
                    value as $ty
                }
            }
        )*
    }
}

impl_zero_one! {
    u8,
    u16,
    u32,
    u64,
    usize
}

#[cfg(test)]
mod tests {
    use super::BitVector;

    #[test]
    fn test_with_capacity() {
        //TODO rewrite range checks to use iter

        let vec_32_1024 = BitVector::<u32>::with_capacity(1024);
        for i in 0..1024 {
            assert_eq!(vec_32_1024[i], false);
        }

        let vec_32_1000 = BitVector::<u32>::with_capacity(1000);
        for i in 0..1000 {
            assert_eq!(vec_32_1000[i], false);
        }
    }

    //TODO in index() check out of bounds
}
