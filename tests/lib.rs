extern crate bit_vector;
extern crate crossbeam;

use bit_vector::{BitVector,BitSlice,BitSliceMut,BitStorage};

#[test]
fn test_parallel_immutable() {
	let mut vec: BitVector<u8> = BitVector::with_capacity(32, false);

	vec.set(0, true);
	vec.set(1, true);
	vec.set(2, false);
	vec.set(3, false);
	vec.set(4, true);
	vec.set(5, false);
	vec.set(6, true);
	vec.set(7, false);

	vec.set(8, false);
	vec.set(9, true);
	vec.set(10, false);
	vec.set(11, true);
	vec.set(12, true);
	vec.set(13, true);
	vec.set(14, false);
	vec.set(15, true);

	vec.set(16, false);
	vec.set(17, false);
	vec.set(18, true);
	vec.set(19, false);
	vec.set(20, true);
	vec.set(21, false);
	vec.set(22, true);
	vec.set(23, true);

	vec.set(24, true);
	vec.set(25, true);
	vec.set(26, false);
	vec.set(27, true);
	vec.set(28, true);
	vec.set(29, false);
	vec.set(30, false);
	vec.set(31, true);

	let (left, right) = vec.split_at(16);

	let (first, second) = left.split_at(8);
	let (third, fourth) = right.split_at(8);

	let mut handles = vec![];

	crossbeam::scope(|scope| {
		let handle = scope.spawn(|| {
			assert_eq!(first[0], true);
			assert_eq!(first[1], true);
			assert_eq!(first[2], false);
			assert_eq!(first[3], false);
			assert_eq!(first[4], true);
			assert_eq!(first[5], false);
			assert_eq!(first[6], true);
			assert_eq!(first[7], false);
		});
		handles.push(handle);

		let handle = scope.spawn(|| {
			assert_eq!(second[0], false);
			assert_eq!(second[1], true);
			assert_eq!(second[2], false);
			assert_eq!(second[3], true);
			assert_eq!(second[4], true);
			assert_eq!(second[5], true);
			assert_eq!(second[6], false);
			assert_eq!(second[7], true);
		});
		handles.push(handle);

		let handle = scope.spawn(|| {
			assert_eq!(third[0], false);
			assert_eq!(third[1], false);
			assert_eq!(third[2], true);
			assert_eq!(third[3], false);
			assert_eq!(third[4], true);
			assert_eq!(third[5], false);
			assert_eq!(third[6], true);
			assert_eq!(third[7], true);
		});
		handles.push(handle);

		let handle = scope.spawn(|| {
			assert_eq!(fourth[0], true);
			assert_eq!(fourth[1], true);
			assert_eq!(fourth[2], false);
			assert_eq!(fourth[3], true);
			assert_eq!(fourth[4], true);
			assert_eq!(fourth[5], false);
			assert_eq!(fourth[6], false);
			assert_eq!(fourth[7], true);
		});
		handles.push(handle);
	});

	for handle in handles {
		handle.join();
	}
}

