use core::{sync::atomic::{Ordering}, task::Poll};
#[cfg(not(debug_assertions))]
use core::hint::unreachable_unchecked;
use crate::{queue::WakerQueue, Flag, FALSE, TRUE};

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        /// A mutex that isn't linked to any data, and has to be managed manually
        #[derive(Debug)]
        pub struct MovableMutex {
            inner: Flag,
            queue: WakerQueue
        }
    } else {
        /// A mutex that isn't linked to any data, and has to be managed manually
        #[derive(Debug)]
        pub struct MovableMutex {
            inner: Flag
        }
    }
}

impl MovableMutex {
    /// Creates a new mutex with a default capacity for 8 concurrent wakers
    #[inline(always)]
    pub fn new () -> Self {
        Self::with_capacity(8)
    }

    /// Creates a new mutex with the given capacity of concurrent wakers
    #[cfg(feature = "futures")]
    #[inline(always)]
    pub fn with_capacity (cap: usize) -> Self {
        Self {
            inner: Flag::new(FALSE),
            queue: WakerQueue::with_capacity(cap)
        }
    }

    /// Creates a new mutex with the given capacity of concurrent wakers
    #[cfg(not(feature = "futures"))]
    #[inline(always)]
    pub fn with_capacity (cap: usize) -> Self {
        Self {
            inner: Flag::new(FALSE)
        }
    }

    /// Attempts to acquire the lock.
    #[inline(always)]
    pub fn try_lock (&self) -> bool {
        match self.inner.compare_exchange(FALSE, TRUE, Ordering::Acquire, Ordering::Acquire) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    /// Blocks the current task until the lock is acquired.
    #[inline(always)]
    pub fn lock (&self) {
        while !self.try_lock() { core::hint::spin_loop() }
    }

    /// Returns a future that will resolve when the lock is acquired.
    #[cfg(feature = "futures")]
    #[inline(always)]
    pub fn lock_async (&self) -> MovableMutexFuture<'_> {
        MovableMutexFuture { inner: self }
    }

    /// Releases the lock. It's up to the caller to guarantee the safety of this operation
    #[inline(always)]
    pub unsafe fn unlock (&self) {
        #[cfg(debug_assertions)]
        assert_eq!(TRUE, self.inner.swap(FALSE, Ordering::Release));
        #[cfg(not(debug_assertions))]
        self.inner.set(FALSE, Ordering::Release);
        #[cfg(feature = "futures")]
        self.queue.wake_next()
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        use futures::Future;

        #[repr(transparent)]
        pub struct MovableMutexFuture<'a> {
            inner: &'a MovableMutex
        }
        
        impl Future for MovableMutexFuture<'_> {
            type Output = ();
        
            #[inline(always)]
            fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
                if self.inner.try_lock() {
                    return Poll::Ready(());
                }
        
                self.inner.queue.register(cx.waker());
                Poll::Pending
            }
        }
    }
}