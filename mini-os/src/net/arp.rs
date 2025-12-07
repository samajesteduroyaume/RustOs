/// Module ARP (Address Resolution Protocol)
/// 
/// Résolution d'adresses IPv4 en adresses MAC

use alloc::collections::BTreeMap;
use spin::Mutex;
use super::ethernet::MacAddress;

/// Adresse IPv4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Address([a, b, c, d])
    }
    
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Ipv4Address(bytes)
    }
}

impl core::fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// Type d'opération ARP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
}

/// Packet ARP
#[derive(Debug, Clone)]
pub struct ArpPacket {
    /// Opération (Request/Reply)
    pub operation: ArpOperation,
    /// Adresse MAC source
    pub sender_mac: MacAddress,
    /// Adresse IP source
    pub sender_ip: Ipv4Address,
    /// Adresse MAC destination
    pub target_mac: MacAddress,
    /// Adresse IP destination
    pub target_ip: Ipv4Address,
}

impl ArpPacket {
    /// Taille d'un packet ARP (Ethernet + IPv4)
    pub const SIZE: usize = 28;
    
    /// Crée un packet ARP Request
    pub fn request(sender_mac: MacAddress, sender_ip: Ipv4Address, target_ip: Ipv4Address) -> Self {
        Self {
            operation: ArpOperation::Request,
            sender_mac,
            sender_ip,
            target_mac: MacAddress::ZERO,
            target_ip,
        }
    }
    
    /// Crée un packet ARP Reply
    pub fn reply(sender_mac: MacAddress, sender_ip: Ipv4Address, 
                 target_mac: MacAddress, target_ip: Ipv4Address) -> Self {
        Self {
            operation: ArpOperation::Reply,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        }
    }
    
    /// Parse un packet ARP
    pub fn parse(data: &[u8]) -> Result<Self, ArpError> {
        if data.len() < Self::SIZE {
            return Err(ArpError::TooShort);
        }
        
        // Hardware type (2 bytes) - devrait être 1 pour Ethernet
        let hw_type = u16::from_be_bytes([data[0], data[1]]);
        if hw_type != 1 {
            return Err(ArpError::InvalidHardwareType);
        }
        
        // Protocol type (2 bytes) - devrait être 0x0800 pour IPv4
        let proto_type = u16::from_be_bytes([data[2], data[3]]);
        if proto_type != 0x0800 {
            return Err(ArpError::InvalidProtocolType);
        }
        
        // Operation
        let op = u16::from_be_bytes([data[6], data[7]]);
        let operation = match op {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            _ => return Err(ArpError::InvalidOperation),
        };
        
        // Adresses
        let sender_mac = MacAddress::new([data[8], data[9], data[10], data[11], data[12], data[13]]);
        let sender_ip = Ipv4Address::from_bytes([data[14], data[15], data[16], data[17]]);
        let target_mac = MacAddress::new([data[18], data[19], data[20], data[21], data[22], data[23]]);
        let target_ip = Ipv4Address::from_bytes([data[24], data[25], data[26], data[27]]);
        
        Ok(Self {
            operation,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        })
    }
    
    /// Sérialise le packet
    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        
        // Hardware type (Ethernet = 1)
        bytes[0..2].copy_from_slice(&1u16.to_be_bytes());
        
        // Protocol type (IPv4 = 0x0800)
        bytes[2..4].copy_from_slice(&0x0800u16.to_be_bytes());
        
        // Hardware size (6 pour MAC)
        bytes[4] = 6;
        
        // Protocol size (4 pour IPv4)
        bytes[5] = 4;
        
        // Operation
        let op: u16 = match self.operation {
            ArpOperation::Request => 1,
            ArpOperation::Reply => 2,
        };
        bytes[6..8].copy_from_slice(&op.to_be_bytes());
        
        // Sender MAC
        bytes[8..14].copy_from_slice(&self.sender_mac.0);
        
        // Sender IP
        bytes[14..18].copy_from_slice(&self.sender_ip.0);
        
        // Target MAC
        bytes[18..24].copy_from_slice(&self.target_mac.0);
        
        // Target IP
        bytes[24..28].copy_from_slice(&self.target_ip.0);
        
        bytes
    }
}

/// Entrée de cache ARP
#[derive(Debug, Clone)]
struct ArpCacheEntry {
    mac: MacAddress,
    timestamp: u64,
}

/// Cache ARP
pub struct ArpCache {
    /// Entrées (IP -> MAC)
    entries: BTreeMap<Ipv4Address, ArpCacheEntry>,
    /// Timeout (en secondes)
    timeout: u64,
}

impl ArpCache {
    /// Crée un nouveau cache
    pub fn new(timeout: u64) -> Self {
        Self {
            entries: BTreeMap::new(),
            timeout,
        }
    }
    
    /// Ajoute une entrée
    pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress) {
        let entry = ArpCacheEntry {
            mac,
            timestamp: 0, // TODO: Utiliser vrai timestamp
        };
        self.entries.insert(ip, entry);
    }
    
    /// Récupère une adresse MAC
    pub fn get(&self, ip: &Ipv4Address) -> Option<MacAddress> {
        self.entries.get(ip).map(|entry| entry.mac)
    }
    
    /// Supprime les entrées expirées
    pub fn cleanup(&mut self, current_time: u64) {
        self.entries.retain(|_, entry| {
            current_time - entry.timestamp < self.timeout
        });
    }
    
    /// Retourne le nombre d'entrées
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Erreurs ARP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpError {
    TooShort,
    InvalidHardwareType,
    InvalidProtocolType,
    InvalidOperation,
}

/// Instance globale du cache ARP
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ARP_CACHE: Mutex<ArpCache> = Mutex::new(ArpCache::new(300)); // 5 minutes
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_ipv4_address() {
        let ip = Ipv4Address::new(192, 168, 1, 1);
        assert_eq!(ip.0, [192, 168, 1, 1]);
    }
    
    #[test_case]
    fn test_arp_request() {
        let sender_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let sender_ip = Ipv4Address::new(192, 168, 1, 1);
        let target_ip = Ipv4Address::new(192, 168, 1, 2);
        
        let packet = ArpPacket::request(sender_mac, sender_ip, target_ip);
        assert_eq!(packet.operation, ArpOperation::Request);
    }
    
    #[test_case]
    fn test_arp_serialize_parse() {
        let sender_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let sender_ip = Ipv4Address::new(192, 168, 1, 1);
        let target_ip = Ipv4Address::new(192, 168, 1, 2);
        
        let packet = ArpPacket::request(sender_mac, sender_ip, target_ip);
        let bytes = packet.serialize();
        
        let parsed = ArpPacket::parse(&bytes).unwrap();
        assert_eq!(parsed.operation, ArpOperation::Request);
        assert_eq!(parsed.sender_ip, sender_ip);
    }
    
    #[test_case]
    fn test_arp_cache() {
        let mut cache = ArpCache::new(300);
        let ip = Ipv4Address::new(192, 168, 1, 1);
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        
        cache.insert(ip, mac);
        assert_eq!(cache.get(&ip), Some(mac));
        assert_eq!(cache.len(), 1);
    }
}
