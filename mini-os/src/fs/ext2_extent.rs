/// Module d'optimisation ext2 avec Extent-Based Allocation
/// 
/// Remplace l'allocation bloc par bloc par des extents contigus

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Extent (plage de blocs contigus)
#[derive(Debug, Clone, Copy)]
pub struct Extent {
    /// Bloc logique de début
    pub logical_block: u64,
    /// Bloc physique de début
    pub physical_block: u64,
    /// Nombre de blocs
    pub length: u32,
}

impl Extent {
    pub fn new(logical_block: u64, physical_block: u64, length: u32) -> Self {
        Self {
            logical_block,
            physical_block,
            length,
        }
    }
    
    /// Vérifie si un bloc logique est dans cet extent
    pub fn contains(&self, logical_block: u64) -> bool {
        logical_block >= self.logical_block 
            && logical_block < self.logical_block + self.length as u64
    }
    
    /// Convertit un bloc logique en bloc physique
    pub fn logical_to_physical(&self, logical_block: u64) -> Option<u64> {
        if self.contains(logical_block) {
            let offset = logical_block - self.logical_block;
            Some(self.physical_block + offset)
        } else {
            None
        }
    }
}

/// Arbre d'extents pour un fichier
#[derive(Debug, Clone)]
pub struct ExtentTree {
    /// Extents du fichier
    extents: Vec<Extent>,
    /// Nombre total de blocs
    total_blocks: u64,
}

impl ExtentTree {
    /// Crée un nouvel arbre d'extents
    pub fn new() -> Self {
        Self {
            extents: Vec::new(),
            total_blocks: 0,
        }
    }
    
    /// Ajoute un extent
    pub fn add_extent(&mut self, extent: Extent) {
        // Essayer de fusionner avec l'extent précédent si contigu
        if let Some(last) = self.extents.last_mut() {
            if last.logical_block + last.length as u64 == extent.logical_block
                && last.physical_block + last.length as u64 == extent.physical_block {
                // Fusionner
                last.length += extent.length;
                self.total_blocks += extent.length as u64;
                return;
            }
        }
        
        self.total_blocks += extent.length as u64;
        self.extents.push(extent);
    }
    
    /// Trouve le bloc physique pour un bloc logique
    pub fn get_physical_block(&self, logical_block: u64) -> Option<u64> {
        for extent in &self.extents {
            if let Some(physical) = extent.logical_to_physical(logical_block) {
                return Some(physical);
            }
        }
        None
    }
    
    /// Retourne le nombre d'extents
    pub fn num_extents(&self) -> usize {
        self.extents.len()
    }
    
    /// Retourne le taux de fragmentation (0-100%)
    pub fn fragmentation_rate(&self) -> f64 {
        if self.total_blocks == 0 {
            return 0.0;
        }
        
        // Idéalement 1 extent, plus il y en a, plus c'est fragmenté
        let ideal_extents = 1.0;
        let actual_extents = self.extents.len() as f64;
        
        ((actual_extents - ideal_extents) / actual_extents * 100.0).max(0.0)
    }
}

/// Allocateur d'extents
pub struct ExtentAllocator {
    /// Bitmap des blocs libres (simplifié)
    free_blocks: BTreeMap<u64, u32>,
    /// Prochain bloc à essayer
    next_block: u64,
    /// Nombre de blocs alloués
    allocated_blocks: u64,
}

impl ExtentAllocator {
    /// Crée un nouvel allocateur
    pub fn new(total_blocks: u64) -> Self {
        let mut free_blocks = BTreeMap::new();
        // Initialiser avec un grand extent libre
        free_blocks.insert(0, total_blocks as u32);
        
        Self {
            free_blocks,
            next_block: 0,
            allocated_blocks: 0,
        }
    }
    
    /// Alloue un extent de taille donnée
    pub fn allocate(&mut self, size: u32) -> Option<Extent> {
        // Chercher un extent libre assez grand
        for (&start, &length) in self.free_blocks.iter() {
            if length >= size {
                // Allouer depuis cet extent
                let extent = Extent::new(0, start, size);
                
                // Mettre à jour les blocs libres
                self.free_blocks.remove(&start);
                if length > size {
                    self.free_blocks.insert(start + size as u64, length - size);
                }
                
                self.allocated_blocks += size as u64;
                return Some(extent);
            }
        }
        
        None
    }
    
