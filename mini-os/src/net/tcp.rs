/// Module TCP (Transmission Control Protocol)
/// 
/// Protocole de transport orienté connexion

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use super::arp::Ipv4Address;
use super::udp::Port;

/// État TCP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

/// Flags TCP
#[derive(Debug, Clone, Copy)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
}

impl TcpFlags {
    pub fn new() -> Self {
        Self {
            fin: false,
            syn: false,
            rst: false,
            psh: false,
            ack: false,
            urg: false,
        }
    }
    
    pub fn syn() -> Self {
        let mut flags = Self::new();
        flags.syn = true;
        flags
    }
    
    pub fn syn_ack() -> Self {
        let mut flags = Self::new();
        flags.syn = true;
        flags.ack = true;
        flags
    }
    
    pub fn ack() -> Self {
        let mut flags = Self::new();
        flags.ack = true;
        flags
    }
    
    pub fn to_u8(&self) -> u8 {
        let mut byte = 0u8;
        if self.fin { byte |= 0x01; }
        if self.syn { byte |= 0x02; }
        if self.rst { byte |= 0x04; }
        if self.psh { byte |= 0x08; }
        if self.ack { byte |= 0x10; }
        if self.urg { byte |= 0x20; }
        byte
    }
    
    pub fn from_u8(byte: u8) -> Self {
        Self {
            fin: (byte & 0x01) != 0,
            syn: (byte & 0x02) != 0,
            rst: (byte & 0x04) != 0,
            psh: (byte & 0x08) != 0,
            ack: (byte & 0x10) != 0,
            urg: (byte & 0x20) != 0,
        }
    }
}

/// Segment TCP
#[derive(Debug, Clone)]
pub struct TcpSegment {
    /// Port source
    pub src_port: Port,
    /// Port destination
    pub dst_port: Port,
    /// Numéro de séquence
    pub seq_num: u32,
    /// Numéro d'acquittement
    pub ack_num: u32,
    /// Data offset (en mots de 32 bits)
    pub data_offset: u8,
    /// Flags
    pub flags: TcpFlags,
    /// Fenêtre
    pub window: u16,
    /// Checksum
    pub checksum: u16,
    /// Urgent pointer
    pub urgent_ptr: u16,
    /// Payload
    pub payload: Vec<u8>,
}

impl TcpSegment {
    /// Taille minimale du header TCP
    pub const MIN_HEADER_SIZE: usize = 20;
    
    /// Crée un nouveau segment
    pub fn new(src_port: Port, dst_port: Port, seq_num: u32, ack_num: u32, flags: TcpFlags, payload: Vec<u8>) -> Self {
        Self {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            data_offset: 5, // 5 * 4 = 20 bytes
            flags,
            window: 65535, // Fenêtre maximale
            checksum: 0,
            urgent_ptr: 0,
            payload,
        }
    }
    
    /// Parse un segment TCP
    pub fn parse(data: &[u8]) -> Result<Self, TcpError> {
        if data.len() < Self::MIN_HEADER_SIZE {
            return Err(TcpError::TooShort);
        }
        
        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        
        let data_offset = (data[12] >> 4) & 0x0F;
        let flags = TcpFlags::from_u8(data[13]);
        
        let window = u16::from_be_bytes([data[14], data[15]]);
        let checksum = u16::from_be_bytes([data[16], data[17]]);
        let urgent_ptr = u16::from_be_bytes([data[18], data[19]]);
        
        let header_len = (data_offset as usize) * 4;
        let payload = data[header_len..].to_vec();
        
        Ok(Self {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            data_offset,
            flags,
            window,
            checksum,
            urgent_ptr,
            payload,
        })
    }
    
    /// Calcule le checksum TCP (avec pseudo-header)
    pub fn calculate_checksum(&self, src_ip: Ipv4Address, dst_ip: Ipv4Address) -> u16 {
        let mut sum: u32 = 0;
        
        // Pseudo-header
        for i in 0..4 {
            sum += src_ip.0[i] as u32;
            sum += dst_ip.0[i] as u32;
        }
        sum += 6; // TCP protocol number
        let tcp_len = (self.data_offset as usize * 4) + self.payload.len();
        sum += tcp_len as u32;
        
        // TCP header (sans checksum)
        sum += self.src_port as u32;
        sum += self.dst_port as u32;
        sum += (self.seq_num >> 16) as u32;
        sum += (self.seq_num & 0xFFFF) as u32;
        sum += (self.ack_num >> 16) as u32;
        sum += (self.ack_num & 0xFFFF) as u32;
        
        let data_offset_flags = ((self.data_offset as u16) << 12) | (self.flags.to_u8() as u16);
        sum += data_offset_flags as u32;
        sum += self.window as u32;
        sum += self.urgent_ptr as u32;
        
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
    
    /// Sérialise le segment
    pub fn serialize(&self) -> Vec<u8> {
        let header_len = (self.data_offset as usize) * 4;
        let mut bytes = Vec::with_capacity(header_len + self.payload.len());
        
        bytes.extend_from_slice(&self.src_port.to_be_bytes());
        bytes.extend_from_slice(&self.dst_port.to_be_bytes());
        bytes.extend_from_slice(&self.seq_num.to_be_bytes());
        bytes.extend_from_slice(&self.ack_num.to_be_bytes());
        
        let data_offset_flags = ((self.data_offset as u16) << 12) | (self.flags.to_u8() as u16);
        bytes.extend_from_slice(&data_offset_flags.to_be_bytes());
        
        bytes.extend_from_slice(&self.window.to_be_bytes());
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&self.urgent_ptr.to_be_bytes());
        
        bytes.extend_from_slice(&self.payload);
        
        bytes
    }
}

