use super::{Device, DeviceType, DeviceError};
use crate::vga_buffer::WRITER;
use alloc::string::String;

/// Interface Ethernet
#[derive(Debug, Clone)]
pub struct EthernetInterface {
    pub name: String,
    pub mac_address: [u8; 6],
    pub speed: u32,              // Mbps
    pub duplex: Duplex,
    pub status: InterfaceStatus,
    pub driver: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duplex {
    Half,
    Full,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceStatus {
    Up,
    Down,
    Unknown,
}

impl EthernetInterface {
    pub fn new(name: &str, mac: [u8; 6]) -> Self {
        Self {
            name: name.into(),
            mac_address: mac,
            speed: 1000,
            duplex: Duplex::Full,
            status: InterfaceStatus::Down,
            driver: "e1000".into(),
        }
    }
}

impl Device for EthernetInterface {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Ethernet
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Initialisation interface Ethernet: {}\n",
            self.name
        ));

        self.status = InterfaceStatus::Up;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Arrêt interface Ethernet: {}\n",
            self.name
        ));

        self.status = InterfaceStatus::Down;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_ethernet_interface_creation() {
        let iface = EthernetInterface::new("eth0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(iface.name, "eth0");
        assert_eq!(iface.speed, 1000);
    }

    #[test_case]
    fn test_ethernet_interface_init() {
        let mut iface = EthernetInterface::new("eth0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(iface.init().is_ok());
        assert_eq!(iface.status, InterfaceStatus::Up);
    }

    #[test_case]
    fn test_ethernet_interface_shutdown() {
        let mut iface = EthernetInterface::new("eth0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        iface.init().unwrap();
        assert!(iface.shutdown().is_ok());
        assert_eq!(iface.status, InterfaceStatus::Down);
    }

    #[test_case]
    fn test_ethernet_device_type() {
        let iface = EthernetInterface::new("eth0", [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(iface.device_type(), DeviceType::Ethernet);
    }
}
