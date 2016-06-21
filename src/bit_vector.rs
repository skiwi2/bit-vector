use num;

use std::ops::Index;

use BitStorage;
use BitSlice;
use BitSliceMut;

use TRUE;
use FALSE;

//TODO wait on custom DST on stable and deref BitVector into BitSlice resp BitSliceMut and implement non-structural changing methods on BitSlice/BitSliceMut

pub struct BitVector<S: BitStorage> {
    data: Vec<S>,
    capacity: usize
}

impl<S: BitStorage> BitVector<S> {
    pub fn with_capacity(capacity: usize, default: bool) -> BitVector<S> {
        let len = (capacity / S::storage_size()) + 1;
        let default = if default { S::max_value() } else { S::zero() };
        BitVector { 
            data: vec![default; len],
            capacity: capacity
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
        let (data_index, remainder) = S::compute_data_index_and_remainder(index);
        S::set(&mut self.data[data_index], remainder, value);
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn split_at(&self, index: usize) -> (BitSlice<S>, BitSlice<S>) {
        self.panic_index_not_on_storage_bound(index);
        let data_index = S::compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (data_left, data_right) = self.data.split_at(data_index);

        unsafe {
            let left = BitSlice::from_pointer(data_left.as_ptr(), capacity_left);
            let right = BitSlice::from_pointer(data_right.as_ptr(), capacity_right);
            (left, right)
        }
    }

    pub fn split_at_mut(&mut self, index: usize) -> (BitSliceMut<S>, BitSliceMut<S>) {
        self.panic_index_not_on_storage_bound(index);
        let data_index = S::compute_data_index(index);
        let (capacity_left, capacity_right) = self.compute_capacities(index);
        let (data_left, data_right) = self.data.split_at_mut(data_index);

        unsafe {
            let left = BitSliceMut::from_pointer(data_left.as_mut_ptr(), capacity_left);
            let right = BitSliceMut::from_pointer(data_right.as_mut_ptr(), capacity_right);
            (left, right)
        }
    }

    pub fn iter(&self) -> Iter<S> {
        Iter {
            data: &self.data,
            capacity: self.capacity,
            data_index_counter: 0,
            remainder_counter: 0
        }
    }

    #[inline]
    fn get_unchecked(&self, index: usize) -> bool {
        let (data_index, remainder) = S::compute_data_index_and_remainder(index);
        self.get_unchecked_by_data_index_and_remainder(data_index, remainder)
    }

    #[inline]
    fn get_unchecked_by_data_index_and_remainder(&self, data_index: usize, remainder: S) -> bool {
        S::get(&self.data[data_index], remainder)
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
        if index % S::storage_size() != 0 {
            panic!("Index not on storage bound. Storage size = {}, Index = {}", S::storage_size(), index);
        }
    }
}

impl<S: BitStorage> Index<usize> for BitVector<S> {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        self.panic_index_bounds(index);
        bool_ref!(self.get_unchecked(index))
    }
}

impl<'a, S: BitStorage + 'a> IntoIterator for &'a BitVector<S> {
    type Item = bool;
    type IntoIter = Iter<'a, S>;

    fn into_iter(self) -> Iter<'a, S> {
        self.iter()
    }
}

pub struct Iter<'a, S: BitStorage + 'a> {
    data: &'a Vec<S>,
    capacity: usize,
    data_index_counter: usize,
    remainder_counter: usize
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
        S::get(&self.data[data_index], remainder)
    }

    #[inline]
    fn calculate_index(&self) -> usize {
        (self.data_index_counter * S::storage_size()) + self.remainder_counter
    }
}

#[cfg(test)]
mod tests {
    use super::BitVector;

    #[test]
    fn test_with_capacity() {
        let vec_32_32_false = BitVector::<u32>::with_capacity(32, false);
        assert!(vec_32_32_false.iter().all(|x| !x));

        let vec_32_1024_false = BitVector::<u32>::with_capacity(1024, false);
        assert!(vec_32_1024_false.iter().all(|x| !x));

        let vec_32_1000_false = BitVector::<u32>::with_capacity(1000, false);
        assert!(vec_32_1000_false.iter().all(|x| !x));

        let vec_32_32_true = BitVector::<u32>::with_capacity(32, true);
        assert!(vec_32_32_true.iter().all(|x| x));

        let vec_32_1024_true = BitVector::<u32>::with_capacity(1024, true);
        assert!(vec_32_1024_true.iter().all(|x| x));

        let vec_32_1000_true = BitVector::<u32>::with_capacity(1000, true);
        assert!(vec_32_1000_true.iter().all(|x| x));
    }

