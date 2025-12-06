use super::{NetworkError, IpAddr};
use alloc::vec::Vec;

/// En-tête IPv4
#[derive(Debug, Clone)]
pub struct Ipv4Header {
    pub version_ihl: u8,           // Version (4 bits) + IHL (4 bits)
    pub dscp_ecn: u8,              // DSCP (6 bits) + ECN (2 bits)
    pub total_length: u16,         // Longueur totale
    pub identification: u16,       // Identification
    pub flags_offset: u16,         // Flags (3 bits) + Fragment Offset (13 bits)
    pub ttl: u8,                   // Time To Live
    pub protocol: u8,              // Protocole
    pub checksum: u16,             // Checksum
    pub src_ip: IpAddr,            // Adresse IP source
    pub dest_ip: IpAddr,           // Adresse IP destination
}

impl Ipv4Header {
    pub fn new(src_ip: IpAddr, dest_ip: IpAddr, protocol: u8) -> Self {
        Self {
            version_ihl: 0x45,         // Version 4, IHL 5
            dscp_ecn: 0x00,
            total_length: 20,          // Longueur minimale
            identification: 0,
            flags_offset: 0,           // No flags, offset 0
            ttl: 64,
            protocol,
            checksum: 0,
            src_ip,
            dest_ip,
        }
    }

    pub fn get_version(&self) -> u8 {
        self.version_ihl >> 4
    }

    pub fn get_ihl(&self) -> u8 {
        self.version_ihl & 0x0F
    }

    pub fn get_header_length(&self) -> usize {
        (self.get_ihl() as usize) * 4
    }

    pub fn calculate_checksum(&mut self) {
        self.checksum = 0;
        let mut sum: u32 = 0;

        // Ajouter tous les mots de 16 bits
        sum += ((self.version_ihl as u32) << 8) | (self.dscp_ecn as u32);
        sum += self.total_length as u32;
        sum += self.identification as u32;
        sum += self.flags_offset as u32;
        sum += ((self.ttl as u32) << 8) | (self.protocol as u32);
        sum += ((self.src_ip.octets[0] as u32) << 8) | (self.src_ip.octets[1] as u32);
        sum += ((self.src_ip.octets[2] as u32) << 8) | (self.src_ip.octets[3] as u32);
        sum += ((self.dest_ip.octets[0] as u32) << 8) | (self.dest_ip.octets[1] as u32);
        sum += ((self.dest_ip.octets[2] as u32) << 8) | (self.dest_ip.octets[3] as u32);

        // Replier les bits de débordement
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        self.checksum = !(sum as u16);
    }

    pub fn verify_checksum(&self) -> bool {
        let mut sum: u32 = 0;

        sum += ((self.version_ihl as u32) << 8) | (self.dscp_ecn as u32);
        sum += self.total_length as u32;
        sum += self.identification as u32;
        sum += self.flags_offset as u32;
        sum += ((self.ttl as u32) << 8) | (self.protocol as u32);
        sum += self.checksum as u32;
        sum += ((self.src_ip.octets[0] as u32) << 8) | (self.src_ip.octets[1] as u32);
        sum += ((self.src_ip.octets[2] as u32) << 8) | (self.src_ip.octets[3] as u32);
        sum += ((self.dest_ip.octets[0] as u32) << 8) | (self.dest_ip.octets[1] as u32);
        sum += ((self.dest_ip.octets[2] as u32) << 8) | (self.dest_ip.octets[3] as u32);

        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        sum as u16 == 0xFFFF
    }
}

