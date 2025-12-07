/// USB Controller - Détection et gestion des contrôleurs USB
/// 
/// Supporte UHCI, OHCI, EHCI, et XHCI

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::vga_buffer::WRITER;

/// Types de contrôleurs USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbControllerType {
    UHCI,   // USB 1.1 (Intel)
    OHCI,   // USB 1.1 (Compaq, Microsoft, National Semiconductor)
    EHCI,   // USB 2.0
    XHCI,   // USB 3.0/3.1/3.2
}

impl UsbControllerType {
    pub fn name(&self) -> &str {
        match self {
            UsbControllerType::UHCI => "UHCI (USB 1.1)",
            UsbControllerType::OHCI => "OHCI (USB 1.1)",
            UsbControllerType::EHCI => "EHCI (USB 2.0)",
            UsbControllerType::XHCI => "XHCI (USB 3.x)",
        }
    }

    pub fn max_speed_mbps(&self) -> u32 {
        match self {
            UsbControllerType::UHCI | UsbControllerType::OHCI => 12,
            UsbControllerType::EHCI => 480,
            UsbControllerType::XHCI => 10000,
        }
    }
}

/// Contrôleur USB
#[derive(Debug)]
pub struct UsbController {
    /// Type de contrôleur
    pub controller_type: UsbControllerType,
    
    /// Adresse PCI (bus, device, function)
    pub pci_address: (u8, u8, u8),
    
    /// Adresse de base des registres
    pub base_address: u64,
    
    /// IRQ
    pub irq: u8,
    
    /// Nombre de ports
    pub num_ports: u8,
    
    /// Contrôleur initialisé
    pub initialized: bool,
}

impl UsbController {
    /// Crée un nouveau contrôleur USB
    pub fn new(
        controller_type: UsbControllerType,
        pci_address: (u8, u8, u8),
        base_address: u64,
        irq: u8,
    ) -> Self {
        Self {
            controller_type,
            pci_address,
            base_address,
            irq,
            num_ports: 0,
            initialized: false,
        }
    }

    /// Initialise le contrôleur
    pub fn init(&mut self) -> Result<(), UsbError> {
        WRITER.lock().write_string(&format!(
            "Initialisation contrôleur USB {} à {:04X}:{:02X}:{:02X}\\n",
            self.controller_type.name(),
            self.pci_address.0,
            self.pci_address.1,
            self.pci_address.2
        ));

        match self.controller_type {
            UsbControllerType::UHCI => self.init_uhci(),
            UsbControllerType::OHCI => self.init_ohci(),
            UsbControllerType::EHCI => self.init_ehci(),
            UsbControllerType::XHCI => self.init_xhci(),
        }
    }

    /// Initialise un contrôleur UHCI
    fn init_uhci(&mut self) -> Result<(), UsbError> {
        // TODO: Implémenter l'initialisation UHCI
        // 1. Réinitialiser le contrôleur
        // 2. Configurer la frame list
        // 3. Activer le contrôleur
        self.num_ports = 2; // UHCI a généralement 2 ports
        self.initialized = true;
        Ok(())
    }

    /// Initialise un contrôleur OHCI
    fn init_ohci(&mut self) -> Result<(), UsbError> {
        // TODO: Implémenter l'initialisation OHCI
        // 1. Réinitialiser le contrôleur
        // 2. Configurer HCCA (Host Controller Communications Area)
        // 3. Activer le contrôleur
        self.num_ports = 4; // OHCI a généralement 4 ports
        self.initialized = true;
        Ok(())
    }

    /// Initialise un contrôleur EHCI
    fn init_ehci(&mut self) -> Result<(), UsbError> {
        // TODO: Implémenter l'initialisation EHCI
        // 1. Vérifier la version EHCI
        // 2. Réinitialiser le contrôleur
        // 3. Configurer la periodic frame list
        // 4. Configurer l'async schedule
        // 5. Activer le contrôleur
        self.num_ports = 6; // EHCI a généralement 6 ports
        self.initialized = true;
        Ok(())
    }

    /// Initialise un contrôleur XHCI
    fn init_xhci(&mut self) -> Result<(), UsbError> {
        // TODO: Implémenter l'initialisation XHCI
        // 1. Arrêter le contrôleur
        // 2. Réinitialiser le contrôleur
        // 3. Configurer le Device Context Base Address Array (DCBAA)
        // 4. Configurer les Event Ring Segment Table (ERST)
        // 5. Configurer les Command Ring
        // 6. Démarrer le contrôleur
        self.num_ports = 10; // XHCI peut avoir jusqu'à 15 ports
        self.initialized = true;
        Ok(())
    }

    /// Réinitialise le contrôleur
    pub fn reset(&mut self) -> Result<(), UsbError> {
        WRITER.lock().write_string("Réinitialisation contrôleur USB\\n");
        self.initialized = false;
        self.init()
    }

