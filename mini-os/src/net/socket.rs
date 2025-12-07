/// Module Socket API
/// 
/// Interface BSD-like pour la programmation réseau

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;

use super::tcp::{TcpConnection, TcpState};
use super::udp::UdpDatagram;
use super::ipv4::{Ipv4Packet, IpProtocol};
use super::arp::Ipv4Address;

use super::udp::Port;

/// Type de socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream,    // TCP
    Datagram,  // UDP
}

/// Domaine de socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketDomain {
    Inet,      // IPv4
}

/// Adresse socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketAddr {
    pub ip: Ipv4Address,
    pub port: Port,
}

impl SocketAddr {
    pub fn new(ip: Ipv4Address, port: Port) -> Self {
        Self { ip, port }
    }
}


/// Socket
pub struct Socket {
    /// ID du socket
    pub id: u32,
    /// Type
    pub socket_type: SocketType,
    /// Domaine
    pub domain: SocketDomain,
    /// Adresse locale (bind)
    pub local_addr: Option<SocketAddr>,
    /// Adresse distante (connect)
    pub remote_addr: Option<SocketAddr>,
    /// Connexion TCP (si Stream)
    pub tcp_conn: Option<TcpConnection>,
    /// En écoute (listen)
    pub listening: bool,
    /// Backlog (pour listen)
    pub backlog: usize,
    /// Queue de connexions en attente (TCP)
    pub pending_connections: VecDeque<(u32, SocketAddr)>,
    /// Buffer de réception UDP
    pub udp_recv_buffer: VecDeque<Vec<u8>>,
}


impl Socket {
    /// Crée un nouveau socket
    pub fn new(id: u32, domain: SocketDomain, socket_type: SocketType) -> Self {
        Self {
            id,
            socket_type,
            domain,
            local_addr: None,
            remote_addr: None,
            tcp_conn: None,
            listening: false,
            backlog: 0,
            pending_connections: VecDeque::new(),
            udp_recv_buffer: VecDeque::new(),
        }
    }

    
    /// Bind à une adresse locale
    pub fn bind(&mut self, addr: SocketAddr) -> Result<(), SocketError> {
        if self.local_addr.is_some() {
            return Err(SocketError::AlreadyBound);
        }
        
        self.local_addr = Some(addr);
        Ok(())
    }
    
    /// Connect à une adresse distante (TCP)
    pub fn connect(&mut self, addr: SocketAddr) -> Result<(), SocketError> {
        if self.socket_type != SocketType::Stream {
            return Err(SocketError::InvalidOperation);
        }
        
        let local_addr = self.local_addr.ok_or(SocketError::NotBound)?;
        
        let mut conn = TcpConnection::new(local_addr.port, addr.ip, addr.port);
        let _syn = conn.connect();
        
        self.tcp_conn = Some(conn);
        self.remote_addr = Some(addr);
        
        Ok(())
    }
    
    /// Listen pour connexions entrantes (TCP)
    pub fn listen(&mut self, backlog: usize) -> Result<(), SocketError> {
        if self.socket_type != SocketType::Stream {
            return Err(SocketError::InvalidOperation);
        }
        
        if self.local_addr.is_none() {
            return Err(SocketError::NotBound);
        }
        
        self.listening = true;
        self.backlog = backlog;
        
        Ok(())
    }
    
    /// Accept une connexion entrante (TCP)
    pub fn accept(&mut self) -> Result<(u32, SocketAddr), SocketError> {
        if !self.listening {
            return Err(SocketError::NotListening);
        }
        
        if let Some((new_socket_id, addr)) = self.pending_connections.pop_front() {
            Ok((new_socket_id, addr))
        } else {
            Err(SocketError::WouldBlock)
        }
    }
    
    /// Envoie des données
    pub fn send(&mut self, data: &[u8]) -> Result<usize, SocketError> {
        match self.socket_type {
            SocketType::Stream => {
                let conn = self.tcp_conn.as_mut().ok_or(SocketError::NotConnected)?;
                
                if conn.state != TcpState::Established {
                    return Err(SocketError::NotConnected);
                }
                
                // Ajouter au buffer d'envoi
                conn.send_buffer.extend(data);
                
                Ok(data.len())
            }
            SocketType::Datagram => {
                let remote_addr = self.remote_addr.ok_or(SocketError::NotConnected)?;
                let local_addr = self.local_addr.ok_or(SocketError::NotBound)?;
                
                // Créer datagram UDP
                let udp_dgram = UdpDatagram::new(local_addr.port, remote_addr.port, data.to_vec());
                let udp_bytes = udp_dgram.serialize();
                
                // Encapsuler dans IPv4
                let mut ip_packet = Ipv4Packet::new(
                    local_addr.ip,
                    remote_addr.ip,
                    IpProtocol::UDP,
                    udp_bytes
                );
                let ip_bytes = ip_packet.serialize();
                
                // TODO: Envoyer via interface réseau (Ethernet)
                // Pour l'instant on retourne juste la taille
                Ok(data.len())
            }
        }

    }
    
