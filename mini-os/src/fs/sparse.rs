/// Module Sparse Files (Fichiers Épars)
/// 
/// Support pour fichiers avec trous (holes) pour économiser l'espace disque

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Taille d'un bloc
pub const SPARSE_BLOCK_SIZE: usize = 4096;

/// Région de données (non-sparse)
#[derive(Debug, Clone)]
pub struct DataRegion {
    /// Offset de début
    pub start: u64,
    /// Taille
    pub size: u64,
    /// Numéro de bloc physique
    pub physical_block: u64,
}

/// Fichier épars
#[derive(Debug, Clone)]
pub struct SparseFile {
    /// Taille apparente du fichier
    pub apparent_size: u64,
    /// Régions de données (offset -> région)
    pub regions: BTreeMap<u64, DataRegion>,
    /// Taille réelle utilisée
    pub actual_size: u64,
}

impl SparseFile {
    /// Crée un nouveau fichier épars
    pub fn new() -> Self {
        Self {
            apparent_size: 0,
            regions: BTreeMap::new(),
            actual_size: 0,
        }
    }
    
    /// Lit des données
    pub fn read(&self, offset: u64, size: usize, buffer: &mut [u8]) -> usize {
        let mut bytes_read = 0;
        let mut current_offset = offset;
        
        while bytes_read < size && current_offset < self.apparent_size {
            // Trouver la région contenant cet offset
            if let Some((_, region)) = self.regions.range(..=current_offset).next_back() {
                if current_offset >= region.start && current_offset < region.start + region.size {
                    // Dans une région de données
                    let region_offset = current_offset - region.start;
                    let to_read = core::cmp::min(
                        size - bytes_read,
                        (region.size - region_offset) as usize
                    );
                    
                    // TODO: Lire depuis le bloc physique
                    // Pour l'instant, remplir avec des données de test
                    for i in 0..to_read {
                        if bytes_read + i < buffer.len() {
                            buffer[bytes_read + i] = 0xFF;
                        }
                    }
                    
                    bytes_read += to_read;
                    current_offset += to_read as u64;
                } else {
                    // Dans un trou, retourner des zéros
                    if bytes_read < buffer.len() {
                        buffer[bytes_read] = 0;
                    }
                    bytes_read += 1;
                    current_offset += 1;
                }
            } else {
                // Pas de région, c'est un trou
                if bytes_read < buffer.len() {
                    buffer[bytes_read] = 0;
                }
                bytes_read += 1;
                current_offset += 1;
            }
        }
        
        bytes_read
    }
    
    /// Écrit des données
    pub fn write(&mut self, offset: u64, data: &[u8]) -> Result<usize, SparseError> {
        // Vérifier si les données sont toutes à zéro (créer un trou)
        if data.iter().all(|&b| b == 0) {
            // Ne rien allouer, juste augmenter la taille apparente
            let new_size = offset + data.len() as u64;
            if new_size > self.apparent_size {
                self.apparent_size = new_size;
            }
            return Ok(data.len());
        }
        
        // Allouer un nouveau bloc physique
        let physical_block = self.allocate_block()?;
        
        // Créer une nouvelle région
        let region = DataRegion {
            start: offset,
            size: data.len() as u64,
            physical_block,
        };
        
        self.regions.insert(offset, region);
        self.actual_size += data.len() as u64;
        
        let new_size = offset + data.len() as u64;
        if new_size > self.apparent_size {
            self.apparent_size = new_size;
        }
        
        Ok(data.len())
    }
    
    /// Alloue un bloc physique
    fn allocate_block(&self) -> Result<u64, SparseError> {
        // TODO: Utiliser le vrai allocateur de blocs
        Ok(self.regions.len() as u64 + 1)
    }
    
    /// Tronque le fichier
    pub fn truncate(&mut self, size: u64) {
        // Supprimer les régions au-delà de la nouvelle taille
        let to_remove: Vec<_> = self.regions
            .range(size..)
            .map(|(k, _)| *k)
            .collect();
        
        for key in to_remove {
            if let Some(region) = self.regions.remove(&key) {
                self.actual_size -= region.size;
            }
        }
        
        self.apparent_size = size;
    }
    
