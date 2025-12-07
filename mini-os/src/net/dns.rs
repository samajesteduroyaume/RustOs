/// Module DNS (Domain Name System)
/// 
/// Client DNS basique pour la résolution de noms (Type A / IPv4)

use alloc::vec::Vec;
use alloc::string::String;
use super::ipv4::Ipv4Packet;
use super::udp::UdpDatagram;
use super::socket::{SocketAddr, Socket};
use super::arp::Ipv4Address;

/// Flags DNS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DnsFlags {
    pub qr: bool,      // Query (0) / Response (1)
    pub opcode: u8,    // 0=Standard, 1=Inverse, 2=Status
    pub aa: bool,      // Authoritative Answer
    pub tc: bool,      // Truncated
    pub rd: bool,      // Recursion Desired
    pub ra: bool,      // Recursion Available
    pub rcode: u8,     // Response Code (0=No Error, 3=Name Error, etc.)
}

impl DnsFlags {
    pub fn new() -> Self {
        Self {
            qr: false,
            opcode: 0,
            aa: false,
            tc: false,
            rd: true, // Par défaut, on demande la récursion
            ra: false,
            rcode: 0,
        }
    }
    
    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        if self.qr { word |= 0x8000; }
        word |= (self.opcode as u16 & 0xF) << 11;
        if self.aa { word |= 0x0400; }
        if self.tc { word |= 0x0200; }
        if self.rd { word |= 0x0100; }
        if self.ra { word |= 0x0080; }
        word |= self.rcode as u16 & 0xF;
        word
    }
    
    pub fn from_u16(word: u16) -> Self {
        Self {
            qr: (word & 0x8000) != 0,
            opcode: ((word >> 11) & 0xF) as u8,
            aa: (word & 0x0400) != 0,
            tc: (word & 0x0200) != 0,
            rd: (word & 0x0100) != 0,
            ra: (word & 0x0080) != 0,
            rcode: (word & 0xF) as u8,
        }
    }
}

/// Header DNS
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub q_count: u16,
    pub ans_count: u16,
    pub auth_count: u16,
    pub add_count: u16,
}

impl DnsHeader {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            flags: DnsFlags::new().to_u16(),
            q_count: 1,
            ans_count: 0,
            auth_count: 0,
            add_count: 0,
        }
    }
    
    pub fn serialize(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes[0..2].copy_from_slice(&self.id.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.flags.to_be_bytes());
        bytes[4..6].copy_from_slice(&self.q_count.to_be_bytes());
        bytes[6..8].copy_from_slice(&self.ans_count.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.auth_count.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.add_count.to_be_bytes());
        bytes
    }
}

/// Type de requête DNS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DnsType {
    A = 1,
    CNAME = 5,
    MX = 15,
    TXT = 16,
    AAAA = 28,
}

/// Classe DNS (généralement IN pour Internet)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DnsClass {
    IN = 1,
}

/// Helper pour encoder un nom de domaine (format labels)
pub fn encode_dns_name(domain: &str) -> Vec<u8> {
    let mut encoded = Vec::new();
    for part in domain.split('.') {
        if part.len() > 63 {
            // Label trop long, truncation (devrait retourner erreur)
            continue; 
        }
        encoded.push(part.len() as u8);
        encoded.extend_from_slice(part.as_bytes());
    }
    encoded.push(0); // Terminateur
    encoded
}

/// Requête DNS
pub struct DnsQuestion {
    pub name: String,
    pub qtype: DnsType,
    pub qclass: DnsClass,
}

impl DnsQuestion {
    pub fn new(name: &str, qtype: DnsType) -> Self {
        Self {
            name: String::from(name),
            qtype,
            qclass: DnsClass::IN,
        }
    }
    
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = encode_dns_name(&self.name);
        bytes.extend_from_slice(&(self.qtype as u16).to_be_bytes());
        bytes.extend_from_slice(&(self.qclass as u16).to_be_bytes());
        bytes
    }
}

/// Paquet DNS complet (pour l'envoi)
pub struct DnsPacket {
    pub header: DnsHeader,
    pub question: DnsQuestion,
}

impl DnsPacket {
    pub fn new(name: &str) -> Self {
        // ID aléatoire (devrait être aléatoire, ici fixe pour l'instant ou via rdtsc quand dispo module global)
        let id = unsafe { core::arch::x86_64::_rdtsc() as u16 };
        Self {
            header: DnsHeader::new(id),
            question: DnsQuestion::new(name, DnsType::A),
        }
    }
    
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.serialize());
        bytes.extend_from_slice(&self.question.serialize());
        bytes
    }
}

