extern crate num;

macro_rules! bool_ref {
    ($cond:expr) => (if $cond { &TRUE } else { &FALSE })
}

mod bit_storage;
mod bit_vector;
mod bit_slice;
mod bit_slice_mut;

pub use bit_storage::BitStorage;
pub use bit_vector::BitVector;
pub use bit_slice::BitSlice;
pub use bit_slice_mut::BitSliceMut;

static TRUE: bool = true;
static FALSE: bool = false;