use super::{Device, DeviceType, DeviceError};
use alloc::vec::Vec;
use alloc::string::String;
use crate::vga_buffer::WRITER;

/// Type de périphérique Bluetooth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothDeviceType {
    Headset,
    Keyboard,
    Mouse,
    Speaker,
    Printer,
    Phone,
    Tablet,
    Laptop,
    Smartwatch,
    Fitness,
    Camera,
    Unknown,
}

/// Classe de périphérique Bluetooth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothClass {
    Miscellaneous = 0x000000,
    Computer = 0x010000,
    Phone = 0x020000,
    AudioVideo = 0x040000,
    Peripheral = 0x050000,
    Imaging = 0x060000,
    Wearable = 0x070000,
    Toy = 0x080000,
    HealthDevice = 0x090000,
    Unknown = 0xFFFFFF,
}

/// Périphérique Bluetooth
#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub address: [u8; 6],
    pub name: String,
    pub device_type: BluetoothDeviceType,
    pub device_class: BluetoothClass,
    pub rssi: i8,                // Signal strength (dBm)
    pub tx_power: i8,            // Transmission power (dBm)
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
}

impl BluetoothDevice {
    pub fn new(address: [u8; 6], name: &str) -> Self {
        Self {
            address,
            name: name.into(),
            device_type: BluetoothDeviceType::Unknown,
            device_class: BluetoothClass::Unknown,
            rssi: -100,
            tx_power: 0,
            paired: false,
            connected: false,
            trusted: false,
        }
    }

    pub fn get_signal_strength(&self) -> &'static str {
        match self.rssi {
            -30..=0 => "Excellent",
            -67..=-31 => "Good",
            -70..=-68 => "Fair",
            -100..=-71 => "Poor",
            _ => "Unknown",
        }
    }

    pub fn is_available(&self) -> bool {
        self.rssi > -100
    }
}

/// Adaptateur Bluetooth
#[derive(Debug, Clone)]
pub struct BluetoothAdapter {
    pub name: String,
    pub address: [u8; 6],
    pub version: u8,
    pub manufacturer: u16,
    pub devices: Vec<BluetoothDevice>,
    pub scanning: bool,
    pub powered: bool,
}

impl BluetoothAdapter {
    pub fn new(name: &str, address: [u8; 6]) -> Self {
        Self {
            name: name.into(),
            address,
            version: 5,                    // Bluetooth 5.0
            manufacturer: 0x000D,          // Broadcom
            devices: Vec::new(),
            scanning: false,
            powered: false,
        }
    }

    pub fn add_device(&mut self, device: BluetoothDevice) {
        self.devices.push(device);
    }

    pub fn start_scan(&mut self) -> Result<(), DeviceError> {
        if !self.powered {
            return Err(DeviceError::OperationFailed);
        }
        self.scanning = true;
        WRITER.lock().write_string(&format!(
            "Scan Bluetooth démarré sur {}\n",
            self.name
        ));
        Ok(())
    }

    pub fn stop_scan(&mut self) -> Result<(), DeviceError> {
        self.scanning = false;
        WRITER.lock().write_string(&format!(
            "Scan Bluetooth arrêté sur {}\n",
            self.name
        ));
        Ok(())
    }

    pub fn pair_device(&mut self, address: [u8; 6]) -> Result<(), DeviceError> {
        if let Some(device) = self.devices.iter_mut().find(|d| d.address == address) {
            device.paired = true;
            WRITER.lock().write_string(&format!(
                "Appairage Bluetooth: {}\n",
                device.name
            ));
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn connect_device(&mut self, address: [u8; 6]) -> Result<(), DeviceError> {
        if let Some(device) = self.devices.iter_mut().find(|d| d.address == address) {
            if !device.paired {
                return Err(DeviceError::OperationFailed);
            }
            device.connected = true;
            WRITER.lock().write_string(&format!(
                "Connexion Bluetooth: {}\n",
                device.name
            ));
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn disconnect_device(&mut self, address: [u8; 6]) -> Result<(), DeviceError> {
        if let Some(device) = self.devices.iter_mut().find(|d| d.address == address) {
            device.connected = false;
            WRITER.lock().write_string(&format!(
                "Déconnexion Bluetooth: {}\n",
                device.name
            ));
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn get_paired_devices(&self) -> Vec<&BluetoothDevice> {
        self.devices.iter().filter(|d| d.paired).collect()
    }

    pub fn get_connected_devices(&self) -> Vec<&BluetoothDevice> {
        self.devices.iter().filter(|d| d.connected).collect()
    }

    pub fn get_available_devices(&self) -> Vec<&BluetoothDevice> {
        self.devices.iter().filter(|d| d.is_available()).collect()
    }
}

impl Device for BluetoothAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Bluetooth
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Initialisation Bluetooth: {} (v{}.0)\n",
            self.name, self.version
        ));
        self.powered = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!("Arrêt Bluetooth: {}\n", self.name));
        self.powered = false;
        self.scanning = false;
        Ok(())
    }
}

/// Énumérateur Bluetooth
pub struct BluetoothEnumerator;

impl BluetoothEnumerator {
    pub fn enumerate() -> Result<Vec<BluetoothAdapter>, DeviceError> {
        let mut adapters = Vec::new();

        // Exemple d'adaptateur Bluetooth
        let mut adapter = BluetoothAdapter::new("hci0", [0x5C, 0xF3, 0x70, 0x8B, 0x12, 0x34]);

        // Ajouter des périphériques d'exemple
        let mut device1 = BluetoothDevice::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0x01], "Sony Headset");
        device1.device_type = BluetoothDeviceType::Headset;
        device1.rssi = -45;
        device1.paired = true;
        device1.connected = true;
        adapter.add_device(device1);

        let mut device2 = BluetoothDevice::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0x02], "Logitech Keyboard");
        device2.device_type = BluetoothDeviceType::Keyboard;
        device2.rssi = -55;
        device2.paired = true;
        adapter.add_device(device2);

        let mut device3 = BluetoothDevice::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0x03], "Apple Watch");
        device3.device_type = BluetoothDeviceType::Smartwatch;
        device3.rssi = -65;
        device3.paired = true;
        adapter.add_device(device3);

        adapters.push(adapter);
        Ok(adapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_bluetooth_device_creation() {
        let device = BluetoothDevice::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF], "Test Device");
        assert_eq!(device.name, "Test Device");
        assert!(!device.paired);
    }

    #[test_case]
    fn test_bluetooth_signal_strength() {
        let mut device = BluetoothDevice::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF], "Test");
        device.rssi = -45;
        assert_eq!(device.get_signal_strength(), "Excellent");
        device.rssi = -70;
        assert_eq!(device.get_signal_strength(), "Fair");
    }

    #[test_case]
    fn test_bluetooth_adapter_creation() {
        let adapter = BluetoothAdapter::new("hci0", [0x5C, 0xF3, 0x70, 0x8B, 0x12, 0x34]);
        assert_eq!(adapter.name, "hci0");
        assert!(!adapter.powered);
    }

    #[test_case]
    fn test_bluetooth_enumerator() {
        let adapters = BluetoothEnumerator::enumerate().unwrap();
        assert!(adapters.len() > 0);
        assert!(adapters[0].devices.len() > 0);
    }
}