    /// Reçoit des données
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, SocketError> {
        match self.socket_type {
            SocketType::Stream => {
                let conn = self.tcp_conn.as_mut().ok_or(SocketError::NotConnected)?;
                
                if conn.recv_buffer.is_empty() {
                    return Err(SocketError::WouldBlock);
                }
                
                let to_read = core::cmp::min(buffer.len(), conn.recv_buffer.len());
                
                for i in 0..to_read {
                    buffer[i] = conn.recv_buffer.pop_front().unwrap();
                }
                
                Ok(to_read)
            }
            SocketType::Datagram => {
                if self.udp_recv_buffer.is_empty() {
                    return Err(SocketError::WouldBlock);
                }
                
                let packet = self.udp_recv_buffer.pop_front().unwrap();
                let to_read = core::cmp::min(buffer.len(), packet.len());
                
                buffer[..to_read].copy_from_slice(&packet[..to_read]);
                
                Ok(to_read)
            }
        }

    }
}

/// Table de sockets
pub struct SocketTable {
    /// Sockets par ID
    pub sockets: BTreeMap<u32, Socket>,
    /// Prochain ID
    next_id: u32,
}

impl SocketTable {
    /// Crée une nouvelle table
    pub const fn new() -> Self {
        Self {
            sockets: BTreeMap::new(),
            next_id: 1,
        }
    }
    
    /// Crée un nouveau socket
    pub fn socket(&mut self, domain: SocketDomain, socket_type: SocketType) -> Result<u32, SocketError> {
        let id = self.next_id;
        self.next_id += 1;
        
        let socket = Socket::new(id, domain, socket_type);
        self.sockets.insert(id, socket);
        
        Ok(id)
    }
    
    /// Récupère un socket
    pub fn get(&self, id: u32) -> Option<&Socket> {
        self.sockets.get(&id)
    }
    
    /// Récupère un socket mutable
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Socket> {
        self.sockets.get_mut(&id)
    }
    
    /// Ferme un socket
    pub fn close(&mut self, id: u32) -> Result<(), SocketError> {
        self.sockets.remove(&id).ok_or(SocketError::InvalidSocket)?;
        Ok(())
    }
    
    /// Bind
    pub fn bind(&mut self, id: u32, addr: SocketAddr) -> Result<(), SocketError> {
        let socket = self.sockets.get_mut(&id).ok_or(SocketError::InvalidSocket)?;
        socket.bind(addr)
    }
    
    /// Connect
    pub fn connect(&mut self, id: u32, addr: SocketAddr) -> Result<(), SocketError> {
        let socket = self.sockets.get_mut(&id).ok_or(SocketError::InvalidSocket)?;
        socket.connect(addr)
    }
    
    /// Listen
    pub fn listen(&mut self, id: u32, backlog: usize) -> Result<(), SocketError> {
        let socket = self.sockets.get_mut(&id).ok_or(SocketError::InvalidSocket)?;
        socket.listen(backlog)
    }
    
    /// Accept
    pub fn accept(&mut self, id: u32) -> Result<(u32, SocketAddr), SocketError> {
        let socket = self.sockets.get_mut(&id).ok_or(SocketError::InvalidSocket)?;
        socket.accept()
    }
    
    /// Send
    pub fn send(&mut self, id: u32, data: &[u8]) -> Result<usize, SocketError> {
        let socket = self.sockets.get_mut(&id).ok_or(SocketError::InvalidSocket)?;
        socket.send(data)
    }
    
    /// Recv
    pub fn recv(&mut self, id: u32, buffer: &mut [u8]) -> Result<usize, SocketError> {
        let socket = self.sockets.get_mut(&id).ok_or(SocketError::InvalidSocket)?;
        socket.recv(buffer)
    }
}

/// Erreurs de socket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketError {
    InvalidSocket,
    AlreadyBound,
    NotBound,
    NotConnected,
    NotListening,
    InvalidOperation,
    WouldBlock,
    ConnectionRefused,
}

/// Instance globale de la table de sockets
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SOCKET_TABLE: Mutex<SocketTable> = Mutex::new(SocketTable::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_socket_creation() {
        let mut table = SocketTable::new();
        let id = table.socket(SocketDomain::Inet, SocketType::Stream).unwrap();
        
        assert_eq!(id, 1);
        assert!(table.get(id).is_some());
    }
    
    #[test_case]
    fn test_socket_bind() {
        let mut table = SocketTable::new();
        let id = table.socket(SocketDomain::Inet, SocketType::Stream).unwrap();
        
        let addr = SocketAddr::new(Ipv4Address::new(127, 0, 0, 1), 8080);
        assert!(table.bind(id, addr).is_ok());
        
        let socket = table.get(id).unwrap();
        assert_eq!(socket.local_addr, Some(addr));
    }
    
    #[test_case]
    fn test_socket_listen() {
        let mut table = SocketTable::new();
        let id = table.socket(SocketDomain::Inet, SocketType::Stream).unwrap();
        
        let addr = SocketAddr::new(Ipv4Address::new(127, 0, 0, 1), 8080);
        table.bind(id, addr).unwrap();
        
        assert!(table.listen(id, 10).is_ok());
        
        let socket = table.get(id).unwrap();
        assert!(socket.listening);
        assert_eq!(socket.backlog, 10);
    }
}
