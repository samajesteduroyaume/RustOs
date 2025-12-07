use spin::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use mini_os::process::Process;

/// Sémaphore pour la synchronisation entre processus
pub struct Semaphore {
    count: Mutex<i32>,
    waiters: Mutex<VecDeque<u64>>, // Queue de PIDs en attente
}

impl Semaphore {
    /// Crée un nouveau sémaphore avec une valeur initiale
    pub fn new(initial_count: i32) -> Self {
        Self {
            count: Mutex::new(initial_count),
            waiters: Mutex::new(VecDeque::new()),
        }
    }

    /// Opération P (wait) - décrémente le sémaphore
    pub fn wait(&self, pid: u64) -> Result<(), &'static str> {
        loop {
            let mut count = self.count.lock();
            if *count > 0 {
                *count -= 1;
                return Ok(());
            } else {
                // Ajouter le processus à la queue d'attente
                drop(count); // Libérer le verrou avant d'ajouter à la queue
                self.waiters.lock().push_back(pid);
                // TODO: Bloquer le processus
                return Err("Processus bloqué");
            }
        }
    }

    /// Opération V (signal) - incrémente le sémaphore
    pub fn signal(&self) -> Result<(), &'static str> {
        let mut count = self.count.lock();
        *count += 1;

        // Réveiller un processus en attente
        if let Some(waiting_pid) = self.waiters.lock().pop_front() {
            // TODO: Réveiller le processus avec ce PID
        }

        Ok(())
    }
}

/// Mutex pour l'exclusion mutuelle
pub struct MutexLock {
    locked: Mutex<bool>,
    owner: Mutex<Option<u64>>,
    waiters: Mutex<VecDeque<u64>>,
}

impl MutexLock {
    /// Crée un nouveau mutex
    pub fn new() -> Self {
        Self {
            locked: Mutex::new(false),
            owner: Mutex::new(None),
            waiters: Mutex::new(VecDeque::new()),
        }
    }

    /// Acquiert le mutex
    pub fn lock(&self, pid: u64) -> Result<(), &'static str> {
        loop {
            let mut locked = self.locked.lock();
            if !*locked {
                *locked = true;
                *self.owner.lock() = Some(pid);
                return Ok(());
            } else {
                drop(locked);
                self.waiters.lock().push_back(pid);
                // TODO: Bloquer le processus
                return Err("Processus bloqué");
            }
        }
    }

    /// Libère le mutex
    pub fn unlock(&self, pid: u64) -> Result<(), &'static str> {
        let mut owner = self.owner.lock();
        if *owner != Some(pid) {
            return Err("Le processus ne possède pas le mutex");
        }

        *owner = None;
        *self.locked.lock() = false;

        // Réveiller un processus en attente
        if let Some(waiting_pid) = self.waiters.lock().pop_front() {
            // TODO: Réveiller le processus avec ce PID
        }

        Ok(())
    }

    /// Vérifie si le mutex est verrouillé
    pub fn is_locked(&self) -> bool {
        *self.locked.lock()
    }
}

/// Condition variable pour la synchronisation
pub struct ConditionVariable {
    waiters: Mutex<VecDeque<u64>>,
}

impl ConditionVariable {
    /// Crée une nouvelle variable de condition
    pub fn new() -> Self {
        Self {
            waiters: Mutex::new(VecDeque::new()),
        }
    }

    /// Attend sur la variable de condition
    pub fn wait(&self, pid: u64, mutex: &MutexLock) -> Result<(), &'static str> {
        // Ajouter le processus à la queue d'attente
        self.waiters.lock().push_back(pid);

        // Libérer le mutex
        mutex.unlock(pid)?;

        // TODO: Bloquer le processus

        Ok(())
    }

    /// Signale un processus en attente
    pub fn signal(&self) -> Result<(), &'static str> {
        if let Some(waiting_pid) = self.waiters.lock().pop_front() {
            // TODO: Réveiller le processus avec ce PID
        }

        Ok(())
    }

    /// Signale tous les processus en attente
    pub fn broadcast(&self) -> Result<(), &'static str> {
        while let Some(waiting_pid) = self.waiters.lock().pop_front() {
            // TODO: Réveiller le processus avec ce PID
        }

        Ok(())
    }
}

/// Barrière de synchronisation
pub struct Barrier {
    count: Mutex<usize>,
    total: usize,
    waiters: Mutex<VecDeque<u64>>,
}

impl Barrier {
    /// Crée une nouvelle barrière
    pub fn new(total: usize) -> Self {
        Self {
            count: Mutex::new(0),
            total,
            waiters: Mutex::new(VecDeque::new()),
        }
    }

    /// Attend à la barrière
    pub fn wait(&self, pid: u64) -> Result<(), &'static str> {
        let mut count = self.count.lock();
        *count += 1;

        if *count == self.total {
            // Tous les processus sont arrivés, réveiller tout le monde
            drop(count);
            while let Some(waiting_pid) = self.waiters.lock().pop_front() {
                // TODO: Réveiller le processus avec ce PID
            }
            Ok(())
        } else {
            drop(count);
            self.waiters.lock().push_back(pid);
            // TODO: Bloquer le processus
            Err("Processus bloqué")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_semaphore_creation() {
        let sem = Semaphore::new(1);
        assert_eq!(*sem.count.lock(), 1);
    }

    #[test_case]
    fn test_mutex_creation() {
        let mutex = MutexLock::new();
        assert!(!mutex.is_locked());
    }
}
