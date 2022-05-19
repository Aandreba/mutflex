extern crate alloc;
use core::{ptr::NonNull, alloc::Layout};
use alloc::alloc::alloc;

/// Allocates memory and writes the value after
#[inline(always)]
pub unsafe fn wralloc<T> (val: T) -> Option<NonNull<T>> {
    let alloc = alloc(Layout::new::<T>()) as *mut T;
    if let Some(ptr) = NonNull::new(alloc) {
        core::ptr::write(alloc, val);
        return Some(ptr)
    }

    None
}