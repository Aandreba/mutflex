extern crate alloc;
use core::{task::Waker, sync::atomic::{Ordering, AtomicPtr}, cell::UnsafeCell, ptr::{NonNull, addr_of}, alloc::Layout};
use alloc::{collections::VecDeque, alloc::{alloc, dealloc}, boxed::Box};
use crate::{Flag, FALSE, TRUE, utils::wralloc};

/*#[derive(Debug)]
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
}*/

unsafe impl Send for WakerQueue {}
unsafe impl Sync for WakerQueue {}

#[derive(Debug)]
pub struct WakerQueue {
    first: AtomicPtr<QueueNode>,
    last: AtomicPtr<QueueNode>
}

struct QueueNode {
    value: Waker,
    next: AtomicPtr<QueueNode> 
}

impl QueueNode {
    pub(super) const LAYOUT : Layout = Layout::new::<QueueNode>();
}

impl WakerQueue {
    #[inline(always)]
    pub const fn new () -> Self {
        Self {
            first: AtomicPtr::new(core::ptr::null_mut()),
            last: AtomicPtr::new(core::ptr::null_mut())
        }
    }

    pub fn register (&self, value: Waker) {
        let queue = QueueNode {
            value,
            next: AtomicPtr::new(core::ptr::null_mut())
        };

        let queue = unsafe { wralloc(queue).unwrap() };
        let prev = self.last.swap(queue.as_ptr(), Ordering::Acquire);
        
        // By definition, if last was null, the list was empty, so tail is also null
        if prev.is_null() {
            self.first.store(queue.as_ptr(), Ordering::Release);
            return
        }
        
        let prev = unsafe { &mut *prev };

        #[cfg(debug_assertions)]
        assert_eq!(core::ptr::null_mut(), prev.next.swap(queue.as_ptr(), Ordering::Release));
        #[cfg(not(debug_assertions))]
        prev.next.store(queue.as_ptr(), Ordering::Release);
    }

    pub fn wake_next (&self) {
        let mut ptr = self.first.load(Ordering::Acquire);
        loop {
            if ptr.is_null() { return }
            let first = unsafe { &*ptr };

            match self.first.compare_exchange(ptr, first.next.load(Ordering::Acquire), Ordering::Acquire, Ordering::Acquire) {
                Ok(ptr) => unsafe {
                    let first = &*ptr;
                    let waker = core::ptr::read(addr_of!(first.value));
                    waker.wake();
                    dealloc(ptr.cast(), QueueNode::LAYOUT);
                    return;
                },

                Err(curr) => {
                    ptr = curr; 
                    continue 
                }
            }
        }
    }
}

impl Drop for WakerQueue {
    #[inline(always)]
    fn drop(&mut self) {
        let mut ptr = self.first.load(Ordering::Acquire);
        loop {
            if ptr.is_null() { break }
            unsafe {
                let node = &*ptr;
                let next = node.next.load(Ordering::Acquire);
                
                dealloc(ptr.cast(), QueueNode::LAYOUT);
                ptr = next;
            }
        }
    }
}