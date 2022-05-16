#![no_std]
#![doc = include_str!("../README.md")]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )*
    };
}

pub(crate) mod queue;
pub(crate) mod waker;
flat_mod!(movable, mutex);