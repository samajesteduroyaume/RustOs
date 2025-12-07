/// Module de politique de scheduling
/// 
/// Définit le trait SchedulingPolicy permettant d'abstraire les différentes
/// politiques de scheduling (CFS, Round-Robin, etc.) et de les changer dynamiquement.

use alloc::sync::Arc;
use spin::Mutex;
use crate::process::Process;

/// Trait pour les politiques de scheduling
pub trait SchedulingPolicy: Send + Sync {
    /// Sélectionne le prochain processus à exécuter
    /// 
    /// Retourne None si aucun processus n'est prêt
    fn schedule(&mut self) -> Option<Arc<Mutex<Process>>>;
    
    /// Appelé à chaque tick d'horloge
    /// 
    /// Permet au scheduler de mettre à jour ses métriques et de décider
    /// si un changement de contexte est nécessaire
    fn tick(&mut self);
    
    /// Ajoute un processus au scheduler
    /// 
    /// Le processus doit être dans l'état Ready
    fn add_process(&mut self, process: Arc<Mutex<Process>>);
    
    /// Retire un processus du scheduler
    /// 
    /// Retourne le processus s'il était présent, None sinon
    fn remove_process(&mut self, pid: u64) -> Option<Arc<Mutex<Process>>>;
    
    /// Bloque le processus actuellement en cours d'exécution
    /// 
    /// Le processus passe à l'état Blocked et n'est plus schedulé
    /// jusqu'à ce qu'il soit réveillé
    fn block_current(&mut self);
    
    /// Réveille un processus bloqué
    /// 
    /// Le processus passe à l'état Ready et peut être schedulé
    fn wake_process(&mut self, process: Arc<Mutex<Process>>);
    
    /// Retourne le nom de la politique de scheduling
    fn name(&self) -> &'static str;
    
    /// Retourne les statistiques de la politique
    fn get_stats(&self) -> PolicyStats;
    
    /// Retourne le processus actuellement en cours d'exécution
    fn current(&self) -> Option<Arc<Mutex<Process>>>;
    
    /// Retourne le nombre de processus gérés par le scheduler
    fn process_count(&self) -> usize;
}

/// Statistiques d'une politique de scheduling
#[derive(Debug, Clone, Copy)]
pub struct PolicyStats {
    /// Nom de la politique
    pub name: &'static str,
    /// Nombre de processus dans la runqueue
    pub runqueue_size: usize,
    /// PID du processus actuel (None si aucun)
    pub current_pid: Option<u64>,
    /// Nombre total de context switches
    pub context_switches: usize,
    /// Nombre de ticks écoulés
    pub total_ticks: usize,
}

impl Default for PolicyStats {
    fn default() -> Self {
        Self {
            name: "Unknown",
            runqueue_size: 0,
            current_pid: None,
            context_switches: 0,
            total_ticks: 0,
        }
    }
}

/// Wrapper pour le scheduler CFS implémentant SchedulingPolicy
pub struct CFSPolicy {
    scheduler: crate::scheduler::cfs::CFSScheduler,
    context_switches: usize,
    total_ticks: usize,
}

impl CFSPolicy {
    /// Crée une nouvelle politique CFS
    pub fn new() -> Self {
        Self {
            scheduler: crate::scheduler::cfs::CFSScheduler::new(),
            context_switches: 0,
            total_ticks: 0,
        }
    }
}

impl SchedulingPolicy for CFSPolicy {
    fn schedule(&mut self) -> Option<Arc<Mutex<Process>>> {
        let result = self.scheduler.schedule();
        if result.is_some() {
            self.context_switches += 1;
        }
        result
    }
    
    fn tick(&mut self) {
        self.total_ticks += 1;
        self.scheduler.tick();
    }
    
    fn add_process(&mut self, process: Arc<Mutex<Process>>) {
        self.scheduler.add_process(process);
    }
    
    fn remove_process(&mut self, pid: u64) -> Option<Arc<Mutex<Process>>> {
        self.scheduler.remove_process(pid)
    }
    
    fn block_current(&mut self) {
        self.scheduler.block_current();
    }
    
    fn wake_process(&mut self, process: Arc<Mutex<Process>>) {
        self.scheduler.wake_process(process);
    }
    
    fn name(&self) -> &'static str {
        "CFS (Completely Fair Scheduler)"
    }
    
    fn get_stats(&self) -> PolicyStats {
        let sched_stats = self.scheduler.get_stats();
        PolicyStats {
            name: self.name(),
            runqueue_size: sched_stats.runqueue_len,
            current_pid: sched_stats.current_pid,
            context_switches: self.context_switches,
            total_ticks: self.total_ticks,
        }
    }
    
    fn current(&self) -> Option<Arc<Mutex<Process>>> {
        self.scheduler.current().cloned()
    }
    
    fn process_count(&self) -> usize {
        self.scheduler.process_count()
    }
}

