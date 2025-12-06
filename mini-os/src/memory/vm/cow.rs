use x86_64::{
    structures::paging::{
        Page, PhysFrame, Size4KiB, PageSize, FrameAllocator,
    },
    VirtAddr,
};
use spin::Mutex;
use alloc::collections::BTreeMap;
use lazy_static::lazy_static;

/// Structure pour suivre les pages partagées en CoW
pub struct SharedPage {
    pub frame: PhysFrame,
    pub ref_count: usize,
    pub writable: bool,
}

/// Gestionnaire des pages partagées
pub struct CowManager {
    pub shared_pages: BTreeMap<u64, SharedPage>,
}

impl CowManager {
    pub fn new() -> Self {
        Self {
            shared_pages: BTreeMap::new(),
        }
    }

    /// Marque une page comme partagée en CoW
    pub fn share_page(
        &mut self,
        frame: PhysFrame,
        writable: bool,
    ) -> Result<(), &'static str> {
        let key = frame.start_address().as_u64();
        
        if let Some(shared) = self.shared_pages.get_mut(&key) {
            shared.ref_count += 1;
        } else {
            self.shared_pages.insert(key, SharedPage {
                frame,
                ref_count: 1,
                writable,
            });
        }
        Ok(())
    }

    /// Duplique une page si nécessaire (au premier accès en écriture)
    pub fn handle_cow_fault(
        &mut self,
        fault_addr: VirtAddr,
        frame_allocator: &mut dyn FrameAllocator<Size4KiB>,
    ) -> Result<(), &'static str> {
        let page = Page::<Size4KiB>::containing_address(fault_addr);
        let page_addr = page.start_address().as_u64();
        
        // Vérifier si c'est une page partagée
        if let Some(shared) = self.shared_pages.get_mut(&page_addr) {
            if shared.ref_count > 1 {
                // Allouer une nouvelle trame
                let new_frame = frame_allocator
                    .allocate_frame()
                    .ok_or("Impossible d'allouer une nouvelle trame")?;
                
                // Copier le contenu
                unsafe {
                    let src = shared.frame.start_address().as_u64() as *const u8;
                    let dst = new_frame.start_address().as_u64() as *mut u8;
                    core::ptr::copy_nonoverlapping(src, dst, Size4KiB::SIZE as usize);
                }
                
                // Mettre à jour le compteur de références
                shared.ref_count -= 1;
                
                // Ajouter la nouvelle trame au gestionnaire
                self.share_page(new_frame, true)?;
                
                return Ok(());
            } else {
                // Dernière référence, rendre la page inscriptible
                if let Some(shared) = self.shared_pages.get_mut(&page_addr) {
                    shared.writable = true;
                }
            }
        }
        
        Ok(())
    }

    /// Libère une page partagée
    pub fn unshare_page(&mut self, frame: PhysFrame) -> Result<(), &'static str> {
        let key = frame.start_address().as_u64();
        
        if let Some(shared) = self.shared_pages.get_mut(&key) {
            if shared.ref_count > 1 {
                shared.ref_count -= 1;
            } else {
                self.shared_pages.remove(&key);
            }
            Ok(())
        } else {
            Err("Page non trouvée dans le gestionnaire CoW")
        }
    }
}

lazy_static! {
    pub static ref COW_MANAGER: Mutex<CowManager> = Mutex::new(CowManager::new());
}
