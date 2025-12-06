/// VFS Inode - Représentation en mémoire des fichiers et répertoires
/// 
/// Un inode (index node) contient les métadonnées d'un fichier ou répertoire
/// et fournit les opérations pour manipuler son contenu.

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::String;
use spin::Mutex;
use lazy_static::lazy_static;

use super::vfs_core::*;

/// Structure d'inode en mémoire
pub struct Inode {
    /// Identifiant de l'inode
    pub id: InodeId,
    
    /// Identifiant du système de fichiers
    pub fs_id: FsId,
    
    /// Statistiques du fichier
    pub stat: FileStat,
    
    /// Opérations sur l'inode (spécifiques au système de fichiers)
    pub ops: Arc<Mutex<dyn InodeOps>>,
    
    /// Compteur de références
    pub refcount: u32,
    
    /// Inode modifié (dirty)
    pub dirty: bool,
}

impl Inode {
    /// Crée un nouveau inode
    pub fn new(
        id: InodeId,
        fs_id: FsId,
        file_type: FileType,
        ops: Arc<Mutex<dyn InodeOps>>,
    ) -> Self {
        Self {
            id,
            fs_id,
            stat: FileStat::new(id, file_type),
            ops,
            refcount: 1,
            dirty: false,
        }
    }

    /// Incrémente le compteur de références
    pub fn get(&mut self) {
        self.refcount += 1;
    }

    /// Décrémente le compteur de références
    pub fn put(&mut self) -> u32 {
        if self.refcount > 0 {
            self.refcount -= 1;
        }
        self.refcount
    }

    /// Marque l'inode comme modifié
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Vérifie si l'inode est un répertoire
    pub fn is_dir(&self) -> bool {
        self.stat.file_type == FileType::Directory
    }

    /// Vérifie si l'inode est un fichier régulier
    pub fn is_file(&self) -> bool {
        self.stat.file_type == FileType::Regular
    }

    /// Vérifie si l'inode est un lien symbolique
    pub fn is_symlink(&self) -> bool {
        self.stat.file_type == FileType::Symlink
    }

    /// Lit les données de l'inode
    pub fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        if !self.is_file() && !self.is_symlink() {
            return Err(VfsError::InvalidArgument);
        }
        self.ops.lock().read(offset, buf)
    }

    /// Écrit les données dans l'inode
    pub fn write(&mut self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        if !self.is_file() {
            return Err(VfsError::InvalidArgument);
        }
        let result = self.ops.lock().write(offset, buf);
        if result.is_ok() {
            self.mark_dirty();
        }
        result
    }

    /// Recherche une entrée dans un répertoire
    pub fn lookup(&self, name: &str) -> VfsResult<InodeId> {
        if !self.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        self.ops.lock().lookup(name)
    }

    /// Crée un nouveau fichier dans un répertoire
    pub fn create(&mut self, name: &str, mode: FileMode, file_type: FileType) -> VfsResult<InodeId> {
        if !self.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        let result = self.ops.lock().create(name, mode, file_type);
        if result.is_ok() {
            self.mark_dirty();
        }
        result
    }

    /// Supprime un fichier d'un répertoire
    pub fn unlink(&mut self, name: &str) -> VfsResult<()> {
        if !self.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        let result = self.ops.lock().unlink(name);
        if result.is_ok() {
            self.mark_dirty();
        }
        result
    }

    /// Crée un répertoire
    pub fn mkdir(&mut self, name: &str, mode: FileMode) -> VfsResult<InodeId> {
        if !self.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        let result = self.ops.lock().mkdir(name, mode);
        if result.is_ok() {
            self.mark_dirty();
        }
        result
    }

    /// Supprime un répertoire
    pub fn rmdir(&mut self, name: &str) -> VfsResult<()> {
        if !self.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        let result = self.ops.lock().rmdir(name);
        if result.is_ok() {
            self.mark_dirty();
        }
        result
    }

    /// Lit les entrées d'un répertoire
    pub fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        if !self.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        self.ops.lock().readdir()
    }

    /// Tronque le fichier
    pub fn truncate(&mut self, size: u64) -> VfsResult<()> {
        if !self.is_file() {
            return Err(VfsError::InvalidArgument);
        }
        let result = self.ops.lock().truncate(size);
        if result.is_ok() {
            self.mark_dirty();
        }
        result
    }
}

/// Cache d'inodes global
pub struct InodeCache {
    /// Table de hachage des inodes (clé: (fs_id, inode_id))
    inodes: BTreeMap<(FsId, InodeId), Arc<Mutex<Inode>>>,
    
    /// Nombre maximum d'inodes en cache
    max_inodes: usize,
}

impl InodeCache {
    /// Crée un nouveau cache d'inodes
    pub fn new(max_inodes: usize) -> Self {
        Self {
            inodes: BTreeMap::new(),
            max_inodes,
        }
    }

    /// Obtient un inode du cache
    pub fn get(&self, fs_id: FsId, inode_id: InodeId) -> Option<Arc<Mutex<Inode>>> {
        self.inodes.get(&(fs_id, inode_id)).cloned()
    }