    /// Retourne le ratio de compression
    pub fn compression_ratio(&self) -> f64 {
        if self.apparent_size == 0 {
            return 0.0;
        }
        
        (self.actual_size as f64 / self.apparent_size as f64) * 100.0
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> SparseFileStats {
        SparseFileStats {
            apparent_size: self.apparent_size,
            actual_size: self.actual_size,
            num_regions: self.regions.len(),
            compression_ratio: self.compression_ratio(),
            space_saved: self.apparent_size.saturating_sub(self.actual_size),
        }
    }
}

/// Erreurs de fichiers épars
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparseError {
    AllocationFailed,
    InvalidOffset,
    IoError,
}

/// Statistiques de fichier épars
#[derive(Debug, Clone)]
pub struct SparseFileStats {
    pub apparent_size: u64,
    pub actual_size: u64,
    pub num_regions: usize,
    pub compression_ratio: f64,
    pub space_saved: u64,
}

/// Gestionnaire de fichiers épars
pub struct SparseFileManager {
    /// Fichiers épars par inode
    files: BTreeMap<u64, SparseFile>,
}

impl SparseFileManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }
    
    /// Crée un fichier épars
    pub fn create(&mut self, inode: u64) -> &mut SparseFile {
        self.files.entry(inode).or_insert_with(SparseFile::new)
    }
    
    /// Récupère un fichier
    pub fn get(&self, inode: u64) -> Option<&SparseFile> {
        self.files.get(&inode)
    }
    
    /// Récupère un fichier mutable
    pub fn get_mut(&mut self, inode: u64) -> Option<&mut SparseFile> {
        self.files.get_mut(&inode)
    }
    
    /// Supprime un fichier
    pub fn remove(&mut self, inode: u64) -> Option<SparseFile> {
        self.files.remove(&inode)
    }
    
    /// Retourne les statistiques globales
    pub fn get_global_stats(&self) -> SparseGlobalStats {
        let mut total_apparent = 0;
        let mut total_actual = 0;
        
        for file in self.files.values() {
            total_apparent += file.apparent_size;
            total_actual += file.actual_size;
        }
        
        SparseGlobalStats {
            num_files: self.files.len(),
            total_apparent_size: total_apparent,
            total_actual_size: total_actual,
            total_space_saved: total_apparent.saturating_sub(total_actual),
        }
    }
}

/// Statistiques globales
#[derive(Debug, Clone)]
pub struct SparseGlobalStats {
    pub num_files: usize,
    pub total_apparent_size: u64,
    pub total_actual_size: u64,
    pub total_space_saved: u64,
}

/// Instance globale
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SPARSE_FILE_MANAGER: Mutex<SparseFileManager> = Mutex::new(SparseFileManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_sparse_file_creation() {
        let file = SparseFile::new();
        assert_eq!(file.apparent_size, 0);
        assert_eq!(file.actual_size, 0);
    }
    
    #[test_case]
    fn test_sparse_write_zeros() {
        let mut file = SparseFile::new();
        let zeros = vec![0u8; 4096];
        
        file.write(0, &zeros).unwrap();
        
        // Taille apparente augmente mais pas la taille réelle
        assert_eq!(file.apparent_size, 4096);
        assert_eq!(file.actual_size, 0);
    }
    
    #[test_case]
    fn test_sparse_write_data() {
        let mut file = SparseFile::new();
        let data = vec![0xFFu8; 4096];
        
        file.write(0, &data).unwrap();
        
        assert_eq!(file.apparent_size, 4096);
        assert_eq!(file.actual_size, 4096);
    }
    
    #[test_case]
    fn test_compression_ratio() {
        let mut file = SparseFile::new();
        
        // Écrire 1KB de données réelles
        let data = vec![0xFFu8; 1024];
        file.write(0, &data).unwrap();
        
        // Écrire 3KB de zéros (trou)
        let zeros = vec![0u8; 3072];
        file.write(1024, &zeros).unwrap();
        
        // Ratio devrait être 25% (1KB/4KB)
        assert_eq!(file.compression_ratio(), 25.0);
    }
}
