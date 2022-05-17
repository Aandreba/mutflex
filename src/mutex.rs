use core::{cell::{UnsafeCell}, ops::{Deref, DerefMut}, task::Poll};
use futures::{Future, FutureExt};
use crate::{MovableMutex, MovableMutexFuture};

/// A mutally exclusive lock.
pub struct Mutex<T: ?Sized> {
    inner: MovableMutex,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex with an initial capacity for 8 concurrent wakers
    #[inline(always)]
    pub fn new (data: T) -> Self {
        Self::with_capacity(data, 8)
    }

    /// Creates a new mutex with the given capacity of concurrent wakers
    #[inline(always)]
    pub fn with_capacity (data: T, cap: usize) -> Self {
        Self {
            inner: MovableMutex::with_capacity(cap),
            data: UnsafeCell::new(data)
        }
    }

    /// Consumes the mutex and returns it's inner data
    #[inline(always)]
    pub fn into_inner (self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    #[inline(always)]
    pub fn try_lock (&self) -> Option<MutexGuard<'_, T>> {
        if self.inner.try_lock() {
            return Some(MutexGuard(self))
        }

        None
    }

    /// Blocks the current task until the lock is acquired.
    #[inline(always)]
    pub fn lock (&self) -> MutexGuard<'_, T> {
        self.inner.lock();
        MutexGuard(self)
    }

    /// Returns a future that will resolve when the lock is acquired.
    #[cfg(feature = "futures")]
    #[inline(always)]
    pub fn lock_async (&self) -> MutexFuture<'_, T> {
        let fut = self.inner.lock_async();
        MutexFuture { inner: self, fut: fut }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        /// A future that resolves when the lock is acquired.
        pub struct MutexFuture<'a, T: ?Sized> {
            inner: &'a Mutex<T>,
            fut: MovableMutexFuture<'a>,
        }

        impl<'a, T: ?Sized> Future for MutexFuture<'a, T> {
            type Output = MutexGuard<'a, T>;

            #[inline(always)]
            fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
                if self.fut.poll_unpin(cx).is_ready() {
                    return Poll::Ready(MutexGuard(self.inner))
                }

                Poll::Pending
            }
        }
    }
}

/// A guard for a locked mutex.
#[repr(transparent)]
pub struct MutexGuard<'a, T: ?Sized> (&'a Mutex<T>);

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.inner.unlock() }
    }
}

unsafe impl<T: ?Sized + Sync> Sync for Mutex<T> {}