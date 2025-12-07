use alloc::sync::{Arc, Weak};
use spin::Mutex;
use x86_64::PhysAddr;
use crate::process::{Process, ProcessPriority}; // On réutilisera ProcessPriority ou on le bougera après

/// Identifiant de thread
pub type ThreadId = u64;

/// État d'un thread
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Contexte d'exécution d'un thread
#[derive(Debug, Clone)]
pub struct ThreadContext {
    pub rsp: u64,
    pub rip: u64,
    pub registers: [u64; 16],
    pub rflags: u64,
    pub cr3: u64, // On garde CR3 ici pour switcher rapidement
    pub privilege_level: u8,
}

impl Default for ThreadContext {
    fn default() -> Self {
        Self {
            rsp: 0,
            rip: 0,
            registers: [0; 16],
            rflags: 0x202, // Interrupts enabled by default
            cr3: 0,
            privilege_level: 0,
        }
    }
}

/// Structure représentant un Thread
#[derive(Debug)]
pub struct Thread {
    pub tid: ThreadId,
    pub pid: u64, // Parent Process ID
    pub name: alloc::string::String,
    pub state: ThreadState,
    pub context: ThreadContext,
    pub priority: ProcessPriority, // On utilise la même enum pour l'instant
    pub kstack: Option<PhysAddr>,
    pub vruntime: u64, // Pour CFS
    pub cpu_time: u64,
    pub last_scheduled: u64,
    
    // Le thread peut avoir besoin d'accéder à son processus parent (ex: files, memory)
    // Pour éviter les cycles de référence bloquants (Arc<Process> <-> Arc<Thread>),
    // on pourrait utiliser Weak. Mais pour l'instant, on stocke juste le PID.
    // L'accès au Process se fera via le ProcessManager avec le PID.
}

impl Thread {
    pub fn new(tid: ThreadId, pid: u64, name: &str, priority: ProcessPriority, cr3: u64) -> Self {
        let mut context = ThreadContext::default();
        context.cr3 = cr3;
        
        Self {
            tid,
            pid,
            name: alloc::string::String::from(name),
            state: ThreadState::Ready,
            context,
            priority,
            kstack: None,
            vruntime: 0,
            cpu_time: 0,
            last_scheduled: 0,
        }
    }

    pub fn set_priority(&mut self, priority: ProcessPriority) {
        self.priority = priority;
    }

    pub fn update_vruntime(&mut self, delta_time: u64) {
        let weight = self.priority.weight();
        self.vruntime += (delta_time * 1024) / weight;
        self.cpu_time += delta_time;
    }

    /// Sauvegarde le contexte (simplifié, asm fait le gros du travail normalement)
    pub fn save_context(&mut self) {
        // TODO: Implémentation si nécessaire de logique pré/post switch
    }

    /// Restaure le contexte
    pub fn restore_context(&self) {
        unsafe {
            // La restauration complète se fait en ASM, ici on pourrait juste préparer des choses
            // Si on change de CR3, on le charge
            use x86_64::registers::control::Cr3;
            use x86_64::structures::paging::PhysFrame;
            use x86_64::PhysAddr;
            
            let current_cr3 = Cr3::read().0.start_address().as_u64();
            if self.context.cr3 != 0 && self.context.cr3 != current_cr3 {
                let frame = PhysFrame::containing_address(PhysAddr::new(self.context.cr3));
                // Note: On préserve les flags PCID si on ne les change pas (ici flags vides)
                Cr3::write(frame, x86_64::registers::control::Cr3Flags::empty());
            }
            
            core::arch::asm!(
                "mov rsp, {rsp}",
                // "mov rip, {rip}", // RIP est restauré par ret ou iret
                // "jmp {rip}", // Si on jump directement, mais souvent on 'ret' d'une interruption
                
                // Pour restaurer RIP dans un contexte de switch volontaire (coopératif) ou interruption simulée :
                // On suppose que la stack contient l'adresse de retour si on fait 'ret'
                // Ou alors on push RIP et on ret.
                
                // Ici c'est un placeholder, le vrai switch est dans `scheduler.asm` ou similaire
                // Ou alors on utilise le switch contextuel complet.
                
                rsp = in(reg) self.context.rsp,
                // rip = in(reg) self.context.rip,
            );
        }
    }
}
