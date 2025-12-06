use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

pub mod pci;
pub mod ethernet;
pub mod wifi;
pub mod usb;
pub mod bluetooth;
pub mod audio;
pub mod video;
pub mod hotplug;
pub mod events;

pub use pci::*;
pub use ethernet::*;
pub use wifi::*;
pub use usb::*;
pub use bluetooth::*;
pub use audio::*;
pub use video::*;
pub use hotplug::*;
pub use events::*;

use crate::vga_buffer::WRITER;

/// Types de périphériques
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Ethernet,
    Wifi,
    UsbDisk,
    Bluetooth,
    Audio,
    Video,
    Unknown,
}

/// Erreurs de gestion des périphériques
#[derive(Debug, Clone, Copy)]
pub enum DeviceError {
    NotFound,
    AlreadyRegistered,
    InitializationFailed,
    OperationFailed,
    InvalidArgument,
    NotSupported,
    HotplugFailed,
}

/// Trait pour tous les périphériques
pub trait Device: Send + Sync {
    fn name(&self) -> &str;
    fn device_type(&self) -> DeviceType;
    fn init(&mut self) -> Result<(), DeviceError>;
    fn shutdown(&mut self) -> Result<(), DeviceError>;
}

/// Trait pour les énumérateurs de bus
pub trait BusEnumerator: Send + Sync {
    fn name(&self) -> &str;
    fn enumerate(&self) -> Result<Vec<String>, DeviceError>;
}

/// Trait pour les gestionnaires de hotplug
pub trait HotplugHandler: Send + Sync {
    fn on_device_added(&mut self, device_name: &str) -> Result<(), DeviceError>;
    fn on_device_removed(&mut self, device_name: &str) -> Result<(), DeviceError>;
}

/// Gestionnaire de périphériques
pub struct DeviceManager {
    devices: BTreeMap<String, Box<dyn Device>>,
    buses: BTreeMap<String, Box<dyn BusEnumerator>>,
    hotplug_handlers: Vec<Box<dyn HotplugHandler>>,
    initialized: BTreeMap<String, bool>,
}