    #[test]
    fn test_get_set() {
        let mut vec = BitVector::<u8>::with_capacity(16, false);

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
        let mut vec = BitVector::<u8>::with_capacity(16, false);

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
        let vec = BitVector::<u8>::with_capacity(16, false);

        assert_eq!(vec.get(16), None);
    }

    #[test]
    #[should_panic]
    fn test_set_out_of_bounds() {
        let mut vec = BitVector::<u8>::with_capacity(16, false);

        vec.set(16, true);    
    }

    #[test]
    fn test_index() {
        let mut vec = BitVector::<u8>::with_capacity(16, false);

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
        let vec = BitVector::<u8>::with_capacity(16, false);

        vec[16];
    }

    #[test]
    fn test_capacity() {
        let vec_1000: BitVector<usize> = BitVector::with_capacity(1000, false);
        assert_eq!(vec_1000.capacity(), 1000);

        let vec_1024: BitVector<usize> = BitVector::with_capacity(1024, false);
        assert_eq!(vec_1024.capacity(), 1024);
    }

    #[test]
    fn test_split_at() {
        let mut vec = BitVector::<u8>::with_capacity(16, false);

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
        let vec = BitVector::<u8>::with_capacity(16, false);
        vec.split_at(4);
    }

    #[test]
    fn test_split_at_mut() {
        let mut vec = BitVector::<u8>::with_capacity(16, false);

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

        {
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
        }

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
        let mut vec = BitVector::<u8>::with_capacity(16, false);
        vec.split_at_mut(4);
    }

    #[test]
    fn test_iter() {
        let mut vec_8_4 = BitVector::<u8>::with_capacity(4, false);
        vec_8_4.set(0, true);
        vec_8_4.set(3, true);

        let vec_8_4_iter_vec: Vec<_> = vec_8_4.iter().collect();
        assert_eq!(vec_8_4_iter_vec, [true, false, false, true]);

        let mut vec_8_8 = BitVector::<u8>::with_capacity(8, false);
        vec_8_8.set(0, true);
        vec_8_8.set(3, true);
        vec_8_8.set(4, true);
        vec_8_8.set(6, true);

        let vec_8_8_iter_vec: Vec<_> = vec_8_8.iter().collect();
        assert_eq!(vec_8_8_iter_vec, [true, false, false, true, true, false, true, false]);

        let mut vec_8_16 = BitVector::<u8>::with_capacity(16, false);
        vec_8_16.set(0, true);
        vec_8_16.set(3, true);
        vec_8_16.set(4, true);
        vec_8_16.set(6, true);
        vec_8_16.set(9, true);
        vec_8_16.set(10, true);
        vec_8_16.set(11, true);
        vec_8_16.set(13, true);

        let vec_8_16_iter_vec: Vec<_> = vec_8_16.iter().collect();
        assert_eq!(vec_8_16_iter_vec, [true, false, false, true, true, false, true, false, false, true, true, true, false, true, false, false]);
    }

    #[test]
    fn test_into_iter_on_reference() {
        let mut vec_8_4 = BitVector::<u8>::with_capacity(4, false);
        vec_8_4.set(0, true);
        vec_8_4.set(3, true);

        let vec_8_4_iter_vec: Vec<_> = (&vec_8_4).into_iter().collect();
        assert_eq!(vec_8_4_iter_vec, [true, false, false, true]);

        let mut vec_8_8 = BitVector::<u8>::with_capacity(8, false);
        vec_8_8.set(0, true);
        vec_8_8.set(3, true);
        vec_8_8.set(4, true);
        vec_8_8.set(6, true);

        let vec_8_8_iter_vec: Vec<_> = (&vec_8_8).into_iter().collect();
        assert_eq!(vec_8_8_iter_vec, [true, false, false, true, true, false, true, false]);

        let mut vec_8_16 = BitVector::<u8>::with_capacity(16, false);
        vec_8_16.set(0, true);
        vec_8_16.set(3, true);
        vec_8_16.set(4, true);
        vec_8_16.set(6, true);
        vec_8_16.set(9, true);
        vec_8_16.set(10, true);
        vec_8_16.set(11, true);
        vec_8_16.set(13, true);

        let vec_8_16_iter_vec: Vec<_> = (&vec_8_16).into_iter().collect();
        assert_eq!(vec_8_16_iter_vec, [true, false, false, true, true, false, true, false, false, true, true, true, false, true, false, false]);
    }
}