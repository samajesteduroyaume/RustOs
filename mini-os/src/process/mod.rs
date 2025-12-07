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

pub mod elf;
use self::elf::{ElfFile, PT_LOAD, PF_X, PF_W, PF_R};

pub mod signal;
use self::signal::{SignalQueue, SignalHandlerTable};

/// Niveau de priorité d'un processus
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
    /// Priorité temps réel (la plus haute)
    Realtime = 0,
    /// Priorité haute
    High = 1,
    /// Priorité normale (par défaut)
    Normal = 2,
    /// Priorité basse
    Low = 3,
    /// Priorité idle (la plus basse)
    Idle = 4,
}

impl ProcessPriority {
    /// Convertit un u8 en ProcessPriority
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ProcessPriority::Realtime,
            1 => ProcessPriority::High,
            2 => ProcessPriority::Normal,
            3 => ProcessPriority::Low,
            _ => ProcessPriority::Idle,
        }
    }

    /// Convertit ProcessPriority en u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Retourne le poids du processus pour le scheduler CFS
    /// Plus la priorité est haute, plus le poids est élevé
    pub fn weight(self) -> u64 {
        match self {
            ProcessPriority::Realtime => 1024,
            ProcessPriority::High => 512,
            ProcessPriority::Normal => 256,
            ProcessPriority::Low => 128,
            ProcessPriority::Idle => 64,
        }
    }
}

impl Default for ProcessPriority {
    fn default() -> Self {
        ProcessPriority::Normal
    }
}

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
    /// Niveau de privilège (0 = Ring 0, 3 = Ring 3)
    pub privilege_level: u8,
    /// Pointeur de pile utilisateur (pour Ring 3)
    pub user_rsp: u64,
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            rsp: 0,
            rip: 0,
            registers: [0; 16],
            page_table: Arc::new(Mutex::new(PageTable::new())),
            privilege_level: 0, // Ring 0 par défaut
            user_rsp: 0,
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
    pub priority: ProcessPriority,
    /// Adresse de la pile noyau
    pub kstack: Option<PhysAddr>,
    /// ID de l'espace d'adressage
    pub address_space_id: usize,
    /// Pages marquées en lecture seule pour CoW
    pub cow_pages: Vec<PhysFrame>,
    /// Temps CPU virtuel utilisé (pour CFS scheduler)
    pub vruntime: u64,
    /// Temps CPU réel utilisé (en ticks)
    pub cpu_time: u64,
    /// Timestamp du dernier scheduling
    pub last_scheduled: u64,
    /// Queue de signaux en attente
    pub signal_queue: SignalQueue,
    /// Table des handlers de signaux
    pub signal_handlers: SignalHandlerTable,
}

