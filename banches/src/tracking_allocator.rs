use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicI64, Ordering};

static ALLOCATED_BYTES: AtomicI64 = AtomicI64::new(0);

pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED_BYTES.fetch_add(layout.size() as i64, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED_BYTES.fetch_sub(layout.size() as i64, Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

// --- Public API ---

#[derive(Debug, Clone, Copy)]
pub struct AllocStats {
    pub bytes: usize,
}

impl AllocStats {
    pub fn now() -> Self {
        Self {
            bytes: ALLOCATED_BYTES.load(Ordering::Relaxed) as usize,
        }
    }
}
