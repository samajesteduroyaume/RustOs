use super::{Driver, DriverError};
extern crate alloc;
use alloc::vec::Vec;
use alloc::format;
use crate::vga_buffer::WRITER;

/// Erreurs spécifiques au driver réseau
#[derive(Debug, Clone, Copy)]
pub enum NetworkError {
    InvalidPacket,
    BufferTooSmall,
    NotConnected,
    TransmissionFailed,
    ReceiveFailed,
    Timeout,
}

/// Trame Ethernet
#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub dest_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    /// Crée une nouvelle trame Ethernet
    pub fn new(dest_mac: [u8; 6], src_mac: [u8; 6], ethertype: u16, payload: Vec<u8>) -> Self {
        Self {
            dest_mac,
            src_mac,
            ethertype,
            payload,
        }
    }

    /// Sérialise la trame en octets
    pub fn serialize(&self) -> Vec<u8> {
        let mut frame = Vec::new();
        
        // Adresse MAC destination (6 octets)
        frame.extend_from_slice(&self.dest_mac);
        
        // Adresse MAC source (6 octets)
        frame.extend_from_slice(&self.src_mac);
        
        // Type Ethernet (2 octets, big-endian)
        frame.extend_from_slice(&self.ethertype.to_be_bytes());
        
        // Charge utile
        frame.extend_from_slice(&self.payload);
        
        frame
    }

    /// Désérialise une trame à partir d'octets
    pub fn deserialize(data: &[u8]) -> Result<Self, NetworkError> {
        if data.len() < 14 {
            return Err(NetworkError::InvalidPacket);
        }

        let mut dest_mac = [0u8; 6];
        let mut src_mac = [0u8; 6];
        
        dest_mac.copy_from_slice(&data[0..6]);
        src_mac.copy_from_slice(&data[6..12]);
        
        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        let payload = data[14..].to_vec();

        Ok(EthernetFrame {
            dest_mac,
            src_mac,
            ethertype,
            payload,
        })
    }
}

/// Types Ethernet courants
pub mod ethertype {
    pub const IPV4: u16 = 0x0800;
    pub const ARP: u16 = 0x0806;
    pub const IPV6: u16 = 0x86DD;
}

/// Driver réseau Ethernet
pub struct NetworkDriver {
    pub name: String,
    pub mac_address: [u8; 6],
    pub mtu: u16,
    pub initialized: bool,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
}

impl NetworkDriver {
    /// Crée un nouveau driver réseau
    pub fn new(name: &str, mac_address: [u8; 6]) -> Self {
        Self {
            name: name.into(),
            mac_address,
            mtu: 1500,
            initialized: false,
            tx_packets: 0,
            rx_packets: 0,
            tx_bytes: 0,
            rx_bytes: 0,
        }
    }

    /// Envoie un paquet
    pub fn send_packet(&mut self, packet: &[u8]) -> Result<(), NetworkError> {
        if packet.len() > self.mtu as usize {
            return Err(NetworkError::BufferTooSmall);
        }

        if !self.initialized {
            return Err(NetworkError::NotConnected);
        }

        // TODO: Implémenter l'envoi de paquet
        // 1. Préparer le paquet
        // 2. Envoyer via le contrôleur réseau
        // 3. Attendre la confirmation

        self.tx_packets += 1;
        self.tx_bytes += packet.len() as u64;

        WRITER.lock().write_string(&format!(
            "Envoi de {} octets\n",
            packet.len()
        ));

        Ok(())
    }

    /// Reçoit un paquet
    pub fn receive_packet(&mut self) -> Result<Vec<u8>, NetworkError> {
        if !self.initialized {
            return Err(NetworkError::NotConnected);
        }

        // TODO: Implémenter la réception de paquet
        // 1. Vérifier s'il y a des paquets disponibles
        // 2. Lire le paquet depuis le contrôleur réseau
        // 3. Retourner le paquet

        self.rx_packets += 1;

        WRITER.lock().write_string("Réception de paquet\n");

        Ok(Vec::new())
    }

    /// Obtient l'adresse MAC
    pub fn get_mac_address(&self) -> [u8; 6] {
        self.mac_address
    }

    /// Définit l'adresse MAC
    pub fn set_mac_address(&mut self, mac: [u8; 6]) {
        self.mac_address = mac;
    }

    /// Obtient la MTU (Maximum Transmission Unit)
    pub fn get_mtu(&self) -> u16 {
        self.mtu
    }

    /// Définit la MTU
    pub fn set_mtu(&mut self, mtu: u16) {
        self.mtu = mtu;
    }

    /// Obtient les statistiques
    pub fn get_stats(&self) -> (u64, u64, u64, u64) {
        (self.tx_packets, self.rx_packets, self.tx_bytes, self.rx_bytes)
    }

    /// Réinitialise les statistiques
    pub fn reset_stats(&mut self) {
        self.tx_packets = 0;
        self.rx_packets = 0;
        self.tx_bytes = 0;
        self.rx_bytes = 0;
    }
}

impl Driver for NetworkDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn init(&mut self) -> Result<(), DriverError> {
        WRITER.lock().write_string(&format!(
            "Initialisation du driver réseau: {}\n",
            self.name
        ));

        // TODO: Initialiser le contrôleur réseau
        // 1. Configurer les registres
        // 2. Activer les interruptions
        // 3. Configurer les buffers

        self.initialized = true;

        WRITER.lock().write_string(&format!(
            "Interface {} initialisée (MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X})\n",
            self.name,
            self.mac_address[0],
            self.mac_address[1],
            self.mac_address[2],
            self.mac_address[3],
            self.mac_address[4],
            self.mac_address[5]
        ));

        Ok(())
    }

    fn handle_interrupt(&mut self, irq: u8) {
        WRITER.lock().write_string(&format!(
            "Interruption réseau (IRQ {})\n",
            irq
        ));

        // TODO: Gérer les interruptions réseau
        // - Vérifier le statut du contrôleur
        // - Traiter les paquets reçus
        // - Traiter les erreurs de transmission
    }

    fn shutdown(&mut self) -> Result<(), DriverError> {
        WRITER.lock().write_string(&format!(
            "Arrêt du driver réseau: {}\n",
            self.name
        ));

        self.initialized = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_network_driver_creation() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let driver = NetworkDriver::new("eth0", mac);
        assert_eq!(driver.name, "eth0");
        assert_eq!(driver.mac_address, mac);
        assert_eq!(driver.mtu, 1500);
    }

    #[test_case]
    fn test_ethernet_frame_serialize() {
        let dest_mac = [0xFF; 6];
        let src_mac = [0x00; 6];
        let frame = EthernetFrame::new(dest_mac, src_mac, ethertype::IPV4, vec![1, 2, 3, 4]);
        let serialized = frame.serialize();
        assert_eq!(serialized.len(), 18); // 6 + 6 + 2 + 4
    }

    #[test_case]
    fn test_ethernet_frame_deserialize() {
        let dest_mac = [0xFF; 6];
        let src_mac = [0x00; 6];
        let frame = EthernetFrame::new(dest_mac, src_mac, ethertype::IPV4, vec![1, 2, 3, 4]);
        let serialized = frame.serialize();
        let deserialized = EthernetFrame::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.dest_mac, dest_mac);
        assert_eq!(deserialized.src_mac, src_mac);
    }

    #[test_case]
    fn test_network_driver_stats() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let mut driver = NetworkDriver::new("eth0", mac);
        driver.tx_packets = 10;
        driver.rx_packets = 5;
        let (tx, rx, _, _) = driver.get_stats();
        assert_eq!(tx, 10);
        assert_eq!(rx, 5);
    }
}
