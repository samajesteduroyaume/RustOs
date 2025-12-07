/// Module CFS (Completely Fair Scheduler)
/// 
/// Implémente un scheduler équitable basé sur le temps virtuel d'exécution (vruntime).
/// Chaque processus accumule du vruntime proportionnellement à son temps CPU réel,
/// inversement pondéré par sa priorité. Le processus avec le plus petit vruntime
/// est toujours sélectionné en premier.

use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::process::{Process, ProcessState};

/// Runqueue CFS - file d'attente des processus prêts
pub struct CFSRunqueue {
    /// Processus dans la runqueue, triés par vruntime
    processes: Vec<Arc<Mutex<Process>>>,
    /// Vruntime minimum dans la runqueue
    min_vruntime: u64,
    /// Nombre total de processus dans la runqueue
    count: usize,
}

impl CFSRunqueue {
    /// Crée une nouvelle runqueue CFS
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            min_vruntime: 0,
            count: 0,
        }
    }

    /// Ajoute un processus à la runqueue
    pub fn enqueue(&mut self, process: Arc<Mutex<Process>>) {
        let proc = process.lock();
        let vruntime = proc.vruntime;
        drop(proc);

        // Insérer le processus en maintenant l'ordre par vruntime
        let insert_pos = self.processes
            .iter()
            .position(|p| p.lock().vruntime > vruntime)
            .unwrap_or(self.processes.len());

        self.processes.insert(insert_pos, process);
        self.count += 1;

        // Mettre à jour min_vruntime si nécessaire
        if self.count == 1 || vruntime < self.min_vruntime {
            self.min_vruntime = vruntime;
        }
    }

    /// Retire et retourne le processus avec le plus petit vruntime
    pub fn dequeue(&mut self) -> Option<Arc<Mutex<Process>>> {
        if self.processes.is_empty() {
            return None;
        }

        let process = self.processes.remove(0);
        self.count -= 1;

        // Mettre à jour min_vruntime
        if !self.processes.is_empty() {
            self.min_vruntime = self.processes[0].lock().vruntime;
        }

        Some(process)
    }

    /// Retourne le processus avec le plus petit vruntime sans le retirer
    pub fn peek(&self) -> Option<&Arc<Mutex<Process>>> {
        self.processes.first()
    }

    /// Retire un processus spécifique de la runqueue
    pub fn remove(&mut self, pid: u64) -> Option<Arc<Mutex<Process>>> {
        let pos = self.processes
            .iter()
            .position(|p| p.lock().pid == pid)?;

        let process = self.processes.remove(pos);
        self.count -= 1;

        // Mettre à jour min_vruntime si nécessaire
        if !self.processes.is_empty() && pos == 0 {
            self.min_vruntime = self.processes[0].lock().vruntime;
        }

        Some(process)
    }

    /// Retourne le nombre de processus dans la runqueue
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

    /// Calcule le poids total de tous les processus dans la runqueue
    pub fn total_weight(&self) -> u64 {
        self.processes
            .iter()
            .map(|p| p.lock().priority.weight())
            .sum()
    }
}

/// Scheduler CFS
pub struct CFSScheduler {
    /// Runqueue des processus prêts
    runqueue: CFSRunqueue,
    /// Processus actuellement en cours d'exécution
    current: Option<Arc<Mutex<Process>>>,
    /// Quantum de temps par défaut (en ticks)
    quantum: u64,
    /// Compteur de ticks depuis le dernier scheduling
    tick_count: u64,
    /// Période de scheduling cible (en ticks)
    /// Tous les processus devraient s'exécuter au moins une fois pendant cette période
    sched_period: u64,
}

impl CFSScheduler {
    /// Crée un nouveau scheduler CFS
    pub fn new() -> Self {
        Self {
            runqueue: CFSRunqueue::new(),
            current: None,
            quantum: 10, // 10 ticks par défaut
            tick_count: 0,
            sched_period: 100, // 100 ticks = période de scheduling
        }
    }

    /// Ajoute un processus au scheduler
    pub fn add_process(&mut self, process: Arc<Mutex<Process>>) {
        let mut proc = process.lock();
        
        // Initialiser le vruntime du nouveau processus au min_vruntime
        // pour éviter qu'il monopolise le CPU
        if proc.vruntime == 0 {
            proc.vruntime = self.runqueue.min_vruntime();
        }
        
        proc.state = ProcessState::Ready;
        drop(proc);
        
        self.runqueue.enqueue(process);
    }

