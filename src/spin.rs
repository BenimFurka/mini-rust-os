use core::sync::atomic::{AtomicUsize, Ordering};

// Спинлок без данных
pub struct Spinlock {
    locked: AtomicUsize,
}

impl Spinlock {
    pub const fn new() -> Self {
        Spinlock {
            locked: AtomicUsize::new(0),
        }
    }

    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop(); // Оптимизация для ожидания
        }
    }

    pub fn unlock(&self) {
        self.locked.store(0, Ordering::Release);
    }
}