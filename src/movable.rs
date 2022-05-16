use core::{sync::atomic::{AtomicBool, Ordering}, task::Poll};
use futures::Future;
#[cfg(not(debug_assertions))]
use core::hint::unreachable_unchecked;
use crate::queue::WakerQueue;

/// A mutex that isn't linked to any data, and has to be managed manually
#[derive(Debug)]
pub struct MovableMutex {
    inner: AtomicBool,
    queue: WakerQueue
}

impl MovableMutex {
    /// Creates a new mutex with a default capacity for 1024 concurrent wakers
    #[inline(always)]
    pub fn new () -> Self {
        Self::with_capacity(1024)
    }

    /// Creates a new mutex with the given capacity of concurrent wakers
    #[inline(always)]
    pub fn with_capacity (cap: usize) -> Self {
        Self {
            inner: AtomicBool::new(false),
            queue: WakerQueue::with_capacity(cap)
        }
    }

    /// Attempts to acquire the lock.
    #[inline(always)]
    pub fn try_lock (&self) -> bool {
        match self.inner.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire) {
            Ok(false) => true,
            Err(true) => false,

            // SAFETY: All p
            #[cfg(debug_assertions)]
            _ => unreachable!(),
            #[cfg(not(debug_assertions))]
            _ => unsafe { unreachable_unchecked() }
        }
    }

    /// Blocks the current task until the lock is acquired.
    #[inline(always)]
    pub fn lock_block (&self) {
        while !self.try_lock() { core::hint::spin_loop() }
    }

    /// Returns a future that will resolve when the lock is acquired.
    #[inline(always)]
    pub fn lock_async (&self) -> MovableMutexFuture<'_> {
        MovableMutexFuture { inner: self }
    }

    /// Releases the lock. It's up to the caller to guarantee the safety of this operation
    #[inline(always)]
    pub unsafe fn unlock (&self) {
        #[cfg(debug_assertions)]
        assert_eq!(true, self.inner.swap(false, Ordering::Release));
        #[cfg(not(debug_assertions))]
        self.inner.set(false, Ordering::Release);
        self.queue.wake_next()
    }
}

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

        self.inner.queue.push(cx.waker());
        Poll::Pending
    }
}