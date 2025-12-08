use crate::vga_buffer::WRITER;
use x86_64::instructions::port::Port;

/// Classe PCI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PciClass {
    Unclassified = 0x00,
    MassStorage = 0x01,
    NetworkController = 0x02,
    DisplayController = 0x03,
    MultimediaDevice = 0x04,
    MemoryController = 0x05,
    Bridge = 0x06,
    SimpleCommunication = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDevice = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBusController = 0x0C,
    WirelessController = 0x0D,
    IntelligentIOController = 0x0E,
    SatelliteCommunication = 0x0F,
    EncryptionController = 0x10,
    SignalProcessing = 0x11,
    ProcessingAccelerator = 0x12,
    NonEssentialInstrumentation = 0x13,
    Miscellaneous = 0xFF,
}

/// Périphérique PCI
#[derive(Debug, Clone)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
}

impl PciDevice {
    /// Crée un nouveau périphérique PCI
    pub fn new(bus: u8, slot: u8, function: u8) -> Self {
        Self {
            bus,
            slot,
            function,
            vendor_id: 0,
            device_id: 0,
            class: 0,
            subclass: 0,
            prog_if: 0,
            revision: 0,
            header_type: 0,
        }
    }

    /// Obtient l'adresse du bus PCI
    pub fn bus_address(&self) -> u32 {
        ((self.bus as u32) << 16) | ((self.slot as u32) << 11) | ((self.function as u32) << 8)
    }

    /// Vérifie si le périphérique est valide
    pub fn is_valid(&self) -> bool {
        self.vendor_id != 0xFFFF
    }
}

/// Énumérateur PCI
pub struct PciEnumerator;

impl PciEnumerator {
    /// Énumère tous les périphériques PCI
    pub fn enumerate() -> alloc::vec::Vec<PciDevice> {
        let mut devices = alloc::vec::Vec::new();

        // Énumérer tous les bus (0-255)
        for bus in 0..256u16 {
            // Énumérer tous les slots (0-31)
            for slot in 0..32u16 {
                // Énumérer toutes les fonctions (0-7)
                for function in 0..8u16 {
                    let mut device = PciDevice::new(bus as u8, slot as u8, function as u8);

                    // Lire la configuration PCI
                    let config = Self::read_config(bus as u8, slot as u8, function as u8, 0);

                    device.vendor_id = (config & 0xFFFF) as u16;
                    device.device_id = ((config >> 16) & 0xFFFF) as u16;

                    if device.is_valid() {
                        // Lire la classe et la sous-classe
                        let class_config = Self::read_config(bus as u8, slot as u8, function as u8, 8);
                        device.class = ((class_config >> 24) & 0xFF) as u8;
                        device.subclass = ((class_config >> 16) & 0xFF) as u8;
                        device.prog_if = ((class_config >> 8) & 0xFF) as u8;
                        device.revision = (class_config & 0xFF) as u8;

                        // Lire le type d'en-tête
                        let header_config = Self::read_config(bus as u8, slot as u8, function as u8, 12);
                        let header_type = ((header_config >> 16) & 0xFF) as u8;
                        device.header_type = header_type;

                        devices.push(device);

                        // Si ce n'est pas une fonction multi-fonction, arrêter
                        if function == 0 && (header_type & 0x80) == 0 {
                            break;
                        }
                    }
                }
            }
        }

        devices
    }

    /// Lit une configuration PCI (32 bits)
    pub fn read_config(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
        let address = 0x80000000
            | ((bus as u32) << 16)
            | ((slot as u32) << 11)
            | ((function as u32) << 8)
            | ((offset as u32) & 0xFC);
        
        unsafe {
            let mut port_address = Port::<u32>::new(0xCF8);
            let mut port_data = Port::<u32>::new(0xCFC);
            
            port_address.write(address);
            port_data.read()
        }
    }

    /// Écrit une configuration PCI (32 bits)
    pub fn write_config(bus: u8, slot: u8, function: u8, offset: u8, value: u32) {
        let address = 0x80000000
            | ((bus as u32) << 16)
            | ((slot as u32) << 11)
            | ((function as u32) << 8)
            | ((offset as u32) & 0xFC);
            
        unsafe {
            let mut port_address = Port::<u32>::new(0xCF8);
            let mut port_data = Port::<u32>::new(0xCFC);
            
            port_address.write(address);
            port_data.write(value);
        }
    }

    /// Affiche tous les périphériques PCI
    pub fn print_devices() {
        let devices = Self::enumerate();

        WRITER.lock().write_string("Périphériques PCI détectés:\n");
        WRITER.lock().write_string("Bus:Slot.Func | Vendor:Device | Class:Subclass | Revision\n");
        WRITER.lock().write_string("─────────────────────────────────────────────────────────\n");

        for device in devices {
            WRITER.lock().write_string(&format!(
                "{:02X}:{:02X}.{} | {:04X}:{:04X} | {:02X}:{:02X} | {:02X}\n",
                device.bus,
                device.slot,
                device.function,
                device.vendor_id,
                device.device_id,
                device.class,
                device.subclass,
                device.revision
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_pci_device_creation() {
        let device = PciDevice::new(0, 0, 0);
        assert_eq!(device.bus, 0);
        assert_eq!(device.slot, 0);
        assert_eq!(device.function, 0);
    }

    #[test_case]
    fn test_pci_device_bus_address() {
        let device = PciDevice::new(1, 2, 3);
        let addr = device.bus_address();
        assert!(addr > 0);
    }

    #[test_case]
    fn test_pci_device_invalid() {
        let device = PciDevice::new(0, 0, 0);
        assert!(!device.is_valid());
    }

    #[test_case]
    fn test_pci_device_valid() {
        let mut device = PciDevice::new(0, 0, 0);
        device.vendor_id = 0x8086;
        assert!(device.is_valid());
    }
}
