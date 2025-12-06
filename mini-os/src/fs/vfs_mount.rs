/// VFS Mount - Gestion des points de montage
/// 
/// Le système de montage permet d'attacher différents systèmes de fichiers
/// à l'arborescence VFS globale.

use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

use super::vfs_core::*;
use super::vfs_inode::Inode;
use super::vfs_dentry::Dentry;

/// Point de montage
pub struct MountPoint {
    /// Chemin de montage
    pub path: String,
    
    /// Système de fichiers monté
    pub fs: Arc<dyn FileSystemOps>,
    
    /// Dentry du point de montage
    pub mountpoint: Arc<Mutex<Dentry>>,
    
    /// Inode racine du système de fichiers monté
    pub root: Arc<Mutex<Inode>>,
    
    /// Flags de montage
    pub flags: MountFlags,
}

/// Flags de montage
#[derive(Debug, Clone, Copy)]
pub struct MountFlags(pub u32);

impl MountFlags {
    pub const READONLY: u32 = 0x0001;
    pub const NOEXEC: u32 = 0x0002;
    pub const NOSUID: u32 = 0x0004;
    pub const NODEV: u32 = 0x0008;
    pub const SYNCHRONOUS: u32 = 0x0010;
    pub const REMOUNT: u32 = 0x0020;

    pub fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub fn is_readonly(&self) -> bool {
        (self.0 & Self::READONLY) != 0
    }

    pub fn is_noexec(&self) -> bool {
        (self.0 & Self::NOEXEC) != 0
    }

    pub fn is_nosuid(&self) -> bool {
        (self.0 & Self::NOSUID) != 0
    }

    pub fn is_nodev(&self) -> bool {
        (self.0 & Self::NODEV) != 0
    }
}

impl MountPoint {
    /// Crée un nouveau point de montage
    pub fn new(
        path: String,
        fs: Arc<dyn FileSystemOps>,
        mountpoint: Arc<Mutex<Dentry>>,
        root: Arc<Mutex<Inode>>,
        flags: MountFlags,
    ) -> Self {
        Self {
            path,
            fs,
            mountpoint,
            root,
            flags,
        }
    }
}

/// Gestionnaire de montage
pub struct MountManager {
    /// Table des points de montage (clé: chemin)
    mounts: BTreeMap<String, Arc<Mutex<MountPoint>>>,
    
    /// Point de montage racine
    root_mount: Option<Arc<Mutex<MountPoint>>>,
}

impl MountManager {
    /// Crée un nouveau gestionnaire de montage
    pub fn new() -> Self {
        Self {
            mounts: BTreeMap::new(),
            root_mount: None,
        }
    }

    /// Monte un système de fichiers
    pub fn mount(
        &mut self,
        path: &str,
        fs: Arc<dyn FileSystemOps>,
        mountpoint: Arc<Mutex<Dentry>>,
        flags: MountFlags,
    ) -> VfsResult<()> {
        // Vérifier si le chemin est déjà monté
        if self.mounts.contains_key(path) && !flags.0 & MountFlags::REMOUNT != 0 {
            return Err(VfsError::AlreadyExists);
        }

        // Obtenir l'inode racine du système de fichiers
        let root_inode_id = fs.superblock().root_inode();
        let root_inode_ops = fs.get_inode(root_inode_id)?;
        
        // Créer un inode wrapper
        let root_inode = super::vfs_inode::get_or_create_inode(
            fs.superblock().fs_id(),
            root_inode_id,
            super::vfs_core::FileType::Directory,
            root_inode_ops,
        );

        // Créer le point de montage
        let mount = Arc::new(Mutex::new(MountPoint::new(
            path.into(),
            fs,
            mountpoint,
            root_inode,
            flags,
        )));

        // Ajouter à la table
        self.mounts.insert(path.into(), mount.clone());

        // Si c'est le montage racine, le sauvegarder
        if path == "/" {
            self.root_mount = Some(mount);
        }

        Ok(())
    }

    /// Démonte un système de fichiers
    pub fn unmount(&mut self, path: &str) -> VfsResult<()> {
        // Ne pas permettre de démonter la racine
        if path == "/" {
            return Err(VfsError::InvalidArgument);
        }

        // Vérifier si le chemin est monté
        let mount = self.mounts.get(path).ok_or(VfsError::NotFound)?;

        // Synchroniser le système de fichiers avant de démonter
        let locked_mount = mount.lock();
        locked_mount.fs.sync()?;
        locked_mount.fs.unmount()?;
        drop(locked_mount);

        // Retirer de la table
        self.mounts.remove(path);

        Ok(())
    }

    /// Trouve le point de montage pour un chemin
    pub fn find_mount(&self, path: &str) -> Option<Arc<Mutex<MountPoint>>> {
        // Chercher le point de montage le plus spécifique
        let mut best_match: Option<(&String, &Arc<Mutex<MountPoint>>)> = None;
        let mut best_len = 0;

        for (mount_path, mount) in &self.mounts {
            if path.starts_with(mount_path) && mount_path.len() > best_len {
                best_match = Some((mount_path, mount));
                best_len = mount_path.len();
            }
        }

        best_match.map(|(_, mount)| mount.clone())
    }