/// Wrapper pour le scheduler Round-Robin implémentant SchedulingPolicy
pub struct RoundRobinPolicy {
    processes: alloc::vec::Vec<Arc<Mutex<Process>>>,
    current_index: usize,
    current: Option<Arc<Mutex<Process>>>,
    quantum: usize,
    tick_count: usize,
    context_switches: usize,
    total_ticks: usize,
}

impl RoundRobinPolicy {
    /// Crée une nouvelle politique Round-Robin
    pub fn new(quantum: usize) -> Self {
        Self {
            processes: alloc::vec::Vec::new(),
            current_index: 0,
            current: None,
            quantum,
            tick_count: 0,
            context_switches: 0,
            total_ticks: 0,
        }
    }
    
    /// Définit le quantum (nombre de ticks par processus)
    pub fn set_quantum(&mut self, quantum: usize) {
        self.quantum = quantum;
    }
}

impl SchedulingPolicy for RoundRobinPolicy {
    fn schedule(&mut self) -> Option<Arc<Mutex<Process>>> {
        if self.processes.is_empty() {
            self.current = None;
            return None;
        }
        
        // Remettre le processus actuel dans la liste s'il est toujours prêt
        if let Some(current) = self.current.take() {
            let state = current.lock().state;
            if state == crate::process::ProcessState::Ready || 
               state == crate::process::ProcessState::Running {
                self.processes.push(current);
            }
        }
        
        // Sélectionner le prochain processus (round-robin)
        if !self.processes.is_empty() {
            self.current_index = (self.current_index + 1) % self.processes.len();
            let next = self.processes.remove(self.current_index);
            next.lock().state = crate::process::ProcessState::Running;
            
            self.current = Some(next.clone());
            self.tick_count = 0;
            self.context_switches += 1;
            
            Some(next)
        } else {
            None
        }
    }
    
    fn tick(&mut self) {
        self.total_ticks += 1;
        self.tick_count += 1;
        
        // Préempter si le quantum est écoulé
        if self.tick_count >= self.quantum {
            self.schedule();
        }
    }
    
    fn add_process(&mut self, process: Arc<Mutex<Process>>) {
        process.lock().state = crate::process::ProcessState::Ready;
        self.processes.push(process);
    }
    
    fn remove_process(&mut self, pid: u64) -> Option<Arc<Mutex<Process>>> {
        // Vérifier si c'est le processus actuel
        if let Some(ref current) = self.current {
            if current.lock().pid == pid {
                return self.current.take();
            }
        }
        
        // Chercher dans la liste
        if let Some(pos) = self.processes.iter().position(|p| p.lock().pid == pid) {
            Some(self.processes.remove(pos))
        } else {
            None
        }
    }
    
    fn block_current(&mut self) {
        if let Some(current) = self.current.take() {
            current.lock().state = crate::process::ProcessState::Blocked;
        }
    }
    
    fn wake_process(&mut self, process: Arc<Mutex<Process>>) {
        let mut proc = process.lock();
        if proc.state == crate::process::ProcessState::Blocked {
            proc.state = crate::process::ProcessState::Ready;
            drop(proc);
            self.processes.push(process);
        }
    }
    
    fn name(&self) -> &'static str {
        "Round-Robin"
    }
    
    fn get_stats(&self) -> PolicyStats {
        PolicyStats {
            name: self.name(),
            runqueue_size: self.processes.len(),
            current_pid: self.current.as_ref().map(|p| p.lock().pid),
            context_switches: self.context_switches,
            total_ticks: self.total_ticks,
        }
    }
    
    fn current(&self) -> Option<Arc<Mutex<Process>>> {
        self.current.clone()
    }
    
    fn process_count(&self) -> usize {
        self.processes.len() + if self.current.is_some() { 1 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessPriority;
    
    #[test_case]
    fn test_policy_trait_cfs() {
        let mut policy = CFSPolicy::new();
        assert_eq!(policy.name(), "CFS (Completely Fair Scheduler)");
        assert_eq!(policy.process_count(), 0);
    }
    
    #[test_case]
    fn test_policy_trait_round_robin() {
        let mut policy = RoundRobinPolicy::new(10);
        assert_eq!(policy.name(), "Round-Robin");
        assert_eq!(policy.process_count(), 0);
    }
    
    #[test_case]
    fn test_policy_stats() {
        let policy = CFSPolicy::new();
        let stats = policy.get_stats();
        assert_eq!(stats.name, "CFS (Completely Fair Scheduler)");
        assert_eq!(stats.context_switches, 0);
    }
}
