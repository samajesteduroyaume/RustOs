/// Module ICMP (Internet Control Message Protocol)
/// 
/// Implémente ICMP pour diagnostics réseau (ping)

use alloc::vec::Vec;

/// Type de message ICMP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IcmpType {
    EchoReply = 0,
    DestinationUnreachable = 3,
    EchoRequest = 8,
    TimeExceeded = 11,
    Unknown(u8),
}

impl From<u8> for IcmpType {
    fn from(value: u8) -> Self {
        match value {
            0 => IcmpType::EchoReply,
            3 => IcmpType::DestinationUnreachable,
            8 => IcmpType::EchoRequest,
            11 => IcmpType::TimeExceeded,
            other => IcmpType::Unknown(other),
        }
    }
}

/// Message ICMP
#[derive(Debug, Clone)]
pub struct IcmpMessage {
    /// Type
    pub icmp_type: IcmpType,
    /// Code
    pub code: u8,
    /// Checksum
    pub checksum: u16,
    /// Identifier (pour Echo)
    pub identifier: u16,
    /// Sequence number (pour Echo)
    pub sequence: u16,
    /// Payload
    pub payload: Vec<u8>,
}

impl IcmpMessage {
    /// Taille minimale du header
    pub const MIN_HEADER_SIZE: usize = 8;
    
    /// Crée un Echo Request (ping)
    pub fn echo_request(identifier: u16, sequence: u16, payload: Vec<u8>) -> Self {
        Self {
            icmp_type: IcmpType::EchoRequest,
            code: 0,
            checksum: 0,
            identifier,
            sequence,
            payload,
        }
    }
    
    /// Crée un Echo Reply (pong)
    pub fn echo_reply(identifier: u16, sequence: u16, payload: Vec<u8>) -> Self {
        Self {
            icmp_type: IcmpType::EchoReply,
            code: 0,
            checksum: 0,
            identifier,
            sequence,
            payload,
        }
    }
    
    /// Parse un message ICMP
    pub fn parse(data: &[u8]) -> Result<Self, IcmpError> {
        if data.len() < Self::MIN_HEADER_SIZE {
            return Err(IcmpError::TooShort);
        }
        
        let icmp_type = IcmpType::from(data[0]);
        let code = data[1];
        let checksum = u16::from_be_bytes([data[2], data[3]]);
        let identifier = u16::from_be_bytes([data[4], data[5]]);
        let sequence = u16::from_be_bytes([data[6], data[7]]);
        
        let payload = data[8..].to_vec();
        
        Ok(Self {
            icmp_type,
            code,
            checksum,
            identifier,
            sequence,
            payload,
        })
    }
    
    /// Calcule le checksum ICMP
    pub fn calculate_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        for i in (0..data.len()).step_by(2) {
            if i + 1 < data.len() {
                let word = u16::from_be_bytes([data[i], data[i + 1]]);
                sum += word as u32;
            } else {
                sum += (data[i] as u32) << 8;
            }
        }
        
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        !sum as u16
    }
    
    /// Sérialise le message
    pub fn serialize(&mut self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MIN_HEADER_SIZE + self.payload.len());
        
        // Type et Code
        bytes.push(match self.icmp_type {
            IcmpType::EchoReply => 0,
            IcmpType::DestinationUnreachable => 3,
            IcmpType::EchoRequest => 8,
            IcmpType::TimeExceeded => 11,
            IcmpType::Unknown(v) => v,
        });
        bytes.push(self.code);
        
        // Checksum (temporairement 0)
        bytes.extend_from_slice(&[0, 0]);
        
        // Identifier et Sequence
        bytes.extend_from_slice(&self.identifier.to_be_bytes());
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        
        // Payload
        bytes.extend_from_slice(&self.payload);
        
        // Calculer et insérer le checksum
        let checksum = Self::calculate_checksum(&bytes);
        self.checksum = checksum;
        bytes[2] = (checksum >> 8) as u8;
        bytes[3] = checksum as u8;
        
        bytes
    }
}

/// Erreurs ICMP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpError {
    TooShort,
    ChecksumMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_icmp_echo_request() {
        let payload = vec![1, 2, 3, 4];
        let msg = IcmpMessage::echo_request(1, 1, payload.clone());
        
        assert_eq!(msg.icmp_type, IcmpType::EchoRequest);
        assert_eq!(msg.identifier, 1);
        assert_eq!(msg.sequence, 1);
        assert_eq!(msg.payload, payload);
    }
    
    #[test_case]
    fn test_icmp_serialize_parse() {
        let payload = vec![1, 2, 3, 4];
        let mut msg = IcmpMessage::echo_request(1, 1, payload.clone());
        
        let bytes = msg.serialize();
        let parsed = IcmpMessage::parse(&bytes).unwrap();
        
        assert_eq!(parsed.icmp_type, IcmpType::EchoRequest);
        assert_eq!(parsed.identifier, 1);
        assert_eq!(parsed.sequence, 1);
    }
}
