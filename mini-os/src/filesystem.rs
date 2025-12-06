use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use spin::Mutex;

// Types d'erreur pour le système de fichiers
#[derive(Debug, Clone, PartialEq)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotEmpty,
    NotADirectory,
    IsDirectory,
    InvalidPath,
    PermissionDenied,
    IOError,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FsError::NotFound => write!(f, "File or directory not found"),
            FsError::AlreadyExists => write!(f, "File or directory already exists"),
            FsError::NotEmpty => write!(f, "Directory not empty"),
            FsError::NotADirectory => write!(f, "Not a directory"),
            FsError::IsDirectory => write!(f, "Is a directory"),
            FsError::InvalidPath => write!(f, "Invalid path"),
            FsError::PermissionDenied => write!(f, "Permission denied"),
            FsError::IOError => write!(f, "I/O error"),
        }
    }
}

// Type de nœud dans le système de fichiers
#[derive(Debug, Clone)]
pub enum NodeType {
    File,
    Directory,
}

// Métadonnées d'un nœud
#[derive(Debug, Clone)]
pub struct Metadata {
    pub node_type: NodeType,
    pub size: usize,
    pub created: u64,  // Timestamp
    pub modified: u64, // Timestamp
    pub accessed: u64, // Timestamp
}

// Représente un fichier ou un répertoire
#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub metadata: Metadata,
    pub content: Vec<u8>,
    pub children: BTreeMap<String, Node>,
}

impl Node {
    // Crée un nouveau fichier
    pub fn new_file(name: &str, content: &[u8]) -> Self {
        let now = 0; // TODO: Implémenter un vrai timestamp
        Node {
            name: name.into(),
            metadata: Metadata {
                node_type: NodeType::File,
                size: content.len(),
                created: now,
                modified: now,
                accessed: now,
            },
            content: content.to_vec(),
            children: BTreeMap::new(),
        }
    }

    // Crée un nouveau répertoire
    pub fn new_directory(name: &str) -> Self {
        let now = 0; // TODO: Implémenter un vrai timestamp
        Node {
            name: name.into(),
            metadata: Metadata {
                node_type: NodeType::Directory,
                size: 0,
                created: now,
                modified: now,
                accessed: now,
            },
            content: Vec::new(),
            children: BTreeMap::new(),
        }
    }

    // Vérifie si c'est un répertoire
    pub fn is_dir(&self) -> bool {
        matches!(self.metadata.node_type, NodeType::Directory)
    }

    // Vérifie si c'est un fichier
    pub fn is_file(&self) -> bool {
        matches!(self.metadata.node_type, NodeType::File)
    }
}

// Système de fichiers en mémoire (RAMFS)
pub struct RAMFS {
    root: Node,
    // Cache pour les chemins fréquemment accédés
    path_cache: BTreeMap<String, Node>,
}

impl RAMFS {
    /// Crée une nouvelle instance de RAMFS
    pub fn new() -> Self {
        let mut root = Node::new_directory("");
        // Créer la structure de dossiers par défaut
        root.children.insert("boot".into(), Node::new_directory("boot"));
        root.children.insert("home".into(), Node::new_directory("home"));
        root.children.insert("etc".into(), Node::new_directory("etc"));
        
        RAMFS {
            root,
            path_cache: BTreeMap::new(),
        }
    }

    // Divise un chemin en segments
    fn split_path(&self, path: &str) -> Vec<&str> {
        path.split('/')
            .filter(|s| !s.is_empty())
            .collect()
    }

    // Trouve un nœud à partir d'un chemin
    fn find_node(&self, path: &str) -> Result<&Node, FsError> {
        // Vérifier le cache d'abord
        if let Some(cached) = self.path_cache.get(path) {
            return Ok(cached);
        }

        let segments = self.split_path(path);
        let mut current = &self.root;

        for segment in segments {
            if let Some(child) = current.children.get(segment) {
                current = child;
            } else {
                return Err(FsError::NotFound);
            }
        }

        Ok(current)
    }

