extern crate num;

use std::cmp::Eq;
use std::ops::{BitAnd,Index,Not,Shl};
use num::{One,Zero,Unsigned,NumCast};

pub trait BitStorage: Sized + BitAnd<Self, Output = Self> + Shl<Self, Output = Self> + Not + Eq + Zero + One + Unsigned + NumCast + Copy {}

impl<S> BitStorage for S where S: Sized + BitAnd<S, Output = S> + Shl<S, Output = S> + Not + Eq + Zero + One + Unsigned + NumCast + Copy {}

pub struct BitVector<S: BitStorage = usize> {
    data: Vec<S>,
    capacity: usize
}

impl<S: BitStorage> BitVector<S> {
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

impl<S: BitStorage> Index<usize> for BitVector<S> {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        let data_index = index / (std::mem::size_of::<S>() * 8);
        let remainder = index % (std::mem::size_of::<S>() * 8);
        // we know that remainder is always smaller or equal to the size that S can hold
        // for example if S = u8 then remainder <= 2^8 - 1
        let remainder: S = num::cast(remainder).unwrap();
        bool_ref!((self.data[data_index] & (S::one() << remainder)) != S::zero())
    }
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
