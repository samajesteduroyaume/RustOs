/// Module IPv4 - Network Layer
/// 
/// Implémente le protocole IPv4

use alloc::vec::Vec;
use super::arp::Ipv4Address;

/// Protocole IP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IpProtocol {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
    Unknown(u8),
}

impl From<u8> for IpProtocol {
    fn from(value: u8) -> Self {
        match value {
            1 => IpProtocol::ICMP,
            6 => IpProtocol::TCP,
            17 => IpProtocol::UDP,
            other => IpProtocol::Unknown(other),
        }
    }
}

/// Packet IPv4
#[derive(Debug, Clone)]
pub struct Ipv4Packet {
    /// Version (toujours 4)
    pub version: u8,
    /// Header length (en mots de 32 bits)
    pub ihl: u8,
    /// Type of Service
    pub tos: u8,
    /// Total length
    pub total_length: u16,
    /// Identification
    pub id: u16,
    /// Flags et Fragment offset
    pub flags_fragment: u16,
    /// Time to Live
    pub ttl: u8,
    /// Protocole
    pub protocol: IpProtocol,
    /// Checksum
    pub checksum: u16,
    /// Source IP
    pub src: Ipv4Address,
    /// Destination IP
    pub dst: Ipv4Address,
    /// Payload
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    /// Taille minimale du header (sans options)
    pub const MIN_HEADER_SIZE: usize = 20;
    
    /// Crée un nouveau packet
    pub fn new(src: Ipv4Address, dst: Ipv4Address, protocol: IpProtocol, payload: Vec<u8>) -> Self {
        let total_length = (Self::MIN_HEADER_SIZE + payload.len()) as u16;
        
        Self {
            version: 4,
            ihl: 5, // 5 * 4 = 20 bytes
            tos: 0,
            total_length,
            id: 0,
            flags_fragment: 0,
            ttl: 64,
            protocol,
            checksum: 0,
            src,
            dst,
            payload,
        }
    }
    
    /// Parse un packet IPv4
    pub fn parse(data: &[u8]) -> Result<Self, Ipv4Error> {
        if data.len() < Self::MIN_HEADER_SIZE {
            return Err(Ipv4Error::TooShort);
        }
        
        let version = (data[0] >> 4) & 0x0F;
        if version != 4 {
            return Err(Ipv4Error::InvalidVersion);
        }
        
        let ihl = data[0] & 0x0F;
        let header_len = (ihl as usize) * 4;
        
        if data.len() < header_len {
            return Err(Ipv4Error::TooShort);
        }
        
        let tos = data[1];
        let total_length = u16::from_be_bytes([data[2], data[3]]);
        let id = u16::from_be_bytes([data[4], data[5]]);
        let flags_fragment = u16::from_be_bytes([data[6], data[7]]);
        let ttl = data[8];
        let protocol = IpProtocol::from(data[9]);
        let checksum = u16::from_be_bytes([data[10], data[11]]);
        
        let src = Ipv4Address::from_bytes([data[12], data[13], data[14], data[15]]);
        let dst = Ipv4Address::from_bytes([data[16], data[17], data[18], data[19]]);
        
        let payload = data[header_len..].to_vec();
        
        Ok(Self {
            version,
            ihl,
            tos,
            total_length,
            id,
            flags_fragment,
            ttl,
            protocol,
            checksum,
            src,
            dst,
            payload,
        })
    }
    
    /// Calcule le checksum IPv4
    pub fn calculate_checksum(header: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        for i in (0..header.len()).step_by(2) {
            if i + 1 < header.len() {
                let word = u16::from_be_bytes([header[i], header[i + 1]]);
                sum += word as u32;
            }
        }
        
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        !sum as u16
    }
    
    /// Sérialise le packet
    pub fn serialize(&mut self) -> Vec<u8> {
        let header_len = (self.ihl as usize) * 4;
        let mut bytes = Vec::with_capacity(header_len + self.payload.len());
        
        // Version et IHL
        bytes.push((self.version << 4) | self.ihl);
        bytes.push(self.tos);
        bytes.extend_from_slice(&self.total_length.to_be_bytes());
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.extend_from_slice(&self.flags_fragment.to_be_bytes());
        bytes.push(self.ttl);
        bytes.push(match self.protocol {
            IpProtocol::ICMP => 1,
            IpProtocol::TCP => 6,
            IpProtocol::UDP => 17,
            IpProtocol::Unknown(v) => v,
        });
        
        // Checksum (temporairement 0)
        bytes.extend_from_slice(&[0, 0]);
        
        // Adresses
        bytes.extend_from_slice(&self.src.0);
        bytes.extend_from_slice(&self.dst.0);
        
        // Calculer et insérer le checksum
        let checksum = Self::calculate_checksum(&bytes[..header_len]);
        self.checksum = checksum;
        bytes[10] = (checksum >> 8) as u8;
        bytes[11] = checksum as u8;
        
        // Payload
        bytes.extend_from_slice(&self.payload);
        
        bytes
    }
}

/// Erreurs IPv4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ipv4Error {
    TooShort,
    InvalidVersion,
    ChecksumMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_ipv4_packet_creation() {
        let src = Ipv4Address::new(192, 168, 1, 1);
        let dst = Ipv4Address::new(192, 168, 1, 2);
        let payload = vec![1, 2, 3, 4];
        
        let packet = Ipv4Packet::new(src, dst, IpProtocol::ICMP, payload);
        assert_eq!(packet.version, 4);
        assert_eq!(packet.total_length, 24); // 20 + 4
    }
    
    #[test_case]
    fn test_ipv4_checksum() {
        let header = vec![0x45, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00,
                          0x40, 0x06, 0x00, 0x00, 0xac, 0x10, 0x0a, 0x63,
                          0xac, 0x10, 0x0a, 0x0c];
        let checksum = Ipv4Packet::calculate_checksum(&header);
        assert_ne!(checksum, 0);
    }
}
