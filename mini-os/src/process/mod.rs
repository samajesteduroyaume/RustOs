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
// use crate::memory::vm::{VMManager, VM_MANAGER}; // Disabled - depends on Limine

pub mod elf;
use self::elf::{ElfFile, PT_LOAD, PF_X, PF_W, PF_R};

pub mod thread;
pub use thread::{Thread, ThreadContext, ThreadState};

pub mod signal;
use self::signal::{SignalQueue, SignalHandlerTable};

/// Niveau de priorité d'un processus
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
    Realtime = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Idle = 4,
}

impl ProcessPriority {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ProcessPriority::Realtime,
            1 => ProcessPriority::High,
            2 => ProcessPriority::Normal,
            3 => ProcessPriority::Low,
            _ => ProcessPriority::Idle,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

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
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Représente un processus
pub struct Process {
    /// Identifiant unique du processus (PID)
    pub pid: u64,
    /// Nom du processus
    pub name: String,
    /// État du processus
    pub state: ProcessState,
    /// Priorité du processus
    pub priority: ProcessPriority,
    /// ID de l'espace d'adressage (CR3)
    pub address_space_id: u64,
    /// Pages en copie sur écriture (CoW)
    pub cow_pages: Vec<u64>,
    /// File d'attente des signaux
    pub signal_queue: SignalQueue,
    /// Gestionnaires de signaux
    pub signal_handlers: SignalHandlerTable,
    /// Threads du processus
    pub threads: Vec<Arc<Mutex<Thread>>>,
}

impl Process {
    /// Crée un nouveau processus avec un thread principal
    pub fn new(pid: u64, name: &str, _entry_point: fn() -> !, priority: ProcessPriority) -> Result<Self, &'static str> {
        // VM disabled - using placeholder
        let address_space_id = 0;
        // let address_space_id = VM_MANAGER
        //     .lock()
        //     .as_mut()
        //     .ok_or("Gestionnaire de mémoire virtuelle non initialisé")?
        //     .create_process_space();
            
        let mut process = Self {
            pid,
            name: String::from(name),
            state: ProcessState::Ready,
            priority,
            address_space_id: address_space_id as u64,
            cow_pages: Vec::new(),
            signal_queue: SignalQueue::new(),
            signal_handlers: SignalHandlerTable::new(),
            threads: Vec::new(),
        };

        // Création du thread principal
        // Note: Le TID devrait être unique globalement. Pour l'instant on utilise pid * 1000 (hack).
        // Il faudrait un ThreadManager.
        let main_thread = Arc::new(Mutex::new(Thread::new(
            pid * 1000 + 1, 
            pid, 
            "main", 
            priority,
            0 // CR3 à charger (TODO: récupérer du VMManager)
        )));
        
        // Setup IP/SP du thread
        {
            let mut thread = main_thread.lock();
            thread.context.rip = _entry_point as u64;
            // thread.context.rsp = ...; // Stack setup
        }

        process.threads.push(main_thread);
        
        Ok(process)
    }

    /// Définit la priorité du processus et de tous ses threads
    pub fn set_priority(&mut self, priority: ProcessPriority) {
        self.priority = priority;
        for thread in &self.threads {
            thread.lock().set_priority(priority);
        }
    }
    
    /// Obtient la priorité du processus
    pub fn get_priority(&self) -> ProcessPriority {
        self.priority
    }

    /// Duplique le processus (fork)
    /// Note: Cela duplique l'espace d'adressage et on suppose qu'on fork depuis un thread spécifique qui deviendra le main thread du fils
    pub fn fork(&self, current_thread: &Thread, new_pid: u64) -> Result<Self, &'static str> {
        // VM disabled - using placeholder
        let address_space_id = 0;
        // let address_space_id = VM_MANAGER
        //     .lock()
        //     .as_mut()
        //     .ok_or("Gestionnaire de mémoire virtuelle non initialisé")?
        //     .create_process_space();
        
        // Marquer pages CoW (TODO)
        let cow_pages = Vec::new();

        let mut new_process = Self {
            pid: new_pid,
            name: format!("{}_child", self.name),
            state: ProcessState::Ready,
            priority: self.priority,
            address_space_id: address_space_id as u64,
            cow_pages,
            signal_queue: SignalQueue::new(),
            signal_handlers: self.signal_handlers.clone(),
            threads: Vec::new(),
        };
        
