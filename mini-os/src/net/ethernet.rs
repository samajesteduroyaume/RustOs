/// Module Ethernet - Link Layer
/// 
/// Implémente le protocole Ethernet pour la couche liaison

use alloc::vec::Vec;

/// Adresse MAC (6 octets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Adresse MAC broadcast
    pub const BROADCAST: MacAddress = MacAddress([0xFF; 6]);
    
    /// Adresse MAC nulle
    pub const ZERO: MacAddress = MacAddress([0x00; 6]);
    
    /// Crée une adresse MAC
    pub fn new(bytes: [u8; 6]) -> Self {
        MacAddress(bytes)
    }
    
    /// Vérifie si c'est une adresse broadcast
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF; 6]
    }
    
    /// Vérifie si c'est une adresse multicast
    pub fn is_multicast(&self) -> bool {
        (self.0[0] & 0x01) != 0
    }
}

impl core::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5])
    }
}

/// Type Ethernet (EtherType)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EtherType {
    IPv4 = 0x0800,
    ARP = 0x0806,
    IPv6 = 0x86DD,
    Unknown(u16),
}

impl From<u16> for EtherType {
    fn from(value: u16) -> Self {
        match value {
            0x0800 => EtherType::IPv4,
            0x0806 => EtherType::ARP,
            0x86DD => EtherType::IPv6,
            other => EtherType::Unknown(other),
        }
    }
}

impl From<EtherType> for u16 {
    fn from(ether_type: EtherType) -> u16 {
        match ether_type {
            EtherType::IPv4 => 0x0800,
            EtherType::ARP => 0x0806,
            EtherType::IPv6 => 0x86DD,
            EtherType::Unknown(val) => val,
        }
    }
}

/// Frame Ethernet
#[derive(Debug, Clone)]
pub struct EthernetFrame {
    /// Adresse MAC destination
    pub dst: MacAddress,
    /// Adresse MAC source
    pub src: MacAddress,
    /// Type Ethernet
    pub ether_type: EtherType,
    /// Payload
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    /// Taille minimale d'une frame (sans payload)
    pub const MIN_SIZE: usize = 14; // 6 + 6 + 2
    
    /// Taille maximale du payload
    pub const MAX_PAYLOAD: usize = 1500; // MTU standard
    
    /// Crée une nouvelle frame
    pub fn new(dst: MacAddress, src: MacAddress, ether_type: EtherType, payload: Vec<u8>) -> Self {
        Self {
            dst,
            src,
            ether_type,
            payload,
        }
    }
    
    /// Parse une frame depuis des bytes
    pub fn parse(data: &[u8]) -> Result<Self, EthernetError> {
        if data.len() < Self::MIN_SIZE {
            return Err(EthernetError::TooShort);
        }
        
        let mut dst = [0u8; 6];
        let mut src = [0u8; 6];
        
        dst.copy_from_slice(&data[0..6]);
        src.copy_from_slice(&data[6..12]);
        
        let ether_type_raw = u16::from_be_bytes([data[12], data[13]]);
        let ether_type = EtherType::from(ether_type_raw);
        
        let payload = data[14..].to_vec();
        
        Ok(Self {
            dst: MacAddress(dst),
            src: MacAddress(src),
            ether_type,
            payload,
        })
    }
    
    /// Sérialise la frame en bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MIN_SIZE + self.payload.len());
        
        // Destination MAC
        bytes.extend_from_slice(&self.dst.0);
        
        // Source MAC
        bytes.extend_from_slice(&self.src.0);
        
        // EtherType
        let ether_type_bytes = u16::to_be_bytes(self.ether_type.into());
        bytes.extend_from_slice(&ether_type_bytes);
        
        // Payload
        bytes.extend_from_slice(&self.payload);
        
        bytes
    }
    
    /// Retourne la taille totale de la frame
    pub fn len(&self) -> usize {
        Self::MIN_SIZE + self.payload.len()
    }
}

/// Erreurs Ethernet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EthernetError {
    TooShort,
    InvalidMac,
    PayloadTooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_mac_address() {
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(!mac.is_broadcast());
        assert!(!mac.is_multicast());
        
        assert!(MacAddress::BROADCAST.is_broadcast());
    }
    
    #[test_case]
    fn test_ethernet_frame() {
        let dst = MacAddress::new([0xFF; 6]);
        let src = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let payload = vec![1, 2, 3, 4];
        
        let frame = EthernetFrame::new(dst, src, EtherType::IPv4, payload);
        assert_eq!(frame.len(), 18); // 14 + 4
    }
    
    #[test_case]
    fn test_frame_serialize_parse() {
        let dst = MacAddress::new([0xFF; 6]);
        let src = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let payload = vec![1, 2, 3, 4];
        
        let frame = EthernetFrame::new(dst, src, EtherType::ARP, payload.clone());
        let bytes = frame.serialize();
        
        let parsed = EthernetFrame::parse(&bytes).unwrap();
        assert_eq!(parsed.dst, dst);
        assert_eq!(parsed.src, src);
        assert_eq!(parsed.ether_type, EtherType::ARP);
        assert_eq!(parsed.payload, payload);
    }
}
