/// Gestionnaire de Systèmes de Fichiers
/// 
/// Ce module gère le montage et l'utilisation d'EXT4 comme système de fichiers principal

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::ext4::Ext4;
use crate::fs::{VfsError, JournalMode};
use crate::drivers::disk::Disk;

/// Instance globale du système de fichiers EXT4
lazy_static! {
    pub static ref EXT4_FS: Mutex<Option<Ext4FS>> = Mutex::new(None);
}

/// Wrapper pour le système de fichiers EXT4
pub struct Ext4FS {
    // Pour l'instant, on stocke juste un indicateur
    // Dans une vraie implémentation, on stockerait l'instance EXT4
    mounted: bool,
    mount_point: String,
}

impl Ext4FS {
    pub fn new() -> Self {
        Self {
            mounted: false,
            mount_point: String::from("/"),
        }
    }

    pub fn is_mounted(&self) -> bool {
        self.mounted
    }

    pub fn mount_point(&self) -> &str {
        &self.mount_point
    }
}

/// Initialise le système de fichiers EXT4
pub fn init_ext4() -> Result<(), VfsError> {
    let mut fs = EXT4_FS.lock();
    *fs = Some(Ext4FS::new());
    
    crate::vga_buffer::WRITER.lock()
        .write_string("EXT4: Système de fichiers initialisé\n");
    
    Ok(())
}

/// Monte une partition EXT4
pub fn mount_ext4_partition<D: Disk>(disk: D, mount_point: &str) -> Result<(), VfsError> {
    use crate::ext4::Ext4;
    use crate::fs::JournalMode;
    
    // Créer le filesystem EXT4
    let mut ext4 = Ext4::new(disk, JournalMode::Ordered)
        .map_err(|_| VfsError::IoError)?;
    
    // Monter avec recovery
    ext4.mount()?;
    
    crate::vga_buffer::WRITER.lock()
        .write_string(&alloc::format!("EXT4: Partition montée sur {}\n", mount_point));
    
    // Marquer comme monté
    if let Some(ref mut fs) = *EXT4_FS.lock() {
        fs.mounted = true;
        fs.mount_point = String::from(mount_point);
    }
    
    Ok(())
}

/// API système pour les opérations fichiers EXT4
pub mod syscalls {
    use super::*;
    
    /// Lit un fichier depuis EXT4
    pub fn read_file(path: &str) -> Result<Vec<u8>, VfsError> {
        // Dans une vraie implémentation, on utiliserait l'instance EXT4 globale
        crate::vga_buffer::WRITER.lock()
            .write_string(&alloc::format!("EXT4: Lecture de {}\n", path));
        
        // Pour l'instant, retourner une erreur
        Err(VfsError::NotSupported)
    }
    
    /// Écrit un fichier sur EXT4
    pub fn write_file(path: &str, content: &[u8]) -> Result<(), VfsError> {
        crate::vga_buffer::WRITER.lock()
            .write_string(&alloc::format!("EXT4: Écriture de {} ({} octets)\n", 
                path, content.len()));
        
        // Pour l'instant, retourner une erreur
        Err(VfsError::NotSupported)
    }
    
    /// Crée un répertoire sur EXT4
    pub fn create_dir(path: &str) -> Result<(), VfsError> {
        crate::vga_buffer::WRITER.lock()
            .write_string(&alloc::format!("EXT4: Création du répertoire {}\n", path));
        
        // Pour l'instant, retourner une erreur
        Err(VfsError::NotSupported)
    }
    
    /// Liste un répertoire sur EXT4
    pub fn list_dir(path: &str) -> Result<Vec<String>, VfsError> {
        crate::vga_buffer::WRITER.lock()
            .write_string(&alloc::format!("EXT4: Liste du répertoire {}\n", path));
        
        // Pour l'instant, retourner une erreur
        Err(VfsError::NotSupported)
    }
}

/// Statistiques du système EXT4
pub fn get_stats() -> Option<Ext4Stats> {
    let fs = EXT4_FS.lock();
    if let Some(ref fs_instance) = *fs {
        Some(Ext4Stats {
            mounted: fs_instance.mounted,
            mount_point: fs_instance.mount_point.clone(),
        })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Ext4Stats {
    pub mounted: bool,
    pub mount_point: String,
}
