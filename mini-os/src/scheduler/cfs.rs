/// Module CFS (Completely Fair Scheduler)
/// 
/// Implémente un scheduler équitable basé sur le temps virtuel d'exécution (vruntime).
/// Chaque thread accumule du vruntime proportionnellement à son temps CPU réel,
/// inversement pondéré par sa priorité. Le thread avec le plus petit vruntime
/// est toujours sélectionné en premier.

use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::process::{Thread, ThreadState, ProcessPriority};

/// Runqueue CFS - file d'attente des threads prêts
pub struct CFSRunqueue {
    /// Threads dans la runqueue, triés par vruntime
    pub threads: Vec<Arc<Mutex<Thread>>>,
    /// Vruntime minimum dans la runqueue
    pub min_vruntime: u64,
    /// Nombre total de threads dans la runqueue
    pub count: usize,
}

impl CFSRunqueue {
    /// Crée une nouvelle runqueue CFS
    pub fn new() -> Self {
        Self {
            threads: Vec::new(),
            min_vruntime: 0,
            count: 0,
        }
    }

    /// Ajoute un thread à la runqueue
    pub fn enqueue(&mut self, thread: Arc<Mutex<Thread>>) {
        let th = thread.lock();
        let vruntime = th.vruntime;
        drop(th);

        // Insérer le thread en maintenant l'ordre par vruntime
        let insert_pos = self.threads
            .iter()
            .position(|t| t.lock().vruntime > vruntime)
            .unwrap_or(self.threads.len());

        self.threads.insert(insert_pos, thread);
        self.count += 1;

        // Mettre à jour min_vruntime si nécessaire
        if self.count == 1 || vruntime < self.min_vruntime {
            self.min_vruntime = vruntime;
        }
    }

    /// Retire et retourne le thread avec le plus petit vruntime
    pub fn dequeue(&mut self) -> Option<Arc<Mutex<Thread>>> {
        if self.threads.is_empty() {
            return None;
        }

        let thread = self.threads.remove(0);
        self.count -= 1;

        // Mettre à jour min_vruntime
        if !self.threads.is_empty() {
            self.min_vruntime = self.threads[0].lock().vruntime;
        }

        Some(thread)
    }

    /// Retourne le thread avec le plus petit vruntime sans le retirer
    pub fn peek(&self) -> Option<&Arc<Mutex<Thread>>> {
        self.threads.first()
    }

    /// Retire un thread spécifique de la runqueue
    pub fn remove(&mut self, tid: u64) -> Option<Arc<Mutex<Thread>>> {
        let pos = self.threads
            .iter()
            .position(|t| t.lock().tid == tid)?;

        let thread = self.threads.remove(pos);
        self.count -= 1;

        // Mettre à jour min_vruntime si nécessaire
        if !self.threads.is_empty() && pos == 0 {
            self.min_vruntime = self.threads[0].lock().vruntime;
        }

        Some(thread)
    }

    /// Retourne le nombre de threads dans la runqueue
    pub fn len(&self) -> usize {
        self.count
    }

    /// Vérifie si la runqueue est vide
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Retourne le vruntime minimum
    pub fn min_vruntime(&self) -> u64 {
        self.min_vruntime
    }

    /// Calcule le poids total de tous les threads dans la runqueue
    pub fn total_weight(&self) -> u64 {
        self.threads
            .iter()
            .map(|t| t.lock().priority.weight())
            .sum()
    }
}

/// Scheduler CFS
pub struct CFSScheduler {
    /// Runqueue des threads prêts
    runqueue: CFSRunqueue,
    /// Période de scheduling cible (en ticks)
    sched_period: u64,
}

impl CFSScheduler {
    /// Crée un nouveau scheduler CFS
    pub fn new() -> Self {
        Self {
            runqueue: CFSRunqueue::new(),
            sched_period: 100,
        }
    }

    /// Ajoute un thread au scheduler
    pub fn add_thread(&mut self, thread: Arc<Mutex<Thread>>) {
        let mut th = thread.lock();
        
        // Initialiser le vruntime du nouveau thread au min_vruntime
        if th.vruntime == 0 {
            th.vruntime = self.runqueue.min_vruntime();
        }
        
        th.state = ThreadState::Ready;
        drop(th);
        
        self.runqueue.enqueue(thread);
    }

    /// Retire un thread du scheduler
    pub fn remove_thread(&mut self, tid: u64) -> Option<Arc<Mutex<Thread>>> {
        self.runqueue.remove(tid)
    }

    /// Sélectionne et exécute le prochain thread
    pub fn schedule(&mut self, current_thread: Option<Arc<Mutex<Thread>>>) -> Option<Arc<Mutex<Thread>>> {
        // Remettre le thread actuel dans la runqueue s'il est toujours prêt
        if let Some(current) = current_thread {
            let state = current.lock().state;
            if state == ThreadState::Ready || state == ThreadState::Running {
                current.lock().state = ThreadState::Ready;
                self.runqueue.enqueue(current);
            }
        }

        // Nettoyer les threads terminés de la runqueue
        self.cleanup_terminated_threads();

        // Sélectionner le thread avec le plus petit vruntime
        if let Some(next) = self.runqueue.dequeue() {
            let mut th = next.lock();
            th.state = ThreadState::Running;
            drop(th);
            
            Some(next)
        } else {
            None
        }
    }

    /// Nettoie les threads terminés de la runqueue
    fn cleanup_terminated_threads(&mut self) {
        let mut i = 0;
        while i < self.runqueue.threads.len() {
            let state = self.runqueue.threads[i].lock().state;
            if state == ThreadState::Terminated {
                self.runqueue.threads.remove(i);
                self.runqueue.count -= 1;
            } else {
                i += 1;
            }
        }
        
        if !self.runqueue.threads.is_empty() {
             self.runqueue.min_vruntime = self.runqueue.threads[0].lock().vruntime;
        }
    }

    /// Retourne le nombre de threads dans la runqueue
    pub fn thread_count(&self) -> usize {
        self.runqueue.len()
    }

    /// Réveille un thread bloqué
    pub fn wake_thread(&mut self, thread: Arc<Mutex<Thread>>) {
        let mut th = thread.lock();
        if th.state == ThreadState::Blocked {
            th.state = ThreadState::Ready;
            drop(th);
            self.runqueue.enqueue(thread);
        }
    }
}

impl Default for CFSScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // Tests disabled during SMP refactoring
}