        // Dupliquer le thread courant
        let new_tid = new_pid * 1000 + 1; // Hack TID
        let mut new_thread = Thread::new(
            new_tid,
            new_pid,
            &current_thread.name,
            current_thread.priority,
            0 // CR3 TODO
        );
        
        // Copier le contexte
        new_thread.context = current_thread.context.clone();
        // Ajuster context pour retour de fork (rax=0)
        new_thread.context.registers[0] = 0; // RAX = 0 pour l'enfant

        new_process.threads.push(Arc::new(Mutex::new(new_thread)));
        
        Ok(new_process)
    }

    /// Ajoute un nouveau thread au processus
    pub fn create_thread(&mut self, entry_point: u64) -> Result<Arc<Mutex<Thread>>, &'static str> {
        // Générer un TID (Hack: pid * 1000 + count)
        let tid = self.pid * 1000 + (self.threads.len() as u64) + 1;
        
        let mut thread = Thread::new(
            tid,
            self.pid,
            &format!("{}_th{}", self.name, tid),
            self.priority,
            self.address_space_id // CR3
        );
        
        // Setup IP
        thread.context.rip = entry_point;
        
        // TODO: Allouer une nouvelle pile pour le thread
        // thread.context.rsp = ...
        
        let thread_ref = Arc::new(Mutex::new(thread));
        self.threads.push(thread_ref.clone());
        
        Ok(thread_ref)
    }
}

/// Gestionnaire de processus
pub struct ProcessManager {
    /// Liste des processus
    processes: Vec<Arc<Mutex<Process>>>,
    /// Compteur pour générer des PID uniques
    next_pid: u64,
    // VM disabled - depends on Limine
}

