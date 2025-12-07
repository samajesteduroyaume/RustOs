/// Module NVMe Driver
/// 
/// Implémente le support pour les disques NVMe (Non-Volatile Memory Express)

use alloc::vec::Vec;
use spin::Mutex;

/// Taille d'un bloc NVMe (512 bytes standard)
pub const NVME_BLOCK_SIZE: usize = 512;

/// Nombre maximum de namespaces
pub const MAX_NAMESPACES: usize = 16;

/// Registres NVMe (offsets dans la BAR)
#[repr(C)]
pub struct NVMeRegisters {
    /// Capabilities
    pub cap: u64,
    /// Version
    pub vs: u32,
    /// Interrupt Mask Set
    pub intms: u32,
    /// Interrupt Mask Clear
    pub intmc: u32,
    /// Controller Configuration
    pub cc: u32,
    /// Reserved
    _reserved: u32,
    /// Controller Status
    pub csts: u32,
    /// NVM Subsystem Reset
    pub nssr: u32,
    /// Admin Queue Attributes
    pub aqa: u32,
    /// Admin Submission Queue Base Address
    pub asq: u64,
    /// Admin Completion Queue Base Address
    pub acq: u64,
}

/// Commande NVMe
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NVMeCommand {
    /// Opcode
    pub opcode: u8,
    /// Flags
    pub flags: u8,
    /// Command ID
    pub cid: u16,
    /// Namespace ID
    pub nsid: u32,
    /// Reserved
    _reserved: [u32; 2],
    /// Metadata pointer
    pub mptr: u64,
    /// Data pointer (PRP1)
    pub prp1: u64,
    /// Data pointer (PRP2)
    pub prp2: u64,
    /// Command-specific data
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

impl NVMeCommand {
    /// Crée une commande vide
    pub fn new() -> Self {
        Self {
            opcode: 0,
            flags: 0,
            cid: 0,
            nsid: 0,
            _reserved: [0; 2],
            mptr: 0,
            prp1: 0,
            prp2: 0,
            cdw10: 0,
            cdw11: 0,
            cdw12: 0,
            cdw13: 0,
            cdw14: 0,
            cdw15: 0,
        }
    }
    
    /// Crée une commande READ
    pub fn read(nsid: u32, lba: u64, block_count: u16, prp1: u64) -> Self {
        let mut cmd = Self::new();
        cmd.opcode = 0x02; // NVM Read
        cmd.nsid = nsid;
        cmd.prp1 = prp1;
        cmd.cdw10 = (lba & 0xFFFFFFFF) as u32;
        cmd.cdw11 = (lba >> 32) as u32;
        cmd.cdw12 = block_count as u32;
        cmd
    }
    
    /// Crée une commande WRITE
    pub fn write(nsid: u32, lba: u64, block_count: u16, prp1: u64) -> Self {
        let mut cmd = Self::new();
        cmd.opcode = 0x01; // NVM Write
        cmd.nsid = nsid;
        cmd.prp1 = prp1;
        cmd.cdw10 = (lba & 0xFFFFFFFF) as u32;
        cmd.cdw11 = (lba >> 32) as u32;
        cmd.cdw12 = block_count as u32;
        cmd
    }
}

/// Completion Queue Entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NVMeCompletion {
    /// Command-specific result
    pub result: u32,
    /// Reserved
    _reserved: u32,
    /// Submission Queue Head Pointer
    pub sq_head: u16,
    /// Submission Queue ID
    pub sq_id: u16,
    /// Command ID
    pub cid: u16,
    /// Phase tag and status
    pub status: u16,
}

/// Namespace NVMe
#[derive(Debug, Clone)]
pub struct NVMeNamespace {
    /// ID du namespace
    pub id: u32,
    /// Taille en blocs
    pub size_blocks: u64,
    /// Taille d'un bloc
    pub block_size: usize,
    /// Actif
    pub active: bool,
}

impl NVMeNamespace {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            size_blocks: 0,
            block_size: NVME_BLOCK_SIZE,
            active: false,
        }
    }
}

/// Contrôleur NVMe
pub struct NVMeController {
    /// Namespaces
    namespaces: Vec<NVMeNamespace>,
    /// Nombre de commandes envoyées
    commands_sent: usize,
    /// Nombre de commandes complétées
    commands_completed: usize,
    /// Initialisé
    initialized: bool,
}

