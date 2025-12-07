/// Module de configuration dynamique du scheduler
/// 
/// Permet de changer de politique de scheduling au runtime

use alloc::sync::Arc;
use spin::Mutex;
use crate::process::{Process, ProcessManager};
// use super::policy::{SchedulingPolicy, CFSPolicy, RoundRobinPolicy, PolicyStats};

/// Type de politique de scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerPolicyType {
    /// Completely Fair Scheduler
    CFS,
    /// Round-Robin
    RoundRobin,
}

/// Configuration du scheduler
pub struct SchedulerConfig {
    /// Politique actuellement active
    current_policy: SchedulerPolicyType,
    /// Instance CFS
    cfs: CFSPolicy,
    /// Instance Round-Robin
    round_robin: RoundRobinPolicy,
}

impl SchedulerConfig {
    /// Crée une nouvelle configuration
    pub fn new(initial_policy: SchedulerPolicyType) -> Self {
        Self {
            current_policy: initial_policy,
            cfs: CFSPolicy::new(),
            round_robin: RoundRobinPolicy::new(10), // Quantum de 10 ticks
        }
    }
    
    /// Retourne la politique active
    pub fn current_policy(&self) -> SchedulerPolicyType {
        self.current_policy
    }
    
    /// Change de politique de scheduling
    /// 
    /// Migre tous les processus de l'ancienne politique vers la nouvelle
    pub fn switch_policy(&mut self, new_policy: SchedulerPolicyType) {
        if self.current_policy == new_policy {
            return; // Déjà sur cette politique
        }
        
        // Migrer les processus de l'ancienne vers la nouvelle politique
        let processes = match self.current_policy {
            SchedulerPolicyType::CFS => {
                // Extraire tous les processus du CFS
                let mut procs = alloc::vec::Vec::new();
                while let Some(proc) = self.cfs.schedule() {
                    procs.push(proc);
                }
                procs
            }
            SchedulerPolicyType::RoundRobin => {
                // Extraire tous les processus du Round-Robin
                let mut procs = alloc::vec::Vec::new();
                while let Some(proc) = self.round_robin.schedule() {
                    procs.push(proc);
                }
                procs
            }
        };
        
        // Ajouter les processus à la nouvelle politique
        for proc in processes {
            match new_policy {
                SchedulerPolicyType::CFS => self.cfs.add_process(proc),
                SchedulerPolicyType::RoundRobin => self.round_robin.add_process(proc),
            }
        }
        
        self.current_policy = new_policy;
    }
    
    /// Retourne une référence mutable à la politique active
    pub fn active_policy_mut(&mut self) -> &mut dyn SchedulingPolicy {
        match self.current_policy {
            SchedulerPolicyType::CFS => &mut self.cfs,
            SchedulerPolicyType::RoundRobin => &mut self.round_robin,
        }
    }
    
    /// Retourne les statistiques de la politique active
    pub fn get_active_stats(&self) -> PolicyStats {
        match self.current_policy {
            SchedulerPolicyType::CFS => self.cfs.get_stats(),
            SchedulerPolicyType::RoundRobin => self.round_robin.get_stats(),
        }
    }
    
    /// Retourne les statistiques de toutes les politiques
    pub fn get_all_stats(&self) -> AllPolicyStats {
        AllPolicyStats {
            current_policy: self.current_policy,
            cfs_stats: self.cfs.get_stats(),
            round_robin_stats: self.round_robin.get_stats(),
        }
    }
    
    /// Configure le quantum pour Round-Robin
    pub fn set_round_robin_quantum(&mut self, quantum: usize) {
        self.round_robin.set_quantum(quantum);
    }
}

/// Statistiques de toutes les politiques
#[derive(Debug, Clone)]
pub struct AllPolicyStats {
    pub current_policy: SchedulerPolicyType,
    pub cfs_stats: PolicyStats,
    pub round_robin_stats: PolicyStats,
}

/// Instance globale de la configuration du scheduler
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCHEDULER_CONFIG: Mutex<SchedulerConfig> = 
        Mutex::new(SchedulerConfig::new(SchedulerPolicyType::CFS));
}

/// Change la politique de scheduling globale
pub fn switch_scheduler_policy(new_policy: SchedulerPolicyType) {
    SCHEDULER_CONFIG.lock().switch_policy(new_policy);
}

/// Retourne la politique active
pub fn get_current_policy() -> SchedulerPolicyType {
    SCHEDULER_CONFIG.lock().current_policy()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_scheduler_config_creation() {
        let config = SchedulerConfig::new(SchedulerPolicyType::CFS);
        assert_eq!(config.current_policy(), SchedulerPolicyType::CFS);
    }
    
    #[test_case]
    fn test_policy_switch() {
        let mut config = SchedulerConfig::new(SchedulerPolicyType::CFS);
        config.switch_policy(SchedulerPolicyType::RoundRobin);
        assert_eq!(config.current_policy(), SchedulerPolicyType::RoundRobin);
    }
    
    #[test_case]
    fn test_quantum_configuration() {
        let mut config = SchedulerConfig::new(SchedulerPolicyType::RoundRobin);
        config.set_round_robin_quantum(20);
        // Quantum devrait être mis à jour
    }
}