    /// Libère un extent
    pub fn free(&mut self, extent: &Extent) {
        self.free_blocks.insert(extent.physical_block, extent.length);
        self.allocated_blocks -= extent.length as u64;
        
        // TODO: Fusionner les extents libres adjacents
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> ExtentAllocatorStats {
        ExtentAllocatorStats {
            allocated_blocks: self.allocated_blocks,
            free_extents: self.free_blocks.len(),
        }
    }
}

/// Statistiques de l'allocateur
#[derive(Debug, Clone)]
pub struct ExtentAllocatorStats {
    pub allocated_blocks: u64,
    pub free_extents: usize,
}

/// Gestionnaire d'extents ext2
pub struct Ext2ExtentManager {
    /// Arbres d'extents par inode
    extent_trees: BTreeMap<u64, ExtentTree>,
    /// Allocateur
    allocator: ExtentAllocator,
}

impl Ext2ExtentManager {
    /// Crée un nouveau gestionnaire
    pub fn new(total_blocks: u64) -> Self {
        Self {
            extent_trees: BTreeMap::new(),
            allocator: ExtentAllocator::new(total_blocks),
        }
    }
    
    /// Alloue des blocs pour un inode
    pub fn allocate_blocks(&mut self, inode: u64, num_blocks: u32) -> Result<(), ExtentError> {
        let extent = self.allocator.allocate(num_blocks)
            .ok_or(ExtentError::AllocationFailed)?;
        
        let tree = self.extent_trees.entry(inode).or_insert_with(ExtentTree::new);
        tree.add_extent(extent);
        
        Ok(())
    }
    
    /// Récupère le bloc physique pour un inode et bloc logique
    pub fn get_physical_block(&self, inode: u64, logical_block: u64) -> Option<u64> {
        self.extent_trees.get(&inode)
            .and_then(|tree| tree.get_physical_block(logical_block))
    }
    
    /// Retourne les statistiques pour un inode
    pub fn get_inode_stats(&self, inode: u64) -> Option<InodeExtentStats> {
        self.extent_trees.get(&inode).map(|tree| {
            InodeExtentStats {
                num_extents: tree.num_extents(),
                total_blocks: tree.total_blocks,
                fragmentation_rate: tree.fragmentation_rate(),
            }
        })
    }
}

/// Erreurs d'extent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtentError {
    AllocationFailed,
    InvalidExtent,
}

/// Statistiques d'inode
#[derive(Debug, Clone)]
pub struct InodeExtentStats {
    pub num_extents: usize,
    pub total_blocks: u64,
    pub fragmentation_rate: f64,
}

/// Instance globale
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EXT2_EXTENT_MANAGER: Mutex<Ext2ExtentManager> = 
        Mutex::new(Ext2ExtentManager::new(1024 * 1024)); // 1M blocs
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_extent_creation() {
        let extent = Extent::new(0, 100, 10);
        assert_eq!(extent.logical_block, 0);
        assert_eq!(extent.physical_block, 100);
        assert_eq!(extent.length, 10);
    }
    
    #[test_case]
    fn test_extent_contains() {
        let extent = Extent::new(0, 100, 10);
        assert!(extent.contains(0));
        assert!(extent.contains(5));
        assert!(extent.contains(9));
        assert!(!extent.contains(10));
    }
    
    #[test_case]
    fn test_extent_tree() {
        let mut tree = ExtentTree::new();
        tree.add_extent(Extent::new(0, 100, 10));
        
        assert_eq!(tree.get_physical_block(0), Some(100));
        assert_eq!(tree.get_physical_block(5), Some(105));
        assert_eq!(tree.get_physical_block(10), None);
    }
    
    #[test_case]
    fn test_extent_merge() {
        let mut tree = ExtentTree::new();
        tree.add_extent(Extent::new(0, 100, 10));
        tree.add_extent(Extent::new(10, 110, 10)); // Contigu, devrait fusionner
        
        assert_eq!(tree.num_extents(), 1);
        assert_eq!(tree.total_blocks, 20);
    }
}
