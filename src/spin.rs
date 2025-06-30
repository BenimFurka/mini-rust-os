use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct Spinlock {
    locked: AtomicUsize,
}

impl Spinlock {
    pub const fn new() -> Self {
        Spinlock {
            locked: AtomicUsize::new(0),
        }
    }

    pub fn lock(&self) -> SpinlockGuard {
        while self
            .locked
            .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        SpinlockGuard { lock: self }
    }
}

pub struct SpinlockGuard<'a> {
    lock: &'a Spinlock,
}

impl Drop for SpinlockGuard<'_> {
    fn drop(&mut self) {
        self.lock.locked.store(0, Ordering::Release);
    }
}

pub struct SpinMutex<T> {
    lock: Spinlock,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinMutex<T> where T: Send {}

impl<T> SpinMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: Spinlock::new(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinMutexGuard<T> {
        let guard = self.lock.lock();
        SpinMutexGuard {
            guard,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

pub struct SpinMutexGuard<'a, T> {
    #[allow(dead_code)]
    guard: SpinlockGuard<'a>,
    data: &'a mut T,
}

impl<T> Deref for SpinMutexGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> DerefMut for SpinMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}