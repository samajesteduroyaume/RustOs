use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use lazy_static::lazy_static;

/// Modes d'ouverture de fichier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Lecture seule
    ReadOnly,
    /// Écriture seule
    WriteOnly,
    /// Lecture et écriture
    ReadWrite,
}

/// Descripteur de fichier
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    /// Numéro du descripteur
    pub fd: usize,
    /// Chemin du fichier
    pub path: String,
    /// Mode d'ouverture
    pub mode: OpenMode,
    /// Position actuelle dans le fichier
    pub offset: u64,
    /// Taille du fichier
    pub size: u64,
}

impl FileDescriptor {
    /// Crée un nouveau descripteur de fichier
    pub fn new(fd: usize, path: &str, mode: OpenMode, size: u64) -> Self {
        Self {
            fd,
            path: String::from(path),
            mode,
            offset: 0,
            size,
        }
    }
}

/// Table des descripteurs de fichiers pour un processus
pub struct FileDescriptorTable {
    /// Liste des descripteurs ouverts
    descriptors: Vec<Option<FileDescriptor>>,
    /// Prochain FD disponible
    next_fd: usize,
}

impl FileDescriptorTable {
    /// Crée une nouvelle table de descripteurs
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            next_fd: 3, // 0, 1, 2 sont réservés pour stdin, stdout, stderr
        }
    }

    /// Ouvre un fichier et retourne son descripteur
    pub fn open(&mut self, path: &str, mode: OpenMode, size: u64) -> Result<usize, &'static str> {
        let fd = self.next_fd;
        self.next_fd += 1;

        let descriptor = FileDescriptor::new(fd, path, mode, size);
        
        // Étendre le vecteur si nécessaire
        while self.descriptors.len() <= fd {
            self.descriptors.push(None);
        }

        self.descriptors[fd] = Some(descriptor);
        Ok(fd)
    }

    /// Ferme un descripteur de fichier
    pub fn close(&mut self, fd: usize) -> Result<(), &'static str> {
        if fd < self.descriptors.len() {
            self.descriptors[fd] = None;
            Ok(())
        } else {
            Err("Descripteur invalide")
        }
    }

    /// Obtient un descripteur de fichier
    pub fn get(&self, fd: usize) -> Result<&FileDescriptor, &'static str> {
        if fd < self.descriptors.len() {
            self.descriptors[fd].as_ref().ok_or("Descripteur fermé")
        } else {
            Err("Descripteur invalide")
        }
    }

    /// Obtient un descripteur mutable
    pub fn get_mut(&mut self, fd: usize) -> Result<&mut FileDescriptor, &'static str> {
        if fd < self.descriptors.len() {
            self.descriptors[fd].as_mut().ok_or("Descripteur fermé")
        } else {
            Err("Descripteur invalide")
        }
    }

    /// Duplique un descripteur de fichier (dup2)
    pub fn dup2(&mut self, old_fd: usize, new_fd: usize) -> Result<usize, &'static str> {
        let descriptor = self.get(old_fd)?.clone();
        
        // Fermer le nouveau FD s'il est déjà ouvert
        if new_fd < self.descriptors.len() && self.descriptors[new_fd].is_some() {
            self.close(new_fd)?;
        }

        // Étendre le vecteur si nécessaire
        while self.descriptors.len() <= new_fd {
            self.descriptors.push(None);
        }

        self.descriptors[new_fd] = Some(descriptor);
        Ok(new_fd)
    }

    /// Obtient la liste des descripteurs ouverts
    pub fn list_open(&self) -> Vec<usize> {
        self.descriptors
            .iter()
            .enumerate()
            .filter_map(|(i, fd)| if fd.is_some() { Some(i) } else { None })
            .collect()
    }
}

/// Gestionnaire global des tables de descripteurs
pub struct FileDescriptorManager {
    /// Tables de descripteurs par PID
    tables: Vec<(u64, FileDescriptorTable)>,
}

impl FileDescriptorManager {
    /// Crée un nouveau gestionnaire
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
        }
    }

    /// Crée une nouvelle table pour un processus
    pub fn create_table(&mut self, pid: u64) -> Result<(), &'static str> {
        self.tables.push((pid, FileDescriptorTable::new()));
        Ok(())
    }

    /// Obtient la table d'un processus
    pub fn get_table(&mut self, pid: u64) -> Result<&mut FileDescriptorTable, &'static str> {
        self.tables
            .iter_mut()
            .find(|(p, _)| *p == pid)
            .map(|(_, table)| table)
            .ok_or("Table non trouvée")
    }

    /// Supprime la table d'un processus
    pub fn remove_table(&mut self, pid: u64) -> Result<(), &'static str> {
        if let Some(pos) = self.tables.iter().position(|(p, _)| *p == pid) {
            self.tables.remove(pos);
            Ok(())
        } else {
            Err("Table non trouvée")
        }
    }
}

lazy_static! {
    pub static ref FD_MANAGER: Mutex<FileDescriptorManager> = Mutex::new(FileDescriptorManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_fd_table_creation() {
        let table = FileDescriptorTable::new();
        assert_eq!(table.next_fd, 3);
    }

    #[test_case]
    fn test_fd_open() {
        let mut table = FileDescriptorTable::new();
        let fd = table.open("/test.txt", OpenMode::ReadOnly, 1024).unwrap();
        assert_eq!(fd, 3);
    }

    #[test_case]
    fn test_fd_close() {
        let mut table = FileDescriptorTable::new();
        let fd = table.open("/test.txt", OpenMode::ReadOnly, 1024).unwrap();
        assert!(table.close(fd).is_ok());
    }
}
