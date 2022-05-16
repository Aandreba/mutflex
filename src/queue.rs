extern crate alloc;
use core::{task::Waker, ptr::NonNull, alloc::Layout};
use alloc::alloc::dealloc;
use crate::waker::AtomicWaker;

#[derive(Debug)]
pub struct WakerQueue {
    ptr: NonNull<AtomicWaker>,
    cap: usize
}

impl WakerQueue {
    pub fn with_capacity (cap: usize) -> Self {
        use alloc::alloc::alloc;

        unsafe {
            let layout = Layout::array::<AtomicWaker>(cap).unwrap();
            let ptr = alloc(layout);
            let ptr = NonNull::new(ptr as *mut AtomicWaker).expect("Error allocating to the heap");
            
            // initialize vector
            for i in 0..cap {
                ptr.as_ptr().add(i).write(AtomicWaker::new());
            }

            Self {
                ptr,
                cap
            }
        }
    }

    #[inline(always)]
    pub fn push (&self, waker: &Waker) {
        unsafe {
            let ptr = self.ptr.as_ptr();
            for i in 0..self.cap {
                let inner_waker = &*ptr.add(i);
                if inner_waker.try_register(waker) { break }
            }
        }

        panic!("No empty slot found on the queue")
    }

    #[inline(always)]
    pub fn wake_next (&self) {
        unsafe {
            let ptr = self.ptr.as_ptr();
            for i in 0..self.cap {
                let inner_waker = &*ptr.add(i);
                if inner_waker.try_wake() { break }
            }
        }
    }
}

impl Drop for WakerQueue {
    #[inline(always)]
    fn drop(&mut self) {
        let layout = Layout::array::<AtomicWaker>(self.cap).unwrap();
        unsafe { dealloc(self.ptr.as_ptr().cast(), layout) }
    }
}

unsafe impl Send for WakerQueue {}
unsafe impl Sync for WakerQueue {}