extern crate num;

use std::cmp::Eq;
use std::ops::{BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Index,Not,Shl,ShlAssign,Shr,ShrAssign};
use num::{One,Zero,Unsigned,NumCast};

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
    Eq + Zero + One + Unsigned + NumCast + Copy {}

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
    Eq + Zero + One + Unsigned + NumCast + Copy {}

pub struct BitVector<S: BitStorage> {
    data: Vec<S>,
    capacity: usize,
    storage_size: usize
}

impl<S: BitStorage> BitVector<S> {
    pub fn with_capacity(capacity: usize) -> BitVector<S> {
        let storage_size = std::mem::size_of::<S>() * 8;
        let len = (capacity / storage_size) + 1;
        BitVector { 
            data: vec![S::zero(); len],
            capacity: capacity,
            storage_size: storage_size
        }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        match self.index_in_bounds(index) {
            true => Some(self.get_unchecked(index)),
            false => None
        }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.panic_index_bounds(index);
        let (data_index, remainder) = self.compute_data_index_and_remainder(index);
        if value {
            self.data[data_index] |= S::one() << remainder;
        }
        else {
            self.data[data_index] &= !(S::one() << remainder);
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn get_unchecked(&self, index: usize) -> bool {
        let (data_index, remainder) = self.compute_data_index_and_remainder(index);
        (self.data[data_index] & (S::one() << remainder)) != S::zero()
    }

    #[inline]
    fn compute_data_index_and_remainder(&self, index: usize) -> (usize, S) {
        let data_index = index / self.storage_size;
        let remainder = index % self.storage_size;
        // we know that remainder is always smaller or equal to the size that S can hold
        // for example if S = u8 then remainder <= 2^8 - 1
        let remainder: S = num::cast(remainder).unwrap();
        (data_index, remainder)
    }

    #[inline]
    fn index_in_bounds(&self, index: usize) -> bool {
        index < self.capacity
    }

    #[inline]
    fn panic_index_bounds(&self, index: usize) {
        if !self.index_in_bounds(index) {
            panic!("Index out of bounds. Length = {}, Index = {}", self.capacity, index);
        }
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
        self.panic_index_bounds(index);
        bool_ref!(self.get_unchecked(index))
    }
}

#[cfg(test)]
mod tests {
    use super::BitVector;

    #[test]
    fn test_with_capacity() {
        //TODO rewrite range checks to use iter

        let vec_32_32 = BitVector::<u32>::with_capacity(32);
        for i in 0..32 {
            assert_eq!(vec_32_32[i], false);
        }

        let vec_32_1024 = BitVector::<u32>::with_capacity(1024);
        for i in 0..1024 {
            assert_eq!(vec_32_1024[i], false);
        }

        let vec_32_1000 = BitVector::<u32>::with_capacity(1000);
        for i in 0..1000 {
            assert_eq!(vec_32_1000[i], false);
        }
    }

    #[test]
    fn test_get_set() {
        let mut vec = BitVector::<u8>::with_capacity(16);

        vec.set(0, true);
        vec.set(1, false);
        vec.set(2, true);
        vec.set(3, false);
        vec.set(4, true);
        vec.set(5, false);
        vec.set(6, true);
        vec.set(7, true);
        vec.set(8, true);
        vec.set(9, false);
        vec.set(10, false);
        vec.set(11, false);
        vec.set(12, true);
        vec.set(13, false);
        vec.set(14, false);
        vec.set(15, true);

        assert_eq!(vec.get(0).unwrap(), true);
        assert_eq!(vec.get(1).unwrap(), false);
        assert_eq!(vec.get(2).unwrap(), true);
        assert_eq!(vec.get(3).unwrap(), false);
        assert_eq!(vec.get(4).unwrap(), true);
        assert_eq!(vec.get(5).unwrap(), false);
        assert_eq!(vec.get(6).unwrap(), true);
        assert_eq!(vec.get(7).unwrap(), true);
        assert_eq!(vec.get(8).unwrap(), true);
        assert_eq!(vec.get(9).unwrap(), false);
        assert_eq!(vec.get(10).unwrap(), false);
        assert_eq!(vec.get(11).unwrap(), false);
        assert_eq!(vec.get(12).unwrap(), true);
        assert_eq!(vec.get(13).unwrap(), false);
        assert_eq!(vec.get(14).unwrap(), false);
        assert_eq!(vec.get(15).unwrap(), true);
    }

    #[test]
    fn test_repeated_set() {
        let mut vec = BitVector::<u8>::with_capacity(16);

        for i in 0..16 {
            vec.set(i, false);
        }

        for i in 0..16 {
            assert_eq!(vec[i], false);
        }

        for i in 0..16 {
            vec.set(i, true);
        }

        for i in 0..16 {
            assert_eq!(vec[i], true);
        }

        for i in 0..16 {
            vec.set(i, false);
        }

        for i in 0..16 {
            assert_eq!(vec[i], false);
        }
    }

    #[test]
    fn test_get_out_of_bounds() {
        let vec = BitVector::<u8>::with_capacity(16);

        assert_eq!(vec.get(16), None);
    }

    #[test]
    #[should_panic]
    fn test_set_out_of_bounds() {
        let mut vec = BitVector::<u8>::with_capacity(16);

        vec.set(16, true);    
    }

    #[test]
    fn test_index() {
        let mut vec = BitVector::<u8>::with_capacity(16);

        vec.set(0, true);
        vec.set(1, false);
        vec.set(2, true);
        vec.set(3, false);
        vec.set(4, true);
        vec.set(5, false);
        vec.set(6, true);
        vec.set(7, true);
        vec.set(8, true);
        vec.set(9, false);
        vec.set(10, false);
        vec.set(11, false);
        vec.set(12, true);
        vec.set(13, false);
        vec.set(14, false);
        vec.set(15, true);

        assert_eq!(vec[0], true);
        assert_eq!(vec[1], false);
        assert_eq!(vec[2], true);
        assert_eq!(vec[3], false);
        assert_eq!(vec[4], true);
        assert_eq!(vec[5], false);
        assert_eq!(vec[6], true);
        assert_eq!(vec[7], true);
        assert_eq!(vec[8], true);
        assert_eq!(vec[9], false);
        assert_eq!(vec[10], false);
        assert_eq!(vec[11], false);
        assert_eq!(vec[12], true);
        assert_eq!(vec[13], false);
        assert_eq!(vec[14], false);
        assert_eq!(vec[15], true);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let vec = BitVector::<u8>::with_capacity(16);

        vec[16];
    }

    #[test]
    fn test_capacity() {
        let vec_1000: BitVector<usize> = BitVector::with_capacity(1000);
        assert_eq!(vec_1000.capacity(), 1000);

        let vec_1024: BitVector<usize> = BitVector::with_capacity(1024);
        assert_eq!(vec_1024.capacity(), 1024);
    }
}
