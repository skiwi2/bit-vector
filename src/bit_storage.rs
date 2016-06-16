use std::mem;
use std::ops::{BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Not,Shl,ShlAssign,Shr,ShrAssign};
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
    Eq + Zero + One + Unsigned + NumCast + Copy {
        fn storage_size() -> usize;
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
    Eq + Zero + One + Unsigned + NumCast + Copy {
        #[inline]
        fn storage_size() -> usize {
            mem::size_of::<S>() * 8
        }
    }