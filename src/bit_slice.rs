use num;

use std::marker::PhantomData;
use std::ops::Index;

use BitStorage;

use TRUE;
use FALSE;

pub struct BitSlice<'a, S: BitStorage + 'a> {
    pointer: *const S,
    capacity: usize,
    phantom: PhantomData<&'a S>
}

unsafe impl<'a, S: BitStorage + 'a> Send for BitSlice<'a, S> {}
unsafe impl<'a, S: BitStorage + 'a> Sync for BitSlice<'a, S> {}

impl<'a, S: BitStorage + 'a> BitSlice<'a, S> {
    pub unsafe fn from_pointer(pointer: *const S, capacity: usize) -> BitSlice<'a, S> {
        BitSlice {
            pointer: pointer,
            capacity: capacity,
            phantom: PhantomData
        }
    }

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
        let data_index = S::compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (pointer_left, pointer_right) = self.compute_pointers(data_index);

        unsafe {
            let left = BitSlice::from_pointer(pointer_left, capacity_left);
            let right = BitSlice::from_pointer(pointer_right, capacity_right);
            (left, right)
        }
    }

    pub fn iter(&self) -> Iter<S> {
        Iter {
            pointer: self.pointer,
            capacity: self.capacity,
            data_index_counter: 0,
            remainder_counter: 0,
            phantom: PhantomData
        }
    }

    #[inline]
    fn get_unchecked(&self, index: usize) -> bool {
        let (data_index, remainder) = S::compute_data_index_and_remainder(index);
        self.get_unchecked_by_data_index_and_remainder(data_index, remainder)
    }

    #[inline]
    fn get_unchecked_by_data_index_and_remainder(&self, data_index: usize, remainder: S) -> bool {
        let element = unsafe { *self.pointer.offset(data_index as isize) };
        S::get(&element, remainder)
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
        if index % S::storage_size() != 0 {
            panic!("Index not on storage bound. Storage size = {}, Index = {}", S::storage_size(), index);
        }
    }
}

impl<'a, S: BitStorage + 'a> Index<usize> for BitSlice<'a, S> {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        self.panic_index_bounds(index);
        bool_ref!(self.get_unchecked(index))
    }
}

pub struct Iter<'a, S: BitStorage + 'a> {
    pointer: *const S,
    capacity: usize,
    data_index_counter: usize,
    remainder_counter: usize,
    phantom: PhantomData<&'a S>
}

unsafe impl<'a, S: BitStorage + 'a> Send for Iter<'a, S> {}
unsafe impl<'a, S: BitStorage + 'a> Sync for Iter<'a, S> {}

impl<'a, S: BitStorage + 'a> IntoIterator for &'a BitSlice<'a, S> {
    type Item = bool;
    type IntoIter = Iter<'a, S>;

    fn into_iter(self) -> Iter<'a, S> {
        self.iter()
    }
}

impl<'a, S: BitStorage + 'a> Iterator for Iter<'a, S> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        let remainder: S = num::cast(self.remainder_counter).unwrap();
        let next = self.get_unchecked_by_data_index_and_remainder(self.data_index_counter, remainder);

        if self.calculate_index() == self.capacity {
            return None;
        }

        self.remainder_counter += 1;
        if self.remainder_counter == S::storage_size() {
            self.remainder_counter = 0;
            self.data_index_counter += 1;
        }

        Some(next)
    }
}

impl<'a, S: BitStorage + 'a> Iter<'a, S> {
    #[inline]
    fn get_unchecked_by_data_index_and_remainder(&self, data_index: usize, remainder: S) -> bool {
        let element = unsafe { *self.pointer.offset(data_index as isize) };
        S::get(&element, remainder)
    }

    #[inline]
    fn calculate_index(&self) -> usize {
        (self.data_index_counter * S::storage_size()) + self.remainder_counter
    }
}

#[cfg(test)]
mod tests {
    use super::super::{BitSlice,BitVector};

    fn create_bitslice_u8_16_from_bitvector_u8_32(vec: &BitVector<u8>) -> BitSlice<u8> {
        let (_, right) = vec.split_at(16);
        right
    }

    #[test]
    fn test_from_pointer() {
        let vec: Vec<u8> = vec![0b11001111, 0b01001001];
        let slice = unsafe { BitSlice::from_pointer(vec.as_ptr(), 16) };

        assert_eq!(slice[0], true);
        assert_eq!(slice[1], true);
        assert_eq!(slice[2], true);
        assert_eq!(slice[3], true);
        assert_eq!(slice[4], false);
        assert_eq!(slice[5], false);
        assert_eq!(slice[6], true);
        assert_eq!(slice[7], true);

        assert_eq!(slice[8], true);
        assert_eq!(slice[9], false);
        assert_eq!(slice[10], false);
        assert_eq!(slice[11], true);
        assert_eq!(slice[12], false);
        assert_eq!(slice[13], false);
        assert_eq!(slice[14], true);
        assert_eq!(slice[15], false);
    }

