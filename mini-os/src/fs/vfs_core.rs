/// Virtual File System (VFS) - Architecture de base
/// 
/// Le VFS fournit une abstraction unifiée pour tous les systèmes de fichiers.
/// Il permet de monter différents types de systèmes de fichiers (FAT32, ext2, ext4, etc.)
/// et de les manipuler via une interface commune.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
use core::fmt;

/// Identifiant unique d'inode
pub type InodeId = u64;

/// Identifiant de système de fichiers
pub type FsId = u32;

/// Types de fichiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,        // Fichier régulier
    Directory,      // Répertoire
    Symlink,        // Lien symbolique
    CharDevice,     // Périphérique caractère
    BlockDevice,    // Périphérique bloc
    Fifo,           // Tube nommé (FIFO)
    Socket,         // Socket
}

/// Modes de fichier (permissions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(pub u16);

impl FileMode {
    // Permissions utilisateur
    pub const USER_READ: u16 = 0o400;
    pub const USER_WRITE: u16 = 0o200;
    pub const USER_EXEC: u16 = 0o100;
    
    // Permissions groupe
    pub const GROUP_READ: u16 = 0o040;
    pub const GROUP_WRITE: u16 = 0o020;
    pub const GROUP_EXEC: u16 = 0o010;
    
    // Permissions autres
    pub const OTHER_READ: u16 = 0o004;
    pub const OTHER_WRITE: u16 = 0o002;
    pub const OTHER_EXEC: u16 = 0o001;
    
    // Bits spéciaux
    pub const SUID: u16 = 0o4000;
    pub const SGID: u16 = 0o2000;
    pub const STICKY: u16 = 0o1000;

    pub fn new(mode: u16) -> Self {
        Self(mode)
    }

    pub fn has_permission(&self, permission: u16) -> bool {
        (self.0 & permission) != 0
    }

    pub fn can_read_user(&self) -> bool {
        self.has_permission(Self::USER_READ)
    }

    pub fn can_write_user(&self) -> bool {
        self.has_permission(Self::USER_WRITE)
    }

    pub fn can_exec_user(&self) -> bool {
        self.has_permission(Self::USER_EXEC)
    }
}

/// Flags d'ouverture de fichier
#[derive(Debug, Clone, Copy)]
pub struct OpenFlags(pub u32);

impl OpenFlags {
    pub const READ: u32 = 0x0001;
    pub const WRITE: u32 = 0x0002;
    pub const APPEND: u32 = 0x0004;
    pub const CREATE: u32 = 0x0008;
    pub const TRUNCATE: u32 = 0x0010;
    pub const EXCL: u32 = 0x0020;

    pub fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub fn is_read(&self) -> bool {
        (self.0 & Self::READ) != 0
    }

    pub fn is_write(&self) -> bool {
        (self.0 & Self::WRITE) != 0
    }

    pub fn is_append(&self) -> bool {
        (self.0 & Self::APPEND) != 0
    }

    pub fn is_create(&self) -> bool {
        (self.0 & Self::CREATE) != 0
    }
}

/// Erreurs VFS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    NotFound,           // Fichier/répertoire non trouvé
    AlreadyExists,      // Fichier/répertoire existe déjà
    NotDirectory,       // N'est pas un répertoire
    IsDirectory,        // Est un répertoire
    PermissionDenied,   // Permission refusée
    InvalidArgument,    // Argument invalide
    IoError,            // Erreur d'E/S
    NoSpace,            // Pas d'espace disponible
    ReadOnly,           // Système de fichiers en lecture seule
    NotSupported,       // Opération non supportée
    TooManyLinks,       // Trop de liens symboliques
    NameTooLong,        // Nom trop long
    NotEmpty,           // Répertoire non vide
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VfsError::NotFound => write!(f, "Fichier ou répertoire non trouvé"),
            VfsError::AlreadyExists => write!(f, "Fichier ou répertoire existe déjà"),
            VfsError::NotDirectory => write!(f, "N'est pas un répertoire"),
            VfsError::IsDirectory => write!(f, "Est un répertoire"),
            VfsError::PermissionDenied => write!(f, "Permission refusée"),
            VfsError::InvalidArgument => write!(f, "Argument invalide"),
            VfsError::IoError => write!(f, "Erreur d'entrée/sortie"),
            VfsError::NoSpace => write!(f, "Pas d'espace disponible"),
            VfsError::ReadOnly => write!(f, "Système de fichiers en lecture seule"),
            VfsError::NotSupported => write!(f, "Opération non supportée"),
            VfsError::TooManyLinks => write!(f, "Trop de liens symboliques"),
            VfsError::NameTooLong => write!(f, "Nom de fichier trop long"),
            VfsError::NotEmpty => write!(f, "Répertoire non vide"),
        }
    }
}

pub type VfsResult<T> = Result<T, VfsError>;