impl Process {
    /// Crée un nouveau processus
    pub fn new(name: &str, _entry_point: fn() -> !, priority: ProcessPriority) -> Result<Self, &'static str> {
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
            vruntime: 0,
            cpu_time: 0,
            last_scheduled: 0,
            signal_queue: SignalQueue::new(),
            signal_handlers: SignalHandlerTable::new(),
        })
    }
    
    /// Définit la priorité du processus
    pub fn set_priority(&mut self, priority: ProcessPriority) {
        self.priority = priority;
    }
    
    /// Obtient la priorité du processus
    pub fn get_priority(&self) -> ProcessPriority {
        self.priority
    }
    
    /// Met à jour le temps CPU virtuel (vruntime) pour le scheduler CFS
    pub fn update_vruntime(&mut self, delta_time: u64) {
        // Le vruntime augmente inversement proportionnel au poids
        // Plus le poids est élevé (haute priorité), moins le vruntime augmente
        let weight = self.priority.weight();
        self.vruntime += (delta_time * 1024) / weight;
        self.cpu_time += delta_time;
    }
    
    /// Traite les signaux en attente pour ce processus
    /// Retourne true si le processus doit être terminé
    pub fn deliver_pending_signals(&mut self) -> bool {
        signal::SignalManager::deliver_signals(self)
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
            vruntime: self.vruntime,
            cpu_time: 0, // Le processus enfant commence avec 0 temps CPU
            last_scheduled: 0,
            signal_queue: SignalQueue::new(), // Nouvelle queue pour l'enfant
            signal_handlers: self.signal_handlers.clone(), // Hérite des handlers du parent
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
    
    /// Lance le processus en Ring 3
    pub fn execute_in_ring3(&self) -> ! {
        // Vérifier que le processus est configuré pour Ring 3
        if self.context.privilege_level != 3 {
            panic!("Process not configured for Ring 3 execution");
        }
        
        // TODO: Charger la table des pages du processus
        // TODO: Configurer les registres
        // TODO: Basculer vers Ring 3
        
        // Pour l'instant, boucle infinie
        loop {}
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
    pub fn create_process(&mut self, name: &str, entry_point: fn() -> !, priority: ProcessPriority) -> Result<u64, &'static str> {
        let pid = self.next_pid;
        self.next_pid += 1;
        
        let mut process = Process::new(name, entry_point, priority)?;
        process.pid = pid;
        
        let process = Arc::new(Mutex::new(process));
        self.processes.push(process);
        
        Ok(pid)
    }

    /// Charge et lance un exécutable depuis un fichier
    pub fn spawn(&mut self, path: &str) -> Result<u64, String> {
        let content = crate::fs::vfs_read_file(path)
            .map_err(|_| String::from("File not found"))?;
            
        self.create_process_from_elf(path, &content)
            .map_err(|e| String::from(e))
    }

    /// Crée un nouveau processus à partir de données ELF
    pub fn create_process_from_elf(&mut self, name: &str, elf_data: &[u8]) -> Result<u64, &'static str> {
        let elf = ElfFile::new(elf_data)?;
        elf.header.validate()?;

        // Créer l'espace d'adressage
        let pid = self.next_pid;
        self.next_pid += 1;
        
        // Logique de chargement simulée pour l'instant
        // TODO: Implémenter l'allocation mémoire réelle avec VMManager
        for ph in elf.program_headers() {
            if ph.p_type == PT_LOAD {
                // Segment à charger
                let _vaddr = ph.p_vaddr;
                let _memsz = ph.p_memsz;
                let _filesz = ph.p_filesz;
                let _flags = ph.p_flags;
                
                // Pour un vrai OS :
                // 1. Allouer des frames physiques
                // 2. Mapper vaddr -> frames dans le page table du process
                // 3. Copier les données depuis elf.data[ph.p_offset..]
                // 4. Zero-fill le reste (bss)
            }
        }
        
        let mut vm_manager = self.vm_manager.unwrap().lock();
        if vm_manager.is_none() {
            return Err("VMManager not initialized");
        }
        let vm = vm_manager.as_mut().unwrap();
        let address_space_id = vm.create_process_space();
        
        let entry_point = elf.header.e_entry;
        
        // Création du Process struct
        let mut process = Process {
            pid,
            name: String::from(name),
            state: ProcessState::Ready,
            context: ProcessContext::default(),
            priority: ProcessPriority::Normal,
            kstack: None,
            address_space_id,
            cow_pages: Vec::new(),
            vruntime: 0,
            cpu_time: 0,
            last_scheduled: 0,
            signal_queue: SignalQueue::new(),
            signal_handlers: SignalHandlerTable::new(),
        };
        
        process.context.rip = entry_point;
        // Stack init would normally involve mapping stack pages to the end of user space

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
        let pid = pm.create_process("test", test_process, ProcessPriority::Normal);
        assert_eq!(pid, Ok(1));
        assert_eq!(pm.processes.len(), 1);
    }
}

// Instance globale du gestionnaire de processus
use lazy_static::lazy_static;

lazy_static! {
    /// Gestionnaire de processus global
    pub static ref PROCESS_MANAGER: Mutex<ProcessManager> = Mutex::new(ProcessManager::new());
}

/// Obtient le processus actuellement en cours d'exécution
pub fn current_process() -> Option<Arc<Mutex<Process>>> {
    let pm = PROCESS_MANAGER.lock();
    let current_pid = pm.current_pid?;
    pm.processes.iter()
        .find(|p| p.lock().pid == current_pid)
        .cloned()
}

/// Obtient un processus par son PID
pub fn get_process_by_pid(pid: u64) -> Option<Arc<Mutex<Process>>> {
    PROCESS_MANAGER.lock()
        .processes
        .iter()
        .find(|p| p.lock().pid == pid)
        .cloned()
}
