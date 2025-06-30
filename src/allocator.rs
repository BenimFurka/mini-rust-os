use core::alloc::{GlobalAlloc, Layout};

#[allow(unused)]
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: core::sync::atomic::AtomicUsize,
}

impl BumpAllocator {
    pub const fn new(heap_start: usize, heap_size: usize) -> Self {
        Self {
            heap_start,
            heap_end: heap_start + heap_size,
            next: core::sync::atomic::AtomicUsize::new(heap_start),
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        
        let mut addr = self.next.load(core::sync::atomic::Ordering::Relaxed);
        let rem = addr % align;
        if rem != 0 {
            addr += align - rem;
        }
        
        let new_next = addr.checked_add(size).expect("Allocation overflow");
        if new_next > self.heap_end {
            return core::ptr::null_mut();
        }
        
        self.next.store(new_next, core::sync::atomic::Ordering::Relaxed);
        addr as *mut u8
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        return;
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new(0x4444_4444_0000, 1024 * 1024);
