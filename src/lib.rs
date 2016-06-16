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

//TODO wait on custom DST on stable and deref BitVector into BitSlice resp BitSliceMut and implement non-structural changing methods on BitSlice/BitSliceMut

//TODO should BitVector, BitSlice and BitSliceMut be in different files?

//TODO add storage_size() method with #inline -> will lead to constant after compilation
pub struct BitVector<S: BitStorage> {
    data: Vec<S>,
    capacity: usize,
    storage_size: usize
}

//TODO add storage_size() method with #inline -> will lead to constant after compilation
pub struct BitSlice<S: BitStorage> {
    pointer: *const S,
    capacity: usize,
    storage_size: usize
}

//TODO add storage_size() method with #inline -> will lead to constant after compilation
pub struct BitSliceMut<S: BitStorage> {
    pointer: *mut S,
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

    pub fn split_at(&self, index: usize) -> (BitSlice<S>, BitSlice<S>) {
        self.panic_index_not_on_storage_bound(index);
        let data_index = self.compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (data_left, data_right) = self.data.split_at(data_index);

        let left = BitSlice {
            pointer: data_left.as_ptr(),
            capacity: capacity_left,
            storage_size: self.storage_size
        };
        let right = BitSlice {
            pointer: data_right.as_ptr(),
            capacity: capacity_right,
            storage_size: self.storage_size
        };
        (left, right)
    }

    pub fn split_at_mut(&mut self, index: usize) -> (BitSliceMut<S>, BitSliceMut<S>) {
        self.panic_index_not_on_storage_bound(index);
        let data_index = self.compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (data_left, data_right) = self.data.split_at_mut(data_index);

        let left = BitSliceMut {
            pointer: data_left.as_mut_ptr(),
            capacity: capacity_left,
            storage_size: self.storage_size
        };
        let right = BitSliceMut {
            pointer: data_right.as_mut_ptr(),
            capacity: capacity_right,
            storage_size: self.storage_size
        };
        (left, right)
    }

    #[inline]
    fn get_unchecked(&self, index: usize) -> bool {
        let (data_index, remainder) = self.compute_data_index_and_remainder(index);
        (self.data[data_index] & (S::one() << remainder)) != S::zero()
    }

    #[inline]
    fn compute_data_index_and_remainder(&self, index: usize) -> (usize, S) {
        let data_index = self.compute_data_index(index);
        let remainder = self.compute_data_remainder(index);
        (data_index, remainder)
    }

    #[inline]
    fn compute_data_index(&self, index: usize) -> usize {
        index / self.storage_size
    }

    #[inline]
    fn compute_data_remainder(&self, index: usize) -> S {
        let remainder = index % self.storage_size;
        // we know that remainder is always smaller or equal to the size that S can hold
        // for example if S = u8 then remainder <= 2^8 - 1
        let remainder: S = num::cast(remainder).unwrap();
        remainder
    }

