/// Module de gestion des Huge Pages
/// 
/// Supporte les pages de 2MB et 1GB pour réduire les TLB misses
/// et améliorer les performances pour les grandes allocations.

use core::ptr::NonNull;
use spin::Mutex;
use x86_64::structures::paging::{PhysFrame, FrameAllocator, Size2MiB, Size1GiB, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

/// Tailles de pages supportées
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    /// Page standard de 4KB
    Small4KB = 4096,
    /// Page moyenne de 2MB
    Medium2MB = 2 * 1024 * 1024,
    /// Grande page de 1GB
    Large1GB = 1024 * 1024 * 1024,
}

impl PageSize {
    /// Retourne la taille en bytes
    pub fn size(&self) -> usize {
        *self as usize
    }
    
    /// Retourne l'alignement requis
    pub fn alignment(&self) -> usize {
        self.size()
    }
}

/// Allocateur de huge pages
pub struct HugePageAllocator {
    /// Pool de pages 2MB libres
    pages_2mb: alloc::vec::Vec<PhysAddr>,
    /// Pool de pages 1GB libres
    pages_1gb: alloc::vec::Vec<PhysAddr>,
    /// Nombre total de pages 2MB allouées
    total_2mb_allocated: usize,
    /// Nombre total de pages 1GB allouées
    total_1gb_allocated: usize,
    /// Adresse de début de la mémoire physique
    phys_mem_start: PhysAddr,
    /// Taille de la mémoire physique
    phys_mem_size: usize,
}

impl HugePageAllocator {
    /// Crée un nouvel allocateur de huge pages
    pub const fn new() -> Self {
        Self {
            pages_2mb: alloc::vec::Vec::new(),
            pages_1gb: alloc::vec::Vec::new(),
            total_2mb_allocated: 0,
            total_1gb_allocated: 0,
            phys_mem_start: PhysAddr::new(0),
            phys_mem_size: 0,
        }
    }
    
    /// Initialise l'allocateur avec la mémoire physique disponible
    /// 
    /// # Safety
    /// Doit être appelé une seule fois au démarrage
    pub unsafe fn init(&mut self, phys_mem_start: PhysAddr, phys_mem_size: usize) {
        self.phys_mem_start = phys_mem_start;
        self.phys_mem_size = phys_mem_size;
        
        // Pré-allouer quelques huge pages pour le pool
        self.populate_pools();
    }
    
    /// Remplit les pools de huge pages
    unsafe fn populate_pools(&mut self) {
        let start = self.phys_mem_start.as_u64() as usize;
        let end = start + self.phys_mem_size;
        
        // Aligner au début de la première page 2MB
        let mut current = (start + PageSize::Medium2MB.size() - 1) & !(PageSize::Medium2MB.size() - 1);
        
        // Ajouter des pages 2MB au pool
        while current + PageSize::Medium2MB.size() <= end && self.pages_2mb.len() < 64 {
            self.pages_2mb.push(PhysAddr::new(current as u64));
            current += PageSize::Medium2MB.size();
        }
        
        // Aligner au début de la première page 1GB
        current = (start + PageSize::Large1GB.size() - 1) & !(PageSize::Large1GB.size() - 1);
        
        // Ajouter des pages 1GB au pool (moins nombreuses)
        while current + PageSize::Large1GB.size() <= end && self.pages_1gb.len() < 8 {
            self.pages_1gb.push(PhysAddr::new(current as u64));
            current += PageSize::Large1GB.size();
        }
    }
    
    /// Alloue une huge page de 2MB
    /// 
    /// Retourne None si aucune page n'est disponible
    pub fn alloc_2mb(&mut self) -> Option<PhysAddr> {
        if let Some(addr) = self.pages_2mb.pop() {
            self.total_2mb_allocated += 1;
            Some(addr)
        } else {
            // Essayer d'allouer une nouvelle page depuis la mémoire physique
            self.try_alloc_new_2mb()
        }
    }
    
    /// Alloue une huge page de 1GB
    /// 
    /// Retourne None si aucune page n'est disponible
    pub fn alloc_1gb(&mut self) -> Option<PhysAddr> {
        if let Some(addr) = self.pages_1gb.pop() {
            self.total_1gb_allocated += 1;
            Some(addr)
        } else {
            // Essayer d'allouer une nouvelle page depuis la mémoire physique
            self.try_alloc_new_1gb()
        }
    }
    
    /// Libère une huge page de 2MB
    pub fn dealloc_2mb(&mut self, addr: PhysAddr) {
        // Vérifier l'alignement
        if addr.as_u64() % PageSize::Medium2MB.size() as u64 == 0 {
            self.pages_2mb.push(addr);
            self.total_2mb_allocated = self.total_2mb_allocated.saturating_sub(1);
        }
    }
    
