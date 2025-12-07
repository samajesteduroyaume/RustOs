/// Module Hybrid Allocator pour RustOS
/// 
/// Combine le SLAB allocator (pour petites allocations) et le Buddy allocator
/// (pour grandes allocations) pour obtenir les meilleures performances.
/// 
/// Dispatch automatique :
/// - Taille ≤ 512 bytes → SLAB (O(1), faible fragmentation)
/// - Taille > 512 bytes → Buddy (O(log n), bon pour grandes allocations)

use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;
use crate::memory::{BuddyAllocator, BuddyStats};
use crate::memory::slab::{SlabAllocator, SlabStats};

/// Seuil de dispatch entre SLAB et Buddy (en bytes)
const HYBRID_THRESHOLD: usize = 512;

/// Allocateur hybride combinant SLAB et Buddy
pub struct HybridAllocator {
    /// SLAB allocator pour petites allocations
    slab: Mutex<SlabAllocator>,
    /// Buddy allocator pour grandes allocations
    buddy: Mutex<BuddyAllocator>,
    /// Seuil de dispatch (512 bytes par défaut)
    threshold: usize,
}

impl HybridAllocator {
    /// Crée un nouveau allocateur hybride
    pub const fn new() -> Self {
        Self {
            slab: Mutex::new(SlabAllocator::new()),
            buddy: Mutex::new(BuddyAllocator::new()),
            threshold: HYBRID_THRESHOLD,
        }
    }
    
    /// Initialise l'allocateur hybride
    /// 
    /// # Safety
    /// Doit être appelé une seule fois au démarrage du système
    pub unsafe fn init(&self, start: usize, size: usize) {
        self.buddy.lock().init(start, size);
    }
    
    /// Retourne les statistiques combinées
    pub fn get_stats(&self) -> HybridStats {
        let slab_stats = self.slab.lock().get_stats();
        let buddy_stats = self.buddy.lock().get_stats();
        
        HybridStats {
            slab: slab_stats,
            buddy: buddy_stats,
            threshold: self.threshold,
        }
    }
    
    /// Retourne le seuil de dispatch actuel
    pub fn threshold(&self) -> usize {
        self.threshold
    }
}

unsafe impl GlobalAlloc for HybridAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() <= self.threshold {
            // Petite allocation → SLAB
            let ptr = self.slab.lock().alloc(layout);
            
            if !ptr.is_null() {
                return ptr;
            }
            
            // Fallback vers Buddy si SLAB échoue
            // (peut arriver si la taille n'est pas dans les caches SLAB)
        }
        
        // Grande allocation → Buddy
        self.buddy.lock().alloc_block(layout)
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() <= self.threshold {
            // Essayer de libérer depuis SLAB
            if self.slab.lock().dealloc(ptr, layout) {
                return; // Succès
            }
            
            // Si SLAB ne reconnaît pas le pointeur, c'est du Buddy
        }
        
        // Libérer depuis Buddy
        self.buddy.lock().dealloc_block(ptr, layout);
    }
}

/// Statistiques de l'allocateur hybride
#[derive(Debug, Clone, Copy)]
pub struct HybridStats {
    /// Statistiques du SLAB allocator
    pub slab: SlabStats,
    /// Statistiques du Buddy allocator
    pub buddy: BuddyStats,
    /// Seuil de dispatch (bytes)
    pub threshold: usize,
}

impl HybridStats {
    /// Retourne le nombre total d'allocations
    pub fn total_allocations(&self) -> usize {
        self.slab.total_allocations + self.buddy.total_allocations
    }
    
    /// Retourne le nombre total de libérations
    pub fn total_deallocations(&self) -> usize {
        self.slab.total_deallocations + self.buddy.total_deallocations
    }
    
    /// Retourne le ratio SLAB/Buddy (pourcentage d'allocations via SLAB)
    pub fn slab_ratio(&self) -> f32 {
        let total = self.total_allocations();
        if total == 0 {
            return 0.0;
        }
        (self.slab.total_allocations as f32 / total as f32) * 100.0
    }
    
    /// Affiche un rapport détaillé
    pub fn print_report(&self) {
        // Note: Dans un vrai OS, on utiliserait println! ou un logger
        // Pour l'instant, cette méthode est un placeholder
    }
}

/// Instance globale de l'allocateur hybride
#[global_allocator]
pub static HYBRID_ALLOCATOR: HybridAllocator = HybridAllocator::new();

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_hybrid_small_allocation() {
        unsafe {
            // Allouer 64 bytes → devrait aller vers SLAB
            let layout = Layout::from_size_align_unchecked(64, 8);
            let ptr = HYBRID_ALLOCATOR.alloc(layout);
            
            assert!(!ptr.is_null());
            
            // Vérifier les stats
            let stats = HYBRID_ALLOCATOR.get_stats();
            assert!(stats.slab.total_allocations > 0);
            
            // Libérer
            HYBRID_ALLOCATOR.dealloc(ptr, layout);
        }
    }
    
    #[test_case]
    fn test_hybrid_large_allocation() {
        unsafe {
            // Allouer 4096 bytes → devrait aller vers Buddy
            let layout = Layout::from_size_align_unchecked(4096, 4096);
            let ptr = HYBRID_ALLOCATOR.alloc(layout);
            
            assert!(!ptr.is_null());
            
            // Vérifier les stats
            let stats = HYBRID_ALLOCATOR.get_stats();
            assert!(stats.buddy.total_allocations > 0);
            
            // Libérer
            HYBRID_ALLOCATOR.dealloc(ptr, layout);
        }
    }
    
    #[test_case]
    fn test_hybrid_threshold() {
        unsafe {
            let stats_before = HYBRID_ALLOCATOR.get_stats();
            
            // Allouer juste en dessous du seuil (512 bytes)
            let layout_small = Layout::from_size_align_unchecked(512, 8);
            let ptr_small = HYBRID_ALLOCATOR.alloc(layout_small);
            
            // Allouer juste au-dessus du seuil (513 bytes)
            let layout_large = Layout::from_size_align_unchecked(513, 8);
            let ptr_large = HYBRID_ALLOCATOR.alloc(layout_large);
            
            let stats_after = HYBRID_ALLOCATOR.get_stats();
            
            // Vérifier que SLAB a été utilisé pour la petite allocation
            assert!(stats_after.slab.total_allocations > stats_before.slab.total_allocations);
            
            // Vérifier que Buddy a été utilisé pour la grande allocation
            assert!(stats_after.buddy.total_allocations > stats_before.buddy.total_allocations);
            
            // Libérer
            HYBRID_ALLOCATOR.dealloc(ptr_small, layout_small);
            HYBRID_ALLOCATOR.dealloc(ptr_large, layout_large);
        }
    }
    
    #[test_case]
    fn test_hybrid_mixed_workload() {
        unsafe {
            let mut ptrs = Vec::new();
            
            // Mélange d'allocations petites et grandes
            for i in 0..10 {
                let size = if i % 2 == 0 { 64 } else { 4096 };
                let layout = Layout::from_size_align_unchecked(size, 8);
                let ptr = HYBRID_ALLOCATOR.alloc(layout);
                ptrs.push((ptr, layout));
            }
            
            // Vérifier les stats
            let stats = HYBRID_ALLOCATOR.get_stats();
            assert!(stats.slab.total_allocations >= 5); // Au moins 5 petites
            assert!(stats.buddy.total_allocations >= 5); // Au moins 5 grandes
            
            // Libérer tout
            for (ptr, layout) in ptrs {
                HYBRID_ALLOCATOR.dealloc(ptr, layout);
            }
        }
    }
}
