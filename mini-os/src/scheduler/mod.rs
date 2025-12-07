use alloc::sync::Arc;
use spin::Mutex;
use crate::process::{Thread, ProcessManager}; // ProcessManager peut être utile pour debug ou autre
use core::sync::atomic::{AtomicUsize, Ordering};
use core::arch::asm;

pub mod cfs;
pub use cfs::{CFSScheduler, CFSRunqueue};

// pub mod policy;
// pub use policy::{SchedulingPolicy, PolicyStats, CFSPolicy, RoundRobinPolicy}; // On simplifie pour l'instant

// pub mod config;
// pub use config::{SchedulerConfig, SchedulerPolicyType, SCHEDULER_CONFIG, switch_scheduler_policy, get_current_policy};

/// Planificateur de tâches
pub struct Scheduler {
    cfs: Mutex<CFSScheduler>,
}

impl Scheduler {
    /// Crée un nouveau planificateur
    pub fn new() -> Self {
        Self {
            cfs: Mutex::new(CFSScheduler::new()),
        }
    }
    
    /// Ajoute un thread au planificateur
    pub fn add_thread(&self, thread: Arc<Mutex<Thread>>) {
        self.cfs.lock().add_thread(thread);
    }

    /// Appelé à chaque tick d'horloge
    pub fn tick(&self) {
        // Update vruntime of current thread
        if let Some(current) = self.current_thread() {
            let mut th = current.lock();
            th.update_vruntime(1);
            drop(th);
        }
        
        // In a real OS, we would check quantum in PerCpuData and trigger schedule if needed.
        // For now, we rely on the loop in run() or interrupt to call schedule.
    }
    
    /// Sélectionne le prochain thread à exécuter
    pub fn schedule(&self) -> Option<Arc<Mutex<Thread>>> {
        let current = self.current_thread();
        
        // Acquire lock on Runqueue
        let mut cfs = self.cfs.lock();
        let next = cfs.schedule(current);
        drop(cfs);
        
        // Update Per-CPU current thread
        crate::smp::percpu::set_current_thread(next.clone());
        
        next
    }
    
    /// Démarre le planificateur
    pub fn run(&self) -> ! {
        loop {
            // Scheduling loop
            if let Some(thread) = self.schedule() {
                // Simuler context switch
                let cr3 = thread.lock().context.cr3;
                if cr3 != 0 {
                    // Switch CR3 si nécessaire
                }
                drop(thread);
            }
            unsafe { asm!("hlt") };
        }
    }
    
    /// Bloque le thread courant
    pub fn block_current_thread(&self, reason: crate::process::ThreadState) {
        if let Some(current) = self.current_thread() {
            {
                let mut thread = current.lock();
                thread.state = reason; 
            }
            
            // On force un reschedule immédiat pour passer la main
            // Dans un vrai OS, on appellerait schedule() puis context_switch
            self.schedule();
            
            // Attendre d'être réveillé
            loop {
                unsafe { asm!("hlt") };
                // Si on a été reprogrammé, c'est qu'on est au moins Ready/Running
                if current.lock().state == crate::process::ThreadState::Running {
                    break;
                }
                // Si on est toujours Blocked, on re-schedule un autre thread
                // self.schedule(); 
                // Attention: Si on est bloqué, schedule() ne nous choisira PAS.
                // Donc on doit juste attendre.
            }
        }
    }

    /// Réveille un thread
    pub fn wake_thread(&self, tid: u64) {
        if let Some(thread) = crate::process::get_thread_by_tid(tid) {
            let mut t = thread.lock();
            if t.state == crate::process::ThreadState::Blocked {
                t.state = crate::process::ThreadState::Ready;
                drop(t);
                // On réinsère dans la runqueue
                self.add_thread(thread);
            }
        }
    }
    
    /// Retourne le thread courant (Per-CPU)
    pub fn current_thread(&self) -> Option<Arc<Mutex<Thread>>> {
        crate::smp::percpu::get_current_thread()
    }
}

// Instance globale du scheduler
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCHEDULER: Scheduler = Scheduler::new();
}

/// Helper pour obtenir le thread courant
pub fn current_thread() -> Option<Arc<Mutex<Thread>>> {
    SCHEDULER.current_thread()
}
