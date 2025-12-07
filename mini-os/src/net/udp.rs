/// Module UDP (User Datagram Protocol)
/// 
/// Protocole de transport sans connexion

use alloc::vec::Vec;
use super::arp::Ipv4Address;

/// Port UDP
pub type Port = u16;

/// Datagram UDP
#[derive(Debug, Clone)]
pub struct UdpDatagram {
    /// Port source
    pub src_port: Port,
    /// Port destination
    pub dst_port: Port,
    /// Longueur
    pub length: u16,
    /// Checksum
    pub checksum: u16,
    /// Payload
    pub payload: Vec<u8>,
}

impl UdpDatagram {
    /// Taille du header UDP
    pub const HEADER_SIZE: usize = 8;
    
    /// Crée un nouveau datagram
    pub fn new(src_port: Port, dst_port: Port, payload: Vec<u8>) -> Self {
        let length = (Self::HEADER_SIZE + payload.len()) as u16;
        
        Self {
            src_port,
            dst_port,
            length,
            checksum: 0,
            payload,
        }
    }
    
    /// Parse un datagram UDP
    pub fn parse(data: &[u8]) -> Result<Self, UdpError> {
        if data.len() < Self::HEADER_SIZE {
            return Err(UdpError::TooShort);
        }
        
        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);
        let checksum = u16::from_be_bytes([data[6], data[7]]);
        
        let payload = data[8..].to_vec();
        
        Ok(Self {
            src_port,
            dst_port,
            length,
            checksum,
            payload,
        })
    }
    
    /// Calcule le checksum UDP (avec pseudo-header IPv4)
    pub fn calculate_checksum(&self, src_ip: Ipv4Address, dst_ip: Ipv4Address) -> u16 {
        let mut sum: u32 = 0;
        
        // Pseudo-header
        for i in 0..4 {
            sum += src_ip.0[i] as u32;
            sum += dst_ip.0[i] as u32;
        }
        sum += 17; // UDP protocol number
        sum += self.length as u32;
        
        // UDP header
        sum += self.src_port as u32;
        sum += self.dst_port as u32;
        sum += self.length as u32;
        
        // Payload
        for chunk in self.payload.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            } else {
                sum += (chunk[0] as u32) << 8;
            }
        }
        
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        !sum as u16
    }
    
    /// Sérialise le datagram
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::HEADER_SIZE + self.payload.len());
        
        bytes.extend_from_slice(&self.src_port.to_be_bytes());
        bytes.extend_from_slice(&self.dst_port.to_be_bytes());
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        
        bytes
    }
}

/// Erreurs UDP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpError {
    TooShort,
    ChecksumMismatch,
    PortInUse,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_udp_datagram() {
        let payload = vec![1, 2, 3, 4];
        let dgram = UdpDatagram::new(1234, 5678, payload.clone());
        
        assert_eq!(dgram.src_port, 1234);
        assert_eq!(dgram.dst_port, 5678);
        assert_eq!(dgram.length, 12); // 8 + 4
        assert_eq!(dgram.payload, payload);
    }
    
    #[test_case]
    fn test_udp_serialize_parse() {
        let payload = vec![1, 2, 3, 4];
        let dgram = UdpDatagram::new(1234, 5678, payload.clone());
        
        let bytes = dgram.serialize();
        let parsed = UdpDatagram::parse(&bytes).unwrap();
        
        assert_eq!(parsed.src_port, 1234);
        assert_eq!(parsed.dst_port, 5678);
        assert_eq!(parsed.payload, payload);
    }
}
