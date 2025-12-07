/// Module pour gérer le Mode Utilisateur (Ring 3)
/// 
/// Ce module fournit les structures et fonctions nécessaires pour :
/// - Configurer les segments Ring 3
/// - Gérer le changement de contexte Ring 0 → Ring 3
/// - Gérer les appels système (syscalls) depuis Ring 3

use x86_64::VirtAddr;
use lazy_static::lazy_static;

/// Taille de la pile noyau (16 KB)
const KERNEL_STACK_SIZE: usize = 16 * 1024;

/// Taille de la pile utilisateur (64 KB)
const USER_STACK_SIZE: usize = 64 * 1024;

/// Sélecteurs de segment pour Ring 3
pub struct SegmentSelectors {
    /// Sélecteur de code noyau (Ring 0)
    pub kernel_code: u16,
    /// Sélecteur de données noyau (Ring 0)
    pub kernel_data: u16,
    /// Sélecteur de code utilisateur (Ring 3)
    pub user_code: u16,
    /// Sélecteur de données utilisateur (Ring 3)
    pub user_data: u16,
}

impl SegmentSelectors {
    /// Crée les sélecteurs de segment par défaut
    pub fn new() -> Self {
        Self {
            kernel_code: 0x08,  // Index 1 << 3
            kernel_data: 0x10,  // Index 2 << 3
            user_code: 0x18 | 3,   // Index 3 << 3 | RPL 3
            user_data: 0x20 | 3,   // Index 4 << 3 | RPL 3
        }
    }
}

/// Structure pour gérer Ring 3
pub struct Ring3Manager {
    selectors: SegmentSelectors,
}

impl Ring3Manager {
    /// Crée un nouveau gestionnaire Ring 3
    pub fn new() -> Self {
        Self {
            selectors: SegmentSelectors::new(),
        }
    }
    
    /// Charge les sélecteurs de segment
    pub fn load(&self) {
        // Les sélecteurs sont déjà configurés dans la GDT du bootloader
        // Cette fonction est un placeholder pour la compatibilité
    }
    
    /// Obtient les sélecteurs de segment
    pub fn selectors(&self) -> &SegmentSelectors {
        &self.selectors
    }
}

/// Structure pour représenter le contexte d'exécution Ring 3
#[derive(Debug, Clone)]
pub struct Ring3Context {
    /// Pointeur de pile utilisateur (RSP)
    pub user_rsp: u64,
    /// Pointeur d'instruction (RIP)
    pub rip: u64,
    /// Registres généraux
    pub registers: [u64; 16],
    /// Registre RFLAGS
    pub rflags: u64,
}

impl Ring3Context {
    /// Crée un nouveau contexte Ring 3
    pub fn new(entry_point: u64, user_stack: u64) -> Self {
        Self {
            user_rsp: user_stack,
            rip: entry_point,
            registers: [0; 16],
            rflags: 0x202, // IF=1 (interruptions activées), autres flags par défaut
        }
    }
}

/// Bascule vers Ring 3 avec le contexte spécifié
/// 
/// Cette fonction change le niveau de privilège du CPU de Ring 0 (noyau) à Ring 3 (utilisateur)
/// et saute à l'adresse d'entrée spécifiée.
pub unsafe fn switch_to_ring3(context: &Ring3Context, user_code_selector: u16, user_data_selector: u16) -> ! {
    // Préparer les registres pour le changement de contexte
    // On utilise IRET pour basculer vers Ring 3
    
    // Structure de la pile pour IRET :
    // [RSP + 32] = SS (sélecteur de segment de pile)
    // [RSP + 24] = RSP (pointeur de pile)
    // [RSP + 16] = RFLAGS
    // [RSP + 8]  = CS (sélecteur de segment de code)
    // [RSP + 0]  = RIP (pointeur d'instruction)
    
    core::arch::asm!(
        // Charger les registres généraux
        "mov rax, {rax}",
        "mov rbx, {rbx}",
        "mov rcx, {rcx}",
        "mov rdx, {rdx}",
        "mov rsi, {rsi}",
        "mov rdi, {rdi}",
        
        // Charger les sélecteurs de données utilisateur
        "mov ds, {user_data_selector:x}",
        "mov es, {user_data_selector:x}",
        "mov fs, {user_data_selector:x}",
        "mov gs, {user_data_selector:x}",
        
        // Préparer la pile pour IRET
        // SS (sélecteur de pile utilisateur)
        "push {user_data_selector}",
        // RSP (pointeur de pile utilisateur)
        "push {user_rsp}",
        // RFLAGS
        "push {rflags}",
        // CS (sélecteur de code utilisateur)
        "push {user_code_selector}",
        // RIP (point d'entrée)
        "push {rip}",
        
        // Exécuter IRET pour basculer vers Ring 3
        "iret",
        
        rax = in(reg) context.registers[0],
        rbx = in(reg) context.registers[1],
        rcx = in(reg) context.registers[2],
        rdx = in(reg) context.registers[3],
        rsi = in(reg) context.registers[4],
        rdi = in(reg) context.registers[5],
        user_data_selector = in(reg) user_data_selector as u64,
        user_rsp = in(reg) context.user_rsp,
        rflags = in(reg) context.rflags,
        user_code_selector = in(reg) user_code_selector as u64,
        rip = in(reg) context.rip,
        options(noreturn)
    );
}

/// Bascule de Ring 3 vers Ring 0 (utilisé par les syscalls)
/// 
/// Cette fonction est appelée lors d'un syscall pour revenir en Ring 0
pub unsafe fn switch_to_ring0() {
    // Cette fonction est généralement appelée via l'instruction SYSCALL
    // qui bascule automatiquement vers Ring 0
    // Ici, on peut ajouter du code de nettoyage si nécessaire
}

lazy_static! {
    /// Gestionnaire global Ring 3
    pub static ref RING3_MANAGER: Ring3Manager = Ring3Manager::new();
}
