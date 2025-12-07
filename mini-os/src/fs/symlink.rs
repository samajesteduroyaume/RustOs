/// Module de gestion des liens symboliques
/// 
/// Implémente les liens symboliques (symlinks) et hard links pour le VFS

use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Limite de profondeur pour la résolution de symlinks (éviter boucles infinies)
pub const MAX_SYMLINK_DEPTH: usize = 40;

/// Type de lien
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    /// Lien symbolique (soft link)
    Symbolic,
    /// Lien dur (hard link)
    Hard,
}

/// Métadonnées d'un lien symbolique
#[derive(Debug, Clone)]
pub struct SymlinkMetadata {
    /// Chemin cible du lien
    pub target_path: String,
    /// Type de lien
    pub link_type: LinkType,
    /// Nombre de résolutions (pour détecter les boucles)
    pub resolution_count: usize,
    /// Inode du lien
    pub inode: u64,
    /// UID propriétaire
    pub uid: u32,
    /// GID groupe
    pub gid: u32,
}

impl SymlinkMetadata {
    /// Crée un nouveau symlink
    pub fn new(target_path: String, inode: u64, uid: u32, gid: u32) -> Self {
        Self {
            target_path,
            link_type: LinkType::Symbolic,
            resolution_count: 0,
            inode,
            uid,
            gid,
        }
    }
    
    /// Crée un nouveau hard link
    pub fn new_hardlink(target_path: String, inode: u64, uid: u32, gid: u32) -> Self {
        Self {
            target_path,
            link_type: LinkType::Hard,
            resolution_count: 0,
            inode,
            uid,
            gid,
        }
    }
}

/// Erreurs de gestion des liens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkError {
    /// Trop de niveaux de liens symboliques
    TooManyLevels,
    /// Boucle détectée
    Loop,
    /// Lien non trouvé
    NotFound,
    /// Pas un lien symbolique
    NotSymlink,
    /// Chemin invalide
    InvalidPath,
    /// Permission refusée
    PermissionDenied,
}

/// Gestionnaire de liens symboliques
pub struct SymlinkManager {
    /// Liens indexés par chemin
    symlinks: BTreeMap<String, SymlinkMetadata>,
    /// Prochain numéro d'inode pour les symlinks
    next_inode: u64,
    /// Nombre total de symlinks
    total_symlinks: usize,
    /// Nombre total de hard links
    total_hardlinks: usize,
}

