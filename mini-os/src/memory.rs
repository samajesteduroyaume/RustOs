pub mod vm;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;

const MAX_ORDER: usize = 10; // 2^10 * 4KB = 4MB max block size
const PAGE_SIZE: usize = 4096;

struct Block {
    next: Option<*mut Block>,
}

pub struct BuddyAllocator {
    heap_start: usize,
    heap_size: usize,
    free_lists: [Option<*mut Block>; MAX_ORDER],
}

// SAFETY: BuddyAllocator is only accessed through a Mutex
unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<*mut Block> = None;
        BuddyAllocator {
            heap_start: 0,
            heap_size: 0,
            free_lists: [EMPTY; MAX_ORDER],
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_size = size;
        
        // Initialiser avec un seul grand bloc
        let block = start as *mut Block;
        (*block).next = None;
        self.free_lists[MAX_ORDER - 1] = Some(block);
    }

    unsafe fn alloc_block(&mut self, _order: usize) -> *mut u8 {
        // Version simplifiée : on prend toujours à partir de la plus grande liste libre
        if let Some(block) = self.free_lists[MAX_ORDER - 1] {
            self.free_lists[MAX_ORDER - 1] = None;
            block as *mut u8
        } else {
            null_mut()
        }
    }
}

// Wrapper pour satisfaire les règles d'orphelin
pub struct LockedAllocator(Mutex<BuddyAllocator>);

impl LockedAllocator {
    pub const fn new() -> Self {
        LockedAllocator(Mutex::new(BuddyAllocator::new()))
    }

    pub fn lock(&self) -> spin::MutexGuard<'_, BuddyAllocator> {
        self.0.lock()
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // Pour simplifier, on alloue toujours un bloc de la plus grande taille
        self.0.lock().alloc_block(MAX_ORDER - 1)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // À implémenter : libération de la mémoire
    }
}

#[global_allocator]
pub static ALLOCATOR: LockedAllocator = LockedAllocator::new();