/// Réponse DNS (Record)
#[derive(Debug)]
pub struct DnsRecord {
    pub name: String, // Simplification: nom canonique
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub data_len: u16,
    pub rdata: Vec<u8>,
}

impl DnsRecord {
    pub fn parse(data: &[u8], offset: &mut usize) -> Option<Self> {
        // TODO: Parsing complexe des labels (compression avec ptrs 0xC0)
        // Pour l'instant, skip le nom si c'est un pointeur
        if *offset >= data.len() { return None; }
        
        // Skip name (très simplifié)
        loop {
            if *offset >= data.len() { return None; }
            let len = data[*offset];
            if len == 0 {
                *offset += 1;
                break;
            } else if (len & 0xC0) == 0xC0 {
                *offset += 2; // Pointeur (2 bytes)
                break;
            } else {
                *offset += (len as usize) + 1;
            }
        }
        
        if *offset + 10 > data.len() { return None; }
        
        let rtype = u16::from_be_bytes([data[*offset], data[*offset+1]]);
        let rclass = u16::from_be_bytes([data[*offset+2], data[*offset+3]]);
        let ttl = u32::from_be_bytes([data[*offset+4], data[*offset+5], data[*offset+6], data[*offset+7]]);
        let data_len = u16::from_be_bytes([data[*offset+8], data[*offset+9]]);
        
        *offset += 10;
        
        if *offset + (data_len as usize) > data.len() { return None; }
        let rdata = data[*offset..*offset+(data_len as usize)].to_vec();
        *offset += data_len as usize;
        
        Some(Self {
            name: String::new(), // TODO parse name
            rtype,
            rclass,
            ttl,
            data_len,
            rdata,
        })
    }
}

/// Erreurs DNS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsError {
    SocketError,
    SendError,
    RecvError,
    ParseError,
    NameNotFound,
    Timeout,
}

/// Résout un nom de domaine en adresse IPv4 (enregistrement A)
pub fn resolve(domain: &str, dns_server: Ipv4Address) -> Result<Ipv4Address, DnsError> {
    use super::socket::{SocketDomain, SocketType, SOCKET_TABLE};
    
    // Créer un socket UDP
    let mut table = SOCKET_TABLE.lock();
    let socket_id = table.socket(SocketDomain::Inet, SocketType::Datagram)
        .map_err(|_| DnsError::SocketError)?;
    
    // Bind sur un port éphémère (pseudo-aléatoire ou fixe pour test)
    let local_port = 49152 + (unsafe { core::arch::x86_64::_rdtsc() } % 1000) as u16;
    let local_addr = SocketAddr::new(Ipv4Address::new(0, 0, 0, 0), local_port);
    table.bind(socket_id, local_addr).map_err(|_| DnsError::SocketError)?;
    
    // Connecter au serveur DNS
    let remote_addr = SocketAddr::new(dns_server, 53);
    table.connect(socket_id, remote_addr).map_err(|_| DnsError::SocketError)?;
    
    drop(table); // Libérer le lock avant d'attendre
    
    // Préparer la requête
    let packet = DnsPacket::new(domain);
    let bytes = packet.serialize();
    
    // Envoyer
    let mut table = SOCKET_TABLE.lock();
    table.send(socket_id, &bytes).map_err(|_| DnsError::SendError)?;
    drop(table);
    
    // Attendre la réponse (polling simple avec "timeout" basé sur des boucles)
    let mut buffer = [0u8; 512]; // Taille max standard UDP DNS
    let mut retries = 0;
    loop {
        let mut table = SOCKET_TABLE.lock();
        match table.recv(socket_id, &mut buffer) {
            Ok(len) => {
                // Analyser la réponse
                let mut offset = 0;
                // Skip header (12 bytes)
                if len < 12 { return Err(DnsError::ParseError); }
                
                offset += 12;
                
                // Skip Query (variable length)
                let q_len = encode_dns_name(domain).len() + 4;
                offset += q_len;
                
                if offset >= len { return Err(DnsError::ParseError); }
                
                // Parse Answer
                if let Some(record) = DnsRecord::parse(&buffer[..len], &mut offset) {
                    if record.rtype == 1 && record.data_len == 4 { // Type A, Len 4
                         let ip = Ipv4Address::new(
                             record.rdata[0],
                             record.rdata[1],
                             record.rdata[2],
                             record.rdata[3]
                         );
                         return Ok(ip);
                    }
                }
                return Err(DnsError::NameNotFound);
            },
            Err(super::socket::SocketError::WouldBlock) => {
                // Attendre un peu
                drop(table);
                for _ in 0..10000 { core::hint::spin_loop(); }
                retries += 1;
                if retries > 10000 { return Err(DnsError::Timeout); }
            },
            Err(_) => return Err(DnsError::RecvError),
        }
    }
}