    /// Lit un registre du contrôleur
    pub fn read_register(&self, offset: u32) -> u32 {
        // TODO: Implémenter la lecture de registre
        // unsafe { core::ptr::read_volatile((self.base_address + offset as u64) as *const u32) }
        0
    }

    /// Écrit dans un registre du contrôleur
    pub fn write_register(&mut self, offset: u32, value: u32) {
        // TODO: Implémenter l'écriture de registre
        // unsafe { core::ptr::write_volatile((self.base_address + offset as u64) as *mut u32, value) }
    }

    /// Obtient le statut d'un port
    pub fn get_port_status(&self, port: u8) -> Result<PortStatus, UsbError> {
        if port >= self.num_ports {
            return Err(UsbError::InvalidPort);
        }

        // TODO: Lire le statut réel du port
        Ok(PortStatus {
            connected: false,
            enabled: false,
            suspended: false,
            over_current: false,
            reset: false,
            power: true,
            low_speed: false,
            high_speed: false,
        })
    }

    /// Réinitialise un port
    pub fn reset_port(&mut self, port: u8) -> Result<(), UsbError> {
        if port >= self.num_ports {
            return Err(UsbError::InvalidPort);
        }

        WRITER.lock().write_string(&format!("Réinitialisation port USB {}\\n", port));
        
        // TODO: Implémenter la réinitialisation du port
        // 1. Activer le signal de reset
        // 2. Attendre 10-20ms
        // 3. Désactiver le signal de reset
        // 4. Attendre que le port soit activé

        Ok(())
    }
}

/// Statut d'un port USB
#[derive(Debug, Clone, Copy)]
pub struct PortStatus {
    pub connected: bool,
    pub enabled: bool,
    pub suspended: bool,
    pub over_current: bool,
    pub reset: bool,
    pub power: bool,
    pub low_speed: bool,
    pub high_speed: bool,
}

/// Erreurs USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    NotFound,
    InvalidPort,
    InitializationFailed,
    TransferFailed,
    Timeout,
    Stall,
    NotSupported,
}

/// Gestionnaire de contrôleurs USB
pub struct UsbControllerManager {
    controllers: Vec<UsbController>,
}

impl UsbControllerManager {
    /// Crée un nouveau gestionnaire
    pub fn new() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }

    /// Détecte les contrôleurs USB via PCI
    pub fn detect_controllers(&mut self) -> Result<usize, UsbError> {
        WRITER.lock().write_string("Détection des contrôleurs USB...\\n");

        // TODO: Scanner le bus PCI pour trouver les contrôleurs USB
        // Class Code: 0x0C (Serial Bus Controller)
        // Subclass: 0x03 (USB Controller)
        // Programming Interface:
        //   0x00 = UHCI
        //   0x10 = OHCI
        //   0x20 = EHCI
        //   0x30 = XHCI

        // Exemple de contrôleurs détectés (simulation)
        let ehci = UsbController::new(
            UsbControllerType::EHCI,
            (0, 29, 7),
            0xFEBFF000,
            23,
        );
        self.controllers.push(ehci);

        WRITER.lock().write_string(&format!(
            "{} contrôleur(s) USB détecté(s)\\n",
            self.controllers.len()
        ));

        Ok(self.controllers.len())
    }

    /// Initialise tous les contrôleurs
    pub fn init_all(&mut self) -> Result<(), UsbError> {
        for controller in &mut self.controllers {
            controller.init()?;
        }
        Ok(())
    }

    /// Obtient un contrôleur par index
    pub fn get_controller(&self, index: usize) -> Option<&UsbController> {
        self.controllers.get(index)
    }

    /// Obtient un contrôleur mutable par index
    pub fn get_controller_mut(&mut self, index: usize) -> Option<&mut UsbController> {
        self.controllers.get_mut(index)
    }

    /// Nombre de contrôleurs
    pub fn controller_count(&self) -> usize {
        self.controllers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_controller_creation() {
        let controller = UsbController::new(
            UsbControllerType::EHCI,
            (0, 29, 7),
            0xFEBFF000,
            23,
        );
        assert_eq!(controller.controller_type, UsbControllerType::EHCI);
        assert!(!controller.initialized);
    }

    #[test_case]
    fn test_controller_type_name() {
        assert_eq!(UsbControllerType::UHCI.name(), "UHCI (USB 1.1)");
        assert_eq!(UsbControllerType::EHCI.name(), "EHCI (USB 2.0)");
        assert_eq!(UsbControllerType::XHCI.name(), "XHCI (USB 3.x)");
    }

    #[test_case]
    fn test_controller_manager() {
        let mut manager = UsbControllerManager::new();
        assert_eq!(manager.controller_count(), 0);
    }
}
