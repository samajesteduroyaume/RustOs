/// VFS Dentry - Cache des entrées de répertoire
/// 
/// Le dentry cache (directory entry cache) accélère la résolution de chemins
/// en gardant en mémoire les associations nom -> inode.

use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

use super::vfs_core::*;
use super::vfs_inode::Inode;

/// Entrée de répertoire en cache (dentry)
#[derive(Clone)]
pub struct Dentry {
    /// Nom de l'entrée
    pub name: String,
    
    /// Inode associé
    pub inode: Arc<Mutex<Inode>>,
    
    /// Dentry parent (None pour la racine)
    pub parent: Option<Arc<Mutex<Dentry>>>,
    
    /// Compteur de références
    pub refcount: u32,
    
    /// Hash du nom (pour recherche rapide)
    pub hash: u64,
}

impl Dentry {
    /// Crée une nouvelle dentry
    pub fn new(
        name: String,
        inode: Arc<Mutex<Inode>>,
        parent: Option<Arc<Mutex<Dentry>>>,
    ) -> Self {
        let hash = Self::hash_name(&name);
        Self {
            name,
            inode,
            parent,
            refcount: 1,
            hash,
        }
    }

    /// Calcule le hash d'un nom
    fn hash_name(name: &str) -> u64 {
        // Hash simple (DJB2)
        let mut hash: u64 = 5381;
        for c in name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(c as u64);
        }
        hash
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

    /// Obtient le chemin complet de la dentry
    pub fn get_path(&self) -> String {
        let mut components = Vec::new();
        let current = Some(self);

        while let Some(dentry) = current {
            if dentry.name != "/" {
                components.push(dentry.name.clone());
            }
            // Simplification pour éviter les problèmes de lifetime
            // On ne peut pas traverser la chaîne parent à cause des Mutex
            break;
        }

        if components.is_empty() {
            return "/".into();
        }

        components.reverse();
        let mut path = String::from("/");
        path.push_str(&components.join("/"));
        path
    }
}

/// Cache de dentry
pub struct DentryCache {
    /// Table de hachage des dentries (clé: hash du chemin)
    entries: BTreeMap<u64, Arc<Mutex<Dentry>>>,
    
    /// Nombre maximum de dentries en cache
    max_entries: usize,
}

impl DentryCache {
    /// Crée un nouveau cache de dentry
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries,
        }
    }

    /// Calcule le hash d'un chemin complet
    fn hash_path(parent_hash: u64, name: &str) -> u64 {
        let mut hash = parent_hash;
        for c in name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(c as u64);
        }
        hash
    }

    /// Recherche une dentry dans le cache
    pub fn lookup(&self, parent: &Dentry, name: &str) -> Option<Arc<Mutex<Dentry>>> {
        let hash = Self::hash_path(parent.hash, name);
        self.entries.get(&hash).cloned()
    }

    /// Ajoute une dentry au cache
    pub fn insert(&mut self, dentry: Arc<Mutex<Dentry>>) -> VfsResult<()> {
        let hash = dentry.lock().hash;

        // Vérifier si le cache est plein
        if self.entries.len() >= self.max_entries {
            self.evict_one()?;
        }

        self.entries.insert(hash, dentry);
        Ok(())
    }

    /// Supprime une dentry du cache
    pub fn remove(&mut self, hash: u64) -> Option<Arc<Mutex<Dentry>>> {
        self.entries.remove(&hash)
    }

    /// Évince une dentry du cache
    fn evict_one(&mut self) -> VfsResult<()> {
        // Trouver une dentry avec refcount == 0
        let key_to_remove = self.entries
            .iter()
            .find(|(_, dentry)| {
                let locked = dentry.lock();
                locked.refcount == 0
            })
            .map(|(k, _)| *k);

        if let Some(key) = key_to_remove {
            self.entries.remove(&key);
            Ok(())
        } else {
            // Aucune dentry évictable trouvée
            Err(VfsError::NoSpace)
        }
    }

    /// Nombre de dentries en cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Vérifie si le cache est vide
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Efface toutes les dentries du cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Invalide toutes les dentries d'un système de fichiers
    pub fn invalidate_fs(&mut self, fs_id: FsId) {
        let keys_to_remove: Vec<u64> = self.entries
            .iter()
            .filter(|(_, dentry)| {
                let locked = dentry.lock();
                let inode_locked = locked.inode.lock();
                let matches = inode_locked.fs_id == fs_id;
                drop(inode_locked);
                drop(locked);
                matches
            })
            .map(|(k, _)| *k)
            .collect();

        for key in keys_to_remove {
            self.entries.remove(&key);
        }
    }
}

