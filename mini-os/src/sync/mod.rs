use spin::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::process::ThreadState;
use crate::scheduler::SCHEDULER;
use crate::scheduler::current_thread;

/// Sémaphore pour la synchronisation entre threads
pub struct Semaphore {
    count: Mutex<i32>,
    waiters: Mutex<VecDeque<u64>>, // Queue de TIDs en attente
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
    pub fn wait(&self) {
        let tid = current_thread().expect("No current thread").lock().tid;
        loop {
            let mut count = self.count.lock();
            if *count > 0 {
                *count -= 1;
                return;
            } else {
                // Ajouter le thread à la queue d'attente
                let mut waiters = self.waiters.lock();
                waiters.push_back(tid);
                drop(waiters);
                drop(count); // Libérer le verrou avant de bloquer
                
                SCHEDULER.block_current_thread(ThreadState::Blocked);
            }
        }
    }

    /// Opération V (signal) - incrémente le sémaphore
    pub fn signal(&self) {
        let mut count = self.count.lock();
        *count += 1;

        // Réveiller un thread en attente
        if let Some(waiting_tid) = self.waiters.lock().pop_front() {
            SCHEDULER.wake_thread(waiting_tid);
        }
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
    pub fn lock(&self) {
        let tid = current_thread().expect("No current thread").lock().tid;
        loop {
            let mut locked = self.locked.lock();
            if !*locked {
                *locked = true;
                *self.owner.lock() = Some(tid);
                return;
            } else {
                let mut waiters = self.waiters.lock();
                waiters.push_back(tid);
                drop(waiters);
                drop(locked);
                
                SCHEDULER.block_current_thread(ThreadState::Blocked);
            }
        }
    }

    /// Libère le mutex
    pub fn unlock(&self) {
        let tid = current_thread().expect("No current thread").lock().tid;
        let mut owner = self.owner.lock();
        if *owner != Some(tid) {
            panic!("Le thread ne possède pas le mutex");
        }

        *owner = None;
        *self.locked.lock() = false;

        // Réveiller un thread en attente
        if let Some(waiting_tid) = self.waiters.lock().pop_front() {
            SCHEDULER.wake_thread(waiting_tid);
        }
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
    pub fn wait(&self, mutex: &MutexLock) {
        let tid = current_thread().expect("No current thread").lock().tid;
        
        // Ajouter le thread à la queue d'attente
        self.waiters.lock().push_back(tid);

        // Libérer le mutex
        mutex.unlock();

        // Bloquer le thread
        SCHEDULER.block_current_thread(ThreadState::Blocked);
        
        // Réacquérir le mutex au réveil
        mutex.lock();
    }

    /// Signale un thread en attente
    pub fn signal(&self) {
        if let Some(waiting_tid) = self.waiters.lock().pop_front() {
            SCHEDULER.wake_thread(waiting_tid);
        }
    }

    /// Signale tous les threads en attente
    pub fn broadcast(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(waiting_tid) = waiters.pop_front() {
            SCHEDULER.wake_thread(waiting_tid);
        }
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
    pub fn wait(&self) {
        let tid = current_thread().expect("No current thread").lock().tid;
        
        let mut count = self.count.lock();
        *count += 1;

        if *count == self.total {
            // Tous les threads sont arrivés, réveiller tout le monde
            *count = 0; // Reset pour réutilisation
            drop(count);
            
            let mut waiters = self.waiters.lock();
            while let Some(waiting_tid) = waiters.pop_front() {
                SCHEDULER.wake_thread(waiting_tid);
            }
        } else {
            let mut waiters = self.waiters.lock();
            waiters.push_back(tid);
            drop(waiters);
            drop(count);
            
            SCHEDULER.block_current_thread(ThreadState::Blocked);
        }
    }
}
