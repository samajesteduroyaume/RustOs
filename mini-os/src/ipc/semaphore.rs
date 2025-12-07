/// Module Semaphores
/// 
/// Implémente sémaphores POSIX

use alloc::collections::BTreeMap;
use spin::Mutex;

/// Sémaphore
pub struct Semaphore {
    /// ID du sémaphore
    pub id: u32,
    /// Valeur du compteur
    value: i32,
    /// Valeur maximale
    max_value: i32,
}

impl Semaphore {
    /// Crée un nouveau sémaphore
    pub fn new(id: u32, initial_value: i32, max_value: i32) -> Self {
        Self {
            id,
            value: initial_value,
            max_value,
        }
    }
    
    /// Wait (P operation, décrémente)
    pub fn wait(&mut self) -> Result<(), SemError> {
        if self.value <= 0 {
            return Err(SemError::WouldBlock);
        }
        
        self.value -= 1;
        Ok(())
    }
    
    /// Try wait (non-blocking)
    pub fn try_wait(&mut self) -> Result<(), SemError> {
        if self.value <= 0 {
            return Err(SemError::WouldBlock);
        }
        
        self.value -= 1;
        Ok(())
    }
    
    /// Post (V operation, incrémente)
    pub fn post(&mut self) -> Result<(), SemError> {
        if self.value >= self.max_value {
            return Err(SemError::Overflow);
        }
        
        self.value += 1;
        Ok(())
    }
    
    /// Retourne la valeur actuelle
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

/// Gestionnaire de sémaphores
pub struct SemaphoreManager {
    /// Sémaphores par ID
    semaphores: BTreeMap<u32, Semaphore>,
    /// Prochain ID
    next_id: u32,
}

impl SemaphoreManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            semaphores: BTreeMap::new(),
            next_id: 1,
        }
    }
    
    /// Crée un sémaphore
    pub fn sem_open(&mut self, initial_value: i32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let sem = Semaphore::new(id, initial_value, i32::MAX);
        self.semaphores.insert(id, sem);
        
        id
    }
    
    /// Wait
    pub fn sem_wait(&mut self, id: u32) -> Result<(), SemError> {
        let sem = self.semaphores.get_mut(&id).ok_or(SemError::NotFound)?;
        sem.wait()
    }
    
    /// Try wait
    pub fn sem_trywait(&mut self, id: u32) -> Result<(), SemError> {
        let sem = self.semaphores.get_mut(&id).ok_or(SemError::NotFound)?;
        sem.try_wait()
    }
    
    /// Post
    pub fn sem_post(&mut self, id: u32) -> Result<(), SemError> {
        let sem = self.semaphores.get_mut(&id).ok_or(SemError::NotFound)?;
        sem.post()
    }
    
    /// Get value
    pub fn sem_getvalue(&self, id: u32) -> Result<i32, SemError> {
        let sem = self.semaphores.get(&id).ok_or(SemError::NotFound)?;
        Ok(sem.get_value())
    }
    
    /// Ferme un sémaphore
    pub fn sem_close(&mut self, id: u32) -> Result<(), SemError> {
        self.semaphores.remove(&id).ok_or(SemError::NotFound)?;
        Ok(())
    }
}

/// Erreurs de sémaphore
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemError {
    NotFound,
    WouldBlock,
    Overflow,
}

/// Instance globale
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SEM_MANAGER: Mutex<SemaphoreManager> = Mutex::new(SemaphoreManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_semaphore() {
        let mut sem = Semaphore::new(1, 1, 10);
        
        assert_eq!(sem.get_value(), 1);
        
        sem.wait().unwrap();
        assert_eq!(sem.get_value(), 0);
        
        assert_eq!(sem.wait(), Err(SemError::WouldBlock));
        
        sem.post().unwrap();
        assert_eq!(sem.get_value(), 1);
    }
    
    #[test_case]
    fn test_sem_manager() {
        let mut manager = SemaphoreManager::new();
        let id = manager.sem_open(2);
        
        assert_eq!(manager.sem_getvalue(id).unwrap(), 2);
        
        manager.sem_wait(id).unwrap();
        assert_eq!(manager.sem_getvalue(id).unwrap(), 1);
        
        manager.sem_post(id).unwrap();
        assert_eq!(manager.sem_getvalue(id).unwrap(), 2);
    }
    
    #[test_case]
    fn test_binary_semaphore() {
        let mut sem = Semaphore::new(1, 1, 1);
        
        // Mutex-like behavior
        sem.wait().unwrap();
        assert_eq!(sem.try_wait(), Err(SemError::WouldBlock));
        
        sem.post().unwrap();
        assert_eq!(sem.post(), Err(SemError::Overflow));
    }
}
