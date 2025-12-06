use x86_64::{
    structures::paging::{
        Page, PageTable, PhysFrame, Size4KiB, FrameAllocator, Mapper, OffsetPageTable,
        PageTableFlags, mapper::MapToError,
    },
    VirtAddr, PhysAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

pub mod cow;
pub use cow::{CowManager, COW_MANAGER};

// Gestionnaire de cadre physique
pub struct SimpleFrameAllocator {
    next: usize,
    used: Vec<usize>,
    memory_map: &'static MemoryMap,
}

impl SimpleFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            next: 0,
            used: Vec::new(),
            memory_map,
        }
    }
    
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for SimpleFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        
        if let Some(frame) = frame {
            self.used.push(frame.start_address().as_u64() as usize);
        }
        
        frame
    }
}

// Gestionnaire d'espace d'adressage
pub struct AddressSpace {
    page_table: OffsetPageTable<'static>,
    frame_allocator: Mutex<SimpleFrameAllocator>,
}

impl AddressSpace {
    pub unsafe fn new(phys_offset: u64, memory_map: &'static MemoryMap) -> Self {
        let level_4_table = active_level_4_table(phys_offset);
        let frame_allocator = SimpleFrameAllocator::init(memory_map);
        
        Self {
            page_table: OffsetPageTable::new(level_4_table, VirtAddr::new(phys_offset)),
            frame_allocator: Mutex::new(frame_allocator),
        }
    }
    
    pub fn map_page(&mut self, page: Page, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
        let frame = self.frame_allocator
            .lock()
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
            
        unsafe {
            self.page_table.map_to(
                page,
                frame,
                flags | PageTableFlags::PRESENT,
                &mut *self.frame_allocator.lock(),
            )?.flush();
        }
        
        Ok(())
    }
    
    pub fn clone(&self) -> Self {
        // Implémentation de la copie de l'espace d'adressage avec CoW
        unimplemented!("Clone d'espace d'adressage avec CoW")
    }
}

// Fonction utilitaire pour obtenir la table des pages de niveau 4
unsafe fn active_level_4_table(phys_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    
    let (level_4_table_frame, _) = Cr3::read();
    
    let phys = level_4_table_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + phys_offset);
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
    phys_offset: u64,
}

impl VMManager {
    pub unsafe fn new(phys_offset: u64, memory_map: &'static MemoryMap) -> Self {
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
                let virt = VirtAddr::new(phys.as_u64() + self.phys_offset);
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

pub fn init_vm(phys_offset: u64, memory_map: &'static MemoryMap) {
    unsafe {
        *VM_MANAGER.lock() = Some(VMManager::new(phys_offset, memory_map));
    }
}