impl ProcessManager {
    /// Crée un nouveau gestionnaire de processus
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            next_pid: 1, // Le PID 0 est réservé pour le processus idle (ou kernel)
        }
    }
    
    /// Crée un nouveau processus
    pub fn create_process(&mut self, name: &str, entry_point: fn() -> !, priority: ProcessPriority) -> Result<u64, &'static str> {
        let pid = self.next_pid;
        self.next_pid += 1;
        
        let process_struct = Process::new(pid, name, entry_point, priority)?;
        
        // Récupérer le thread principal avant d'encapsuler dans le Mutex si possible, ou après via lock
        let main_thread = process_struct.threads[0].clone();
        
        let process = Arc::new(Mutex::new(process_struct));
        self.processes.push(process);
        
        // Initialiser la table des descripteurs de fichiers
        crate::fs::FD_MANAGER.lock().create_table(pid).unwrap();
        
        // Ajouter le thread au scheduler
        crate::scheduler::SCHEDULER.add_thread(main_thread);
        
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
        
        // Logique de chargement simulée (TODO: VMManager real alloc)
        // ... (parsing segments) ...
        
        // Création process via new (avec dummy entry point, on overwrite après)
        fn dummy_entry() -> ! { loop {} }
        let process = Process::new(pid, name, dummy_entry, ProcessPriority::Normal)?;
        
        // Overwrite du thread context
        let entry_point = elf.header.e_entry;
        {
            let mut thread = process.threads[0].lock();
            thread.context.rip = entry_point;
            // thread.context.rsp = ...;
        }

        let main_thread = process.threads[0].clone();

        let process = Arc::new(Mutex::new(process));
        self.processes.push(process);
        
        // Initialiser la table des descripteurs de fichiers
        crate::fs::FD_MANAGER.lock().create_table(pid).unwrap();
        
        // Ajouter le thread au scheduler
        crate::scheduler::SCHEDULER.add_thread(main_thread);
        
        Ok(pid)
    }

    /// Remplace l'image du processus actuel par un nouvel exécutable (exec)
    pub fn exec_process(&mut self, current_tid: u64, path: &str) -> Result<u64, String> {
        // 1. Lire le fichier ELF
        let content = crate::fs::vfs_read_file(path)
            .map_err(|_| String::from("File not found"))?;
            
        let elf = ElfFile::new(&content).map_err(|e| String::from(e))?;
        if let Err(e) = elf.header.validate() {
            return Err(String::from(e));
        }
        
        // 2. Trouver le process
        let process_arc = self.processes.iter().find(|p| {
            p.lock().threads.iter().any(|t| t.lock().tid == current_tid)
        }).ok_or(String::from("Process not found"))?.clone();
        
        let mut process = process_arc.lock();
        process.name = String::from(path);
        
        // 3. Réinitialiser le thread
        // Simplification: on assume que c'est le seul thread ou on modifie juste celui-ci
        let thread_arc = process.threads.iter()
            .find(|t| t.lock().tid == current_tid)
            .unwrap()
            .clone();
            
        {
            let mut thread = thread_arc.lock();
            thread.context.rip = elf.header.e_entry;
            // TODO: Reset stack, load segments
        }
        
        Ok(0)
    }

    /// Duplique le processus actuel (fork)
    /// Note: Nécessite de connaitre le thread courant.
    /// Pour l'instant, on laisse en TODO car cela nécessite l'accès au Scheduler global qui n'est pas encore visible ici.
    pub fn fork_process(&mut self, current_tid: u64) -> Result<u64, &'static str> {
        // Trouver le process parent via TID (couteux sans map)
        let parent_proc = self.processes.iter().find(|p| {
            p.lock().threads.iter().any(|t| t.lock().tid == current_tid)
        }).ok_or("Parent process not found")?.clone();
        
        let current_thread_arc = parent_proc.lock().threads.iter()
            .find(|t| t.lock().tid == current_tid)
            .unwrap()
            .clone();
            
        let current_thread = current_thread_arc.lock();
        
        let new_pid = self.next_pid;
        self.next_pid += 1;
        
        let new_process_struct = parent_proc.lock().fork(&current_thread, new_pid)?;
        let main_thread = new_process_struct.threads[0].clone();
        
        let new_process = Arc::new(Mutex::new(new_process_struct));
        self.processes.push(new_process);
        
        // Ajouter le thread au scheduler
        crate::scheduler::SCHEDULER.add_thread(main_thread);
        
        Ok(new_pid)
    }
    
    /// Obtient un thread par son TID
    pub fn get_thread_by_tid(&self, tid: u64) -> Option<Arc<Mutex<Thread>>> {
        for p in &self.processes {
            let p_lock = p.lock();
            for t in &p_lock.threads {
                if t.lock().tid == tid {
                    return Some(t.clone());
                }
            }
        }
        None
    }

    /// Obtient la liste des processus
    pub fn processes(&self) -> &Vec<Arc<Mutex<Process>>> {
        &self.processes
    }

    /// Crée un thread dans un processus existant
    pub fn create_thread(&mut self, pid: u64, entry_point: u64) -> Result<u64, &'static str> {
        let process_lock = self.processes.iter()
            .find(|p| p.lock().pid == pid)
            .ok_or("Process not found")?
            .clone();
            
        let mut process = process_lock.lock();
        let thread = process.create_thread(entry_point)?;
        let tid = thread.lock().tid;
        
        crate::scheduler::SCHEDULER.add_thread(thread);
        
        Ok(tid)
    }

    /// Termine un processus
    pub fn terminate_process(&mut self, target_pid: u64, _status: i32) -> Result<(), &'static str> {
        let process_lock = self.processes.iter()
            .find(|p| p.lock().pid == target_pid)
            .ok_or("Process not found")?
            .clone();
            
        let mut process = process_lock.lock();
        process.state = ProcessState::Terminated;
        
        Ok(())
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
    let thread = crate::scheduler::current_thread()?;
    let tid = thread.lock().tid;
    
    let pm = PROCESS_MANAGER.lock();
    for p in &pm.processes {
        if p.lock().threads.iter().any(|t| t.lock().tid == tid) {
            return Some(p.clone());
        }
    }
    None
}

/// Obtient un processus par son PID
pub fn get_process_by_pid(pid: u64) -> Option<Arc<Mutex<Process>>> {
    PROCESS_MANAGER.lock()
        .processes
        .iter()
        .find(|p| p.lock().pid == pid)
        .cloned()
}

/// Obtient un thread par son TID
pub fn get_thread_by_tid(tid: u64) -> Option<Arc<Mutex<Thread>>> {
    PROCESS_MANAGER.lock().get_thread_by_tid(tid)
}
