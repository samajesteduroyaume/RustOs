use super::{Device, DeviceType, DeviceError};
use alloc::vec::Vec;
use alloc::string::String;
use crate::vga_buffer::WRITER;

/// Vitesses USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    LowSpeed,      // 1.5 Mbps
    FullSpeed,     // 12 Mbps
    HighSpeed,     // 480 Mbps
    SuperSpeed,    // 5 Gbps
    SuperSpeedPlus, // 10 Gbps
}

impl UsbSpeed {
    pub fn to_mbps(&self) -> u32 {
        match self {
            UsbSpeed::LowSpeed => 1,
            UsbSpeed::FullSpeed => 12,
            UsbSpeed::HighSpeed => 480,
            UsbSpeed::SuperSpeed => 5000,
            UsbSpeed::SuperSpeedPlus => 10000,
        }
    }
}

/// Classe USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbClass {
    Audio = 0x01,
    CommunicationsCDC = 0x02,
    HID = 0x03,
    Physical = 0x05,
    Image = 0x06,
    Printer = 0x07,
    MassStorage = 0x08,
    Hub = 0x09,
    CDCData = 0x0A,
    SmartCard = 0x0B,
    ContentSecurity = 0x0D,
    Video = 0x0E,
    PersonalHealthcare = 0x0F,
    AudioVideo = 0x10,
    Billboard = 0x11,
    TypeCBridge = 0x12,
    DiagnosticDevice = 0xDC,
    WirelessController = 0xE0,
    Miscellaneous = 0xEF,
    ApplicationSpecific = 0xFE,
    VendorSpecific = 0xFF,
}

/// Périphérique USB
#[derive(Debug, Clone)]
pub struct UsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub speed: UsbSpeed,
    pub bus_number: u8,
    pub device_number: u8,
    pub port_number: u8,
    pub class: UsbClass,
    pub subclass: u8,
    pub protocol: u8,
}

impl UsbDevice {
    pub fn new(name: &str, vendor_id: u16, product_id: u16, speed: UsbSpeed) -> Self {
        Self {
            vendor_id,
            product_id,
            name: name.into(),
            speed,
            bus_number: 0,
            device_number: 0,
            port_number: 0,
            class: UsbClass::VendorSpecific,
            subclass: 0,
            protocol: 0,
        }
    }

    pub fn is_mass_storage(&self) -> bool {
        self.class == UsbClass::MassStorage
    }

    pub fn is_hid(&self) -> bool {
        self.class == UsbClass::HID
    }

    pub fn is_audio(&self) -> bool {
        self.class == UsbClass::Audio || self.class == UsbClass::AudioVideo
    }
}

impl Device for UsbDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> DeviceType {
        if self.is_mass_storage() {
            DeviceType::UsbDisk
        } else {
            DeviceType::Unknown
        }
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Initialisation USB: {} ({:04X}:{:04X}) - {} Mbps\n",
            self.name,
            self.vendor_id,
            self.product_id,
            self.speed.to_mbps()
        ));
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!("Arrêt USB: {}\n", self.name));
        Ok(())
    }
}

/// Disque USB
#[derive(Debug, Clone)]
pub struct UsbDisk {
    pub device: UsbDevice,
    pub capacity: u64,
    pub block_size: u32,
    pub partitions: Vec<Partition>,
}

/// Partition
#[derive(Debug, Clone)]
pub struct Partition {
    pub number: u8,
    pub start_sector: u64,
    pub size: u64,
    pub filesystem: String,
}

impl UsbDisk {
    pub fn new(device: UsbDevice, capacity: u64) -> Self {
        Self {
            device,
            capacity,
            block_size: 512,
            partitions: Vec::new(),
        }
    }

    pub fn get_size_mb(&self) -> u64 {
        self.capacity / (1024 * 1024)
    }

    pub fn get_size_gb(&self) -> u64 {
        self.capacity / (1024 * 1024 * 1024)
    }

    pub fn add_partition(&mut self, partition: Partition) {
        self.partitions.push(partition);
    }
}

/// Énumérateur USB
pub struct UsbEnumerator;

impl UsbEnumerator {
    pub fn enumerate() -> Result<Vec<UsbDevice>, DeviceError> {
        let mut devices = Vec::new();

        // Exemple de périphériques USB détectés
        let mut device1 = UsbDevice::new("USB Disk 1", 0x0951, 0x1666, UsbSpeed::HighSpeed);
        device1.class = UsbClass::MassStorage;
        device1.bus_number = 1;
        device1.device_number = 1;
        devices.push(device1);

        let mut device2 = UsbDevice::new("USB Keyboard", 0x046D, 0xC31C, UsbSpeed::FullSpeed);
        device2.class = UsbClass::HID;
        device2.bus_number = 1;
        device2.device_number = 2;
        devices.push(device2);

        let mut device3 = UsbDevice::new("USB Mouse", 0x046D, 0xC05A, UsbSpeed::FullSpeed);
        device3.class = UsbClass::HID;
        device3.bus_number = 1;
        device3.device_number = 3;
        devices.push(device3);

        Ok(devices)
    }

    pub fn enumerate_disks() -> Result<Vec<UsbDisk>, DeviceError> {
        let devices = Self::enumerate()?;
        let mut disks = Vec::new();

        for device in devices {
            if device.is_mass_storage() {
                let disk = UsbDisk::new(device, 32 * 1024 * 1024 * 1024); // 32 GB
                disks.push(disk);
            }
        }

        Ok(disks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_usb_device_creation() {
        let device = UsbDevice::new("test", 0x0951, 0x1666, UsbSpeed::HighSpeed);
        assert_eq!(device.vendor_id, 0x0951);
        assert_eq!(device.product_id, 0x1666);
    }

    #[test_case]
    fn test_usb_speed_mbps() {
        assert_eq!(UsbSpeed::FullSpeed.to_mbps(), 12);
        assert_eq!(UsbSpeed::HighSpeed.to_mbps(), 480);
        assert_eq!(UsbSpeed::SuperSpeed.to_mbps(), 5000);
    }

    #[test_case]
    fn test_usb_disk_creation() {
        let device = UsbDevice::new("disk", 0x0951, 0x1666, UsbSpeed::HighSpeed);
        let disk = UsbDisk::new(device, 32 * 1024 * 1024 * 1024);
        assert_eq!(disk.get_size_gb(), 32);
    }

    #[test_case]
    fn test_usb_enumerator() {
        let devices = UsbEnumerator::enumerate().unwrap();
        assert!(devices.len() > 0);
    }
}

