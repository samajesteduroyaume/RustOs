/// Module de Demand Paging (Pagination à la demande)
/// 
/// Permet d'allouer des pages virtuelles sans allouer immédiatement
/// les pages physiques correspondantes. Les pages physiques sont allouées
/// uniquement lors du premier accès (page fault).

use alloc::collections::BTreeMap;
use x86_64::{VirtAddr, PhysAddr};
use x86_64::structures::paging::PageTableFlags;
use spin::Mutex;

/// Page lazy (non encore allouée physiquement)
#[derive(Debug, Clone)]
pub struct LazyPage {
    /// Adresse virtuelle
    pub virt_addr: VirtAddr,
    /// Page physique allouée (None si pas encore allouée)
    pub phys_addr: Option<PhysAddr>,
    /// Flags de la page
    pub flags: PageTableFlags,
    /// Nombre d'accès
    pub access_count: usize,
}

impl LazyPage {
    /// Crée une nouvelle page lazy
    pub fn new(virt_addr: VirtAddr, flags: PageTableFlags) -> Self {
        Self {
            virt_addr,
            phys_addr: None,
            flags,
            access_count: 0,
        }
    }
    
    /// Vérifie si la page est allouée
    pub fn is_allocated(&self) -> bool {
        self.phys_addr.is_some()
    }
}

/// Gestionnaire de demand paging
pub struct DemandPagingManager {
    /// Pages lazy indexées par adresse virtuelle
    lazy_pages: BTreeMap<u64, LazyPage>,
    /// Nombre total de pages lazy
    total_lazy_pages: usize,
    /// Nombre de page faults traités
    page_faults_handled: usize,
}

impl DemandPagingManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            lazy_pages: BTreeMap::new(),
            total_lazy_pages: 0,
            page_faults_handled: 0,
        }
    }
    
    /// Enregistre une page lazy
    pub fn register_lazy_page(&mut self, virt_addr: VirtAddr, flags: PageTableFlags) {
        let page = LazyPage::new(virt_addr, flags);
        self.lazy_pages.insert(virt_addr.as_u64(), page);
        self.total_lazy_pages += 1;
    }
    
    /// Traite un page fault
    /// 
    /// Retourne true si c'était une page lazy et qu'elle a été allouée
    pub fn handle_page_fault(&mut self, fault_addr: VirtAddr) -> bool {
        // Aligner l'adresse sur une page
        let page_addr = fault_addr.align_down(4096u64);
        
        if let Some(lazy_page) = self.lazy_pages.get_mut(&page_addr.as_u64()) {
            // C'est une page lazy
            if !lazy_page.is_allocated() {
                // Allouer la page physique
                // TODO: utiliser un vrai allocateur de frames
                let phys_addr = PhysAddr::new(0x1000_0000); // Placeholder
                lazy_page.phys_addr = Some(phys_addr);
                lazy_page.access_count += 1;
                
                // TODO: mapper la page dans la table de pages
                
                self.page_faults_handled += 1;
                return true;
            }
        }
        
        false
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> DemandPagingStats {
        DemandPagingStats {
            total_lazy_pages: self.total_lazy_pages,
            allocated_pages: self.lazy_pages.values().filter(|p| p.is_allocated()).count(),
            page_faults_handled: self.page_faults_handled,
        }
    }
}

/// Statistiques de demand paging
#[derive(Debug, Clone, Copy)]
pub struct DemandPagingStats {
    pub total_lazy_pages: usize,
    pub allocated_pages: usize,
    pub page_faults_handled: usize,
}

/// Instance globale du gestionnaire de demand paging
pub static DEMAND_PAGING_MANAGER: Mutex<DemandPagingManager> = Mutex::new(DemandPagingManager::new());

/// Handler de page fault (à appeler depuis le gestionnaire d'interruptions)
pub fn handle_page_fault(fault_addr: VirtAddr) -> bool {
    DEMAND_PAGING_MANAGER.lock().handle_page_fault(fault_addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_lazy_page() {
        let page = LazyPage::new(VirtAddr::new(0x1000), PageTableFlags::PRESENT);
        assert!(!page.is_allocated());
        assert_eq!(page.access_count, 0);
    }
    
    #[test_case]
    fn test_demand_paging_register() {
        let mut manager = DemandPagingManager::new();
        manager.register_lazy_page(VirtAddr::new(0x1000), PageTableFlags::PRESENT);
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_lazy_pages, 1);
        assert_eq!(stats.allocated_pages, 0);
    }
}
