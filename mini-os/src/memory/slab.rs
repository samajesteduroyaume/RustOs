/// Module SLAB Allocator pour RustOS
/// 
/// Implémente un allocateur de type SLAB pour optimiser les petites allocations.
/// Les objets de tailles communes (32, 64, 128, 256, 512 bytes) sont alloués
/// depuis des caches dédiés pour une performance O(1).

use core::alloc::Layout;
use core::ptr::{null_mut, NonNull};
use spin::Mutex;
use crate::memory::ALLOCATOR;

/// Tailles d'objets supportées par le SLAB allocator
const SLAB_SIZES: [usize; 5] = [32, 64, 128, 256, 512];
const SLAB_PAGE_SIZE: usize = 4096;

/// Représente un slab (page de 4KB divisée en objets de taille fixe)
struct Slab {
    /// Adresse de base du slab
    base: *mut u8,
    /// Taille de chaque objet dans ce slab
    object_size: usize,
    /// Nombre d'objets dans ce slab
    object_count: usize,
    /// Bitmap pour tracker les objets libres (1 = libre, 0 = utilisé)
    /// Chaque bit représente un objet
    bitmap: [u64; 8], // 512 bits max = 512 objets max
    /// Nombre d'objets libres
    free_count: usize,
    /// Pointeur vers le prochain slab dans la liste
    next: Option<NonNull<Slab>>,
}

impl Slab {
    /// Crée un nouveau slab pour une taille d'objet donnée
    unsafe fn new(object_size: usize) -> Option<NonNull<Slab>> {
        // Allouer une page de 4KB depuis le Buddy Allocator
        let layout = Layout::from_size_align_unchecked(SLAB_PAGE_SIZE, SLAB_PAGE_SIZE);
        let page = ALLOCATOR.lock().alloc_block(layout);
        
        if page.is_null() {
            return None;
        }
        
        // Calculer combien d'objets peuvent tenir dans une page
        // Réserver de l'espace pour la structure Slab elle-même
        let slab_header_size = core::mem::size_of::<Slab>();
        let available_space = SLAB_PAGE_SIZE - slab_header_size;
        let object_count = available_space / object_size;
        
        // La structure Slab est au début de la page
        let slab_ptr = page as *mut Slab;
        
        // Initialiser le slab
        (*slab_ptr).base = page.add(slab_header_size);
        (*slab_ptr).object_size = object_size;
        (*slab_ptr).object_count = object_count;
        (*slab_ptr).bitmap = [0xFFFFFFFFFFFFFFFF; 8]; // Tous les bits à 1 (tous libres)
        (*slab_ptr).free_count = object_count;
        (*slab_ptr).next = None;
        
        // Marquer les objets au-delà de object_count comme non disponibles
        for i in object_count..512 {
            let word_idx = i / 64;
            let bit_idx = i % 64;
            (*slab_ptr).bitmap[word_idx] &= !(1u64 << bit_idx);
        }
        
        NonNull::new(slab_ptr)
    }
    
    /// Alloue un objet depuis ce slab
    unsafe fn alloc_object(&mut self) -> Option<*mut u8> {
        if self.free_count == 0 {
            return None;
        }
        
        // Trouver le premier bit à 1 (objet libre)
        for word_idx in 0..8 {
            if self.bitmap[word_idx] != 0 {
                // Trouver le premier bit à 1 dans ce mot
                let bit_idx = self.bitmap[word_idx].trailing_zeros() as usize;
                let object_idx = word_idx * 64 + bit_idx;
                
                if object_idx < self.object_count {
                    // Marquer l'objet comme utilisé
                    self.bitmap[word_idx] &= !(1u64 << bit_idx);
                    self.free_count -= 1;
                    
                    // Calculer l'adresse de l'objet
                    let object_ptr = self.base.add(object_idx * self.object_size);
                    return Some(object_ptr);
                }
            }
        }
        
        None
    }
    
