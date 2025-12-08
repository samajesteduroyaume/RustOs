// pub mod vm; // Disabled - depends on Limine
pub mod slab;
pub mod hybrid;
pub mod shm;
pub mod mmap;

pub use hybrid::{HYBRID_ALLOCATOR, HybridStats};
pub use shm::{SHM_MANAGER, ShmManager, ShmError, ShmCmd};
pub use mmap::{MMAP_MANAGER, MmapManager, MmapError, MmapRegion};

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{null_mut, NonNull};
use core::cmp::max;
use spin::Mutex;

// 2^12 * 4096 = 16MB max block size, enough for our small heap
const MAX_ORDER: usize = 12; 
const PAGE_SIZE: usize = 4096;

struct Block {
    next: Option<NonNull<Block>>,
}

unsafe impl Send for Block {}
unsafe impl Sync for Block {}

impl Block {
    // Defines standard methods for the block manipulation
    fn as_ptr(&self) -> *mut u8 {
        self as *const _ as *mut u8
    }
}

pub struct BuddyAllocator {
    heap_start: usize,
    heap_end: usize,
    free_lists: [Option<NonNull<Block>>; MAX_ORDER],
    total_allocations: usize,
    total_deallocations: usize,
    // Nouvelles métriques
    pub fragmentation_internal: usize,  // Bytes gaspillés (alloués mais non utilisés)
    pub current_memory_usage: usize,    // Utilisation mémoire actuelle
    pub peak_memory_usage: usize,       // Utilisation mémoire maximale
}

