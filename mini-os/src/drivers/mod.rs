use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

#[cfg(feature = "usb")]
pub mod usb_controller;
#[cfg(feature = "usb")]
pub mod usb_protocol;
#[cfg(feature = "usb")]
pub mod usb_mass_storage;
#[cfg(feature = "usb")]
pub mod usb_hid;

pub mod serial_trait;
pub mod mock_serial;
pub mod disk;

// Ré-exports
pub use serial_trait::SerialPort;
pub use mock_serial::MockSerial;

#[cfg(feature = "bluetooth")]
pub mod bluetooth_hci;
#[cfg(feature = "bluetooth")]
pub mod bluetooth_l2cap;

#[cfg(feature = "usb")]
pub use usb_controller::*;
#[cfg(feature = "usb")]
pub use usb_mass_storage::*;
#[cfg(feature = "usb")]
pub use usb_hid::*;
#[cfg(feature = "bluetooth")]
pub use bluetooth_hci::*;
#[cfg(feature = "bluetooth")]
pub use bluetooth_l2cap::*;

/// Erreurs possibles des drivers
#[derive(Debug, Clone, Copy)]
pub enum DriverError {
    NotFound,
    AlreadyRegistered,
    InitializationFailed,
    OperationFailed,
    InvalidArgument,
    NotSupported,
}

/// Trait que tous les drivers doivent implémenter
pub trait Driver: Send + Sync {
    fn name(&self) -> &str;
    fn init(&mut self) -> Result<(), DriverError>;
    fn handle_interrupt(&mut self, irq: u8);
    fn shutdown(&mut self) -> Result<(), DriverError>;
}

/// Gestionnaire de drivers
pub struct DriverManager {
    drivers: BTreeMap<String, Box<dyn Driver>>,
    initialized: BTreeMap<String, bool>,
}

impl DriverManager {
    /// Crée un nouveau gestionnaire de drivers
    pub fn new() -> Self {
        Self {
            drivers: BTreeMap::new(),
            initialized: BTreeMap::new(),
        }
    }

    /// Enregistre un nouveau driver
    pub fn register_driver(&mut self, name: &str, driver: Box<dyn Driver>) -> Result<(), DriverError> {
        if self.drivers.contains_key(name) {
            return Err(DriverError::AlreadyRegistered);
        }

        self.drivers.insert(name.into(), driver);
        self.initialized.insert(name.into(), false);
        Ok(())
    }

    /// Initialise un driver
    pub fn init_driver(&mut self, name: &str) -> Result<(), DriverError> {
        if !self.drivers.contains_key(name) {
            return Err(DriverError::NotFound);
        }

        if let Some(driver) = self.drivers.get_mut(name) {
            driver.init()?;
            self.initialized.insert(name.into(), true);
            Ok(())
        } else {
            Err(DriverError::InitializationFailed)
        }
    }

    /// Initialise tous les drivers
    pub fn init_all_drivers(&mut self) -> Result<(), DriverError> {
        let driver_names: Vec<String> = self.drivers.keys().cloned().collect();
        
        for name in driver_names {
            if let Err(e) = self.init_driver(&name) {
                log::error!("Erreur initialisation driver {}: {:?}", name, e);
            }
        }

        Ok(())
    }

    /// Obtient un driver par son nom
    pub fn get_driver(&self, name: &str) -> Option<&dyn Driver> {
        self.drivers.get(name).map(|d| d.as_ref())
    }

    /// Obtient un driver mutable par son nom
    pub fn get_driver_mut(&mut self, name: &str) -> Option<&mut (dyn Driver + '_)> {
        if let Some(d) = self.drivers.get_mut(name) {
            Some(&mut **d)
        } else {
            None
        }
    }

    /// Vérifie si un driver est initialisé
    pub fn is_initialized(&self, name: &str) -> bool {
        self.initialized.get(name).copied().unwrap_or(false)
    }

    /// Liste tous les drivers
    pub fn list_drivers(&self) -> Vec<(String, bool)> {
        self.drivers
            .keys()
            .map(|name| {
                let initialized = self.initialized.get(name).copied().unwrap_or(false);
                (name.clone(), initialized)
            })
            .collect()
    }

    /// Gère une interruption pour un driver
    pub fn handle_interrupt(&mut self, driver_name: &str, irq: u8) -> Result<(), DriverError> {
        if let Some(driver) = self.drivers.get_mut(driver_name) {
            driver.handle_interrupt(irq);
            Ok(())
        } else {
            Err(DriverError::NotFound)
        }
    }

    /// Arrête un driver
    pub fn shutdown_driver(&mut self, name: &str) -> Result<(), DriverError> {
        if let Some(driver) = self.drivers.get_mut(name) {
            driver.shutdown()?;
            self.initialized.insert(name.into(), false);
            Ok(())
        } else {
            Err(DriverError::NotFound)
        }
    }

    /// Arrête tous les drivers
    pub fn shutdown_all_drivers(&mut self) -> Result<(), DriverError> {
        let driver_names: Vec<String> = self.drivers.keys().cloned().collect();
        
        for name in driver_names {
            let _ = self.shutdown_driver(&name);
        }

        Ok(())
    }
}

lazy_static! {
    pub static ref DRIVER_MANAGER: Mutex<DriverManager> = Mutex::new(DriverManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDriver {
        name: String,
        initialized: bool,
    }

    impl Driver for TestDriver {
        fn name(&self) -> &str {
            &self.name
        }

        fn init(&mut self) -> Result<(), DriverError> {
            self.initialized = true;
            Ok(())
        }

        fn handle_interrupt(&mut self, _irq: u8) {}

        fn shutdown(&mut self) -> Result<(), DriverError> {
            self.initialized = false;
            Ok(())
        }
    }

    #[test_case]
    fn test_driver_manager_creation() {
        let manager = DriverManager::new();
        assert_eq!(manager.drivers.len(), 0);
    }

    #[test_case]
    fn test_register_driver() {
        let mut manager = DriverManager::new();
        let driver = Box::new(TestDriver {
            name: "test".into(),
            initialized: false,
        });
        assert!(manager.register_driver("test", driver).is_ok());
        assert_eq!(manager.drivers.len(), 1);
    }

    #[test_case]
    fn test_init_driver() {
        let mut manager = DriverManager::new();
        let driver = Box::new(TestDriver {
            name: "test".into(),
            initialized: false,
        });
        manager.register_driver("test", driver).unwrap();
        assert!(manager.init_driver("test").is_ok());
        assert!(manager.is_initialized("test"));
    }
}
