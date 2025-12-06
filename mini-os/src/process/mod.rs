use x86_64::{
    structures::paging::{
        PageTable, PhysFrame, Size4KiB
    },
    PhysAddr,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use core::arch::asm;
use crate::memory::vm::{VMManager, VM_MANAGER};

/// État d'un processus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Le processus est prêt à être exécuté
    Ready,
    /// Le processus est en cours d'exécution
    Running,
    /// Le processus est bloqué en attente d'une ressource
    Blocked,
    /// Le processus a terminé son exécution
    Terminated,
}

/// Représente le contexte d'exécution d'un processus
#[derive(Debug, Clone)]
pub struct ProcessContext {
    /// Pointeur de pile (RSP)
    pub rsp: u64,
    /// Pointeur d'instruction (RIP)
    pub rip: u64,
    /// Registres généraux
    pub registers: [u64; 16],
    /// Pointeur vers la table des pages
    pub page_table: Arc<Mutex<PageTable>>,
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            rsp: 0,
            rip: 0,
            registers: [0; 16],
            page_table: Arc::new(Mutex::new(PageTable::new())),
        }
    }
}

/// Représente un processus
pub struct Process {
    /// Identifiant unique du processus
    pub pid: u64,
    /// Nom du processus
    pub name: String,
    /// État actuel du processus
    pub state: ProcessState,
    /// Contexte d'exécution
    pub context: ProcessContext,
    /// Priorité du processus
    pub priority: u8,
    /// Adresse de la pile noyau
    pub kstack: Option<PhysAddr>,
    /// ID de l'espace d'adressage
    pub address_space_id: usize,
    /// Pages marquées en lecture seule pour CoW
    pub cow_pages: Vec<PhysFrame>,
}

impl Process {
    /// Crée un nouveau processus
    pub fn new(name: &str, _entry_point: fn() -> !, priority: u8) -> Result<Self, &'static str> {
        // Créer un nouvel espace d'adressage pour le processus
        let address_space_id = VM_MANAGER
            .lock()
            .as_mut()
            .ok_or("Gestionnaire de mémoire virtuelle non initialisé")?
            .create_process_space();
        
        // TODO: Allouer une pile noyau
        // TODO: Configurer la pile utilisateur
        // TODO: Initialiser le contexte d'exécution
        
        Ok(Self {
            pid: 0, // Le PID sera défini par le gestionnaire de processus
            name: String::from(name),
            state: ProcessState::Ready,
            context: ProcessContext::default(),
            priority,
            kstack: None,
            address_space_id,
            cow_pages: Vec::new(),
        })
    }
    
    /// Duplique le processus (fork)
    pub fn fork(&self) -> Result<Self, &'static str> {
        // Créer un nouvel espace d'adressage en copiant l'actuel
        let address_space_id = VM_MANAGER
            .lock()
            .as_mut()
            .ok_or("Gestionnaire de mémoire virtuelle non initialisé")?
            .create_process_space();
        
        // Marquer toutes les pages en lecture seule pour CoW
        let cow_pages = Vec::new();
        // TODO: Itérer sur toutes les pages du processus et les marquer en lecture seule
        
        let new_process = Self {
            pid: 0, // Le PID sera défini par le gestionnaire de processus
            name: format!("{}_child", self.name),
            state: ProcessState::Ready,
            context: self.context.clone(),
            priority: self.priority,
            kstack: None, // La pile noyau sera dupliquée
            address_space_id,
            cow_pages,
        };
        
        // TODO: Configurer le contexte pour le retour de fork
        
        Ok(new_process)
    }
    
    /// Sauvegarde le contexte d'exécution actuel
    pub fn save_context(&mut self) {
        // TODO: Implémenter la sauvegarde du contexte
        unimplemented!()
    }
    
    /// Restaure le contexte d'exécution
    pub fn restore_context(&self) {
        // TODO: Implémenter la restauration du contexte
        unsafe {
            asm!(
                "mov rsp, {rsp}",
                "ret",
                rsp = in(reg) self.context.rsp,
                options(noreturn)
            );
        }
    }
}

/// Gestionnaire de processus
pub struct ProcessManager {
    /// Liste des processus
    processes: Vec<Arc<Mutex<Process>>>,
    /// PID du processus actuellement en cours d'exécution
    current_pid: Option<u64>,
    /// Compteur pour générer des PID uniques
    next_pid: u64,
    /// Gestionnaire de mémoire virtuelle
    vm_manager: Option<&'static Mutex<Option<VMManager>>>,
}

impl ProcessManager {
    /// Crée un nouveau gestionnaire de processus
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_pid: None,
            next_pid: 1, // Le PID 0 est réservé pour le processus idle
            vm_manager: Some(&VM_MANAGER),
        }
    }
    
    /// Crée un nouveau processus
    pub fn create_process(&mut self, name: &str, entry_point: fn() -> !, priority: u8) -> Result<u64, &'static str> {
        let pid = self.next_pid;
        self.next_pid += 1;
        
        let mut process = Process::new(name, entry_point, priority)?;
        process.pid = pid;
        
        let process = Arc::new(Mutex::new(process));
        self.processes.push(process);
        
        Ok(pid)
    }
    
    /// Duplique le processus actuel (fork)
    pub fn fork_process(&mut self) -> Result<u64, &'static str> {
        let current_pid = self.current_pid.ok_or("Aucun processus en cours d'exécution")?;
        
        let current_process = self.processes
            .iter()
            .find(|p| p.lock().pid == current_pid)
            .ok_or("Processus courant introuvable")?;
        
        let new_process = current_process.lock().fork()?;
        let new_pid = self.next_pid;
        
        let mut new_process = new_process;
        new_process.pid = new_pid;
        
        self.next_pid += 1;
        self.processes.push(Arc::new(Mutex::new(new_process)));
        
        Ok(new_pid)
    }
    
    /// Planifie le prochain processus à exécuter
    pub fn schedule(&mut self) -> Option<Arc<Mutex<Process>>> {
        // TODO: Implémenter un algorithme de planification
        // Pour l'instant, on utilise un simple round-robin
        
        let current_pos = self.current_pid.and_then(|pid| {
            self.processes.iter().position(|p| p.lock().pid == pid)
        }).unwrap_or(0);
        
        let next_pos = (current_pos + 1) % self.processes.len();
        
        if let Some(process) = self.processes.get(next_pos) {
            self.current_pid = Some(process.lock().pid);
            Some(process.clone())
        } else {
            None
        }
    }
    
    /// Obtient le PID du processus actuel
    pub fn current_pid(&self) -> Option<u64> {
        self.current_pid
    }
    
    /// Obtient la liste des processus
    pub fn processes(&self) -> &Vec<Arc<Mutex<Process>>> {
        &self.processes
    }
}

// Fonction de test pour démontrer la création de processus
pub fn test_process() -> ! {
    // Ceci est une fonction de test qui sera exécutée dans un processus
    loop {
        // Faire quelque chose d'utile
        unsafe { asm!("hlt") };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_process_creation() {
        let mut pm = ProcessManager::new();
        let pid = pm.create_process("test", test_process, 1);
        assert_eq!(pid, 1);
        assert_eq!(pm.processes.len(), 1);
    }
}