lazy_static! {
    /// Cache de dentry global
    pub static ref DENTRY_CACHE: Mutex<DentryCache> = Mutex::new(DentryCache::new(2048));
}

/// Résout un chemin en dentry
pub fn path_lookup(path: &str, root: Arc<Mutex<Dentry>>) -> VfsResult<Arc<Mutex<Dentry>>> {
    if path.is_empty() {
        return Err(VfsError::InvalidArgument);
    }

    // Chemin absolu commence à la racine
    let mut current = if path.starts_with('/') {
        root.clone()
    } else {
        root.clone() // Pour l'instant, on utilise toujours la racine
    };

    // Séparer le chemin en composants
    let components: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();

    // Résoudre chaque composant
    for component in components {
        // Gérer ".."
        if component == ".." {
            let parent = current.lock().parent.clone();
            if let Some(p) = parent {
                current = p;
            }
            continue;
        }

        // Vérifier le cache de dentry
        let cache = DENTRY_CACHE.lock();
        let cached = cache.lookup(&current.lock(), component);
        drop(cache);

        if let Some(dentry) = cached {
            current = dentry;
            continue;
        }

        // Pas en cache, rechercher dans l'inode
        let current_inode = current.lock().inode.clone();
        let inode_id = current_inode.lock().lookup(component)?;

        // Créer une nouvelle dentry (simplifié - devrait obtenir l'inode réel)
        // Pour l'instant, on retourne une erreur
        return Err(VfsError::NotFound);
    }

    Ok(current)
}

/// Crée une dentry racine
pub fn create_root_dentry(root_inode: Arc<Mutex<Inode>>) -> Arc<Mutex<Dentry>> {
    Arc::new(Mutex::new(Dentry::new(
        "/".into(),
        root_inode,
        None,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vfs_inode::*;

    struct DummyInodeOps;

    impl InodeOps for DummyInodeOps {
        fn read(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
            Ok(0)
        }

        fn write(&mut self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
            Ok(0)
        }

        fn stat(&self) -> VfsResult<FileStat> {
            Ok(FileStat::new(1, FileType::Directory))
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
    fn test_dentry_creation() {
        let ops = Arc::new(Mutex::new(DummyInodeOps));
        let inode = Arc::new(Mutex::new(Inode::new(1, 0, FileType::Directory, ops)));
        let dentry = Dentry::new("/".into(), inode, None);
        
        assert_eq!(dentry.name, "/");
        assert_eq!(dentry.refcount, 1);
    }

    #[test_case]
    fn test_dentry_cache() {
        let mut cache = DentryCache::new(10);
        let ops = Arc::new(Mutex::new(DummyInodeOps));
        let inode = Arc::new(Mutex::new(Inode::new(1, 0, FileType::Directory, ops)));
        let dentry = Arc::new(Mutex::new(Dentry::new("test".into(), inode, None)));
        
        assert!(cache.insert(dentry.clone()).is_ok());
        assert_eq!(cache.len(), 1);
    }

    #[test_case]
    fn test_hash_name() {
        let hash1 = Dentry::hash_name("test");
        let hash2 = Dentry::hash_name("test");
        let hash3 = Dentry::hash_name("other");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
