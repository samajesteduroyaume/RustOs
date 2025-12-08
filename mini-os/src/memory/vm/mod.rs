use x86_64::{
    structures::paging::{
        Page, PageTable, PhysFrame, Size4KiB, Mapper, OffsetPageTable,
        PageTableFlags, mapper::MapToError, FrameAllocator as PageFrameAllocator,
    },
    VirtAddr, PhysAddr,
};
// Limine removed - using Multiboot2
use core::ptr::NonNull;
use alloc::vec::Vec;
use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::memory::{FrameAllocator, FRAME_ALLOCATOR};

pub mod cow;
pub use cow::{CowManager, COW_MANAGER};

pub mod hugepage;
pub use hugepage::{HugePageAllocator, PageSize, HugePageStats, HUGE_PAGE_ALLOCATOR};

pub mod demand;
pub use demand::{DemandPagingManager, LazyPage, DEMAND_PAGING_MANAGER, handle_page_fault};

pub mod lru;
pub use lru::{LRUPageCache, PageEntry, LRU_PAGE_CACHE};

pub mod pagecache;
pub use pagecache::{PageCache, PageCacheEntry, PAGE_CACHE};

pub mod swap;
pub use swap::{SwapDaemon, SwapEntry, SWAP_DAEMON};

// Wrapper thread-safe pour la memory map de Limine
#[derive(Clone, Copy)]
pub struct LimineMemoryMap(pub &'static [NonNull<MemmapEntry>]);

unsafe impl Send for LimineMemoryMap {}
unsafe impl Sync for LimineMemoryMap {}

impl LimineMemoryMap {
    pub fn iter(&self) -> impl Iterator<Item = &NonNull<MemmapEntry>> {
        self.0.iter()
    }
}

// Gestionnaire de cadre physique
pub struct SimpleFrameAllocator {
    next: usize,
    used: Vec<usize>,
    memory_map: LimineMemoryMap,
}    

impl SimpleFrameAllocator {
    pub unsafe fn init(memory_map: LimineMemoryMap) -> Self {
        // Commencer l'allocation après le kernel et ses segments
        const KERNEL_END: u64 = 0x200000; // 2 MB
        
        let mut used = Vec::new();
        
        // Marquer toutes les frames avant KERNEL_END comme utilisées
        for addr in (0..KERNEL_END).step_by(4096) {
            used.push(addr as usize);
        }
        
        // Marquer toutes les frames dans les régions NON-Usable comme utilisées
        for entry in memory_map.iter() {
            let entry: &MemmapEntry = unsafe { entry.as_ref() };
            if entry.typ != MemoryMapEntryType::Usable {
                let start = entry.base;
                let end = entry.base + entry.len;
                for addr in (start..end).step_by(4096) {
                    let frame_addr = addr as usize;
                    if !used.contains(&frame_addr) {
                        used.push(frame_addr);
                    }
                }
            }
        }
        
        Self {
            next: 0,
            used,
            memory_map,
        }
    }
    
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .map(|e: &NonNull<MemmapEntry>| unsafe { e.as_ref() })
            .filter(|e| e.typ == MemoryMapEntryType::Usable);
        
        let addr_ranges = usable_regions.map(|e| e.base..(e.base + e.len));
        let frame_addresses = addr_ranges.flat_map(|r: core::ops::Range<u64>| r.step_by(4096));
        
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}



unsafe impl FrameAllocator<Size4KiB> for SimpleFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // Chercher une frame non utilisée
        loop {
            let frame = self.usable_frames().nth(self.next)?;
            self.next += 1;
            
            let frame_addr = frame.start_address().as_u64() as usize;
            
            // Vérifier si cette frame n'est pas déjà utilisée
            if !self.used.contains(&frame_addr) {
                self.used.push(frame_addr);
                return Some(frame);
            }
            // Sinon, continuer à chercher la prochaine frame
        }
    }
}

// Gestionnaire d'espace d'adressage
pub struct AddressSpace {
    page_table: OffsetPageTable<'static>,
    frame_allocator: Mutex<SimpleFrameAllocator>,
}