    /// Libère un objet dans ce slab
    unsafe fn dealloc_object(&mut self, ptr: *mut u8) -> bool {
        // Vérifier que le pointeur appartient à ce slab
        let offset = ptr as usize - self.base as usize;
        
        if offset >= self.object_count * self.object_size {
            return false; // Pas dans ce slab
        }
        
        let object_idx = offset / self.object_size;
        
        // Vérifier l'alignement
        if offset % self.object_size != 0 {
            return false; // Pointeur mal aligné
        }
        
        // Marquer l'objet comme libre
        let word_idx = object_idx / 64;
        let bit_idx = object_idx % 64;
        
        // Vérifier que l'objet n'était pas déjà libre (double free)
        if (self.bitmap[word_idx] & (1u64 << bit_idx)) != 0 {
            return false; // Déjà libre
        }
        
        self.bitmap[word_idx] |= 1u64 << bit_idx;
        self.free_count += 1;
        
        true
    }
    
    /// Vérifie si le slab est complètement vide
    fn is_empty(&self) -> bool {
        self.free_count == self.object_count
    }
    
    /// Vérifie si le slab est complètement plein
    fn is_full(&self) -> bool {
        self.free_count == 0
    }
    
    /// Détruit le slab et retourne la mémoire au Buddy Allocator
    unsafe fn destroy(slab_ptr: *mut Slab) {
        let layout = Layout::from_size_align_unchecked(SLAB_PAGE_SIZE, SLAB_PAGE_SIZE);
        ALLOCATOR.lock().dealloc_block(slab_ptr as *mut u8, layout);
    }
}

// SAFETY: Slab est uniquement accédé via un Mutex
unsafe impl Send for Slab {}
unsafe impl Sync for Slab {}

/// Cache pour une taille d'objet spécifique
struct SlabCache {
    /// Taille des objets dans ce cache
    object_size: usize,
    /// Liste des slabs partiellement pleins (ont des objets libres)
    partial_slabs: Option<NonNull<Slab>>,
    /// Liste des slabs complètement pleins
    full_slabs: Option<NonNull<Slab>>,
    /// Nombre total de slabs dans ce cache
    slab_count: usize,
    /// Nombre total d'allocations depuis ce cache
    total_allocations: usize,
    /// Nombre total de libérations dans ce cache
    total_deallocations: usize,
}

impl SlabCache {
    /// Crée un nouveau cache pour une taille d'objet donnée
    const fn new(object_size: usize) -> Self {
        Self {
            object_size,
            partial_slabs: None,
            full_slabs: None,
            slab_count: 0,
            total_allocations: 0,
            total_deallocations: 0,
        }
    }
    
    /// Alloue un objet depuis ce cache
    unsafe fn alloc(&mut self) -> *mut u8 {
        // Essayer d'allouer depuis un slab partiel
        if let Some(mut slab_ptr) = self.partial_slabs {
            let slab = slab_ptr.as_mut();
            
            if let Some(obj) = slab.alloc_object() {
                self.total_allocations += 1;
                
                // Si le slab est maintenant plein, le déplacer vers full_slabs
                if slab.is_full() {
                    self.partial_slabs = slab.next;
                    slab.next = self.full_slabs;
                    self.full_slabs = Some(slab_ptr);
                }
                
                return obj;
            }
        }
        
        // Aucun slab partiel disponible, créer un nouveau slab
        if let Some(mut new_slab_ptr) = Slab::new(self.object_size) {
            let new_slab = new_slab_ptr.as_mut();
            
            // Allouer le premier objet
            if let Some(obj) = new_slab.alloc_object() {
                // Ajouter le nouveau slab à la liste des slabs partiels
                new_slab.next = self.partial_slabs;
                self.partial_slabs = Some(new_slab_ptr);
                self.slab_count += 1;
                self.total_allocations += 1;
                
                return obj;
            }
        }
        
        // Échec de l'allocation
        null_mut()
    }
    
    /// Libère un objet dans ce cache
    unsafe fn dealloc(&mut self, ptr: *mut u8) -> bool {
        // Chercher dans les slabs pleins d'abord
        let mut current_opt = self.full_slabs;
        let mut prev_ptr: *mut Option<NonNull<Slab>> = &mut self.full_slabs;
        
        while let Some(mut current) = current_opt {
            let slab = current.as_mut();
            
            if slab.dealloc_object(ptr) {
                self.total_deallocations += 1;
                
                // Le slab n'est plus plein, le déplacer vers partial_slabs
                *prev_ptr = slab.next;
                slab.next = self.partial_slabs;
                self.partial_slabs = Some(current);
                
                return true;
            }
            
            prev_ptr = &mut current.as_mut().next;
            current_opt = slab.next;
        }
        
        // Chercher dans les slabs partiels
        current_opt = self.partial_slabs;
        prev_ptr = &mut self.partial_slabs;
        
        while let Some(mut current) = current_opt {
            let slab = current.as_mut();
            
            if slab.dealloc_object(ptr) {
                self.total_deallocations += 1;
                
                // Si le slab est maintenant complètement vide, le détruire
                // (mais garder au moins 1 slab vide pour éviter le thrashing)
                if slab.is_empty() && self.slab_count > 1 {
                    *prev_ptr = slab.next;
                    Slab::destroy(current.as_ptr());
                    self.slab_count -= 1;
                }
                
                return true;
            }
            
            prev_ptr = &mut current.as_mut().next;
            current_opt = slab.next;
        }
        
        false
    }
}

