use core::{cell::UnsafeCell, sync::atomic::{AtomicBool, Ordering}, task::Waker};
#[cfg(not(debug_assertions))]
use core::hint::unreachable_unchecked;

pub struct AtomicWaker {
    waiting: AtomicBool,
    waker: UnsafeCell<Option<Waker>>,
}

impl AtomicWaker {
    #[inline(always)]
    pub const fn new () -> Self {
        Self {
            waiting: AtomicBool::new(true),
            waker: UnsafeCell::new(None),
        }
    }

    pub fn try_register (&self, waker: &Waker) -> bool {
        match self.waiting.compare_exchange(true, false, Ordering::Acquire, Ordering::Acquire) {
            Ok(true) => unsafe {
                let ptr = self.waker.get();
                match &mut *ptr {
                    Some(_) => false,
                    None => {
                        *ptr = Some(waker.clone());
                        true
                    }
                }
            },

            Err(false) => false,
            #[cfg(debug_assertions)]
            _ => unreachable!(),
            #[cfg(not(debug_assertions))]
            _ => unsafe { unreachable_unchecked() }
        }
    }

    pub fn try_wake (&self) -> bool {
        match self.waiting.compare_exchange(true, false, Ordering::Acquire, Ordering::Acquire) {
            Ok(true) => unsafe {
                let waker = core::mem::take(&mut *self.waker.get());
                match waker {
                    Some(waker) => {
                        waker.wake();
                        todo!()
                    },

                    None => false
                }
            },
            
            Err(false) => false,
            #[cfg(debug_assertions)]
            _ => unreachable!(),
            #[cfg(not(debug_assertions))]
            _ => unsafe { unreachable_unchecked() }
        }
    }
}