    /// Ajoute un inode au cache
    pub fn insert(&mut self, inode: Arc<Mutex<Inode>>) -> VfsResult<()> {
        let locked_inode = inode.lock();
        let key = (locked_inode.fs_id, locked_inode.id);
        drop(locked_inode);

        // Vérifier si le cache est plein
        if self.inodes.len() >= self.max_inodes {
            self.evict_one()?;
        }

        self.inodes.insert(key, inode);
        Ok(())
    }

    /// Supprime un inode du cache
    pub fn remove(&mut self, fs_id: FsId, inode_id: InodeId) -> Option<Arc<Mutex<Inode>>> {
        self.inodes.remove(&(fs_id, inode_id))
    }

    /// Évince un inode du cache (politique LRU simplifiée)
    fn evict_one(&mut self) -> VfsResult<()> {
        // Trouver un inode avec refcount == 0 et non dirty
        let key_to_remove = self.inodes
            .iter()
            .find(|(_, inode)| {
                let locked = inode.lock();
                locked.refcount == 0 && !locked.dirty
            })
            .map(|(k, _)| *k);

        if let Some(key) = key_to_remove {
            self.inodes.remove(&key);
            Ok(())
        } else {
            // Aucun inode évictable trouvé
            Err(VfsError::NoSpace)
        }
    }

    /// Synchronise tous les inodes dirty
    pub fn sync_all(&mut self) -> VfsResult<()> {
        for (_, inode) in self.inodes.iter() {
            let mut locked = inode.lock();
            if locked.dirty {
                // Synchroniser l'inode (appeler les opérations du système de fichiers)
                locked.dirty = false;
            }
        }
        Ok(())
    }

    /// Nombre d'inodes en cache
    pub fn len(&self) -> usize {
        self.inodes.len()
    }

    /// Vérifie si le cache est vide
    pub fn is_empty(&self) -> bool {
        self.inodes.is_empty()
    }

    /// Efface tous les inodes du cache
    pub fn clear(&mut self) {
        self.inodes.clear();
    }
}

lazy_static! {
    /// Cache d'inodes global
    pub static ref INODE_CACHE: Mutex<InodeCache> = Mutex::new(InodeCache::new(1024));
}

/// Obtient ou crée un inode
pub fn get_or_create_inode(
    fs_id: FsId,
    inode_id: InodeId,
    file_type: FileType,
    ops: Arc<Mutex<dyn InodeOps>>,
) -> Arc<Mutex<Inode>> {
    let mut cache = INODE_CACHE.lock();
    
    // Vérifier si l'inode est déjà en cache
    if let Some(inode) = cache.get(fs_id, inode_id) {
        inode.lock().get();
        return inode;
    }

    // Créer un nouvel inode
    let inode = Arc::new(Mutex::new(Inode::new(inode_id, fs_id, file_type, ops)));
    let _ = cache.insert(inode.clone());
    inode
}

/// Libère un inode
pub fn put_inode(inode: Arc<Mutex<Inode>>) {
    let mut locked = inode.lock();
    let refcount = locked.put();
    
    // Si le refcount atteint 0, l'inode peut être évincé du cache
    if refcount == 0 {
        // L'éviction sera gérée par le cache lors de l'insertion d'un nouvel inode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyInodeOps;

    impl InodeOps for DummyInodeOps {
        fn read(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
            Ok(0)
        }

        fn write(&mut self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
            Ok(0)
        }

        fn stat(&self) -> VfsResult<FileStat> {
            Ok(FileStat::new(1, FileType::Regular))
        }

        fn lookup(&self, _name: &str) -> VfsResult<InodeId> {
            Err(VfsError::NotFound)
        }

        fn create(&mut self, _name: &str, _mode: FileMode, _file_type: FileType) -> VfsResult<InodeId> {
            Ok(2)
        }

        fn unlink(&mut self, _name: &str) -> VfsResult<()> {
            Ok(())
        }

        fn mkdir(&mut self, _name: &str, _mode: FileMode) -> VfsResult<InodeId> {
            Ok(3)
        }

        fn rmdir(&mut self, _name: &str) -> VfsResult<()> {
            Ok(())
        }

        fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
            Ok(Vec::new())
        }

        fn truncate(&mut self, _size: u64) -> VfsResult<()> {
            Ok(())
        }
    }

    #[test_case]
    fn test_inode_creation() {
        let ops = Arc::new(Mutex::new(DummyInodeOps));
        let inode = Inode::new(1, 0, FileType::Regular, ops);
        assert_eq!(inode.id, 1);
        assert_eq!(inode.refcount, 1);
        assert!(!inode.dirty);
    }

    #[test_case]
    fn test_inode_refcount() {
        let ops = Arc::new(Mutex::new(DummyInodeOps));
        let mut inode = Inode::new(1, 0, FileType::Regular, ops);
        inode.get();
        assert_eq!(inode.refcount, 2);
        inode.put();
        assert_eq!(inode.refcount, 1);
    }

    #[test_case]
    fn test_inode_cache() {
        let mut cache = InodeCache::new(10);
        let ops = Arc::new(Mutex::new(DummyInodeOps));
        let inode = Arc::new(Mutex::new(Inode::new(1, 0, FileType::Regular, ops)));
        
        assert!(cache.insert(inode.clone()).is_ok());
        assert_eq!(cache.len(), 1);
        
        let cached = cache.get(0, 1);
        assert!(cached.is_some());
    }
}
