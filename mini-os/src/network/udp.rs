use super::NetworkError;
use alloc::vec::Vec;

/// En-tête UDP
#[derive(Debug, Clone)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dest_port: u16,
    pub length: u16,
    pub checksum: u16,
}

impl UdpHeader {
    pub fn new(src_port: u16, dest_port: u16, payload_len: usize) -> Self {
        Self {
            src_port,
            dest_port,
            length: (8 + payload_len) as u16,
            checksum: 0,
        }
    }

    pub fn calculate_checksum(&mut self) {
        // TODO: Implémenter le checksum UDP
        self.checksum = 0;
    }
}

/// Paquet UDP
#[derive(Debug, Clone)]
pub struct UdpPacket {
    pub header: UdpHeader,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn new(src_port: u16, dest_port: u16, payload: Vec<u8>) -> Self {
        let mut header = UdpHeader::new(src_port, dest_port, payload.len());
        header.calculate_checksum();

        Self { header, payload }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(&self.header.src_port.to_be_bytes());
        packet.extend_from_slice(&self.header.dest_port.to_be_bytes());
        packet.extend_from_slice(&self.header.length.to_be_bytes());
        packet.extend_from_slice(&self.header.checksum.to_be_bytes());
        packet.extend_from_slice(&self.payload);

        packet
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, NetworkError> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidPacket);
        }

        let header = UdpHeader {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dest_port: u16::from_be_bytes([data[2], data[3]]),
            length: u16::from_be_bytes([data[4], data[5]]),
            checksum: u16::from_be_bytes([data[6], data[7]]),
        };

        let payload = if data.len() > 8 {
            data[8..].to_vec()
        } else {
            Vec::new()
        };

        Ok(Self { header, payload })
    }
}

/// Socket UDP
pub struct UdpSocket {
    pub local_port: u16,
    pub remote_port: u16,
    pub remote_ip: [u8; 4],
    pub bound: bool,
}

impl UdpSocket {
    pub fn new() -> Self {
        Self {
            local_port: 0,
            remote_port: 0,
            remote_ip: [0, 0, 0, 0],
            bound: false,
        }
    }

    pub fn bind(&mut self, port: u16) -> Result<(), NetworkError> {
        self.local_port = port;
        self.bound = true;
        Ok(())
    }

    pub fn sendto(&mut self, data: &[u8], addr: ([u8; 4], u16)) -> Result<usize, NetworkError> {
        if !self.bound {
            return Err(NetworkError::PortUnreachable);
        }

        self.remote_ip = addr.0;
        self.remote_port = addr.1;

        // TODO: Envoyer le paquet
        Ok(data.len())
    }

    pub fn recvfrom(&mut self) -> Result<(Vec<u8>, ([u8; 4], u16)), NetworkError> {
        if !self.bound {
            return Err(NetworkError::PortUnreachable);
        }

        // TODO: Recevoir le paquet
        Ok((Vec::new(), ([0, 0, 0, 0], 0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_udp_header_creation() {
        let header = UdpHeader::new(1234, 5678, 100);
        assert_eq!(header.src_port, 1234);
        assert_eq!(header.dest_port, 5678);
        assert_eq!(header.length, 108);
    }

    #[test_case]
    fn test_udp_packet_serialize() {
        let packet = UdpPacket::new(1234, 5678, vec![1, 2, 3, 4]);
        let serialized = packet.serialize();
        assert!(serialized.len() >= 12);
    }

    #[test_case]
    fn test_udp_socket_creation() {
        let socket = UdpSocket::new();
        assert!(!socket.bound);
    }
}