    // Trouve un nœud mutable à partir d'un chemin
    fn find_node_mut(&mut self, path: &str) -> Result<&mut Node, FsError> {
        let segments = self.split_path(path);
        let mut current = &mut self.root;

        for segment in segments {
            if !current.children.contains_key(segment) {
                return Err(FsError::NotFound);
            }
            current = current.children.get_mut(segment).unwrap();
        }

        Ok(current)
    }

    // Trouve le nœud parent d'un chemin
    fn find_parent_node(&mut self, path: &str) -> Result<&mut Node, FsError> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Ok(&mut self.root);
        }

        let parent_path = &segments[..segments.len() - 1].join("/");
        self.find_node_mut(parent_path)
    }

    /// Crée un nouveau fichier
    pub fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Err(FsError::InvalidPath);
        }

        let filename = segments.last().unwrap();
        let parent_path = &segments[..segments.len() - 1].join("/");
        
        let parent = if parent_path.is_empty() {
            &mut self.root
        } else {
            self.find_node_mut(parent_path).map_err(|_| FsError::NotFound)?
        };

        if !parent.is_dir() {
            return Err(FsError::NotADirectory);
        }

        if parent.children.contains_key(*filename) {
            return Err(FsError::AlreadyExists);
        }

        let file = Node::new_file(filename, content);
        parent.children.insert(filename.into(), file);
        
        // Mettre à jour le cache
        self.path_cache.insert(path.into(), self.find_node(path).unwrap().clone());
        
        Ok(())
    }

    /// Lit le contenu d'un fichier
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let node = self.find_node(path)?;
        
        if !node.is_file() {
            return Err(FsError::NotADirectory);
        }
        
        // Mettre à jour le timestamp d'accès
        // Note: Dans une implémentation complète, nous devrions le faire de manière mutable
        
        Ok(node.content.clone())
    }

    /// Écrit dans un fichier existant
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let node = self.find_node_mut(path)?;
        
        if !node.is_file() {
            return Err(FsError::IsDirectory);
        }
        
        node.content = content.to_vec();
        node.metadata.size = content.len();
        // Mettre à jour le timestamp de modification
        // node.metadata.modified = get_current_timestamp();
        
        // Mettre à jour le cache
        self.path_cache.insert(path.into(), node.clone());
        
        Ok(())
    }

    /// Crée un nouveau répertoire
    pub fn create_dir(&mut self, path: &str) -> Result<(), FsError> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Err(FsError::InvalidPath);
        }

        let dirname = segments.last().unwrap();
        let parent_path = &segments[..segments.len() - 1].join("/");
        
        let parent = if parent_path.is_empty() {
            &mut self.root
        } else {
            self.find_node_mut(parent_path).map_err(|_| FsError::NotFound)?
        };

        if !parent.is_dir() {
            return Err(FsError::NotADirectory);
        }

        if parent.children.contains_key(*dirname) {
            return Err(FsError::AlreadyExists);
        }

        let dir = Node::new_directory(dirname);
        parent.children.insert(dirname.into(), dir);
        
        Ok(())
    }

    /// Supprime un fichier
    pub fn remove_file(&mut self, path: &str) -> Result<(), FsError> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Err(FsError::InvalidPath);
        }

        let filename = segments.last().unwrap();
        let parent_path = &segments[..segments.len() - 1].join("/");
        
        let parent = if parent_path.is_empty() {
            &mut self.root
        } else {
            self.find_node_mut(parent_path).map_err(|_| FsError::NotFound)?
        };

        if !parent.is_dir() {
            return Err(FsError::NotADirectory);
        }

        if let Some(node) = parent.children.get(*filename) {
            if node.is_dir() {
                return Err(FsError::IsDirectory);
            }
        } else {
            return Err(FsError::NotFound);
        }

        parent.children.remove(*filename);
        
        // Supprimer du cache
        self.path_cache.remove(path);
        
        Ok(())
    }

    /// Supprime un répertoire vide
    pub fn remove_dir(&mut self, path: &str) -> Result<(), FsError> {
        let node = self.find_node(path)?;
        
        if !node.is_dir() {
            return Err(FsError::NotADirectory);
        }
        
        if !node.children.is_empty() {
            return Err(FsError::NotEmpty);
        }
        
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Err(FsError::InvalidPath);
        }
        
        let dirname = segments.last().unwrap();
        let parent_path = &segments[..segments.len() - 1].join("/");
        
        let parent = if parent_path.is_empty() {
            &mut self.root
        } else {
            self.find_node_mut(parent_path).map_err(|_| FsError::NotFound)?
        };
        
        parent.children.remove(*dirname);
        
        // Supprimer du cache
        self.path_cache.remove(path);
        
        Ok(())
    }

    /// Liste le contenu d'un répertoire
    pub fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let node = self.find_node(path)?;
        
        if !node.is_dir() {
            return Err(FsError::NotADirectory);
        }
        
        Ok(node.children.keys().cloned().collect())
    }

    /// Vérifie si un fichier ou un répertoire existe
    pub fn exists(&self, path: &str) -> bool {
        self.find_node(path).is_ok()
    }

    /// Vérifie si le chemin est un fichier
    pub fn is_file(&self, path: &str) -> bool {
        self.find_node(path).map(|n| n.is_file()).unwrap_or(false)
    }

    /// Vérifie si le chemin est un répertoire
    pub fn is_dir(&self, path: &str) -> bool {
        self.find_node(path).map(|n| n.is_dir()).unwrap_or(false)
    }
}

