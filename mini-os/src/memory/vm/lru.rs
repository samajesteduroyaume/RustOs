/// Module de Page Replacement avec algorithme LRU (version simplifiée)
use alloc::vec::Vec;
use x86_64::{VirtAddr, PhysAddr};
use spin::Mutex;

/// Entrée de page dans le cache LRU
#[derive(Debug, Clone)]
pub struct PageEntry {
    pub phys_addr: PhysAddr,
    pub virt_addr: VirtAddr,
    pub last_access: u64,
    pub dirty: bool,
    pub owner_pid: u64,
}

impl PageEntry {
    pub fn new(phys_addr: PhysAddr, virt_addr: VirtAddr, pid: u64) -> Self {
        Self {
            phys_addr,
            virt_addr,
            last_access: 0,
            dirty: false,
            owner_pid: pid,
        }
    }
}

/// Cache de pages avec LRU
pub struct LRUPageCache {
    pages: Vec<PageEntry>,
    max_pages: usize,
    eviction_count: usize,
}

impl LRUPageCache {
    pub fn new(max_pages: usize) -> Self {
        Self {
            pages: Vec::new(),
            max_pages,
            eviction_count: 0,
        }
    }
    
    pub fn add_page(&mut self, entry: PageEntry) {
        if self.pages.len() >= self.max_pages {
            self.pages.remove(0);
            self.eviction_count += 1;
        }
        self.pages.push(entry);
    }
}

/// Statistiques LRU
#[derive(Debug, Clone, Copy)]
pub struct LRUStats {
    pub total_pages: usize,
    pub eviction_count: usize,
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref LRU_PAGE_CACHE: Mutex<LRUPageCache> = Mutex::new(LRUPageCache::new(1024));
}