impl NVMeController {
    /// Crée un nouveau contrôleur
    pub const fn new() -> Self {
        Self {
            namespaces: Vec::new(),
            commands_sent: 0,
            commands_completed: 0,
            initialized: false,
        }
    }
    
    /// Initialise le contrôleur
    pub fn init(&mut self) -> Result<(), NVMeError> {
        // TODO: Initialisation réelle du hardware
        // 1. Mapper les registres BAR
        // 2. Reset du contrôleur
        // 3. Configurer Admin Queue
        // 4. Identifier les namespaces
        
        // Pour l'instant, créer un namespace de test
        let mut ns = NVMeNamespace::new(1);
        ns.size_blocks = 1024 * 1024; // 512 MB
        ns.block_size = NVME_BLOCK_SIZE;
        ns.active = true;
        
        self.namespaces.push(ns);
        self.initialized = true;
        
        Ok(())
    }
    
    /// Lit des blocs
    pub fn read_blocks(&mut self, nsid: u32, lba: u64, count: u16, buffer: &mut [u8]) -> Result<usize, NVMeError> {
        if !self.initialized {
            return Err(NVMeError::NotInitialized);
        }
        
        // Vérifier le namespace
        let ns = self.namespaces.iter()
            .find(|n| n.id == nsid && n.active)
            .ok_or(NVMeError::InvalidNamespace)?;
        
        // Vérifier la taille du buffer
        let required_size = count as usize * ns.block_size;
        if buffer.len() < required_size {
            return Err(NVMeError::BufferTooSmall);
        }
        
        // TODO: Créer et soumettre la commande NVMe
        // Pour l'instant, simuler la lecture
        self.commands_sent += 1;
        self.commands_completed += 1;
        
        Ok(required_size)
    }
    
    /// Écrit des blocs
    pub fn write_blocks(&mut self, nsid: u32, lba: u64, count: u16, buffer: &[u8]) -> Result<usize, NVMeError> {
        if !self.initialized {
            return Err(NVMeError::NotInitialized);
        }
        
        let ns = self.namespaces.iter()
            .find(|n| n.id == nsid && n.active)
            .ok_or(NVMeError::InvalidNamespace)?;
        
        let required_size = count as usize * ns.block_size;
        if buffer.len() < required_size {
            return Err(NVMeError::BufferTooSmall);
        }
        
        // TODO: Créer et soumettre la commande NVMe
        self.commands_sent += 1;
        self.commands_completed += 1;
        
        Ok(required_size)
    }
    
    /// Retourne les namespaces
    pub fn get_namespaces(&self) -> &[NVMeNamespace] {
        &self.namespaces
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> NVMeStats {
        NVMeStats {
            namespaces: self.namespaces.len(),
            commands_sent: self.commands_sent,
            commands_completed: self.commands_completed,
            initialized: self.initialized,
        }
    }
}

/// Erreurs NVMe
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NVMeError {
    NotInitialized,
    InvalidNamespace,
    BufferTooSmall,
    CommandFailed,
    Timeout,
}

/// Statistiques NVMe
#[derive(Debug, Clone)]
pub struct NVMeStats {
    pub namespaces: usize,
    pub commands_sent: usize,
    pub commands_completed: usize,
    pub initialized: bool,
}

/// Instance globale du contrôleur NVMe
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NVME_CONTROLLER: Mutex<NVMeController> = Mutex::new(NVMeController::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_nvme_controller_creation() {
        let controller = NVMeController::new();
        assert!(!controller.initialized);
        assert_eq!(controller.commands_sent, 0);
    }
    
    #[test_case]
    fn test_nvme_init() {
        let mut controller = NVMeController::new();
        assert!(controller.init().is_ok());
        assert!(controller.initialized);
        assert_eq!(controller.namespaces.len(), 1);
    }
    
    #[test_case]
    fn test_nvme_read() {
        let mut controller = NVMeController::new();
        controller.init().unwrap();
        
        let mut buffer = vec![0u8; 4096];
        let result = controller.read_blocks(1, 0, 8, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4096);
    }
    
    #[test_case]
    fn test_nvme_command_creation() {
        let cmd = NVMeCommand::read(1, 100, 8, 0x1000);
        assert_eq!(cmd.opcode, 0x02);
        assert_eq!(cmd.nsid, 1);
        assert_eq!(cmd.cdw10, 100);
    }
}