// Instance globale du système de fichiers
pub static FS: Mutex<RAMFS> = Mutex::new(RAMFS::new());

/// Interface de haut niveau pour le système de fichiers
pub fn init_ramfs() -> Result<(), FsError> {
    let mut fs = FS.lock();
    
    // Créer quelques fichiers système par défaut
    fs.create_file("/boot/kernel", b"Mini OS Kernel")?;
    fs.create_file("/home/readme.txt", b"Bienvenue sur Mini OS!")?;
    fs.create_file("/etc/motd", b"Bienvenue sur Mini OS!\n")?;
    
    Ok(())
}

/// Liste le contenu d'un répertoire
pub fn read_dir(path: &str) -> Result<Vec<String>, FsError> {
    FS.lock().read_dir(path)
}

/// Lit un fichier
pub fn read_file(path: &str) -> Result<Vec<u8>, FsError> {
    FS.lock().read_file(path)
}

/// Écrit dans un fichier
pub fn write_file(path: &str, content: &[u8]) -> Result<(), FsError> {
    FS.lock().write_file(path, content)
}

/// Crée un nouveau fichier
pub fn create_file(path: &str, content: &[u8]) -> Result<(), FsError> {
    FS.lock().create_file(path, content)
}

/// Crée un nouveau répertoire
pub fn create_dir(path: &str) -> Result<(), FsError> {
    FS.lock().create_dir(path)
}

/// Supprime un fichier
pub fn remove_file(path: &str) -> Result<(), FsError> {
    FS.lock().remove_file(path)
}

/// Supprime un répertoire vide
pub fn remove_dir(path: &str) -> Result<(), FsError> {
    FS.lock().remove_dir(path)
}

/// Vérifie si un fichier ou un répertoire existe
pub fn exists(path: &str) -> bool {
    FS.lock().exists(path)
}

/// Vérifie si le chemin est un fichier
pub fn is_file(path: &str) -> bool {
    FS.lock().is_file(path)
}

/// Vérifie si le chemin est un répertoire
pub fn is_dir(path: &str) -> bool {
    FS.lock().is_dir(path)
}
