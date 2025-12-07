/// Module Pipes
/// 
/// Implémente pipes anonymes et named pipes (FIFO)

use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use spin::Mutex;

/// Taille du buffer de pipe
pub const PIPE_BUF_SIZE: usize = 4096;

/// Pipe
pub struct Pipe {
    /// ID du pipe
    pub id: u32,
    /// Buffer circulaire
    buffer: VecDeque<u8>,
    /// Capacité maximale
    capacity: usize,
    /// Nombre de lecteurs
    readers: usize,
    /// Nombre d'écrivains
    writers: usize,
    /// Named pipe (FIFO)
    pub name: Option<String>,
}

impl Pipe {
    /// Crée un nouveau pipe
    pub fn new(id: u32, capacity: usize) -> Self {
        Self {
            id,
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            readers: 0,
            writers: 0,
            name: None,
        }
    }
    
    /// Crée un named pipe (FIFO)
    pub fn named(id: u32, name: String, capacity: usize) -> Self {
        let mut pipe = Self::new(id, capacity);
        pipe.name = Some(name);
        pipe
    }
    
    /// Ouvre le pipe en lecture
    pub fn open_read(&mut self) {
        self.readers += 1;
    }
    
    /// Ouvre le pipe en écriture
    pub fn open_write(&mut self) {
        self.writers += 1;
    }
    
    /// Ferme le lecteur
    pub fn close_read(&mut self) {
        if self.readers > 0 {
            self.readers -= 1;
        }
    }
    
    /// Ferme l'écrivain
    pub fn close_write(&mut self) {
        if self.writers > 0 {
            self.writers -= 1;
        }
    }
    
    /// Écrit dans le pipe
    pub fn write(&mut self, data: &[u8]) -> Result<usize, PipeError> {
        if self.readers == 0 {
            return Err(PipeError::BrokenPipe);
        }
        
        let available = self.capacity - self.buffer.len();
        if available == 0 {
            return Err(PipeError::WouldBlock);
        }
        
        let to_write = core::cmp::min(data.len(), available);
        
        for i in 0..to_write {
            self.buffer.push_back(data[i]);
        }
        
        Ok(to_write)
    }
    
    /// Lit depuis le pipe
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, PipeError> {
        if self.buffer.is_empty() {
            if self.writers == 0 {
                return Ok(0); // EOF
            }
            return Err(PipeError::WouldBlock);
        }
        
        let to_read = core::cmp::min(buffer.len(), self.buffer.len());
        
        for i in 0..to_read {
            buffer[i] = self.buffer.pop_front().unwrap();
        }
        
        Ok(to_read)
    }
    
    /// Vérifie si le pipe est vide
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    /// Vérifie si le pipe est plein
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }
    
    /// Retourne le nombre d'octets disponibles
    pub fn available(&self) -> usize {
        self.buffer.len()
    }
}

/// Gestionnaire de pipes
pub struct PipeManager {
    /// Pipes par ID
    pipes: BTreeMap<u32, Pipe>,
    /// Named pipes par nom
    named_pipes: BTreeMap<String, u32>,
    /// Prochain ID
    next_id: u32,
}

impl PipeManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            pipes: BTreeMap::new(),
            named_pipes: BTreeMap::new(),
            next_id: 1,
        }
    }
    
    /// Crée un pipe anonyme
    pub fn create_pipe(&mut self) -> (u32, u32) {
        let id = self.next_id;
        self.next_id += 1;
        
        let mut pipe = Pipe::new(id, PIPE_BUF_SIZE);
        pipe.open_read();
        pipe.open_write();
        
        self.pipes.insert(id, pipe);
        
        // Retourne (read_fd, write_fd)
        (id, id)
    }
    
    /// Crée un named pipe (FIFO)
    pub fn mkfifo(&mut self, name: String) -> Result<u32, PipeError> {
        if self.named_pipes.contains_key(&name) {
            return Err(PipeError::AlreadyExists);
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
        let pipe = Pipe::named(id, name.clone(), PIPE_BUF_SIZE);
        self.pipes.insert(id, pipe);
        self.named_pipes.insert(name, id);
        
        Ok(id)
    }
    
    /// Ouvre un named pipe
    pub fn open_fifo(&mut self, name: &str, for_write: bool) -> Result<u32, PipeError> {
        let id = *self.named_pipes.get(name).ok_or(PipeError::NotFound)?;
        
        let pipe = self.pipes.get_mut(&id).ok_or(PipeError::NotFound)?;
        
        if for_write {
            pipe.open_write();
        } else {
            pipe.open_read();
        }
        
        Ok(id)
    }
    
    /// Écrit dans un pipe
    pub fn write(&mut self, id: u32, data: &[u8]) -> Result<usize, PipeError> {
        let pipe = self.pipes.get_mut(&id).ok_or(PipeError::NotFound)?;
        pipe.write(data)
    }
    
    /// Lit depuis un pipe
    pub fn read(&mut self, id: u32, buffer: &mut [u8]) -> Result<usize, PipeError> {
        let pipe = self.pipes.get_mut(&id).ok_or(PipeError::NotFound)?;
        pipe.read(buffer)
    }
    
    /// Ferme un pipe
    pub fn close(&mut self, id: u32, for_write: bool) -> Result<(), PipeError> {
        let pipe = self.pipes.get_mut(&id).ok_or(PipeError::NotFound)?;
        
        if for_write {
            pipe.close_write();
        } else {
            pipe.close_read();
        }
        
        // Supprimer si plus de lecteurs ni d'écrivains
        if pipe.readers == 0 && pipe.writers == 0 {
            if let Some(name) = &pipe.name {
                self.named_pipes.remove(name);
            }
            self.pipes.remove(&id);
        }
        
        Ok(())
    }
}

/// Erreurs de pipe
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeError {
    NotFound,
    BrokenPipe,
    WouldBlock,
    AlreadyExists,
}

/// Instance globale
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PIPE_MANAGER: Mutex<PipeManager> = Mutex::new(PipeManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_pipe_creation() {
        let mut pipe = Pipe::new(1, 1024);
        pipe.open_read();
        pipe.open_write();
        
        assert_eq!(pipe.readers, 1);
        assert_eq!(pipe.writers, 1);
        assert!(pipe.is_empty());
    }
    
    #[test_case]
    fn test_pipe_write_read() {
        let mut pipe = Pipe::new(1, 1024);
        pipe.open_read();
        pipe.open_write();
        
        let data = b"Hello, World!";
        let written = pipe.write(data).unwrap();
        assert_eq!(written, data.len());
        
        let mut buffer = [0u8; 20];
        let read = pipe.read(&mut buffer).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&buffer[..read], data);
    }
    
    #[test_case]
    fn test_pipe_manager() {
        let mut manager = PipeManager::new();
        let (read_fd, write_fd) = manager.create_pipe();
        
        assert_eq!(read_fd, write_fd);
        
        let data = b"Test";
        manager.write(write_fd, data).unwrap();
        
        let mut buffer = [0u8; 10];
        let n = manager.read(read_fd, &mut buffer).unwrap();
        assert_eq!(n, data.len());
    }
    
    #[test_case]
    fn test_named_pipe() {
        let mut manager = PipeManager::new();
        let id = manager.mkfifo("test_fifo".into()).unwrap();
        
        let write_fd = manager.open_fifo("test_fifo", true).unwrap();
        let read_fd = manager.open_fifo("test_fifo", false).unwrap();
        
        assert_eq!(write_fd, read_fd);
        assert_eq!(write_fd, id);
    }
}
