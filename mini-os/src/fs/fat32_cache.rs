/// Module de Cache FAT pour FAT32
/// 
/// Cache la File Allocation Table en mémoire pour améliorer les performances

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Entrée FAT (32 bits pour FAT32)
pub type FatEntry = u32;

/// Valeurs spéciales FAT
pub const FAT_FREE: u32 = 0x00000000;
pub const FAT_EOC: u32 = 0x0FFFFFF8; // End of Chain
pub const FAT_BAD: u32 = 0x0FFFFFF7;

/// Entrée de cache FAT
#[derive(Debug, Clone)]
pub struct CachedFatEntry {
    /// Numéro de cluster
    pub cluster: u32,
    /// Valeur FAT (prochain cluster)
    pub value: FatEntry,
    /// Timestamp dernier accès
    pub last_access: u64,
    /// Nombre d'accès
    pub access_count: usize,
}

impl CachedFatEntry {
    pub fn new(cluster: u32, value: FatEntry) -> Self {
        Self {
            cluster,
            value,
            last_access: 0,
            access_count: 0,
        }
    }
    
    pub fn mark_accessed(&mut self) {
        self.access_count += 1;
        self.last_access = Self::get_timestamp();
    }
    
    fn get_timestamp() -> u64 {
        // TODO: Utiliser vrai timestamp
        0
    }
}

/// Cache FAT
pub struct FatCache {
    /// Entrées en cache
    entries: BTreeMap<u32, CachedFatEntry>,
    /// Taille maximale du cache
    max_entries: usize,
    /// Statistiques
    hits: usize,
    misses: usize,
}

impl FatCache {
    /// Crée un nouveau cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }
    
    /// Lit une entrée FAT
    pub fn get(&mut self, cluster: u32) -> Option<FatEntry> {
        if let Some(entry) = self.entries.get_mut(&cluster) {
            entry.mark_accessed();
            self.hits += 1;
            Some(entry.value)
        } else {
            self.misses += 1;
            None
        }
    }
    
    /// Met à jour une entrée FAT
    pub fn put(&mut self, cluster: u32, value: FatEntry) {
        // Vérifier si le cache est plein
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&cluster) {
            self.evict_lru();
        }
        
        if let Some(entry) = self.entries.get_mut(&cluster) {
            entry.value = value;
            entry.mark_accessed();
        } else {
            let mut entry = CachedFatEntry::new(cluster, value);
            entry.mark_accessed();
            self.entries.insert(cluster, entry);
        }
    }
    
    /// Évince l'entrée LRU
    fn evict_lru(&mut self) {
        if let Some((&cluster, _)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.last_access) {
            self.entries.remove(&cluster);
        }
    }
    
    /// Suit une chaîne de clusters
    pub fn follow_chain(&mut self, start_cluster: u32, max_length: usize) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut current = start_cluster;
        
        while chain.len() < max_length {
            chain.push(current);
            
            // Lire la prochaine entrée
            if let Some(next) = self.get(current) {
                if next >= FAT_EOC || next == FAT_FREE {
                    break;
                }
                current = next;
            } else {
                // Pas en cache, il faudrait lire depuis le disque
                break;
            }
        }
        
        chain
    }
    
    /// Invalide une entrée
    pub fn invalidate(&mut self, cluster: u32) {
        self.entries.remove(&cluster);
    }
    
    /// Invalide tout le cache
    pub fn invalidate_all(&mut self) {
        self.entries.clear();
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> FatCacheStats {
        let total_requests = self.hits + self.misses;
        let hit_rate = if total_requests > 0 {
            (self.hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        FatCacheStats {
            entries: self.entries.len(),
            max_entries: self.max_entries,
            hits: self.hits,
            misses: self.misses,
            hit_rate,
        }
    }
}

/// Statistiques du cache FAT
#[derive(Debug, Clone)]
pub struct FatCacheStats {
    pub entries: usize,
    pub max_entries: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
}

/// Gestionnaire de cache FAT32
pub struct Fat32CacheManager {
    /// Cache FAT
    fat_cache: FatCache,
    /// Cache de chaînes complètes (cluster -> chaîne)
    chain_cache: BTreeMap<u32, Vec<u32>>,
}

impl Fat32CacheManager {
    /// Crée un nouveau gestionnaire
    pub fn new(cache_size: usize) -> Self {
        Self {
            fat_cache: FatCache::new(cache_size),
            chain_cache: BTreeMap::new(),
        }
    }
    
    /// Lit une entrée FAT
    pub fn read_fat_entry(&mut self, cluster: u32) -> Option<FatEntry> {
        self.fat_cache.get(cluster)
    }
    
    /// Écrit une entrée FAT
    pub fn write_fat_entry(&mut self, cluster: u32, value: FatEntry) {
        self.fat_cache.put(cluster, value);
        
        // Invalider les chaînes qui pourraient être affectées
        self.chain_cache.retain(|&start, _| start != cluster);
    }
    
    /// Récupère une chaîne de clusters (avec cache)
    pub fn get_cluster_chain(&mut self, start_cluster: u32) -> Vec<u32> {
        // Vérifier le cache de chaînes
        if let Some(chain) = self.chain_cache.get(&start_cluster) {
            return chain.clone();
        }
        
        // Construire la chaîne
        let chain = self.fat_cache.follow_chain(start_cluster, 1024);
        
        // Mettre en cache
        self.chain_cache.insert(start_cluster, chain.clone());
        
        chain
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> Fat32CacheStats {
        Fat32CacheStats {
            fat_cache: self.fat_cache.get_stats(),
            cached_chains: self.chain_cache.len(),
        }
    }
}

/// Statistiques globales
#[derive(Debug, Clone)]
pub struct Fat32CacheStats {
    pub fat_cache: FatCacheStats,
    pub cached_chains: usize,
}

/// Instance globale
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FAT32_CACHE: Mutex<Fat32CacheManager> = 
        Mutex::new(Fat32CacheManager::new(2048)); // 2048 entrées
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_fat_cache_creation() {
        let cache = FatCache::new(100);
        assert_eq!(cache.max_entries, 100);
        assert_eq!(cache.hits, 0);
    }
    
    #[test_case]
    fn test_fat_cache_put_get() {
        let mut cache = FatCache::new(100);
        
        cache.put(10, 20);
        assert_eq!(cache.get(10), Some(20));
        assert_eq!(cache.hits, 1);
        
        assert_eq!(cache.get(999), None);
        assert_eq!(cache.misses, 1);
    }
    
    #[test_case]
    fn test_follow_chain() {
        let mut cache = FatCache::new(100);
        
        // Créer une chaîne: 10 -> 20 -> 30 -> EOC
        cache.put(10, 20);
        cache.put(20, 30);
        cache.put(30, FAT_EOC);
        
        let chain = cache.follow_chain(10, 100);
        assert_eq!(chain, vec![10, 20, 30]);
    }
    
    #[test_case]
    fn test_cache_manager() {
        let mut manager = Fat32CacheManager::new(100);
        
        manager.write_fat_entry(10, 20);
        assert_eq!(manager.read_fat_entry(10), Some(20));
    }
}