    /// Obtient le point de montage racine
    pub fn root_mount(&self) -> Option<Arc<Mutex<MountPoint>>> {
        self.root_mount.clone()
    }

    /// Liste tous les points de montage
    pub fn list_mounts(&self) -> Vec<String> {
        self.mounts.keys().cloned().collect()
    }

    /// Synchronise tous les systèmes de fichiers montés
    pub fn sync_all(&self) -> VfsResult<()> {
        for (_, mount) in &self.mounts {
            let locked = mount.lock();
            locked.fs.sync()?;
        }
        Ok(())
    }

    /// Démonte tous les systèmes de fichiers
    pub fn unmount_all(&mut self) -> VfsResult<()> {
        // Collecter les chemins (sauf la racine)
        let paths: Vec<String> = self.mounts
            .keys()
            .filter(|p| *p != "/")
            .cloned()
            .collect();

        // Démonter dans l'ordre inverse (les plus profonds d'abord)
        let mut sorted_paths = paths;
        sorted_paths.sort_by(|a, b| b.len().cmp(&a.len()));

        for path in sorted_paths {
            let _ = self.unmount(&path);
        }

        Ok(())
    }

    /// Nombre de systèmes de fichiers montés
    pub fn mount_count(&self) -> usize {
        self.mounts.len()
    }
}

lazy_static! {
    /// Gestionnaire de montage global
    pub static ref MOUNT_MANAGER: Mutex<MountManager> = Mutex::new(MountManager::new());
}

/// Monte le système de fichiers racine
pub fn mount_root(fs: Arc<dyn FileSystemOps>, flags: MountFlags) -> VfsResult<Arc<Mutex<Dentry>>> {
    let mut manager = MOUNT_MANAGER.lock();

    // Obtenir l'inode racine
    let root_inode_id = fs.superblock().root_inode();
    let root_inode_ops = fs.get_inode(root_inode_id)?;
    
    // Créer un inode wrapper
    let root_inode = super::vfs_inode::get_or_create_inode(
        fs.superblock().fs_id(),
        root_inode_id,
        super::vfs_core::FileType::Directory,
        root_inode_ops,
    );

    // Créer la dentry racine
    let root_dentry = super::vfs_dentry::create_root_dentry(root_inode);

    // Monter le système de fichiers
    manager.mount("/", fs, root_dentry.clone(), flags)?;

    Ok(root_dentry)
}

/// Monte un système de fichiers à un chemin donné
pub fn mount_fs(
    path: &str,
    fs: Arc<dyn FileSystemOps>,
    flags: MountFlags,
) -> VfsResult<()> {
    let mut manager = MOUNT_MANAGER.lock();

    // Résoudre le chemin du point de montage
    let root_mount = manager.root_mount().ok_or(VfsError::NotFound)?;
    let root_dentry = root_mount.lock().mountpoint.clone();
    drop(manager);

    // Trouver la dentry du point de montage
    let mountpoint = super::vfs_dentry::path_lookup(path, root_dentry)?;

    // Monter le système de fichiers
    let mut manager = MOUNT_MANAGER.lock();
    manager.mount(path, fs, mountpoint, flags)?;

    Ok(())
}

/// Démonte un système de fichiers
pub fn unmount_fs(path: &str) -> VfsResult<()> {
    let mut manager = MOUNT_MANAGER.lock();
    manager.unmount(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummySuperblock;

    impl Superblock for DummySuperblock {
        fn fs_name(&self) -> &str {
            "dummy"
        }

        fn fs_id(&self) -> FsId {
            0
        }

        fn block_size(&self) -> u32 {
            4096
        }

        fn total_blocks(&self) -> u64 {
            1000
        }

        fn free_blocks(&self) -> u64 {
            500
        }

        fn total_inodes(&self) -> u64 {
            100
        }

        fn free_inodes(&self) -> u64 {
            50
        }

        fn is_readonly(&self) -> bool {
            false
        }

        fn root_inode(&self) -> InodeId {
            1
        }
    }

    struct DummyFileSystem;

    impl FileSystemOps for DummyFileSystem {
        fn superblock(&self) -> Arc<dyn Superblock> {
            Arc::new(DummySuperblock)
        }

        fn get_inode(&self, _inode_id: InodeId) -> VfsResult<Arc<Mutex<dyn InodeOps>>> {
            Err(VfsError::NotSupported)
        }

        fn sync(&self) -> VfsResult<()> {
            Ok(())
        }

        fn unmount(&self) -> VfsResult<()> {
            Ok(())
        }
    }

    #[test_case]
    fn test_mount_flags() {
        let flags = MountFlags::new(MountFlags::READONLY | MountFlags::NOEXEC);
        assert!(flags.is_readonly());
        assert!(flags.is_noexec());
        assert!(!flags.is_nosuid());
    }

    #[test_case]
    fn test_mount_manager() {
        let mut manager = MountManager::new();
        assert_eq!(manager.mount_count(), 0);
        
        let paths = manager.list_mounts();
        assert_eq!(paths.len(), 0);
    }
}
