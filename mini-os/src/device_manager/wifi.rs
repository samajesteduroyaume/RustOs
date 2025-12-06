use super::{Device, DeviceType, DeviceError};
use crate::vga_buffer::WRITER;
use alloc::string::String;

/// Interface Wi-Fi
#[derive(Debug, Clone)]
pub struct WifiInterface {
    pub name: String,
    pub mac_address: [u8; 6],
    pub standard: WifiStandard,
    pub channels: alloc::vec::Vec<u8>,
    pub power: u8,
    pub status: super::ethernet::InterfaceStatus,
    pub driver: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiStandard {
    A,
    B,
    G,
    N,
    AC,
    AX,
    Unknown,
}

impl WifiInterface {
    pub fn new(name: &str, mac: [u8; 6]) -> Self {
        Self {
            name: name.into(),
            mac_address: mac,
            standard: WifiStandard::AC,
            channels: alloc::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            power: 20,
            status: super::ethernet::InterfaceStatus::Down,
            driver: "iwlwifi".into(),
        }
    }
}

impl Device for WifiInterface {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Wifi
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Initialisation interface Wi-Fi: {}\n",
            self.name
        ));

        self.status = super::ethernet::InterfaceStatus::Up;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Arrêt interface Wi-Fi: {}\n",
            self.name
        ));

        self.status = super::ethernet::InterfaceStatus::Down;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_wifi_interface_creation() {
        let iface = WifiInterface::new("wlan0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(iface.name, "wlan0");
        assert_eq!(iface.standard, WifiStandard::AC);
    }

    #[test_case]
    fn test_wifi_interface_init() {
        let mut iface = WifiInterface::new("wlan0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(iface.init().is_ok());
    }

    #[test_case]
    fn test_wifi_device_type() {
        let iface = WifiInterface::new("wlan0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(iface.device_type(), DeviceType::Wifi);
    }

    #[test_case]
    fn test_wifi_channels() {
        let iface = WifiInterface::new("wlan0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(iface.channels.len() > 0);
    }
}