#[test]
fn test_parallel_mutable() {
	let mut vec: BitVector<u8> = BitVector::with_capacity(32, false);

	{
		let (left, right) = vec.split_at_mut(16);

		let (mut first, mut second) = left.split_at_mut(8);
		let (mut third, mut fourth) = right.split_at_mut(8);

		let mut handles = vec![];

		crossbeam::scope(|scope| {
			let handle = scope.spawn(|| {
				first.set(0, true);
				first.set(1, true);
				first.set(2, false);
				first.set(3, false);
				first.set(4, true);
				first.set(5, false);
				first.set(6, true);
				first.set(7, false);
			});
			handles.push(handle);

			let handle = scope.spawn(|| {
				second.set(0, false);
				second.set(1, true);
				second.set(2, false);
				second.set(3, true);
				second.set(4, true);
				second.set(5, true);
				second.set(6, false);
				second.set(7, true);
			});
			handles.push(handle);

			let handle = scope.spawn(|| {
				third.set(0, false);
				third.set(1, false);
				third.set(2, true);
				third.set(3, false);
				third.set(4, true);
				third.set(5, false);
				third.set(6, true);
				third.set(7, true);
			});
			handles.push(handle);

			let handle = scope.spawn(|| {
				fourth.set(0, true);
				fourth.set(1, true);
				fourth.set(2, false);
				fourth.set(3, true);
				fourth.set(4, true);
				fourth.set(5, false);
				fourth.set(6, false);
				fourth.set(7, true);
			});
			handles.push(handle);
		});

		for handle in handles {
			handle.join();
		}
	}

	assert_eq!(vec[0], true);
	assert_eq!(vec[1], true);
	assert_eq!(vec[2], false);
	assert_eq!(vec[3], false);
	assert_eq!(vec[4], true);
	assert_eq!(vec[5], false);
	assert_eq!(vec[6], true);
	assert_eq!(vec[7], false);

	assert_eq!(vec[8], false);
	assert_eq!(vec[9], true);
	assert_eq!(vec[10], false);
	assert_eq!(vec[11], true);
	assert_eq!(vec[12], true);
	assert_eq!(vec[13], true);
	assert_eq!(vec[14], false);
	assert_eq!(vec[15], true);

	assert_eq!(vec[16], false);
	assert_eq!(vec[17], false);
	assert_eq!(vec[18], true);
	assert_eq!(vec[19], false);
	assert_eq!(vec[20], true);
	assert_eq!(vec[21], false);
	assert_eq!(vec[22], true);
	assert_eq!(vec[23], true);

	assert_eq!(vec[24], true);
	assert_eq!(vec[25], true);
	assert_eq!(vec[26], false);
	assert_eq!(vec[27], true);
	assert_eq!(vec[28], true);
	assert_eq!(vec[29], false);
	assert_eq!(vec[30], false);
	assert_eq!(vec[31], true);
}

#[test]
fn test_reborrow_immutable() {
    let mut vector: BitVector<u32> = BitVector::with_capacity(1000, false);
    let indices = vec![128, 224, 320, 416, 512, 608, 704, 800, 928];

    let slices = split_into_bit_slices(&mut vector, &indices);
    assert_eq!(slices.len(), 10);
}

fn split_into_bit_slices<'a, S: BitStorage>(bit_vector: &'a mut BitVector<S>, indices: &[usize]) -> Vec<BitSlice<'a, S>> {
    let mut bit_slices = vec![];

    bit_slices.push(bit_vector.split_at(0).1);
    let mut split_indices = 0;
    for index in indices {
        let last_slice = bit_slices.pop().unwrap();
        let (new_slice, remainder) = last_slice.split_at(index - split_indices);
        split_indices = *index;
        bit_slices.push(new_slice);
        bit_slices.push(remainder);
    }

    bit_slices
}

#[test]
fn test_reborrow_mutable() {
    let mut vector: BitVector<u32> = BitVector::with_capacity(1000, false);
    let indices = vec![128, 224, 320, 416, 512, 608, 704, 800, 928];

    let slices = split_into_bit_slices_mut(&mut vector, &indices);
    assert_eq!(slices.len(), 10);
}

fn split_into_bit_slices_mut<'a, S: BitStorage>(bit_vector: &'a mut BitVector<S>, indices: &[usize]) -> Vec<BitSliceMut<'a, S>> {
    let mut bit_slices = vec![];

    bit_slices.push(bit_vector.split_at_mut(0).1);
    let mut split_indices = 0;
    for index in indices {
        let last_slice = bit_slices.pop().unwrap();
        let (new_slice, remainder) = last_slice.split_at_mut(index - split_indices);
        split_indices = *index;
        bit_slices.push(new_slice);
        bit_slices.push(remainder);
    }

    bit_slices
}