/// Connexion TCP
#[derive(Debug, Clone)]
pub struct TcpConnection {
    /// État
    pub state: TcpState,
    /// Port local
    pub local_port: Port,
    /// Port distant
    pub remote_port: Port,
    /// IP distante
    pub remote_ip: Ipv4Address,
    /// Numéro de séquence
    pub seq_num: u32,
    /// Numéro d'acquittement
    pub ack_num: u32,
    /// Buffer de réception
    pub recv_buffer: VecDeque<u8>,
    /// Buffer d'envoi
    pub send_buffer: VecDeque<u8>,
}

impl TcpConnection {
    /// Crée une nouvelle connexion
    pub fn new(local_port: Port, remote_ip: Ipv4Address, remote_port: Port) -> Self {
        // Utiliser RDTSC pour générer un ISN (Initial Sequence Number) pseudo-aléatoire
        let isn = unsafe { core::arch::x86_64::_rdtsc() } as u32;
        
        Self {
            state: TcpState::Closed,
            local_port,
            remote_port,
            remote_ip,
            seq_num: isn,
            ack_num: 0,
            recv_buffer: VecDeque::new(),
            send_buffer: VecDeque::new(),
        }
    }
    
    /// Démarre le handshake (SYN)
    pub fn connect(&mut self) -> TcpSegment {
        self.state = TcpState::SynSent;
        TcpSegment::new(
            self.local_port,
            self.remote_port,
            self.seq_num,
            0,
            TcpFlags::syn(),
            Vec::new()
        )
    }
    
    /// Traite un segment reçu
    pub fn handle_segment(&mut self, segment: &TcpSegment) -> Option<TcpSegment> {
        match self.state {
            TcpState::SynSent => {
                if segment.flags.syn && segment.flags.ack {
                    self.ack_num = segment.seq_num + 1;
                    self.seq_num += 1;
                    self.state = TcpState::Established;
                    
                    // Envoyer ACK
                    return Some(TcpSegment::new(
                        self.local_port,
                        self.remote_port,
                        self.seq_num,
                        self.ack_num,
                        TcpFlags::ack(),
                        Vec::new()
                    ));
                }
            }
            TcpState::Established => {
                if !segment.payload.is_empty() {
                    // Ajouter au buffer de réception
                    self.recv_buffer.extend(&segment.payload);
                    self.ack_num += segment.payload.len() as u32;
                    
                    // Envoyer ACK
                    return Some(TcpSegment::new(
                        self.local_port,
                        self.remote_port,
                        self.seq_num,
                        self.ack_num,
                        TcpFlags::ack(),
                        Vec::new()
                    ));
                }
            }
            _ => {}
        }
        
        None
    }
}

/// Erreurs TCP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpError {
    TooShort,
    ChecksumMismatch,
    InvalidState,
    ConnectionRefused,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_tcp_flags() {
        let flags = TcpFlags::syn();
        assert!(flags.syn);
        assert!(!flags.ack);
        
        let byte = flags.to_u8();
        assert_eq!(byte, 0x02);
        
        let parsed = TcpFlags::from_u8(byte);
        assert!(parsed.syn);
    }
    
    #[test_case]
    fn test_tcp_segment() {
        let payload = vec![1, 2, 3, 4];
        let segment = TcpSegment::new(1234, 5678, 1000, 2000, TcpFlags::ack(), payload.clone());
        
        assert_eq!(segment.src_port, 1234);
        assert_eq!(segment.dst_port, 5678);
        assert_eq!(segment.seq_num, 1000);
        assert_eq!(segment.ack_num, 2000);
    }
    
    #[test_case]
    fn test_tcp_connection() {
        let remote_ip = Ipv4Address::new(192, 168, 1, 1);
        let mut conn = TcpConnection::new(1234, remote_ip, 80);
        
        assert_eq!(conn.state, TcpState::Closed);
        
        let syn = conn.connect();
        assert_eq!(conn.state, TcpState::SynSent);
        assert!(syn.flags.syn);
    }
}
