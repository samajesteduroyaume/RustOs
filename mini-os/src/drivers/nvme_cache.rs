/// Module d'intégration NVMe avec Buffer Cache
/// 
/// Fournit une couche d'abstraction qui combine NVMe et cache

use alloc::vec;
use alloc::vec::Vec;
use spin::Mutex;
use super::nvme::{NVME_CONTROLLER, NVMeError};
use crate::fs::cache::{BUFFER_CACHE, READAHEAD_MANAGER, WRITEBACK_DAEMON};

/// Taille d'un bloc (aligné sur NVMe et cache)
pub const BLOCK_SIZE: usize = 4096;

/// Nombre de blocs NVMe par bloc cache (4096 / 512 = 8)
const NVME_BLOCKS_PER_CACHE_BLOCK: u16 = 8;

/// Gestionnaire de stockage avec cache
pub struct CachedStorage {
    /// Namespace ID par défaut
    default_nsid: u32,
    /// Statistiques
    cache_hits: usize,
    cache_misses: usize,
    reads: usize,
    writes: usize,
}

impl CachedStorage {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            default_nsid: 1,
            cache_hits: 0,
            cache_misses: 0,
            reads: 0,
            writes: 0,
        }
    }
    
    /// Lit un bloc avec cache
    pub fn read_block(&mut self, block_num: u64) -> Result<Vec<u8>, StorageError> {
        self.reads += 1;
        
        // 1. Vérifier le cache
        let mut cache = BUFFER_CACHE.lock();
        if let Some(data) = cache.read_block(block_num) {
            self.cache_hits += 1;
            drop(cache);
            
            // Notifier read-ahead
            READAHEAD_MANAGER.lock().on_read(0, block_num);
            
            return Ok(data);
        }
        
        self.cache_misses += 1;
        drop(cache);
        
        // 2. Lire depuis NVMe
        let data = self.read_from_nvme(block_num)?;
        
        // 3. Stocker dans le cache
        BUFFER_CACHE.lock().write_block(block_num, data.clone());
        
        // 4. Notifier read-ahead
        READAHEAD_MANAGER.lock().on_read(0, block_num);
        
        Ok(data)
    }
    
    /// Écrit un bloc avec cache
    pub fn write_block(&mut self, block_num: u64, data: Vec<u8>) -> Result<(), StorageError> {
        self.writes += 1;
        
        if data.len() != BLOCK_SIZE {
            return Err(StorageError::InvalidBlockSize);
        }
        
        // Écrire dans le cache (marqué dirty)
        BUFFER_CACHE.lock().write_block(block_num, data);
        
        // Le write-back daemon s'occupera de l'écriture disque
        
        Ok(())
    }
    
    /// Lit depuis NVMe (sans cache)
    fn read_from_nvme(&self, block_num: u64) -> Result<Vec<u8>, StorageError> {
        let mut buffer = vec![0u8; BLOCK_SIZE];
        
        // Convertir numéro de bloc cache en LBA NVMe
        let lba = block_num * NVME_BLOCKS_PER_CACHE_BLOCK as u64;
        
        NVME_CONTROLLER.lock()
            .read_blocks(self.default_nsid, lba, NVME_BLOCKS_PER_CACHE_BLOCK, &mut buffer)
            .map_err(|_| StorageError::ReadError)?;
        
        Ok(buffer)
    }
    
    /// Écrit vers NVMe (sans cache)
    fn write_to_nvme(&self, block_num: u64, data: &[u8]) -> Result<(), StorageError> {
        if data.len() != BLOCK_SIZE {
            return Err(StorageError::InvalidBlockSize);
        }
        
        let lba = block_num * NVME_BLOCKS_PER_CACHE_BLOCK as u64;
        
        NVME_CONTROLLER.lock()
            .write_blocks(self.default_nsid, lba, NVME_BLOCKS_PER_CACHE_BLOCK, data)
            .map_err(|_| StorageError::WriteError)?;
        
        Ok(())
    }
    
    /// Flush un bloc spécifique vers le disque
    pub fn flush_block(&mut self, block_num: u64) -> Result<(), StorageError> {
        if let Some(data) = BUFFER_CACHE.lock().flush_block(block_num) {
            self.write_to_nvme(block_num, &data)?;
        }
        Ok(())
    }
    
    /// Flush tous les blocs dirty
    pub fn flush_all(&mut self) -> Result<(), StorageError> {
        let blocks = BUFFER_CACHE.lock().flush_all();
        
        for (block_num, data) in blocks {
            self.write_to_nvme(block_num, &data)?;
        }
        
        Ok(())
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> CachedStorageStats {
        let cache_stats = BUFFER_CACHE.lock().get_stats();
        
        CachedStorageStats {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            cache_hit_rate: if self.reads > 0 {
                (self.cache_hits as f64 / self.reads as f64) * 100.0
            } else {
                0.0
            },
            reads: self.reads,
            writes: self.writes,
            cache_size: cache_stats.total_entries,
            dirty_blocks: cache_stats.dirty_blocks,
        }
    }
}

/// Erreurs de stockage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    ReadError,
    WriteError,
    InvalidBlockSize,
    NotInitialized,
}

/// Statistiques du stockage avec cache
#[derive(Debug, Clone)]
pub struct CachedStorageStats {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_hit_rate: f64,
    pub reads: usize,
    pub writes: usize,
    pub cache_size: usize,
    pub dirty_blocks: usize,
}

/// Instance globale du stockage avec cache
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CACHED_STORAGE: Mutex<CachedStorage> = Mutex::new(CachedStorage::new());
}

/// Initialise le système de stockage
pub fn init_storage() -> Result<(), StorageError> {
    // Initialiser NVMe
    NVME_CONTROLLER.lock()
        .init()
        .map_err(|_| StorageError::NotInitialized)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_cached_storage_creation() {
        let storage = CachedStorage::new();
        assert_eq!(storage.reads, 0);
        assert_eq!(storage.writes, 0);
    }
    
    #[test_case]
    fn test_block_size_conversion() {
        assert_eq!(NVME_BLOCKS_PER_CACHE_BLOCK, 8);
        assert_eq!(BLOCK_SIZE, 4096);
    }
}
