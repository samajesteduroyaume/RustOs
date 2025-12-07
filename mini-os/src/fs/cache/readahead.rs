/// Module Read-Ahead
/// 
/// Détecte les lectures séquentielles et pré-charge les blocs suivants

use alloc::collections::BTreeMap;
use spin::Mutex;
use super::buffer::BUFFER_CACHE;

/// Contexte de read-ahead pour un fichier/device
#[derive(Debug, Clone)]
struct ReadAheadContext {
    /// Dernier bloc lu
    last_block: u64,
    /// Nombre de lectures séquentielles détectées
    sequential_count: usize,
    /// Taille de la fenêtre de read-ahead
    window_size: usize,
}

impl ReadAheadContext {
    fn new() -> Self {
        Self {
            last_block: 0,
            sequential_count: 0,
            window_size: 4, // Commencer avec 4 blocs
        }
    }
    
    /// Met à jour le contexte avec une nouvelle lecture
    fn update(&mut self, block_num: u64) -> bool {
        // Vérifier si c'est une lecture séquentielle
        if block_num == self.last_block + 1 {
            self.sequential_count += 1;
            
            // Augmenter la fenêtre si beaucoup de lectures séquentielles
            if self.sequential_count > 10 && self.window_size < 32 {
                self.window_size *= 2;
            }
            
            self.last_block = block_num;
            true
        } else {
            // Lecture aléatoire, réinitialiser
            self.sequential_count = 0;
            self.window_size = 4;
            self.last_block = block_num;
            false
        }
    }
}

/// Gestionnaire de read-ahead
pub struct ReadAheadManager {
    /// Contextes par device/fichier
    contexts: BTreeMap<u64, ReadAheadContext>,
    /// Nombre de blocs pré-chargés
    prefetched_blocks: usize,
    /// Nombre de hits sur blocs pré-chargés
    prefetch_hits: usize,
    /// Activer le read-ahead
    enabled: bool,
}

impl ReadAheadManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            contexts: BTreeMap::new(),
            prefetched_blocks: 0,
            prefetch_hits: 0,
            enabled: true,
        }
    }
    
    /// Notifie une lecture de bloc
    /// 
    /// Retourne true si du read-ahead a été effectué
    pub fn on_read(&mut self, device_id: u64, block_num: u64) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Obtenir ou créer le contexte
        let context = self.contexts.entry(device_id).or_insert_with(ReadAheadContext::new);
        
        // Mettre à jour et vérifier si séquentiel
        let is_sequential = context.update(block_num);
        let should_prefetch = is_sequential && context.sequential_count >= 2;
        let window_size = context.window_size;
        
        if should_prefetch {
            // Effectuer le read-ahead
            self.prefetch_blocks(device_id, block_num + 1, window_size);
            true
        } else {
            false
        }
    }
    
    /// Pré-charge des blocs
    fn prefetch_blocks(&mut self, _device_id: u64, start_block: u64, count: usize) {
        let mut cache = BUFFER_CACHE.lock();
        
        for i in 0..count {
            let block_num = start_block + i as u64;
            
            // Vérifier si le bloc n'est pas déjà en cache
            if cache.read_block(block_num).is_none() {
                // TODO: Lire le bloc depuis le disque de manière asynchrone
                // Pour l'instant, juste compter
                self.prefetched_blocks += 1;
            }
        }
        
        drop(cache);
    }
    
    /// Notifie un hit sur un bloc pré-chargé
    pub fn on_prefetch_hit(&mut self) {
        self.prefetch_hits += 1;
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> ReadAheadStats {
        let hit_rate = if self.prefetched_blocks > 0 {
            (self.prefetch_hits as f64 / self.prefetched_blocks as f64) * 100.0
        } else {
            0.0
        };
        
        ReadAheadStats {
            prefetched_blocks: self.prefetched_blocks,
            prefetch_hits: self.prefetch_hits,
            hit_rate,
            active_contexts: self.contexts.len(),
            enabled: self.enabled,
        }
    }
}

/// Statistiques du read-ahead
#[derive(Debug, Clone)]
pub struct ReadAheadStats {
    pub prefetched_blocks: usize,
    pub prefetch_hits: usize,
    pub hit_rate: f64,
    pub active_contexts: usize,
    pub enabled: bool,
}

/// Instance globale du read-ahead manager
use lazy_static::lazy_static;

lazy_static! {
    pub static ref READAHEAD_MANAGER: Mutex<ReadAheadManager> = Mutex::new(ReadAheadManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_readahead_manager_creation() {
        let manager = ReadAheadManager::new();
        assert!(manager.enabled);
        assert_eq!(manager.prefetched_blocks, 0);
    }
    
    #[test_case]
    fn test_sequential_detection() {
        let mut manager = ReadAheadManager::new();
        
        // Première lecture
        assert!(!manager.on_read(0, 1));
        
        // Deuxième lecture séquentielle
        assert!(!manager.on_read(0, 2));
        
        // Troisième lecture séquentielle, devrait déclencher read-ahead
        assert!(manager.on_read(0, 3));
        assert!(manager.prefetched_blocks > 0);
    }
    
    #[test_case]
    fn test_random_reads() {
        let mut manager = ReadAheadManager::new();
        
        // Lectures aléatoires
        assert!(!manager.on_read(0, 1));
        assert!(!manager.on_read(0, 10));
        assert!(!manager.on_read(0, 5));
        
        // Pas de read-ahead
        assert_eq!(manager.prefetched_blocks, 0);
    }
}
