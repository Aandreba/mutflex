#![no_std]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "nightly", feature(cfg_target_has_atomic))]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )*
    };
}

cfg_if::cfg_if! {
    if #[cfg(any(not(feature = "nightly"), target_has_atomic_load_store = "8"))] {
        use core::sync::atomic::AtomicBool;
        pub(crate) type Flag = AtomicBool;

        pub(crate) const TRUE : bool = true;
        pub(crate) const FALSE : bool = false;
    } else if #[cfg(target_has_atomic_load_store = "16")] {
        use core::sync::atomic::AtomicU16;
        pub(crate) type Flag = AtomicU16;

        pub(crate) const TRUE : u16 = 1;
        pub(crate) const FALSE : u16 = 0;
    } else if #[cfg(target_has_atomic_load_store = "32")] {
        use core::sync::atomic::AtomicU32;
        pub(crate) type Flag = AtomicU32;

        pub(crate) const TRUE : u32 = 1;
        pub(crate) const FALSE : u32 = 0;
    } else if #[cfg(target_has_atomic_load_store = "64")] {
        use core::sync::atomic::AtomicU64;
        pub(crate) type Flag = AtomicU64;

        pub(crate) const TRUE : u64 = 1;
        pub(crate) const FALSE : u64 = 0;
    } else if #[cfg(target_has_atomic_load_store = "ptr")] {
        use core::sync::atomic::AtomicUsize;
        pub(crate) type Flag = AtomicUsize;

        pub(crate) const TRUE : usize = 1;
        pub(crate) const FALSE : usize = 0;
    } else {
        compile_error!("This target doesn't have atomic load/store");
    }
}

#[cfg(feature = "futures")]
pub(crate) mod queue;
flat_mod!(movable, mutex);