impl SymlinkManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            symlinks: BTreeMap::new(),
            next_inode: 1000000, // Commencer à 1M pour éviter conflits
            total_symlinks: 0,
            total_hardlinks: 0,
        }
    }
    
    /// Crée un lien symbolique
    /// 
    /// # Arguments
    /// * `link_path` - Chemin du lien à créer
    /// * `target_path` - Chemin cible du lien
    /// * `uid` - UID du créateur
    /// * `gid` - GID du créateur
    pub fn create_symlink(
        &mut self,
        link_path: String,
        target_path: String,
        uid: u32,
        gid: u32,
    ) -> Result<u64, SymlinkError> {
        // Vérifier que le lien n'existe pas déjà
        if self.symlinks.contains_key(&link_path) {
            return Err(SymlinkError::InvalidPath);
        }
        
        let inode = self.next_inode;
        self.next_inode += 1;
        
        let metadata = SymlinkMetadata::new(target_path, inode, uid, gid);
        self.symlinks.insert(link_path, metadata);
        self.total_symlinks += 1;
        
        Ok(inode)
    }
    
    /// Crée un hard link
    pub fn create_hardlink(
        &mut self,
        link_path: String,
        target_path: String,
        uid: u32,
        gid: u32,
    ) -> Result<u64, SymlinkError> {
        if self.symlinks.contains_key(&link_path) {
            return Err(SymlinkError::InvalidPath);
        }
        
        let inode = self.next_inode;
        self.next_inode += 1;
        
        let metadata = SymlinkMetadata::new_hardlink(target_path, inode, uid, gid);
        self.symlinks.insert(link_path, metadata);
        self.total_hardlinks += 1;
        
        Ok(inode)
    }
    
    /// Lit le contenu d'un lien symbolique
    pub fn readlink(&self, link_path: &str) -> Result<String, SymlinkError> {
        if let Some(metadata) = self.symlinks.get(link_path) {
            if metadata.link_type == LinkType::Symbolic {
                Ok(metadata.target_path.clone())
            } else {
                Err(SymlinkError::NotSymlink)
            }
        } else {
            Err(SymlinkError::NotFound)
        }
    }
    
    /// Résout un lien symbolique (suit les liens jusqu'au fichier final)
    /// 
    /// # Arguments
    /// * `path` - Chemin à résoudre
    /// * `depth` - Profondeur actuelle de résolution
    pub fn resolve_symlink(&self, path: &str, depth: usize) -> Result<String, SymlinkError> {
        // Vérifier la profondeur maximale
        if depth >= MAX_SYMLINK_DEPTH {
            return Err(SymlinkError::TooManyLevels);
        }
        
        // Vérifier si c'est un symlink
        if let Some(metadata) = self.symlinks.get(path) {
            if metadata.link_type == LinkType::Symbolic {
                // Résoudre récursivement
                self.resolve_symlink(&metadata.target_path, depth + 1)
            } else {
                // Hard link, retourner le chemin cible
                Ok(metadata.target_path.clone())
            }
        } else {
            // Pas un lien, retourner le chemin tel quel
            Ok(String::from(path))
        }
    }
    
    /// Vérifie si un chemin est un lien symbolique
    pub fn is_symlink(&self, path: &str) -> bool {
        if let Some(metadata) = self.symlinks.get(path) {
            metadata.link_type == LinkType::Symbolic
        } else {
            false
        }
    }
    
    /// Vérifie si un chemin est un hard link
    pub fn is_hardlink(&self, path: &str) -> bool {
        if let Some(metadata) = self.symlinks.get(path) {
            metadata.link_type == LinkType::Hard
        } else {
            false
        }
    }
    
    /// Supprime un lien
    pub fn remove_link(&mut self, path: &str) -> Result<(), SymlinkError> {
        if let Some(metadata) = self.symlinks.remove(path) {
            match metadata.link_type {
                LinkType::Symbolic => self.total_symlinks -= 1,
                LinkType::Hard => self.total_hardlinks -= 1,
            }
            Ok(())
        } else {
            Err(SymlinkError::NotFound)
        }
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> SymlinkStats {
        SymlinkStats {
            total_symlinks: self.total_symlinks,
            total_hardlinks: self.total_hardlinks,
            total_links: self.total_symlinks + self.total_hardlinks,
        }
    }
}

/// Statistiques des liens
#[derive(Debug, Clone, Copy)]
pub struct SymlinkStats {
    pub total_symlinks: usize,
    pub total_hardlinks: usize,
    pub total_links: usize,
}

/// Instance globale du gestionnaire de symlinks
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SYMLINK_MANAGER: Mutex<SymlinkManager> = Mutex::new(SymlinkManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_create_symlink() {
        let mut manager = SymlinkManager::new();
        let result = manager.create_symlink(
            "/link".to_string(),
            "/target".to_string(),
            1000,
            1000,
        );
        
        assert!(result.is_ok());
        assert_eq!(manager.total_symlinks, 1);
    }
    
    #[test_case]
    fn test_readlink() {
        let mut manager = SymlinkManager::new();
        manager.create_symlink(
            "/link".to_string(),
            "/target".to_string(),
            1000,
            1000,
        ).unwrap();
        
        let target = manager.readlink("/link").unwrap();
        assert_eq!(target, "/target");
    }
    
    #[test_case]
    fn test_resolve_symlink() {
        let mut manager = SymlinkManager::new();
        manager.create_symlink(
            "/link1".to_string(),
            "/link2".to_string(),
            1000,
            1000,
        ).unwrap();
        manager.create_symlink(
            "/link2".to_string(),
            "/target".to_string(),
            1000,
            1000,
        ).unwrap();
        
        let resolved = manager.resolve_symlink("/link1", 0).unwrap();
        assert_eq!(resolved, "/target");
    }
    
    #[test_case]
    fn test_too_many_levels() {
        let mut manager = SymlinkManager::new();
        
        // Créer une boucle
        manager.create_symlink(
            "/link1".to_string(),
            "/link2".to_string(),
            1000,
            1000,
        ).unwrap();
        manager.create_symlink(
            "/link2".to_string(),
            "/link1".to_string(),
            1000,
            1000,
        ).unwrap();
        
        let result = manager.resolve_symlink("/link1", 0);
        assert_eq!(result, Err(SymlinkError::TooManyLevels));
    }
}