// SAFETY: BuddyAllocator is only accessed through a Mutex
unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<NonNull<Block>> = None;
        BuddyAllocator {
            heap_start: 0,
            heap_end: 0,
            free_lists: [EMPTY; MAX_ORDER],
            total_allocations: 0,
            total_deallocations: 0,
            fragmentation_internal: 0,
            current_memory_usage: 0,
            peak_memory_usage: 0,
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_end = start + size;
        self.total_allocations = 0;
        self.total_deallocations = 0;
        self.fragmentation_internal = 0;
        self.current_memory_usage = 0;
        self.peak_memory_usage = 0;
        
        // Add the entire range to the free lists
        self.add_free_memory(start, size);
    }
    
    // Add a range of memory to the allocator
    unsafe fn add_free_memory(&mut self, start: usize, size: usize) {
        // We need to break down the range into power-of-two blocks
        let mut current_start = start;
        let mut remaining_size = size;
        
        while remaining_size > 0 {
            // Find the largest order that fits and is aligned
            let order = self.find_largest_fit(current_start, remaining_size);
            let block_size = 1 << (order + 12); // block size = 2^order * 4096
            
            // Add to free list
            self.push_block(order, current_start as *mut Block);
            
            current_start += block_size;
            remaining_size -= block_size;
        }
    }
    
    // Find largest order that fits in size and is aligned to current_addr
    fn find_largest_fit(&self, addr: usize, size: usize) -> usize {
        let mut order = MAX_ORDER - 1;
        while order > 0 {
            let block_size = 1 << (order + 12);
            if size >= block_size && (addr % block_size) == 0 {
                return order;
            }
            order -= 1;
        }
        0 // Order 0 (4KB)
    }

    unsafe fn alloc_block(&mut self, layout: Layout) -> *mut u8 {
        let size = max(layout.size().max(layout.align()), PAGE_SIZE);
        let order = self.size_to_order(size);
        
        // Find the first available block in the requested order or higher
        for i in order..MAX_ORDER {
            if let Some(block_ptr) = self.free_lists[i] {
                // Remove block from list
                self.pop_block(i);
                
                let block_addr = block_ptr.as_ptr() as usize;
                
                // If we found a larger block, split it down to the requested order
                for j in (order..i).rev() {
                    let split_order = j;
                    let split_size = 1 << (split_order + 12);
                    let buddy_addr = block_addr + split_size;
                    
                    self.push_block(split_order, buddy_addr as *mut Block);
                }
                
                self.total_allocations += 1;
                
                // Mettre à jour les métriques
                let block_size = 1 << (order + 12);
                self.current_memory_usage += block_size;
                if self.current_memory_usage > self.peak_memory_usage {
                    self.peak_memory_usage = self.current_memory_usage;
                }
                
                // Calculer la fragmentation interne
                let requested_size = max(layout.size().max(layout.align()), PAGE_SIZE);
                self.fragmentation_internal += block_size - requested_size;
                
                return block_addr as *mut u8;
            }
        }
        
        null_mut()
    }
    
    unsafe fn dealloc_block(&mut self, ptr: *mut u8, layout: Layout) {
        let size = max(layout.size().max(layout.align()), PAGE_SIZE);
        let mut order = self.size_to_order(size);
        let mut current_addr = ptr as usize;
        
        self.total_deallocations += 1;
        
        // Mettre à jour les métriques
        let initial_size = 1 << (order + 12);
        self.current_memory_usage = self.current_memory_usage.saturating_sub(initial_size);
        
        // Réduire la fragmentation interne
        let requested_size = max(layout.size().max(layout.align()), PAGE_SIZE);
        self.fragmentation_internal = self.fragmentation_internal.saturating_sub(initial_size - requested_size);
        
        // Try to merge with buddies
        while order < MAX_ORDER - 1 {
            let block_size = 1 << (order + 12);
            let buddy_addr = current_addr ^ block_size;
            
            // Check if buddy is in the free list
            if self.remove_from_list(order, buddy_addr) {
                // Buddy found and removed, merge
                current_addr = core::cmp::min(current_addr, buddy_addr);
                order += 1;
            } else {
                // Buddy not free, stop merging
                break;
            }
        }
        
        // Add the (possibly merged) block to the free list
        self.push_block(order, current_addr as *mut Block);
    }
    
    // Helper: Push a block to the front of the free list
    unsafe fn push_block(&mut self, order: usize, ptr: *mut Block) {
        let mut block = NonNull::new_unchecked(ptr);
        block.as_mut().next = self.free_lists[order];
        self.free_lists[order] = Some(block);
    }
    
    // Helper: Pop a block from the front of the free list
    unsafe fn pop_block(&mut self, order: usize) -> Option<NonNull<Block>> {
        let block = self.free_lists[order]?;
        self.free_lists[order] = block.as_ref().next;
        return Some(block);
    }
    
    // Helper: Remove a specific block (by address) from the free list
    // Returns true if found and removed
    unsafe fn remove_from_list(&mut self, order: usize, addr: usize) -> bool {
        let mut current_opt = self.free_lists[order];
        let mut prev_ptr: *mut Option<NonNull<Block>> = &mut self.free_lists[order];
        
        while let Some(mut current) = current_opt {
            if current.as_ptr() as usize == addr {
                // Found it, unlink
                *prev_ptr = current.as_ref().next;
                return true;
            }
            
            prev_ptr = &mut current.as_mut().next;
            current_opt = current.as_ref().next;
        }
        
        false
    }
    
    fn size_to_order(&self, size: usize) -> usize {
        let mut order = 0;
        let mut block_size = PAGE_SIZE;
        
        while block_size < size {
            block_size *= 2;
            order += 1;
        }
        
        order
    }
    
    /// Retourne le ratio de fragmentation interne (0.0 - 1.0)
    pub fn get_fragmentation_ratio(&self) -> f32 {
        if self.current_memory_usage == 0 {
            return 0.0;
        }
        self.fragmentation_internal as f32 / self.current_memory_usage as f32
    }
    
    /// Retourne les statistiques de l'allocateur
    pub fn get_stats(&self) -> BuddyStats {
        BuddyStats {
            total_allocations: self.total_allocations,
            total_deallocations: self.total_deallocations,
            current_memory_usage: self.current_memory_usage,
            peak_memory_usage: self.peak_memory_usage,
            fragmentation_internal: self.fragmentation_internal,
            fragmentation_ratio: self.get_fragmentation_ratio(),
        }
    }
}

/// Statistiques du Buddy Allocator
#[derive(Debug, Clone, Copy)]
pub struct BuddyStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub current_memory_usage: usize,
    pub peak_memory_usage: usize,
    pub fragmentation_internal: usize,
    pub fragmentation_ratio: f32,
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
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc_block(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc_block(ptr, layout)
    }
}

// NOTE: L'allocateur global est maintenant défini dans hybrid.rs
// L'ancien LockedAllocator est conservé pour compatibilité mais n'est plus utilisé
// Utilisez HYBRID_ALLOCATOR à la place de ALLOCATOR pour bénéficier du SLAB

// #[global_allocator]
// pub static ALLOCATOR: LockedAllocator = LockedAllocator::new();

/// Allocateur Buddy legacy - conservé pour compatibilité
/// IMPORTANT: Utilisez HYBRID_ALLOCATOR pour de meilleures performances
pub static ALLOCATOR: LockedAllocator = LockedAllocator::new();