/// Statistiques de fichier
#[derive(Debug, Clone)]
pub struct FileStat {
    pub inode: InodeId,
    pub file_type: FileType,
    pub mode: FileMode,
    pub nlinks: u32,        // Nombre de liens durs
    pub uid: u32,           // User ID
    pub gid: u32,           // Group ID
    pub size: u64,          // Taille en octets
    pub atime: u64,         // Dernier accès (timestamp)
    pub mtime: u64,         // Dernière modification (timestamp)
    pub ctime: u64,         // Dernier changement de métadonnées (timestamp)
    pub blksize: u32,       // Taille de bloc
    pub blocks: u64,        // Nombre de blocs alloués
}

impl FileStat {
    pub fn new(inode: InodeId, file_type: FileType) -> Self {
        Self {
            inode,
            file_type,
            mode: FileMode::new(0o644),
            nlinks: 1,
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            blksize: 4096,
            blocks: 0,
        }
    }
}

/// Superblock - Métadonnées du système de fichiers
pub trait Superblock: Send + Sync {
    /// Nom du système de fichiers
    fn fs_name(&self) -> &str;
    
    /// Identifiant du système de fichiers
    fn fs_id(&self) -> FsId;
    
    /// Taille de bloc
    fn block_size(&self) -> u32;
    
    /// Nombre total de blocs
    fn total_blocks(&self) -> u64;
    
    /// Nombre de blocs libres
    fn free_blocks(&self) -> u64;
    
    /// Nombre total d'inodes
    fn total_inodes(&self) -> u64;
    
    /// Nombre d'inodes libres
    fn free_inodes(&self) -> u64;
    
    /// Système de fichiers en lecture seule ?
    fn is_readonly(&self) -> bool;
    
    /// Obtenir l'inode racine
    fn root_inode(&self) -> InodeId;
}

/// Opérations sur les inodes
pub trait InodeOps: Send + Sync {
    /// Lire les données de l'inode
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize>;
    
    /// Écrire les données de l'inode
    fn write(&mut self, offset: u64, buf: &[u8]) -> VfsResult<usize>;
    
    /// Obtenir les statistiques de l'inode
    fn stat(&self) -> VfsResult<FileStat>;
    
    /// Rechercher une entrée dans un répertoire
    fn lookup(&self, name: &str) -> VfsResult<InodeId>;
    
    /// Créer un nouveau fichier
    fn create(&mut self, name: &str, mode: FileMode, file_type: FileType) -> VfsResult<InodeId>;
    
    /// Supprimer un fichier
    fn unlink(&mut self, name: &str) -> VfsResult<()>;
    
    /// Créer un répertoire
    fn mkdir(&mut self, name: &str, mode: FileMode) -> VfsResult<InodeId>;
    
    /// Supprimer un répertoire
    fn rmdir(&mut self, name: &str) -> VfsResult<()>;
    
    /// Lire les entrées d'un répertoire
    fn readdir(&self) -> VfsResult<Vec<DirEntry>>;
    
    /// Tronquer le fichier à une taille donnée
    fn truncate(&mut self, size: u64) -> VfsResult<()>;
}

/// Entrée de répertoire
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub inode: InodeId,
    pub name: String,
    pub file_type: FileType,
}

impl DirEntry {
    pub fn new(inode: InodeId, name: String, file_type: FileType) -> Self {
        Self {
            inode,
            name,
            file_type,
        }
    }
}

/// Opérations sur le système de fichiers
pub trait FileSystemOps: Send + Sync {
    /// Obtenir le superblock
    fn superblock(&self) -> Arc<dyn Superblock>;
    
    /// Obtenir un inode par son ID
    fn get_inode(&self, inode_id: InodeId) -> VfsResult<Arc<Mutex<dyn InodeOps>>>;
    
    /// Synchroniser le système de fichiers
    fn sync(&self) -> VfsResult<()>;
    
    /// Démonter le système de fichiers
    fn unmount(&self) -> VfsResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_file_mode_permissions() {
        let mode = FileMode::new(0o755);
        assert!(mode.can_read_user());
        assert!(mode.can_write_user());
        assert!(mode.can_exec_user());
        assert!(mode.has_permission(FileMode::GROUP_READ));
        assert!(mode.has_permission(FileMode::GROUP_EXEC));
        assert!(!mode.has_permission(FileMode::GROUP_WRITE));
    }

    #[test_case]
    fn test_open_flags() {
        let flags = OpenFlags::new(OpenFlags::READ | OpenFlags::WRITE);
        assert!(flags.is_read());
        assert!(flags.is_write());
        assert!(!flags.is_append());
    }

    #[test_case]
    fn test_file_stat_creation() {
        let stat = FileStat::new(1, FileType::Regular);
        assert_eq!(stat.inode, 1);
        assert_eq!(stat.file_type, FileType::Regular);
        assert_eq!(stat.nlinks, 1);
    }
}