// SAFETY: SlabCache est uniquement accédé via un Mutex
unsafe impl Send for SlabCache {}
unsafe impl Sync for SlabCache {}

/// Allocateur SLAB principal
pub struct SlabAllocator {
    /// Caches pour chaque taille d'objet
    caches: [SlabCache; 5],
}

impl SlabAllocator {
    /// Crée un nouveau SLAB allocator
    pub const fn new() -> Self {
        Self {
            caches: [
                SlabCache::new(32),
                SlabCache::new(64),
                SlabCache::new(128),
                SlabCache::new(256),
                SlabCache::new(512),
            ],
        }
    }
    
    /// Trouve le cache approprié pour une taille donnée
    fn find_cache(&mut self, size: usize) -> Option<&mut SlabCache> {
        for cache in &mut self.caches {
            if size <= cache.object_size {
                return Some(cache);
            }
        }
        None
    }
    
    /// Alloue de la mémoire
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if let Some(cache) = self.find_cache(layout.size()) {
            cache.alloc()
        } else {
            null_mut()
        }
    }
    
    /// Libère de la mémoire
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) -> bool {
        if let Some(cache) = self.find_cache(layout.size()) {
            cache.dealloc(ptr)
        } else {
            false
        }
    }
    
    /// Retourne les statistiques du SLAB allocator
    pub fn get_stats(&self) -> SlabStats {
        let mut stats = SlabStats::default();
        
        for cache in &self.caches {
            stats.total_slabs += cache.slab_count;
            stats.total_allocations += cache.total_allocations;
            stats.total_deallocations += cache.total_deallocations;
        }
        
        stats
    }
}

/// Statistiques du SLAB allocator
#[derive(Debug, Clone, Copy, Default)]
pub struct SlabStats {
    pub total_slabs: usize,
    pub total_allocations: usize,
    pub total_deallocations: usize,
}

/// Instance globale du SLAB allocator
pub static SLAB_ALLOCATOR: Mutex<SlabAllocator> = Mutex::new(SlabAllocator::new());

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_slab_basic_allocation() {
        unsafe {
            let mut allocator = SlabAllocator::new();
            
            // Allouer un objet de 32 bytes
            let layout = Layout::from_size_align_unchecked(32, 8);
            let ptr = allocator.alloc(layout);
            
            assert!(!ptr.is_null());
            
            // Libérer l'objet
            assert!(allocator.dealloc(ptr, layout));
        }
    }
    
    #[test_case]
    fn test_slab_multiple_allocations() {
        unsafe {
            let mut allocator = SlabAllocator::new();
            let layout = Layout::from_size_align_unchecked(64, 8);
            
            // Allouer 10 objets
            let mut ptrs = [null_mut(); 10];
            for i in 0..10 {
                ptrs[i] = allocator.alloc(layout);
                assert!(!ptrs[i].is_null());
            }
            
            // Libérer tous les objets
            for ptr in ptrs.iter() {
                assert!(allocator.dealloc(*ptr, layout));
            }
        }
    }
    
    #[test_case]
    fn test_slab_reuse() {
        unsafe {
            let mut allocator = SlabAllocator::new();
            let layout = Layout::from_size_align_unchecked(128, 8);
            
            // Allouer un objet
            let ptr1 = allocator.alloc(layout);
            assert!(!ptr1.is_null());
            
            // Libérer l'objet
            assert!(allocator.dealloc(ptr1, layout));
            
            // Réallouer - devrait réutiliser le même emplacement
            let ptr2 = allocator.alloc(layout);
            assert_eq!(ptr1, ptr2);
        }
    }
}