    /// Libère une huge page de 1GB
    pub fn dealloc_1gb(&mut self, addr: PhysAddr) {
        // Vérifier l'alignement
        if addr.as_u64() % PageSize::Large1GB.size() as u64 == 0 {
            self.pages_1gb.push(addr);
            self.total_1gb_allocated = self.total_1gb_allocated.saturating_sub(1);
        }
    }
    
    /// Essaie d'allouer une nouvelle page 2MB depuis la mémoire physique
    fn try_alloc_new_2mb(&mut self) -> Option<PhysAddr> {
        // Chercher un espace libre aligné sur 2MB
        let start = self.phys_mem_start.as_u64() as usize;
        let end = start + self.phys_mem_size;
        
        let mut current = (start + PageSize::Medium2MB.size() - 1) & !(PageSize::Medium2MB.size() - 1);
        
        // Avancer jusqu'à trouver un espace libre
        // (Dans une vraie implémentation, on devrait tracker les zones utilisées)
        current += self.total_2mb_allocated * PageSize::Medium2MB.size();
        
        if current + PageSize::Medium2MB.size() <= end {
            self.total_2mb_allocated += 1;
            Some(PhysAddr::new(current as u64))
        } else {
            None
        }
    }
    
    /// Essaie d'allouer une nouvelle page 1GB depuis la mémoire physique
    fn try_alloc_new_1gb(&mut self) -> Option<PhysAddr> {
        // Chercher un espace libre aligné sur 1GB
        let start = self.phys_mem_start.as_u64() as usize;
        let end = start + self.phys_mem_size;
        
        let mut current = (start + PageSize::Large1GB.size() - 1) & !(PageSize::Large1GB.size() - 1);
        
        // Avancer jusqu'à trouver un espace libre
        current += self.total_1gb_allocated * PageSize::Large1GB.size();
        
        if current + PageSize::Large1GB.size() <= end {
            self.total_1gb_allocated += 1;
            Some(PhysAddr::new(current as u64))
        } else {
            None
        }
    }
    
    /// Retourne les statistiques de l'allocateur
    pub fn get_stats(&self) -> HugePageStats {
        HugePageStats {
            pages_2mb_free: self.pages_2mb.len(),
            pages_2mb_allocated: self.total_2mb_allocated,
            pages_1gb_free: self.pages_1gb.len(),
            pages_1gb_allocated: self.total_1gb_allocated,
        }
    }
}

/// Statistiques des huge pages
#[derive(Debug, Clone, Copy)]
pub struct HugePageStats {
    /// Nombre de pages 2MB libres
    pub pages_2mb_free: usize,
    /// Nombre de pages 2MB allouées
    pub pages_2mb_allocated: usize,
    /// Nombre de pages 1GB libres
    pub pages_1gb_free: usize,
    /// Nombre de pages 1GB allouées
    pub pages_1gb_allocated: usize,
}

impl HugePageStats {
    /// Retourne la mémoire totale allouée en bytes
    pub fn total_allocated_bytes(&self) -> usize {
        self.pages_2mb_allocated * PageSize::Medium2MB.size() +
        self.pages_1gb_allocated * PageSize::Large1GB.size()
    }
    
    /// Retourne la mémoire totale libre en bytes
    pub fn total_free_bytes(&self) -> usize {
        self.pages_2mb_free * PageSize::Medium2MB.size() +
        self.pages_1gb_free * PageSize::Large1GB.size()
    }
}

/// Instance globale de l'allocateur de huge pages
pub static HUGE_PAGE_ALLOCATOR: Mutex<HugePageAllocator> = Mutex::new(HugePageAllocator::new());

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_page_size() {
        assert_eq!(PageSize::Small4KB.size(), 4096);
        assert_eq!(PageSize::Medium2MB.size(), 2 * 1024 * 1024);
        assert_eq!(PageSize::Large1GB.size(), 1024 * 1024 * 1024);
    }
    
    #[test_case]
    fn test_page_alignment() {
        assert_eq!(PageSize::Medium2MB.alignment(), 2 * 1024 * 1024);
        assert_eq!(PageSize::Large1GB.alignment(), 1024 * 1024 * 1024);
    }
    
    #[test_case]
    fn test_huge_page_stats() {
        let stats = HugePageStats {
            pages_2mb_free: 10,
            pages_2mb_allocated: 5,
            pages_1gb_free: 2,
            pages_1gb_allocated: 1,
        };
        
        assert_eq!(stats.total_allocated_bytes(), 
                   5 * 2 * 1024 * 1024 + 1 * 1024 * 1024 * 1024);
    }
}