impl AddressSpace {
    pub unsafe fn new(phys_offset: VirtAddr, memory_map: LimineMemoryMap) -> Self {
        let level_4_table = active_level_4_table(phys_offset);
        let frame_allocator = SimpleFrameAllocator::init(memory_map);
        
        Self {
            page_table: OffsetPageTable::new(level_4_table, phys_offset),
            frame_allocator: Mutex::new(frame_allocator),
        }
    }
    
    pub fn map_page(&mut self, page: Page, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
        let frame = self.frame_allocator
            .lock()
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
            
        unsafe {
            match self.page_table.map_to(
                page,
                frame,
                flags | PageTableFlags::PRESENT,
                &mut *self.frame_allocator.lock(),
            ) {
                Ok(t) => {
                    t.flush();
                    Ok(())
                },
                Err(MapToError::PageAlreadyMapped(_)) => {
                    // Page is already mapped. We accept this as "success" (idempotent).
                    // The 'frame' we allocated is unused.
                    // TODO: Deallocate 'frame' when FrameDeallocator is implemented.
                    Ok(())
                },
                Err(e) => Err(e),
            }
        }
    }
    
    pub fn clone(&self) -> Self {
        // Implémentation de la copie de l'espace d'adressage avec CoW
        unimplemented!("Clone d'espace d'adressage avec CoW")
    }
}

// Initialise le mapper de pages
pub unsafe fn init_mapper(phys_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(phys_offset);
    OffsetPageTable::new(level_4_table, phys_offset)
}

// Fonction utilitaire pour obtenir la table des pages de niveau 4
unsafe fn active_level_4_table(phys_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    
    let (level_4_table_frame, _) = Cr3::read();
    
    let phys = level_4_table_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    
    &mut *page_table_ptr
}

// Implémentation de la copie sur écriture
pub struct CowPage {
    frame: PhysFrame,
    ref_count: usize,
    writable: bool,
}

impl CowPage {
    pub fn new(frame: PhysFrame, writable: bool) -> Self {
        Self {
            frame,
            ref_count: 1,
            writable,
        }
    }
    
    pub fn make_writable(&mut self) {
        // Implémenter la copie si nécessaire
        if !self.writable {
            // TODO: Allouer une nouvelle page et copier les données
            self.writable = true;
        }
    }
}

// Gestionnaire de la mémoire virtuelle
pub struct VMManager {
    kernel_space: AddressSpace,
    process_spaces: Vec<AddressSpace>,
    phys_offset: VirtAddr,
}

impl VMManager {
    pub unsafe fn new(phys_offset: VirtAddr, memory_map: LimineMemoryMap) -> Self {
        let kernel_space = AddressSpace::new(phys_offset, memory_map);
        
        Self {
            kernel_space,
            process_spaces: Vec::new(),
            phys_offset,
        }
    }
    
    pub fn create_process_space(&mut self) -> usize {
        // Créer un nouvel espace d'adressage pour un processus
        // en copiant l'espace noyau avec CoW
        let new_space = self.kernel_space.clone();
        let id = self.process_spaces.len();
        self.process_spaces.push(new_space);
        id
    }
    
    pub fn switch_space(&self, space_id: usize) {
        // Changer d'espace d'adressage
        if let Some(_space) = self.process_spaces.get(space_id) {
            unsafe {
                use x86_64::registers::control::Cr3;
                use x86_64::structures::paging::PageTable;
                
                let (frame, _) = Cr3::read();
                let phys = frame.start_address();
                let virt = self.phys_offset + phys.as_u64();
                let _page_table_ptr: *mut PageTable = virt.as_mut_ptr();
                
                // Mettre à jour le pointeur de la table des pages
                Cr3::write(frame, Cr3::read().1);
            }
        }
    }
}

// Initialisation du gestionnaire de mémoire virtuelle
lazy_static! {
    pub static ref VM_MANAGER: Mutex<Option<VMManager>> = Mutex::new(None);
}

pub fn init_vm(phys_offset: VirtAddr, memory_map: LimineMemoryMap) {
    unsafe {
        *VM_MANAGER.lock() = Some(VMManager::new(phys_offset, memory_map));
    }
}
