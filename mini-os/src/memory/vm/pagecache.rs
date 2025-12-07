/// Module de Page Cache pour fichiers
/// 
/// Cache les pages de fichiers en mémoire pour améliorer les performances
/// des opérations de lecture/écriture.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Entrée de cache pour une page de fichier
#[derive(Debug, Clone)]
pub struct PageCacheEntry {
    /// ID du fichier (inode number)
    pub file_id: u64,
    /// Offset de la page dans le fichier
    pub page_offset: u64,
    /// Données de la page (4KB)
    pub data: Vec<u8>,
    /// Page modifiée (dirty)
    pub dirty: bool,
    /// Timestamp du dernier accès
    pub last_access: u64,
    /// Nombre d'accès
    pub access_count: usize,
}

impl PageCacheEntry {
    /// Crée une nouvelle entrée de cache
    pub fn new(file_id: u64, page_offset: u64, data: Vec<u8>) -> Self {
        Self {
            file_id,
            page_offset,
            data,
            dirty: false,
            last_access: 0, // TODO: timestamp réel
            access_count: 0,
        }
    }
    
    /// Marque la page comme accédée
    pub fn touch(&mut self) {
        self.last_access = 0; // TODO: timestamp réel
        self.access_count += 1;
    }
}

/// Cache de pages de fichiers
pub struct PageCache {
    /// Entrées de cache indexées par (file_id, page_offset)
    entries: BTreeMap<(u64, u64), PageCacheEntry>,
    /// Nombre maximum d'entrées
    max_entries: usize,
    /// Nombre de hits (lectures depuis le cache)
    cache_hits: usize,
    /// Nombre de misses (lectures depuis le disque)
    cache_misses: usize,
    /// Nombre de writebacks (écritures vers le disque)
    writebacks: usize,
}

impl PageCache {
    /// Crée un nouveau cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries,
            cache_hits: 0,
            cache_misses: 0,
            writebacks: 0,
        }
    }
    
    /// Lit une page depuis le cache
    /// 
    /// Retourne Some(data) si la page est dans le cache, None sinon
    pub fn read_page(&mut self, file_id: u64, page_offset: u64) -> Option<Vec<u8>> {
        let key = (file_id, page_offset);
        
        if let Some(entry) = self.entries.get_mut(&key) {
            // Cache hit
            entry.touch();
            self.cache_hits += 1;
            Some(entry.data.clone())
        } else {
            // Cache miss
            self.cache_misses += 1;
            None
        }
    }
    
    /// Écrit une page dans le cache
    pub fn write_page(&mut self, file_id: u64, page_offset: u64, data: Vec<u8>) {
        let key = (file_id, page_offset);
        
        // Si le cache est plein, évincer une page
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&key) {
            self.evict_lru();
        }
        
        // Ajouter ou mettre à jour l'entrée
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.data = data;
            entry.dirty = true;
            entry.touch();
        } else {
            let mut entry = PageCacheEntry::new(file_id, page_offset, data);
            entry.dirty = true;
            self.entries.insert(key, entry);
        }
    }
    
    /// Flush toutes les pages dirty vers le disque
    pub fn flush_all(&mut self) {
        let keys: Vec<_> = self.entries.iter()
            .filter(|(_, e)| e.dirty)
            .map(|(k, _)| *k)
            .collect();
        
        for key in keys {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.dirty = false;
                self.writebacks += 1;
            }
        }
    }
    
    /// Flush les pages dirty d'un fichier spécifique
    pub fn flush_file(&mut self, file_id: u64) {
        let keys: Vec<_> = self.entries.iter()
            .filter(|((fid, _), e)| *fid == file_id && e.dirty)
            .map(|(k, _)| *k)
            .collect();
        
        for key in keys {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.dirty = false;
                self.writebacks += 1;
            }
        }
    }
    
    /// Évince la page LRU (la moins récemment utilisée)
    fn evict_lru(&mut self) {
        // Trouver l'entrée avec le plus petit last_access
        let lru_key = self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(key, _)| *key);
        
        if let Some(key) = lru_key {
            if let Some(entry) = self.entries.remove(&key) {
                // Si la page est dirty, l'écrire sur disque
                if entry.dirty {
                    self.writeback_page(&entry);
                }
            }
        }
    }
    
    /// Écrit une page sur le disque
    fn writeback_page(&mut self, entry: &PageCacheEntry) {
        // TODO: implémenter l'écriture réelle sur disque
        self.writebacks += 1;
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> PageCacheStats {
        let dirty_pages = self.entries.values().filter(|e| e.dirty).count();
        let hit_rate = if self.cache_hits + self.cache_misses > 0 {
            (self.cache_hits as f32 / (self.cache_hits + self.cache_misses) as f32) * 100.0
        } else {
            0.0
        };
        
        PageCacheStats {
            total_entries: self.entries.len(),
            max_entries: self.max_entries,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            hit_rate,
            dirty_pages,
            writebacks: self.writebacks,
        }
    }
}

/// Statistiques du page cache
#[derive(Debug, Clone, Copy)]
pub struct PageCacheStats {
    pub total_entries: usize,
    pub max_entries: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub hit_rate: f32,
    pub dirty_pages: usize,
    pub writebacks: usize,
}

/// Instance globale du page cache
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PAGE_CACHE: Mutex<PageCache> = Mutex::new(PageCache::new(512));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_page_cache_read_miss() {
        let mut cache = PageCache::new(10);
        let result = cache.read_page(1, 0);
        
        assert!(result.is_none());
        assert_eq!(cache.cache_misses, 1);
    }
    
    #[test_case]
    fn test_page_cache_write_read() {
        let mut cache = PageCache::new(10);
        let data = vec![1, 2, 3, 4];
        
        cache.write_page(1, 0, data.clone());
        let result = cache.read_page(1, 0);
        
        assert_eq!(result, Some(data));
        assert_eq!(cache.cache_hits, 1);
    }
    
    #[test_case]
    fn test_page_cache_eviction() {
        let mut cache = PageCache::new(2);
        
        cache.write_page(1, 0, vec![1]);
        cache.write_page(2, 0, vec![2]);
        cache.write_page(3, 0, vec![3]); // Should evict LRU
        
        assert_eq!(cache.entries.len(), 2);
    }
}
