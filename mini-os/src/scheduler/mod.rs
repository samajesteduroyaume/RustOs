use alloc::sync::Arc;
use spin::Mutex;
use crate::process::{Process, ProcessManager};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::arch::asm;

pub mod cfs;
pub use cfs::{CFSScheduler, CFSRunqueue};

pub mod policy;
pub use policy::{SchedulingPolicy, PolicyStats, CFSPolicy, RoundRobinPolicy};

pub mod config;
pub use config::{SchedulerConfig, SchedulerPolicyType, SCHEDULER_CONFIG, switch_scheduler_policy, get_current_policy};

/// Algorithme de planification
#[derive(Debug, Clone, Copy)]
pub enum SchedulerPolicy {
    /// Tourniquet (Round Robin)
    RoundRobin,
    /// Par priorité
    Priority,
    /// Premier arrivé, premier servi
    Fifo,
}

/// Planificateur de tâches
pub struct Scheduler {
    /// Gestionnaire de processus
    process_manager: Arc<Mutex<ProcessManager>>,
    /// Politique de planification
    policy: SchedulerPolicy,
    /// Compteur de ticks
    tick_count: AtomicUsize,
    /// Quantum (nombre de ticks par processus)
    quantum: usize,
}

impl Scheduler {
    /// Crée un nouveau planificateur
    pub fn new(process_manager: Arc<Mutex<ProcessManager>>, policy: SchedulerPolicy) -> Self {
        Self {
            process_manager,
            policy,
            tick_count: AtomicUsize::new(0),
            quantum: 10, // Valeur par défaut
        }
    }
    
    /// Définit le quantum
    pub fn set_quantum(&mut self, quantum: usize) {
        self.quantum = quantum;
    }
    
    /// Appelé à chaque tick d'horloge
    pub fn tick(&self) {
        let tick = self.tick_count.fetch_add(1, Ordering::SeqCst) + 1;
        
        // Vérifier si on doit effectuer un changement de contexte
        if tick % self.quantum == 0 {
            self.schedule();
        }
    }
    
    /// Sélectionne le prochain processus à exécuter
    pub fn schedule(&self) -> Option<Arc<Mutex<Process>>> {
        let mut pm = self.process_manager.lock();
        
        match self.policy {
            SchedulerPolicy::RoundRobin => {
                // Implémentation simple du round-robin
                if let Some(next_process) = pm.schedule() {
                    // Sauvegarder le contexte du processus actuel
                    if let Some(current_pid) = pm.current_pid() {
                        if let Some(proc) = pm.processes().iter().find(|p| p.lock().pid == current_pid) {
                            proc.lock().save_context();
                        }
                    }
                    
                    // Restaurer le contexte du prochain processus
                    next_process.lock().restore_context();
                    Some(next_process)
                } else {
                    None
                }
            }
            _ => {
                // TODO: Implémenter d'autres algorithmes de planification
                None
            }
        }
    }
    
    /// Démarre le planificateur
    pub fn run(&self) -> ! {
        loop {
            if let Some(process) = self.schedule() {
                // Le contexte sera restauré par schedule()
                drop(process); // Libérer le verrou avant de dormir
                unsafe { asm!("hlt") }; // Attendre la prochaine interruption
            } else {
                // Aucun processus à exécuter, attendre une interruption
                unsafe { asm!("hlt") };
            }
        }
    }
}
