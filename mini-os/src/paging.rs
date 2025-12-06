use x86_64::{
    structures::paging::{
        Mapper, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, mapper::MapToError,
        FrameAllocator, OffsetPageTable, PageTableIndex, MappedPageTable,
    },
    VirtAddr, PhysAddr, structures::paging::page_table::PageTableEntry,
};

/// Initialise la pagination de base
pub fn init_paging(mapper: &mut impl Mapper<Size4KiB>) {
    // Exemple de mapping d'une page physique vers une adresse virtuelle
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000)); // Adresse du buffer VGA
    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0xb8000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    
    unsafe {
        // Note: Cette implémentation est simplifiée et nécessite un FrameAllocator
        // En production, il faudrait gérer correctement les erreurs
        let _ = mapper.map_to(
            page, 
            frame, 
            flags, 
            &mut DummyFrameAllocator
        ).unwrap().flush();
    }
}

/// Crée une nouvelle table des pages
pub unsafe fn create_page_table() -> &'static mut PageTable {
    // Allouer une page alignée pour la table des pages
    use x86_64::structures::paging::PageTableFlags as Flags;
    
    let page = Page::from_start_address(VirtAddr::new(0x1000)).unwrap();
    let frame = PhysFrame::containing_address(PhysAddr::new(0x1000));
    
    // Créer un mappage identité
    let mut mapper = unsafe { 
        let level_4_table = active_level_4_table();
        let (level_4_table, _) = level_4_table;
        level_4_table
    };
    
    // Mapper la page
    unsafe {
        mapper.map_to(
            page,
            frame,
            Flags::PRESENT | Flags::WRITABLE,
            &mut DummyFrameAllocator,
        ).unwrap().flush();
    }
    
    let page_table_ptr: *mut PageTable = page.start_address().as_mut_ptr();
    &mut *page_table_ptr
}

/// Obtient une référence mutable vers la table des pages de niveau 4 active
pub unsafe fn active_level_4_table() -> (&'static mut PageTable, u16) {
    use x86_64::registers::control::Cr3;
    
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64());
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    
    (&mut *page_table_ptr, 0)
}

/// Allocateur de frames factice (à remplacer par un vrai allocateur)
pub struct DummyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for DummyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None // À implémenter avec un vrai allocateur de frames
    }
}