    /// Retire un processus du scheduler
    pub fn remove_process(&mut self, pid: u64) -> Option<Arc<Mutex<Process>>> {
        // Vérifier si c'est le processus actuel
        if let Some(ref current) = self.current {
            if current.lock().pid == pid {
                return self.current.take();
            }
        }
        
        // Sinon, chercher dans la runqueue
        self.runqueue.remove(pid)
    }

    /// Appelé à chaque tick d'horloge
    pub fn tick(&mut self) {
        self.tick_count += 1;

        // Mettre à jour le vruntime du processus actuel
        if let Some(ref current) = self.current {
            let mut proc = current.lock();
            proc.update_vruntime(1); // 1 tick
            
            // Traiter les signaux en attente
            let should_terminate = proc.deliver_pending_signals();
            drop(proc);
            
            // Si le processus doit être terminé, le retirer
            if should_terminate {
                self.current = None;
                // Le processus sera retiré de la liste lors du prochain schedule
            }
        }

        // Vérifier si on doit effectuer un changement de contexte
        if self.should_preempt() {
            self.schedule();
        }
    }

    /// Détermine si on doit préempter le processus actuel
    fn should_preempt(&self) -> bool {
        // Pas de processus actuel, on doit scheduler
        if self.current.is_none() {
            return !self.runqueue.is_empty();
        }

        // Pas de processus en attente
        if self.runqueue.is_empty() {
            return false;
        }

        let current = self.current.as_ref().unwrap();
        let current_vruntime = current.lock().vruntime;
        let next_vruntime = self.runqueue.peek().unwrap().lock().vruntime;

        // Préempter si le prochain processus a un vruntime significativement plus petit
        // ou si le quantum est écoulé
        next_vruntime + self.quantum < current_vruntime || self.tick_count >= self.quantum
    }

    /// Calcule le quantum dynamique basé sur le nombre de processus
    fn calculate_quantum(&self) -> u64 {
        if self.runqueue.is_empty() {
            return self.quantum;
        }

        let nr_running = self.runqueue.len() + if self.current.is_some() { 1 } else { 0 };
        
        // Diviser la période de scheduling par le nombre de processus
        let quantum = self.sched_period / nr_running as u64;
        
        // Assurer un quantum minimum de 1 tick
        quantum.max(1)
    }

    /// Sélectionne et exécute le prochain processus
    pub fn schedule(&mut self) -> Option<Arc<Mutex<Process>>> {
        // Remettre le processus actuel dans la runqueue s'il est toujours prêt
        if let Some(current) = self.current.take() {
            let state = current.lock().state;
            if state == ProcessState::Ready || state == ProcessState::Running {
                current.lock().state = ProcessState::Ready;
                self.runqueue.enqueue(current);
            }
            // Si le processus est terminé ou bloqué, il n'est pas remis dans la runqueue
        }

        // Nettoyer les processus terminés de la runqueue
        self.cleanup_terminated_processes();

        // Sélectionner le processus avec le plus petit vruntime
        if let Some(next) = self.runqueue.dequeue() {
            // Traiter les signaux en attente avant de commencer l'exécution
            let mut proc = next.lock();
            let should_terminate = proc.deliver_pending_signals();
            
            if should_terminate {
                // Le processus a été terminé par un signal, chercher le suivant
                drop(proc);
                return self.schedule(); // Récursion pour trouver le prochain processus valide
            }
            
            proc.state = ProcessState::Running;
            proc.last_scheduled = self.tick_count;
            drop(proc);
            
            self.current = Some(next.clone());
            self.tick_count = 0; // Réinitialiser le compteur de ticks
            
            // Recalculer le quantum
            self.quantum = self.calculate_quantum();
            
            Some(next)
        } else {
            None
        }
    }

    /// Nettoie les processus terminés de la runqueue
    fn cleanup_terminated_processes(&mut self) {
        // Retirer tous les processus terminés
        let mut i = 0;
        while i < self.runqueue.processes.len() {
            let state = self.runqueue.processes[i].lock().state;
            if state == ProcessState::Terminated {
                self.runqueue.processes.remove(i);
                self.runqueue.count -= 1;
            } else {
                i += 1;
            }
        }
        
        // Mettre à jour min_vruntime si nécessaire
        if !self.runqueue.processes.is_empty() {
            self.runqueue.min_vruntime = self.runqueue.processes[0].lock().vruntime;
        }
    }

    /// Retourne le processus actuellement en cours d'exécution
    pub fn current(&self) -> Option<&Arc<Mutex<Process>>> {
        self.current.as_ref()
    }

    /// Retourne le nombre de processus dans le scheduler
    pub fn process_count(&self) -> usize {
        self.runqueue.len() + if self.current.is_some() { 1 } else { 0 }
    }

