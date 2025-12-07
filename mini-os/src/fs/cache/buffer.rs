/// Module de Buffer Cache
/// 
/// Cache les blocs disque en mémoire pour améliorer les performances I/O

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Taille d'un bloc (4KB)
pub const BLOCK_SIZE: usize = 4096;

/// Entrée de cache pour un bloc disque
#[derive(Debug, Clone)]
pub struct BufferCacheEntry {
    /// Numéro de bloc
    pub block_num: u64,
    /// Données du bloc
    pub data: Vec<u8>,
    /// Bloc modifié (dirty)
    pub dirty: bool,
    /// Timestamp dernier accès
    pub last_access: u64,
    /// Nombre d'accès
    pub access_count: usize,
}

impl BufferCacheEntry {
    /// Crée une nouvelle entrée
    pub fn new(block_num: u64, data: Vec<u8>) -> Self {
        Self {
            block_num,
            data,
            dirty: false,
            last_access: 0,
            access_count: 0,
        }
    }
    
    /// Marque comme accédé
    pub fn mark_accessed(&mut self) {
        self.access_count += 1;
        self.last_access = Self::get_timestamp();
    }
    
    /// Marque comme modifié
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    /// Obtient un timestamp (placeholder)
    fn get_timestamp() -> u64 {
        // TODO: Utiliser un vrai timestamp
        0
    }
}

/// Cache de blocs disque
pub struct BufferCache {
    /// Entrées de cache indexées par numéro de bloc
    entries: BTreeMap<u64, BufferCacheEntry>,
    /// Taille maximale du cache (en nombre de blocs)
    max_entries: usize,
    /// Nombre de hits
    hits: usize,
    /// Nombre de misses
    misses: usize,
    /// Nombre de writebacks
    writebacks: usize,
    /// Nombre d'évictions
    evictions: usize,
}

impl BufferCache {
    /// Crée un nouveau cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries,
            hits: 0,
            misses: 0,
            writebacks: 0,
            evictions: 0,
        }
    }
    
    /// Lit un bloc depuis le cache
    /// 
    /// Retourne None si le bloc n'est pas en cache
    pub fn read_block(&mut self, block_num: u64) -> Option<Vec<u8>> {
        if let Some(entry) = self.entries.get_mut(&block_num) {
            entry.mark_accessed();
            self.hits += 1;
            Some(entry.data.clone())
        } else {
            self.misses += 1;
            None
        }
    }
    
    /// Écrit un bloc dans le cache
    /// 
    /// Si le cache est plein, évince le bloc LRU
    pub fn write_block(&mut self, block_num: u64, data: Vec<u8>) {
        // Vérifier si le cache est plein
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&block_num) {
            self.evict_lru();
        }
        
        if let Some(entry) = self.entries.get_mut(&block_num) {
            // Bloc déjà en cache, mettre à jour
            entry.data = data;
            entry.mark_dirty();
            entry.mark_accessed();
        } else {
            // Nouveau bloc
            let mut entry = BufferCacheEntry::new(block_num, data);
            entry.mark_dirty();
            entry.mark_accessed();
            self.entries.insert(block_num, entry);
        }
    }
    
    /// Flush un bloc spécifique vers le disque
    pub fn flush_block(&mut self, block_num: u64) -> Option<Vec<u8>> {
        if let Some(entry) = self.entries.get_mut(&block_num) {
            if entry.dirty {
                entry.dirty = false;
                self.writebacks += 1;
                Some(entry.data.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Flush tous les blocs dirty vers le disque
    pub fn flush_all(&mut self) -> Vec<(u64, Vec<u8>)> {
        // Collecter les blocs dirty
        let dirty_blocks: Vec<(u64, Vec<u8>)> = self.entries
            .iter()
            .filter(|(_, entry)| entry.dirty)
            .map(|(k, v)| (*k, v.data.clone()))
            .collect();
        
        // Marquer comme non-dirty
        for (block_num, _) in &dirty_blocks {
            if let Some(entry) = self.entries.get_mut(block_num) {
                entry.dirty = false;
                self.writebacks += 1;
            }
        }
        
        dirty_blocks
    }
    
    /// Évince le bloc LRU (Least Recently Used)
    fn evict_lru(&mut self) {
        // Trouver le bloc avec le plus ancien last_access
        let lru_block = self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(k, _)| *k);
        
        if let Some(block_num) = lru_block {
            if let Some(entry) = self.entries.remove(&block_num) {
                if entry.dirty {
                    // TODO: Écrire le bloc sur disque avant de l'évincer
                    self.writebacks += 1;
                }
                self.evictions += 1;
            }
        }
    }
    
    /// Invalide un bloc (le retire du cache)
    pub fn invalidate_block(&mut self, block_num: u64) {
        self.entries.remove(&block_num);
    }
    
    /// Invalide tous les blocs
    pub fn invalidate_all(&mut self) {
        self.entries.clear();
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> BufferCacheStats {
        let total_requests = self.hits + self.misses;
        let hit_rate = if total_requests > 0 {
            (self.hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        BufferCacheStats {
            total_entries: self.entries.len(),
            max_entries: self.max_entries,
            hits: self.hits,
            misses: self.misses,
            hit_rate,
            writebacks: self.writebacks,
            evictions: self.evictions,
            dirty_blocks: self.entries.values().filter(|e| e.dirty).count(),
        }
    }
}

/// Statistiques du buffer cache
#[derive(Debug, Clone)]
pub struct BufferCacheStats {
    pub total_entries: usize,
    pub max_entries: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
    pub writebacks: usize,
    pub evictions: usize,
    pub dirty_blocks: usize,
}

/// Instance globale du buffer cache
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUFFER_CACHE: Mutex<BufferCache> = Mutex::new(BufferCache::new(1024));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_buffer_cache_creation() {
        let cache = BufferCache::new(10);
        assert_eq!(cache.max_entries, 10);
        assert_eq!(cache.hits, 0);
        assert_eq!(cache.misses, 0);
    }
    
    #[test_case]
    fn test_read_write() {
        let mut cache = BufferCache::new(10);
        let data = vec![1, 2, 3, 4];
        
        // Write
        cache.write_block(1, data.clone());
        
        // Read hit
        let read_data = cache.read_block(1);
        assert!(read_data.is_some());
        assert_eq!(read_data.unwrap(), data);
        assert_eq!(cache.hits, 1);
        
        // Read miss
        let miss_data = cache.read_block(999);
        assert!(miss_data.is_none());
        assert_eq!(cache.misses, 1);
    }
    
    #[test_case]
    fn test_lru_eviction() {
        let mut cache = BufferCache::new(2);
        
        cache.write_block(1, vec![1]);
        cache.write_block(2, vec![2]);
        cache.write_block(3, vec![3]); // Devrait évincer le bloc 1
        
        assert_eq!(cache.entries.len(), 2);
        assert_eq!(cache.evictions, 1);
        assert!(cache.read_block(1).is_none());
    }
    
    #[test_case]
    fn test_flush() {
        let mut cache = BufferCache::new(10);
        cache.write_block(1, vec![1, 2, 3]);
        cache.write_block(2, vec![4, 5, 6]);
        
        let flushed = cache.flush_all();
        assert_eq!(flushed.len(), 2);
        assert_eq!(cache.writebacks, 2);
    }
}
