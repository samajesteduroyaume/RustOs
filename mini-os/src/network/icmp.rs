use super::NetworkError;
use alloc::vec::Vec;

/// Types de messages ICMP
pub mod icmp_type {
    pub const ECHO_REPLY: u8 = 0;
    pub const ECHO_REQUEST: u8 = 8;
    pub const DESTINATION_UNREACHABLE: u8 = 3;
    pub const TIME_EXCEEDED: u8 = 11;
}

/// Codes ICMP
pub mod icmp_code {
    pub const ECHO_REPLY: u8 = 0;
    pub const ECHO_REQUEST: u8 = 0;
    pub const HOST_UNREACHABLE: u8 = 1;
    pub const PROTOCOL_UNREACHABLE: u8 = 2;
    pub const PORT_UNREACHABLE: u8 = 3;
}

/// Paquet ICMP
#[derive(Debug, Clone)]
pub struct IcmpPacket {
    pub msg_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub identifier: u16,
    pub sequence: u16,
    pub data: Vec<u8>,
}

impl IcmpPacket {
    /// Crée une requête echo (ping)
    pub fn echo_request(identifier: u16, sequence: u16, data: Vec<u8>) -> Self {
        let mut packet = Self {
            msg_type: icmp_type::ECHO_REQUEST,
            code: icmp_code::ECHO_REQUEST,
            checksum: 0,
            identifier,
            sequence,
            data,
        };
        packet.calculate_checksum();
        packet
    }

    /// Crée une réponse echo (pong)
    pub fn echo_reply(identifier: u16, sequence: u16, data: Vec<u8>) -> Self {
        let mut packet = Self {
            msg_type: icmp_type::ECHO_REPLY,
            code: icmp_code::ECHO_REPLY,
            checksum: 0,
            identifier,
            sequence,
            data,
        };
        packet.calculate_checksum();
        packet
    }

    /// Calcule le checksum ICMP
    pub fn calculate_checksum(&mut self) {
        self.checksum = 0;
        let mut sum: u32 = 0;

        // Ajouter le type et le code
        sum += ((self.msg_type as u32) << 8) | (self.code as u32);

        // Ajouter l'identifiant et la séquence
        sum += self.identifier as u32;
        sum += self.sequence as u32;

        // Ajouter les données
        for i in (0..self.data.len()).step_by(2) {
            if i + 1 < self.data.len() {
                sum += ((self.data[i] as u32) << 8) | (self.data[i + 1] as u32);
            } else {
                sum += (self.data[i] as u32) << 8;
            }
        }

        // Replier les bits de débordement
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        self.checksum = !(sum as u16);
    }

    /// Vérifie le checksum ICMP
    pub fn verify_checksum(&self) -> bool {
        let mut sum: u32 = 0;

        sum += ((self.msg_type as u32) << 8) | (self.code as u32);
        sum += self.checksum as u32;
        sum += self.identifier as u32;
        sum += self.sequence as u32;

        for i in (0..self.data.len()).step_by(2) {
            if i + 1 < self.data.len() {
                sum += ((self.data[i] as u32) << 8) | (self.data[i + 1] as u32);
            } else {
                sum += (self.data[i] as u32) << 8;
            }
        }

        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        sum as u16 == 0xFFFF
    }

    /// Sérialise le paquet ICMP
    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.push(self.msg_type);
        packet.push(self.code);
        packet.extend_from_slice(&self.checksum.to_be_bytes());
        packet.extend_from_slice(&self.identifier.to_be_bytes());
        packet.extend_from_slice(&self.sequence.to_be_bytes());
        packet.extend_from_slice(&self.data);

        packet
    }

    /// Désérialise un paquet ICMP
    pub fn deserialize(data: &[u8]) -> Result<Self, NetworkError> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidPacket);
        }

        let packet = Self {
            msg_type: data[0],
            code: data[1],
            checksum: u16::from_be_bytes([data[2], data[3]]),
            identifier: u16::from_be_bytes([data[4], data[5]]),
            sequence: u16::from_be_bytes([data[6], data[7]]),
            data: if data.len() > 8 {
                data[8..].to_vec()
            } else {
                Vec::new()
            },
        };

        if !packet.verify_checksum() {
            return Err(NetworkError::InvalidChecksum);
        }

        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_icmp_echo_request() {
        let packet = IcmpPacket::echo_request(1, 1, vec![1, 2, 3, 4]);
        assert_eq!(packet.msg_type, icmp_type::ECHO_REQUEST);
        assert!(packet.verify_checksum());
    }

    #[test_case]
    fn test_icmp_echo_reply() {
        let packet = IcmpPacket::echo_reply(1, 1, vec![1, 2, 3, 4]);
        assert_eq!(packet.msg_type, icmp_type::ECHO_REPLY);
        assert!(packet.verify_checksum());
    }

    #[test_case]
    fn test_icmp_serialize() {
        let packet = IcmpPacket::echo_request(1, 1, vec![1, 2, 3, 4]);
        let serialized = packet.serialize();
        assert!(serialized.len() >= 12);
    }

    #[test_case]
    fn test_icmp_deserialize() {
        let packet = IcmpPacket::echo_request(1, 1, vec![1, 2, 3, 4]);
        let serialized = packet.serialize();
        let deserialized = IcmpPacket::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.msg_type, icmp_type::ECHO_REQUEST);
        assert_eq!(deserialized.identifier, 1);
    }
}