    #[inline]
    fn compute_capacities(&self, index_to_split: usize) -> (usize, usize) {
        (index_to_split, self.capacity - index_to_split)
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

    #[inline]
    fn panic_index_not_on_storage_bound(&self, index: usize) {
        if index % self.storage_size != 0 {
            panic!("Index not on storage bound. Storage size = {}, Index = {}", self.storage_size, index);
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

impl<S: BitStorage> BitSlice<S> {
    pub fn get(&self, index: usize) -> Option<bool> {
        match self.index_in_bounds(index) {
            true => Some(self.get_unchecked(index)),
            false => None
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn split_at(&self, index: usize) -> (BitSlice<S>, BitSlice<S>) {
        self.panic_index_not_on_storage_bound(index);
        let data_index = self.compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (pointer_left, pointer_right) = self.compute_pointers(data_index);

        let left = BitSlice {
            pointer: pointer_left,
            capacity: capacity_left,
            storage_size: self.storage_size
        };
        let right = BitSlice {
            pointer: pointer_right,
            capacity: capacity_right,
            storage_size: self.storage_size
        };
        (left, right)
    }

    #[inline]
    fn get_unchecked(&self, index: usize) -> bool {
        let (data_index, remainder) = self.compute_data_index_and_remainder(index);
        let element = unsafe { *self.pointer.offset(data_index as isize) };
        (element & (S::one() << remainder)) != S::zero()
    }

    #[inline]
    fn compute_data_index_and_remainder(&self, index: usize) -> (usize, S) {
        let data_index = self.compute_data_index(index);
        let remainder = self.compute_data_remainder(index);
        (data_index, remainder)
    }

    #[inline]
    fn compute_data_index(&self, index: usize) -> usize {
        index / self.storage_size
    }

    #[inline]
    fn compute_data_remainder(&self, index: usize) -> S {
        let remainder = index % self.storage_size;
        // we know that remainder is always smaller or equal to the size that S can hold
        // for example if S = u8 then remainder <= 2^8 - 1
        let remainder: S = num::cast(remainder).unwrap();
        remainder
    }

    #[inline]
    fn compute_capacities(&self, index_to_split: usize) -> (usize, usize) {
        (index_to_split, self.capacity - index_to_split)
    }

    #[inline]
    fn compute_pointers(&self, data_index_to_split: usize) -> (*const S, *const S) {
        let pointer_left = self.pointer;
        let pointer_right = unsafe { self.pointer.offset(data_index_to_split as isize) };
        (pointer_left, pointer_right)
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

    #[inline]
    fn panic_index_not_on_storage_bound(&self, index: usize) {
        if index % self.storage_size != 0 {
            panic!("Index not on storage bound. Storage size = {}, Index = {}", self.storage_size, index);
        }
    }
}

impl<S: BitStorage> Index<usize> for BitSlice<S> {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        self.panic_index_bounds(index);
        bool_ref!(self.get_unchecked(index))
    }
}

impl<S: BitStorage> BitSliceMut<S> {
    pub fn get(&self, index: usize) -> Option<bool> {
        match self.index_in_bounds(index) {
            true => Some(self.get_unchecked(index)),
            false => None
        }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.panic_index_bounds(index);
        let (data_index, remainder) = self.compute_data_index_and_remainder(index);
        unsafe {
            let element_pointer = self.pointer.offset(data_index as isize);
            let mut element = *element_pointer;
            if value {
                element |= S::one() << remainder;
            }
            else {
                element &= !(S::one() << remainder);
            }
            std::ptr::write(element_pointer, element);
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn split_at(&self, index: usize) -> (BitSlice<S>, BitSlice<S>) {
        self.panic_index_not_on_storage_bound(index);
        let data_index = self.compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (pointer_left, pointer_right) = self.compute_pointers(data_index);

        let left = BitSlice {
            pointer: pointer_left,
            capacity: capacity_left,
            storage_size: self.storage_size
        };
        let right = BitSlice {
            pointer: pointer_right,
            capacity: capacity_right,
            storage_size: self.storage_size
        };
        (left, right)
    }

    pub fn split_at_mut(&mut self, index: usize) -> (BitSliceMut<S>, BitSliceMut<S>) {
        self.panic_index_not_on_storage_bound(index);
        let data_index = self.compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (pointer_left, pointer_right) = self.compute_mut_pointers(data_index);

        let left = BitSliceMut {
            pointer: pointer_left,
            capacity: capacity_left,
            storage_size: self.storage_size
        };
        let right = BitSliceMut {
            pointer: pointer_right,
            capacity: capacity_right,
            storage_size: self.storage_size
        };
        (left, right)
    }

    #[inline]
    fn get_unchecked(&self, index: usize) -> bool {
        let (data_index, remainder) = self.compute_data_index_and_remainder(index);
        let element = unsafe { *self.pointer.offset(data_index as isize) };
        (element & (S::one() << remainder)) != S::zero()
    }

    #[inline]
    fn compute_data_index_and_remainder(&self, index: usize) -> (usize, S) {
        let data_index = self.compute_data_index(index);
        let remainder = self.compute_data_remainder(index);
        (data_index, remainder)
    }

    #[inline]
    fn compute_data_index(&self, index: usize) -> usize {
        index / self.storage_size
    }

    #[inline]
    fn compute_data_remainder(&self, index: usize) -> S {
        let remainder = index % self.storage_size;
        // we know that remainder is always smaller or equal to the size that S can hold
        // for example if S = u8 then remainder <= 2^8 - 1
        let remainder: S = num::cast(remainder).unwrap();
        remainder
    }

    #[inline]
    fn compute_capacities(&self, index_to_split: usize) -> (usize, usize) {
        (index_to_split, self.capacity - index_to_split)
    }

    #[inline]
    fn compute_pointers(&self, data_index_to_split: usize) -> (*const S, *const S) {
        let pointer_left = self.pointer;
        let pointer_right = unsafe { self.pointer.offset(data_index_to_split as isize) };
        (pointer_left, pointer_right)
    }

    #[inline]
    fn compute_mut_pointers(&self, data_index_to_split: usize) -> (*mut S, *mut S) {
        let (pointer_left, pointer_right) = self.compute_pointers(data_index_to_split);
        (pointer_left as *mut _, pointer_right as *mut _)
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

    #[inline]
    fn panic_index_not_on_storage_bound(&self, index: usize) {
        if index % self.storage_size != 0 {
            panic!("Index not on storage bound. Storage size = {}, Index = {}", self.storage_size, index);
        }
    }
}

impl<S: BitStorage> Index<usize> for BitSliceMut<S> {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        self.panic_index_bounds(index);
        bool_ref!(self.get_unchecked(index))
    }
}

#[cfg(test)]
mod tests_bitvector {
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

    #[test]
    fn test_split_at() {
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

        let (left, right) = vec.split_at(8);

        assert_eq!(left[0], true);
        assert_eq!(left[1], false);
        assert_eq!(left[2], true);
        assert_eq!(left[3], false);
        assert_eq!(left[4], true);
        assert_eq!(left[5], false);
        assert_eq!(left[6], true);
        assert_eq!(left[7], true);

        assert_eq!(right[0], true);
        assert_eq!(right[1], false);
        assert_eq!(right[2], false);
        assert_eq!(right[3], false);
        assert_eq!(right[4], true);
        assert_eq!(right[5], false);
        assert_eq!(right[6], false);
        assert_eq!(right[7], true);
    }

    #[test]
    #[should_panic]
    fn test_split_at_not_on_storage_bound() {
        let vec = BitVector::<u8>::with_capacity(16);
        vec.split_at(4);
    }

    #[test]
    fn test_split_at_mut() {
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

        let (mut left, mut right) = vec.split_at_mut(8);

        assert_eq!(left[0], true);
        assert_eq!(left[1], false);
        assert_eq!(left[2], true);
        assert_eq!(left[3], false);
        assert_eq!(left[4], true);
        assert_eq!(left[5], false);
        assert_eq!(left[6], true);
        assert_eq!(left[7], true);

        assert_eq!(right[0], true);
        assert_eq!(right[1], false);
        assert_eq!(right[2], false);
        assert_eq!(right[3], false);
        assert_eq!(right[4], true);
        assert_eq!(right[5], false);
        assert_eq!(right[6], false);
        assert_eq!(right[7], true);

        left.set(0, false);
        left.set(1, true);
        left.set(2, false);
        left.set(3, true);
        left.set(4, false);
        left.set(5, true);
        left.set(6, false);
        left.set(7, false);

        right.set(0, false);
        right.set(1, true);
        right.set(2, true);
        right.set(3, true);
        right.set(4, false);
        right.set(5, true);
        right.set(6, true);
        right.set(7, false);

        assert_eq!(vec[0], false);
        assert_eq!(vec[1], true);
        assert_eq!(vec[2], false);
        assert_eq!(vec[3], true);
        assert_eq!(vec[4], false);
        assert_eq!(vec[5], true);
        assert_eq!(vec[6], false);
        assert_eq!(vec[7], false);
        assert_eq!(vec[8], false);
        assert_eq!(vec[9], true);
        assert_eq!(vec[10], true);
        assert_eq!(vec[11], true);
        assert_eq!(vec[12], false);
        assert_eq!(vec[13], true);
        assert_eq!(vec[14], true);
        assert_eq!(vec[15], false);
    }

    #[test]
    #[should_panic]
    fn test_split_at_mut_not_on_storage_bound() {
        let mut vec = BitVector::<u8>::with_capacity(16);
        vec.split_at_mut(4);
    }
}

#[cfg(test)]
mod tests_bitslice {
    use super::{BitSlice,BitVector};

    fn create_bitslice_u8_16() -> BitSlice<u8> {
        let vec_8_32: BitVector<u8> = BitVector::with_capacity(32);
        let (_, right) = vec_8_32.split_at(16);
        right
    }

    #[test]
    fn test_index_bits_already_set() {
        let mut vec_8_32: BitVector<u8> = BitVector::with_capacity(32);

        vec_8_32.set(1, true);
        vec_8_32.set(3, true);
        vec_8_32.set(5, true);
        vec_8_32.set(7, true);
        vec_8_32.set(11, true);
        vec_8_32.set(13, true);
        vec_8_32.set(17, true);
        vec_8_32.set(19, true);
        vec_8_32.set(23, true);
        vec_8_32.set(29, true);

        let (left, right) = vec_8_32.split_at(16);

        assert_eq!(left[0], false);
        assert_eq!(left[1], true);
        assert_eq!(left[2], false);
        assert_eq!(left[3], true);
        assert_eq!(left[4], false);
        assert_eq!(left[5], true);
        assert_eq!(left[6], false);
        assert_eq!(left[7], true);
        assert_eq!(left[8], false);
        assert_eq!(left[9], false);
        assert_eq!(left[10], false);
        assert_eq!(left[11], true);
        assert_eq!(left[12], false);
        assert_eq!(left[13], true);
        assert_eq!(left[14], false);
        assert_eq!(left[15], false);

        assert_eq!(right[0], false);
        assert_eq!(right[1], true);
        assert_eq!(right[2], false);
        assert_eq!(right[3], true);
        assert_eq!(right[4], false);
        assert_eq!(right[5], false);
        assert_eq!(right[6], false);
        assert_eq!(right[7], true);
        assert_eq!(right[8], false);
        assert_eq!(right[9], false);
        assert_eq!(right[10], false);
        assert_eq!(right[11], false);
        assert_eq!(right[12], false);
        assert_eq!(right[13], true);
        assert_eq!(right[14], false);
        assert_eq!(right[15], false);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let slice = create_bitslice_u8_16();

        assert_eq!(slice.get(16), None);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let slice = create_bitslice_u8_16();

        slice[16];
    }

    #[test]
    fn test_capacity() {
        let slice = create_bitslice_u8_16();
        assert_eq!(slice.capacity(), 16);
    }

    #[test]
    fn test_split_at() {
        let mut vec_8_32: BitVector<u8> = BitVector::with_capacity(32);

        vec_8_32.set(1, true);
        vec_8_32.set(3, true);
        vec_8_32.set(5, true);
        vec_8_32.set(7, true);
        vec_8_32.set(11, true);
        vec_8_32.set(13, true);
        vec_8_32.set(17, true);
        vec_8_32.set(19, true);
        vec_8_32.set(23, true);
        vec_8_32.set(29, true);

        let (left_slice_from_vec, right_slice_from_vec) = vec_8_32.split_at(16);
        let (left_left, left_right) = left_slice_from_vec.split_at(8);
        let (right_left, right_right) = right_slice_from_vec.split_at(8);

        assert_eq!(left_left[0], false);
        assert_eq!(left_left[1], true);
        assert_eq!(left_left[2], false);
        assert_eq!(left_left[3], true);
        assert_eq!(left_left[4], false);
        assert_eq!(left_left[5], true);
        assert_eq!(left_left[6], false);
        assert_eq!(left_left[7], true);

        assert_eq!(left_right[0], false);
        assert_eq!(left_right[1], false);
        assert_eq!(left_right[2], false);
        assert_eq!(left_right[3], true);
        assert_eq!(left_right[4], false);
        assert_eq!(left_right[5], true);
        assert_eq!(left_right[6], false);
        assert_eq!(left_right[7], false);

        assert_eq!(right_left[0], false);
        assert_eq!(right_left[1], true);
        assert_eq!(right_left[2], false);
        assert_eq!(right_left[3], true);
        assert_eq!(right_left[4], false);
        assert_eq!(right_left[5], false);
        assert_eq!(right_left[6], false);
        assert_eq!(right_left[7], true);

        assert_eq!(right_right[0], false);
        assert_eq!(right_right[1], false);
        assert_eq!(right_right[2], false);
        assert_eq!(right_right[3], false);
        assert_eq!(right_right[4], false);
        assert_eq!(right_right[5], true);
        assert_eq!(right_right[6], false);
        assert_eq!(right_right[7], false);
    }

    #[test]
    #[should_panic]
    fn test_split_at_not_on_storage_bound() {
        let slice = create_bitslice_u8_16();
        slice.split_at(4);
    }
}

#[cfg(test)]
mod tests_bitslicemut {
    use super::{BitSliceMut,BitVector};

    fn create_bitslice_mut_u8_16() -> BitSliceMut<u8> {
        let mut vec_8_32: BitVector<u8> = BitVector::with_capacity(32);
        let (_, right) = vec_8_32.split_at_mut(16);
        right
    }

    #[test]
    fn test_index_bits_already_set() {
        let mut vec_8_32: BitVector<u8> = BitVector::with_capacity(32);

        vec_8_32.set(1, true);
        vec_8_32.set(3, true);
        vec_8_32.set(5, true);
        vec_8_32.set(7, true);
        vec_8_32.set(11, true);
        vec_8_32.set(13, true);
        vec_8_32.set(17, true);
        vec_8_32.set(19, true);
        vec_8_32.set(23, true);
        vec_8_32.set(29, true);

        let (left, right) = vec_8_32.split_at_mut(16);

        assert_eq!(left[0], false);
        assert_eq!(left[1], true);
        assert_eq!(left[2], false);
        assert_eq!(left[3], true);
        assert_eq!(left[4], false);
        assert_eq!(left[5], true);
        assert_eq!(left[6], false);
        assert_eq!(left[7], true);
        assert_eq!(left[8], false);
        assert_eq!(left[9], false);
        assert_eq!(left[10], false);
        assert_eq!(left[11], true);
        assert_eq!(left[12], false);
        assert_eq!(left[13], true);
        assert_eq!(left[14], false);
        assert_eq!(left[15], false);

        assert_eq!(right[0], false);
        assert_eq!(right[1], true);
        assert_eq!(right[2], false);
        assert_eq!(right[3], true);
        assert_eq!(right[4], false);
        assert_eq!(right[5], false);
        assert_eq!(right[6], false);
        assert_eq!(right[7], true);
        assert_eq!(right[8], false);
        assert_eq!(right[9], false);
        assert_eq!(right[10], false);
        assert_eq!(right[11], false);
        assert_eq!(right[12], false);
        assert_eq!(right[13], true);
        assert_eq!(right[14], false);
        assert_eq!(right[15], false);
    }

    #[test]
    fn test_get_set() {
        let mut slice = create_bitslice_mut_u8_16();

        slice.set(0, true);
        slice.set(1, false);
        slice.set(2, true);
        slice.set(3, false);
        slice.set(4, true);
        slice.set(5, false);
        slice.set(6, true);
        slice.set(7, true);
        slice.set(8, true);
        slice.set(9, false);
        slice.set(10, false);
        slice.set(11, false);
        slice.set(12, true);
        slice.set(13, false);
        slice.set(14, false);
        slice.set(15, true);

        assert_eq!(slice.get(0).unwrap(), true);
        assert_eq!(slice.get(1).unwrap(), false);
        assert_eq!(slice.get(2).unwrap(), true);
        assert_eq!(slice.get(3).unwrap(), false);
        assert_eq!(slice.get(4).unwrap(), true);
        assert_eq!(slice.get(5).unwrap(), false);
        assert_eq!(slice.get(6).unwrap(), true);
        assert_eq!(slice.get(7).unwrap(), true);
        assert_eq!(slice.get(8).unwrap(), true);
        assert_eq!(slice.get(9).unwrap(), false);
        assert_eq!(slice.get(10).unwrap(), false);
        assert_eq!(slice.get(11).unwrap(), false);
        assert_eq!(slice.get(12).unwrap(), true);
        assert_eq!(slice.get(13).unwrap(), false);
        assert_eq!(slice.get(14).unwrap(), false);
        assert_eq!(slice.get(15).unwrap(), true);
    }

    #[test]
    fn test_repeated_set() {
        let mut slice = create_bitslice_mut_u8_16();

        for i in 0..16 {
            slice.set(i, false);
        }

        for i in 0..16 {
            assert_eq!(slice[i], false);
        }

        for i in 0..16 {
            slice.set(i, true);
        }

        for i in 0..16 {
            assert_eq!(slice[i], true);
        }

        for i in 0..16 {
            slice.set(i, false);
        }

        for i in 0..16 {
            assert_eq!(slice[i], false);
        }
    }

    #[test]
    fn test_get_out_of_bounds() {
        let slice = create_bitslice_mut_u8_16();

        assert_eq!(slice.get(16), None);
    }

    #[test]
    #[should_panic]
    fn test_set_out_of_bounds() {
        let mut slice = create_bitslice_mut_u8_16();

        slice.set(16, true);    
    }

    #[test]
    fn test_index() {
        let mut slice = create_bitslice_mut_u8_16();

        slice.set(0, true);
        slice.set(1, false);
        slice.set(2, true);
        slice.set(3, false);
        slice.set(4, true);
        slice.set(5, false);
        slice.set(6, true);
        slice.set(7, true);
        slice.set(8, true);
        slice.set(9, false);
        slice.set(10, false);
        slice.set(11, false);
        slice.set(12, true);
        slice.set(13, false);
        slice.set(14, false);
        slice.set(15, true);

        assert_eq!(slice[0], true);
        assert_eq!(slice[1], false);
        assert_eq!(slice[2], true);
        assert_eq!(slice[3], false);
        assert_eq!(slice[4], true);
        assert_eq!(slice[5], false);
        assert_eq!(slice[6], true);
        assert_eq!(slice[7], true);
        assert_eq!(slice[8], true);
        assert_eq!(slice[9], false);
        assert_eq!(slice[10], false);
        assert_eq!(slice[11], false);
        assert_eq!(slice[12], true);
        assert_eq!(slice[13], false);
        assert_eq!(slice[14], false);
        assert_eq!(slice[15], true);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let slice = create_bitslice_mut_u8_16();

        slice[16];
    }

    #[test]
    fn test_capacity() {
        let slice = create_bitslice_mut_u8_16();
        assert_eq!(slice.capacity(), 16);
    }

    #[test]
    fn test_split_at() {
        let mut slice = create_bitslice_mut_u8_16();

        slice.set(0, true);
        slice.set(1, false);
        slice.set(2, true);
        slice.set(3, false);
        slice.set(4, true);
        slice.set(5, false);
        slice.set(6, true);
        slice.set(7, true);
        slice.set(8, true);
        slice.set(9, false);
        slice.set(10, false);
        slice.set(11, false);
        slice.set(12, true);
        slice.set(13, false);
        slice.set(14, false);
        slice.set(15, true);

        let (left, right) = slice.split_at(8);

        assert_eq!(left[0], true);
        assert_eq!(left[1], false);
        assert_eq!(left[2], true);
        assert_eq!(left[3], false);
        assert_eq!(left[4], true);
        assert_eq!(left[5], false);
        assert_eq!(left[6], true);
        assert_eq!(left[7], true);

        assert_eq!(right[0], true);
        assert_eq!(right[1], false);
        assert_eq!(right[2], false);
        assert_eq!(right[3], false);
        assert_eq!(right[4], true);
        assert_eq!(right[5], false);
        assert_eq!(right[6], false);
        assert_eq!(right[7], true);
    }

    #[test]
    #[should_panic]
    fn test_split_at_not_on_storage_bound() {
        let slice = create_bitslice_mut_u8_16();
        slice.split_at(4);
    }

    #[test]
    fn test_split_at_mut() {
        let mut slice = create_bitslice_mut_u8_16();

        slice.set(0, true);
        slice.set(1, false);
        slice.set(2, true);
        slice.set(3, false);
        slice.set(4, true);
        slice.set(5, false);
        slice.set(6, true);
        slice.set(7, true);
        slice.set(8, true);
        slice.set(9, false);
        slice.set(10, false);
        slice.set(11, false);
        slice.set(12, true);
        slice.set(13, false);
        slice.set(14, false);
        slice.set(15, true);

        let (mut left, mut right) = slice.split_at_mut(8);

        assert_eq!(left[0], true);
        assert_eq!(left[1], false);
        assert_eq!(left[2], true);
        assert_eq!(left[3], false);
        assert_eq!(left[4], true);
        assert_eq!(left[5], false);
        assert_eq!(left[6], true);
        assert_eq!(left[7], true);

        assert_eq!(right[0], true);
        assert_eq!(right[1], false);
        assert_eq!(right[2], false);
        assert_eq!(right[3], false);
        assert_eq!(right[4], true);
        assert_eq!(right[5], false);
        assert_eq!(right[6], false);
        assert_eq!(right[7], true);

        left.set(0, false);
        left.set(1, true);
        left.set(2, false);
        left.set(3, true);
        left.set(4, false);
        left.set(5, true);
        left.set(6, false);
        left.set(7, false);

        right.set(0, false);
        right.set(1, true);
        right.set(2, true);
        right.set(3, true);
        right.set(4, false);
        right.set(5, true);
        right.set(6, true);
        right.set(7, false);

        assert_eq!(slice[0], false);
        assert_eq!(slice[1], true);
        assert_eq!(slice[2], false);
        assert_eq!(slice[3], true);
        assert_eq!(slice[4], false);
        assert_eq!(slice[5], true);
        assert_eq!(slice[6], false);
        assert_eq!(slice[7], false);
        assert_eq!(slice[8], false);
        assert_eq!(slice[9], true);
        assert_eq!(slice[10], true);
        assert_eq!(slice[11], true);
        assert_eq!(slice[12], false);
        assert_eq!(slice[13], true);
        assert_eq!(slice[14], true);
        assert_eq!(slice[15], false);
    }

    #[test]
    #[should_panic]
    fn test_split_at_mut_not_on_storage_bound() {
        let mut slice = create_bitslice_mut_u8_16();
        slice.split_at_mut(4);
    }
}