    /// Bloque le processus actuel
    pub fn block_current(&mut self) {
        if let Some(current) = self.current.take() {
            current.lock().state = ProcessState::Blocked;
            // Le processus bloqué n'est pas remis dans la runqueue
            // Il sera réveillé plus tard par un événement
        }
    }

    /// Réveille un processus bloqué
    pub fn wake_process(&mut self, process: Arc<Mutex<Process>>) {
        let mut proc = process.lock();
        if proc.state == ProcessState::Blocked {
            proc.state = ProcessState::Ready;
            drop(proc);
            self.runqueue.enqueue(process);
        }
    }

    /// Retourne les statistiques du scheduler
    pub fn get_stats(&self) -> SchedulerStats {
        SchedulerStats {
            runqueue_len: self.runqueue.len(),
            current_pid: self.current.as_ref().map(|p| p.lock().pid),
            min_vruntime: self.runqueue.min_vruntime(),
            quantum: self.quantum,
            sched_period: self.sched_period,
        }
    }
}

/// Statistiques du scheduler
#[derive(Debug, Clone, Copy)]
pub struct SchedulerStats {
    pub runqueue_len: usize,
    pub current_pid: Option<u64>,
    pub min_vruntime: u64,
    pub quantum: u64,
    pub sched_period: u64,
}

impl Default for CFSScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessPriority;

    fn create_test_process(pid: u64, priority: ProcessPriority) -> Arc<Mutex<Process>> {
        Arc::new(Mutex::new(Process {
            pid,
            name: alloc::format!("test_{}", pid),
            state: ProcessState::Ready,
            context: crate::process::ProcessContext::default(),
            priority,
            kstack: None,
            address_space_id: 0,
            cow_pages: alloc::vec::Vec::new(),
            vruntime: 0,
            cpu_time: 0,
            last_scheduled: 0,
            signal_queue: crate::process::signal::SignalQueue::new(),
            signal_handlers: crate::process::signal::SignalHandlerTable::new(),
        }))
    }

    #[test_case]
    fn test_cfs_runqueue_order() {
        let mut runqueue = CFSRunqueue::new();
        
        let p1 = create_test_process(1, ProcessPriority::Normal);
        let p2 = create_test_process(2, ProcessPriority::Normal);
        let p3 = create_test_process(3, ProcessPriority::Normal);
        
        p1.lock().vruntime = 100;
        p2.lock().vruntime = 50;
        p3.lock().vruntime = 75;
        
        runqueue.enqueue(p1);
        runqueue.enqueue(p2);
        runqueue.enqueue(p3);
        
        // Le processus avec le plus petit vruntime devrait être en premier
        assert_eq!(runqueue.peek().unwrap().lock().pid, 2);
        assert_eq!(runqueue.dequeue().unwrap().lock().pid, 2);
        assert_eq!(runqueue.dequeue().unwrap().lock().pid, 3);
        assert_eq!(runqueue.dequeue().unwrap().lock().pid, 1);
    }

    #[test_case]
    fn test_cfs_scheduler_fairness() {
        let mut scheduler = CFSScheduler::new();
        
        let p1 = create_test_process(1, ProcessPriority::Normal);
        let p2 = create_test_process(2, ProcessPriority::Normal);
        
        scheduler.add_process(p1);
        scheduler.add_process(p2);
        
        // Simuler plusieurs ticks
        for _ in 0..20 {
            scheduler.tick();
        }
        
        // Les deux processus devraient avoir un vruntime similaire
        // (à quelques ticks près)
        let processes: Vec<_> = scheduler.runqueue.processes.iter()
            .chain(scheduler.current.iter())
            .collect();
        
        if processes.len() >= 2 {
            let vr1 = processes[0].lock().vruntime;
            let vr2 = processes[1].lock().vruntime;
            let diff = if vr1 > vr2 { vr1 - vr2 } else { vr2 - vr1 };
            
            // La différence devrait être petite (< 20 ticks)
            assert!(diff < 20);
        }
    }

    #[test_case]
    fn test_priority_weighting() {
        let mut scheduler = CFSScheduler::new();
        
        let p_high = create_test_process(1, ProcessPriority::High);
        let p_low = create_test_process(2, ProcessPriority::Low);
        
        scheduler.add_process(p_high.clone());
        scheduler.add_process(p_low.clone());
        
        // Simuler plusieurs ticks
        for _ in 0..100 {
            scheduler.tick();
        }
        
        // Le processus haute priorité devrait avoir moins de vruntime
        // (car il accumule du vruntime plus lentement)
        let vr_high = p_high.lock().vruntime;
        let vr_low = p_low.lock().vruntime;
        
        assert!(vr_high < vr_low);
    }
}
