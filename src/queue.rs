extern crate alloc;
use core::{task::Waker, sync::atomic::{Ordering}, cell::UnsafeCell};
use alloc::{collections::VecDeque};
use crate::{Flag, FALSE, TRUE};

#[derive(Debug)]
pub struct WakerQueue {
    locked: Flag,
    inner: UnsafeCell<VecDeque<Waker>>
}

impl WakerQueue {
    #[inline(always)]
    pub fn new () -> Self {
        Self {
            locked: Flag::new(FALSE),
            inner: UnsafeCell::new(VecDeque::new())
        }
    }

    #[inline(always)]
    pub fn with_capacity (cap: usize) -> Self {
        Self {
            locked: Flag::new(FALSE),
            inner: UnsafeCell::new(VecDeque::with_capacity(cap))
        }
    }

    #[inline(always)]
    pub fn len (&self) -> usize {
        self.lock();
        let len = self.get_inner().len();
        self.unlock();
        len
    }

    #[inline(always)]
    pub fn capacity (&self) -> usize {
        self.lock();
        let cap = self.get_inner().capacity();
        self.unlock();
        cap
    }

    #[inline(always)]
    pub fn register (&self, waker: &Waker) {        
        self.lock();
        unsafe { self.get_inner_mut().push_back(waker.clone()) };
        self.unlock();
    }

    #[inline(always)]
    pub fn wake_next (&self) {
        let waker;
        self.lock();
        unsafe { waker = self.get_inner_mut().pop_front() };
        self.unlock();

        if let Some(waker) = waker {
            waker.wake();
        }
    }

    /* UTILS */
    #[inline(always)]
    fn get_inner (&self) -> &VecDeque<Waker> {
        unsafe { &*self.inner.get() }
    }

    #[inline(always)]
    unsafe fn get_inner_mut (&self) -> &mut VecDeque<Waker> {
        &mut *self.inner.get()
    }

    #[inline(always)]
    fn try_lock (&self) -> bool {
        match self.locked.compare_exchange(FALSE, TRUE, Ordering::Acquire, Ordering::Acquire) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    #[inline(always)]
    fn lock (&self) {
        while !self.try_lock() { core::hint::spin_loop() }
    }

    #[inline(always)]
    fn unlock (&self) {
        #[cfg(debug_assertions)]
        assert_eq!(TRUE, self.locked.swap(FALSE, Ordering::Release));
        #[cfg(not(debug_assertions))]
        self.locked.set(FALSE, Ordering::Release);
    }
}

unsafe impl Send for WakerQueue {}
unsafe impl Sync for WakerQueue {}