/// Paquet IPv4
#[derive(Debug, Clone)]
pub struct Ipv4Packet {
    pub header: Ipv4Header,
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    pub fn new(src_ip: IpAddr, dest_ip: IpAddr, protocol: u8, payload: Vec<u8>) -> Self {
        let mut header = Ipv4Header::new(src_ip, dest_ip, protocol);
        header.total_length = (20 + payload.len()) as u16;
        header.calculate_checksum();

        Self { header, payload }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        // Version + IHL
        packet.push(self.header.version_ihl);

        // DSCP + ECN
        packet.push(self.header.dscp_ecn);

        // Total Length
        packet.extend_from_slice(&self.header.total_length.to_be_bytes());

        // Identification
        packet.extend_from_slice(&self.header.identification.to_be_bytes());

        // Flags + Fragment Offset
        packet.extend_from_slice(&self.header.flags_offset.to_be_bytes());

        // TTL
        packet.push(self.header.ttl);

        // Protocol
        packet.push(self.header.protocol);

        // Checksum
        packet.extend_from_slice(&self.header.checksum.to_be_bytes());

        // Source IP
        packet.extend_from_slice(&self.header.src_ip.octets);

        // Destination IP
        packet.extend_from_slice(&self.header.dest_ip.octets);

        // Payload
        packet.extend_from_slice(&self.payload);

        packet
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, NetworkError> {
        if data.len() < 20 {
            return Err(NetworkError::InvalidPacket);
        }

        let version_ihl = data[0];
        let ihl = (version_ihl & 0x0F) as usize;
        let header_len = ihl * 4;

        if data.len() < header_len {
            return Err(NetworkError::InvalidPacket);
        }

        let mut header = Ipv4Header {
            version_ihl,
            dscp_ecn: data[1],
            total_length: u16::from_be_bytes([data[2], data[3]]),
            identification: u16::from_be_bytes([data[4], data[5]]),
            flags_offset: u16::from_be_bytes([data[6], data[7]]),
            ttl: data[8],
            protocol: data[9],
            checksum: u16::from_be_bytes([data[10], data[11]]),
            src_ip: IpAddr::from_bytes(&[data[12], data[13], data[14], data[15]]),
            dest_ip: IpAddr::from_bytes(&[data[16], data[17], data[18], data[19]]),
        };

        if !header.verify_checksum() {
            return Err(NetworkError::InvalidChecksum);
        }

        let payload = if data.len() > header_len {
            data[header_len..].to_vec()
        } else {
            Vec::new()
        };

        Ok(Self { header, payload })
    }
}

/// Protocoles IPv4
pub mod protocol {
    pub const ICMP: u8 = 1;
    pub const TCP: u8 = 6;
    pub const UDP: u8 = 17;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_ipv4_header_creation() {
        let src = IpAddr::new(192, 168, 1, 1);
        let dst = IpAddr::new(192, 168, 1, 2);
        let header = Ipv4Header::new(src, dst, protocol::UDP);
        assert_eq!(header.get_version(), 4);
        assert_eq!(header.get_ihl(), 5);
    }

    #[test_case]
    fn test_ipv4_checksum() {
        let src = IpAddr::new(192, 168, 1, 1);
        let dst = IpAddr::new(192, 168, 1, 2);
        let mut header = Ipv4Header::new(src, dst, protocol::UDP);
        header.calculate_checksum();
        assert!(header.verify_checksum());
    }

    #[test_case]
    fn test_ipv4_packet_serialize() {
        let src = IpAddr::new(192, 168, 1, 1);
        let dst = IpAddr::new(192, 168, 1, 2);
        let packet = Ipv4Packet::new(src, dst, protocol::UDP, vec![1, 2, 3, 4]);
        let serialized = packet.serialize();
        assert!(serialized.len() >= 24);
    }

    #[test_case]
    fn test_ipv4_packet_deserialize() {
        let src = IpAddr::new(192, 168, 1, 1);
        let dst = IpAddr::new(192, 168, 1, 2);
        let packet = Ipv4Packet::new(src, dst, protocol::UDP, vec![1, 2, 3, 4]);
        let serialized = packet.serialize();
        let deserialized = Ipv4Packet::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.header.src_ip, src);
        assert_eq!(deserialized.header.dest_ip, dst);
    }
}