    #[test]
    fn test_index_bits_already_set() {
        let mut vec_8_32: BitVector<u8> = BitVector::with_capacity(32, false);

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
        let vec_8_32: BitVector<u8> = BitVector::with_capacity(32, false);
        let slice = create_bitslice_u8_16_from_bitvector_u8_32(&vec_8_32);

        assert_eq!(slice.get(16), None);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let vec_8_32: BitVector<u8> = BitVector::with_capacity(32, false);
        let slice = create_bitslice_u8_16_from_bitvector_u8_32(&vec_8_32);

        slice[16];
    }

    #[test]
    fn test_capacity() {
        let vec_8_32: BitVector<u8> = BitVector::with_capacity(32, false);
        let slice = create_bitslice_u8_16_from_bitvector_u8_32(&vec_8_32);
        assert_eq!(slice.capacity(), 16);
    }

    #[test]
    fn test_split_at() {
        let mut vec_8_32: BitVector<u8> = BitVector::with_capacity(32, false);

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
        let vec_8_32: BitVector<u8> = BitVector::with_capacity(32, false);
        let slice = create_bitslice_u8_16_from_bitvector_u8_32(&vec_8_32);
        slice.split_at(4);
    }

    #[test]
    fn test_iter() {
        let mut vec_8_4 = BitVector::<u8>::with_capacity(4, false);
        vec_8_4.set(0, true);
        vec_8_4.set(3, true);

        let (_, slice_8_4) = vec_8_4.split_at(0);
        let slice_8_4_iter_vec: Vec<_> = slice_8_4.iter().collect();
        assert_eq!(slice_8_4_iter_vec, [true, false, false, true]);

        let mut vec_8_8 = BitVector::<u8>::with_capacity(8, false);
        vec_8_8.set(0, true);
        vec_8_8.set(3, true);
        vec_8_8.set(4, true);
        vec_8_8.set(6, true);

        let (_, slice_8_8) = vec_8_8.split_at(0);
        let slice_8_8_iter_vec: Vec<_> = slice_8_8.iter().collect();
        assert_eq!(slice_8_8_iter_vec, [true, false, false, true, true, false, true, false]);

        let mut vec_8_16 = BitVector::<u8>::with_capacity(16, false);
        vec_8_16.set(0, true);
        vec_8_16.set(3, true);
        vec_8_16.set(4, true);
        vec_8_16.set(6, true);
        vec_8_16.set(9, true);
        vec_8_16.set(10, true);
        vec_8_16.set(11, true);
        vec_8_16.set(13, true);

        let (_, slice_8_16) = vec_8_16.split_at(0);
        let slice_8_16_iter_vec: Vec<_> = slice_8_16.iter().collect();
        assert_eq!(slice_8_16_iter_vec, [true, false, false, true, true, false, true, false, false, true, true, true, false, true, false, false]);

        let (left, right) = vec_8_16.split_at(8);
        let vec_8_16_iter_vec: Vec<_> = vec_8_16.iter().collect();
        let left_plus_right_iter_vec: Vec<_> = left.iter().chain(right.iter()).collect();

        assert_eq!(vec_8_16_iter_vec, left_plus_right_iter_vec);
    }

    #[test]
    fn test_into_iter_on_reference() {
        let mut vec_8_4 = BitVector::<u8>::with_capacity(4, false);
        vec_8_4.set(0, true);
        vec_8_4.set(3, true);

        let (_, slice_8_4) = vec_8_4.split_at(0);
        let slice_8_4_iter_vec: Vec<_> = (&slice_8_4).into_iter().collect();
        assert_eq!(slice_8_4_iter_vec, [true, false, false, true]);

        let mut vec_8_8 = BitVector::<u8>::with_capacity(8, false);
        vec_8_8.set(0, true);
        vec_8_8.set(3, true);
        vec_8_8.set(4, true);
        vec_8_8.set(6, true);

        let (_, slice_8_8) = vec_8_8.split_at(0);
        let slice_8_8_iter_vec: Vec<_> = (&slice_8_8).into_iter().collect();
        assert_eq!(slice_8_8_iter_vec, [true, false, false, true, true, false, true, false]);

        let mut vec_8_16 = BitVector::<u8>::with_capacity(16, false);
        vec_8_16.set(0, true);
        vec_8_16.set(3, true);
        vec_8_16.set(4, true);
        vec_8_16.set(6, true);
        vec_8_16.set(9, true);
        vec_8_16.set(10, true);
        vec_8_16.set(11, true);
        vec_8_16.set(13, true);

        let (_, slice_8_16) = vec_8_16.split_at(0);
        let slice_8_16_iter_vec: Vec<_> = (&slice_8_16).into_iter().collect();
        assert_eq!(slice_8_16_iter_vec, [true, false, false, true, true, false, true, false, false, true, true, true, false, true, false, false]);

        let (left, right) = vec_8_16.split_at(8);
        let vec_8_16_iter_vec: Vec<_> = vec_8_16.iter().collect();
        let left_iter = (&left).into_iter();
        let left_plus_right_iter_vec: Vec<_> = left_iter.chain(&right).collect();

        assert_eq!(vec_8_16_iter_vec, left_plus_right_iter_vec);
    }
}