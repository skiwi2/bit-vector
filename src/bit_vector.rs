use num;

use BitStorage;
use std::marker::PhantomData;
use BitSlice;
use BitSliceMut;
use std::mem;
use std::ops::Index;
use TRUE;
use FALSE;

//TODO add storage_size() method with #inline -> will lead to constant after compilation
pub struct BitVector<S: BitStorage> {
    data: Vec<S>,
    capacity: usize,
    storage_size: usize
}

impl<S: BitStorage> Index<usize> for BitVector<S> {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        self.panic_index_bounds(index);
        bool_ref!(self.get_unchecked(index))
    }
}

impl<S: BitStorage> BitVector<S> {
    pub fn with_capacity(capacity: usize) -> BitVector<S> {
        let storage_size = mem::size_of::<S>() * 8;
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
            storage_size: self.storage_size,
            phantom: PhantomData
        };
        let right = BitSlice {
            pointer: data_right.as_ptr(),
            capacity: capacity_right,
            storage_size: self.storage_size,
            phantom: PhantomData
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
            storage_size: self.storage_size,
            phantom: PhantomData
        };
        let right = BitSliceMut {
            pointer: data_right.as_mut_ptr(),
            capacity: capacity_right,
            storage_size: self.storage_size,
            phantom: PhantomData
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

