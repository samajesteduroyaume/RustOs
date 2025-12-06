use x86_64::instructions::port::Port;
use crate::vga_buffer::WRITER;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
}

impl PciDevice {
    pub fn new(bus: u8, device: u8, function: u8) -> Option<Self> {
        let vendor_id = pci_config_read_u16(bus, device, function, 0x00);
        if vendor_id == 0xFFFF {
            return None;
        }
        
        let device_id = pci_config_read_u16(bus, device, function, 0x02);
        
        Some(Self {
            bus,
            device,
            function,
            vendor_id,
            device_id,
        })
    }
    
    pub fn class_code(&self) -> (u8, u8, u8) {
        let class_rev = pci_config_read_u32(self.bus, self.device, self.function, 0x08);
        (
            ((class_rev >> 24) & 0xFF) as u8,
            ((class_rev >> 16) & 0xFF) as u8,
            ((class_rev >> 8) & 0xFF) as u8,
        )
    }
}

pub fn scan_pci() {
    WRITER.lock().write_string("Scanning PCI devices...\n");
    
    for bus in 0..=255 {
        for device in 0..32 {
            // Check for multi-function devices
            if device == 0 {
                let header_type = pci_config_read_u8(bus, device, 0, 0x0E);
                if (header_type & 0x80) == 0 {
                    // Single function device
                    if let Some(dev) = PciDevice::new(bus, device, 0) {
                        print_pci_device(&dev);
                    }
                    continue;
                }
            }
            
            // Check all functions
            for function in 0..8 {
                if let Some(dev) = PciDevice::new(bus, device, function) {
                    print_pci_device(&dev);
                }
            }
        }
    }
}

fn print_pci_device(dev: &PciDevice) {
    let (class, subclass, _) = dev.class_code();
    WRITER.lock().write_string(&format!(
        "PCI: {:02X}:{:02X}.{} {:04X}:{:04X} Class {:02X}:{:02X}\n",
        dev.bus, dev.device, dev.function,
        dev.vendor_id, dev.device_id,
        class, subclass
    ));
}

fn pci_config_read_u8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let address = 0x80000000
        | ((bus as u32) << 16)
        | ((device as u32 & 0x1F) << 11)
        | ((function as u32 & 0x07) << 8)
        | (offset as u32 & 0xFC);
    
    let mut port = Port::<u32>::new(0xCF8);
    unsafe { port.write(address); }
    
    let mut port = Port::<u32>::new(0xCFC);
    let value = unsafe { port.read() };
    
    ((value >> ((offset & 0x03) * 8)) & 0xFF) as u8
}

fn pci_config_read_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let address = 0x80000000
        | ((bus as u32) << 16)
        | ((device as u32 & 0x1F) << 11)
        | ((function as u32 & 0x07) << 8)
        | (offset as u32 & 0xFC);
    
    let mut port = Port::<u32>::new(0xCF8);
    unsafe { port.write(address); }
    
    let mut port = Port::<u32>::new(0xCFC);
    let value = unsafe { port.read() };
    
    if (offset & 0x02) == 0 {
        (value & 0xFFFF) as u16
    } else {
        ((value >> 16) & 0xFFFF) as u16
    }
}

fn pci_config_read_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = 0x80000000
        | ((bus as u32) << 16)
        | ((device as u32 & 0x1F) << 11)
        | ((function as u32 & 0x07) << 8)
        | (offset as u32 & 0xFC);
    
    let mut port = Port::<u32>::new(0xCF8);
    unsafe { port.write(address); }
    
    let mut port = Port::<u32>::new(0xCFC);
    unsafe { port.read() }
}