impl DeviceManager {
    /// Crée un nouveau gestionnaire de périphériques
    pub fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
            buses: BTreeMap::new(),
            hotplug_handlers: Vec::new(),
            initialized: BTreeMap::new(),
        }
    }

    /// Enregistre un périphérique
    pub fn register_device(&mut self, name: &str, device: Box<dyn Device>) -> Result<(), DeviceError> {
        if self.devices.contains_key(name) {
            return Err(DeviceError::AlreadyRegistered);
        }

        self.devices.insert(name.into(), device);
        self.initialized.insert(name.into(), false);
        Ok(())
    }

    /// Enregistre un énumérateur de bus
    pub fn register_bus_enumerator(&mut self, name: &str, enumerator: Box<dyn BusEnumerator>) -> Result<(), DeviceError> {
        if self.buses.contains_key(name) {
            return Err(DeviceError::AlreadyRegistered);
        }

        self.buses.insert(name.into(), enumerator);
        Ok(())
    }

    /// Enregistre un gestionnaire de hotplug
    pub fn register_hotplug_handler(&mut self, handler: Box<dyn HotplugHandler>) {
        self.hotplug_handlers.push(handler);
    }

    /// Initialise un périphérique
    pub fn init_device(&mut self, name: &str) -> Result<(), DeviceError> {
        if !self.devices.contains_key(name) {
            return Err(DeviceError::NotFound);
        }

        if let Some(device) = self.devices.get_mut(name) {
            device.init()?;
            self.initialized.insert(name.into(), true);
            
            WRITER.lock().write_string(&format!(
                "Périphérique initialisé: {}\n",
                name
            ));

            Ok(())
        } else {
            Err(DeviceError::InitializationFailed)
        }
    }

    /// Initialise tous les périphériques
    pub fn init_all_devices(&mut self) -> Result<(), DeviceError> {
        let device_names: Vec<String> = self.devices.keys().cloned().collect();
        
        for name in device_names {
            let _ = self.init_device(&name);
        }

        Ok(())
    }

    /// Détecte tous les périphériques
    pub fn detect_all_devices(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string("Détection des périphériques...\n");

        // Énumérer tous les bus
        let bus_names: Vec<String> = self.buses.keys().cloned().collect();
        
        for bus_name in bus_names {
            if let Some(bus) = self.buses.get(&bus_name) {
                match bus.enumerate() {
                    Ok(devices) => {
                        WRITER.lock().write_string(&format!(
                            "Bus {}: {} périphériques détectés\n",
                            bus_name, devices.len()
                        ));
                    }
                    Err(e) => {
                        WRITER.lock().write_string(&format!(
                            "Erreur énumération bus {}: {:?}\n",
                            bus_name, e
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Obtient un périphérique
    pub fn get_device(&self, name: &str) -> Option<&dyn Device> {
        self.devices.get(name).map(|d| d.as_ref())
    }

    /// Obtient un périphérique mutable
    pub fn get_device_mut(&mut self, name: &str) -> Option<&mut (dyn Device + '_)> {
        if let Some(d) = self.devices.get_mut(name) {
            Some(&mut **d)
        } else {
            None
        }
    }

    /// Vérifie si un périphérique est initialisé
    pub fn is_initialized(&self, name: &str) -> bool {
        self.initialized.get(name).copied().unwrap_or(false)
    }

    /// Liste tous les périphériques
    pub fn list_devices(&self) -> Vec<(String, DeviceType, bool)> {
        self.devices
            .iter()
            .map(|(name, device)| {
                let initialized = self.initialized.get(name).copied().unwrap_or(false);
                (name.clone(), device.device_type(), initialized)
            })
            .collect()
    }

    /// Gère un événement de hotplug
    pub fn handle_hotplug_add(&mut self, device_name: &str) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Périphérique ajouté: {}\n",
            device_name
        ));

        for handler in &mut self.hotplug_handlers {
            let _ = handler.on_device_added(device_name);
        }

        Ok(())
    }

    /// Gère un événement de retrait de hotplug
    pub fn handle_hotplug_remove(&mut self, device_name: &str) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Périphérique retiré: {}\n",
            device_name
        ));

        for handler in &mut self.hotplug_handlers {
            let _ = handler.on_device_removed(device_name);
        }

        Ok(())
    }

    /// Arrête un périphérique
    pub fn shutdown_device(&mut self, name: &str) -> Result<(), DeviceError> {
        if let Some(device) = self.devices.get_mut(name) {
            device.shutdown()?;
            self.initialized.insert(name.into(), false);
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    /// Arrête tous les périphériques
    pub fn shutdown_all_devices(&mut self) -> Result<(), DeviceError> {
        let device_names: Vec<String> = self.devices.keys().cloned().collect();
        
        for name in device_names {
            let _ = self.shutdown_device(&name);
        }

        Ok(())
    }
}

lazy_static! {
    pub static ref DEVICE_MANAGER: Mutex<DeviceManager> = Mutex::new(DeviceManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDevice {
        name: String,
        initialized: bool,
    }

    impl Device for TestDevice {
        fn name(&self) -> &str {
            &self.name
        }

        fn device_type(&self) -> DeviceType {
            DeviceType::Unknown
        }

        fn init(&mut self) -> Result<(), DeviceError> {
            self.initialized = true;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), DeviceError> {
            self.initialized = false;
            Ok(())
        }
    }

    #[test_case]
    fn test_device_manager_creation() {
        let manager = DeviceManager::new();
        assert_eq!(manager.devices.len(), 0);
    }

    #[test_case]
    fn test_register_device() {
        let mut manager = DeviceManager::new();
        let device = Box::new(TestDevice {
            name: "test".into(),
            initialized: false,
        });
        assert!(manager.register_device("test", device).is_ok());
        assert_eq!(manager.devices.len(), 1);
    }

    #[test_case]
    fn test_init_device() {
        let mut manager = DeviceManager::new();
        let device = Box::new(TestDevice {
            name: "test".into(),
            initialized: false,
        });
        manager.register_device("test", device).unwrap();
        assert!(manager.init_device("test").is_ok());
        assert!(manager.is_initialized("test"));
    }

    #[test_case]
    fn test_list_devices() {
        let mut manager = DeviceManager::new();
        let device = Box::new(TestDevice {
            name: "test".into(),
            initialized: false,
        });
        manager.register_device("test", device).unwrap();
        let devices = manager.list_devices();
        assert_eq!(devices.len(), 1);
    }
}
