/// Module DHCP (Dynamic Host Configuration Protocol)
/// 
/// Client DHCP basique (Discover -> Offer -> Request -> Ack)

use alloc::vec::Vec;
use super::ipv4::Ipv4Packet;
use super::udp::UdpDatagram;
use super::socket::{SocketAddr, Socket};
use super::ethernet::MacAddress;
use super::arp::Ipv4Address;
use super::interface::NETWORK_INTERFACE;

/// OpCodes DHCP
pub const DHCP_OP_BOOTREQUEST: u8 = 1;
pub const DHCP_OP_BOOTREPLY: u8 = 2;

/// Message Types DHCP (Option 53)
pub const DHCP_MSG_DISCOVER: u8 = 1;
pub const DHCP_MSG_OFFER: u8 = 2;
pub const DHCP_MSG_REQUEST: u8 = 3;
pub const DHCP_MSG_DECLINE: u8 = 4;
pub const DHCP_MSG_ACK: u8 = 5;
pub const DHCP_MSG_NAK: u8 = 6;
pub const DHCP_MSG_RELEASE: u8 = 7;
pub const DHCP_MSG_INFORM: u8 = 8;

/// Paquet DHCP (structure fixe BOOTP + options)
#[derive(Debug, Clone)]
pub struct DhcpPacket {
    pub op: u8,
    pub htype: u8,
    pub hlen: u8,
    pub hops: u8,
    pub xid: u32,
    pub secs: u16,
    pub flags: u16,
    pub ciaddr: Ipv4Address, // Client IP
    pub yiaddr: Ipv4Address, // Your IP
    pub siaddr: Ipv4Address, // Server IP
    pub giaddr: Ipv4Address, // Gateway IP
    pub chaddr: MacAddress,  // Client Hardware Address
    pub options: Vec<u8>,
    // sname et file ignorés pour simplification (0 filled)
}

impl DhcpPacket {
    pub fn new_discover(mac: MacAddress) -> Self {
        let xid = unsafe { core::arch::x86_64::_rdtsc() as u32 };
        
        let mut packet = Self {
            op: DHCP_OP_BOOTREQUEST,
            htype: 1, // Ethernet
            hlen: 6,  // MAC len
            hops: 0,
            xid,
            secs: 0,
            flags: 0x8000, // Broadcast
            ciaddr: Ipv4Address::new(0,0,0,0),
            yiaddr: Ipv4Address::new(0,0,0,0),
            siaddr: Ipv4Address::new(0,0,0,0),
            giaddr: Ipv4Address::new(0,0,0,0),
            chaddr: mac,
            options: Vec::new(),
        };
        
        // Options de base pour Discover
        packet.add_option(53, &[DHCP_MSG_DISCOVER]); // DHCP Message Type
        packet.add_option(55, &[1, 3, 6]); // Parameter Request List (Subnet Mask, Router, DNS)
        
        packet
    }
    
    pub fn new_request(mac: MacAddress, xid: u32, requested_ip: Ipv4Address, server_id: Ipv4Address) -> Self {
        let mut packet = Self {
            op: DHCP_OP_BOOTREQUEST,
            htype: 1,
            hlen: 6,
            hops: 0,
            xid,
            secs: 0,
            flags: 0x8000,
            ciaddr: Ipv4Address::new(0,0,0,0),
            yiaddr: Ipv4Address::new(0,0,0,0),
            siaddr: Ipv4Address::new(0,0,0,0),
            giaddr: Ipv4Address::new(0,0,0,0),
            chaddr: mac,
            options: Vec::new(),
        };
        
        packet.add_option(53, &[DHCP_MSG_REQUEST]);
        packet.add_option(50, &requested_ip.0); // Requested IP Address
        packet.add_option(54, &server_id.0);    // Server Identifier
        
        packet
    }
    
    pub fn add_option(&mut self, code: u8, data: &[u8]) {
        self.options.push(code);
        self.options.push(data.len() as u8);
        self.options.extend_from_slice(data);
    }
    
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(300);
        
        bytes.push(self.op);
        bytes.push(self.htype);
        bytes.push(self.hlen);
        bytes.push(self.hops);
        bytes.extend_from_slice(&self.xid.to_be_bytes());
        bytes.extend_from_slice(&self.secs.to_be_bytes());
        bytes.extend_from_slice(&self.flags.to_be_bytes());
        bytes.extend_from_slice(&self.ciaddr.0);
        bytes.extend_from_slice(&self.yiaddr.0);
        bytes.extend_from_slice(&self.siaddr.0);
        bytes.extend_from_slice(&self.giaddr.0);
        
        // chaddr (16 bytes)
        bytes.extend_from_slice(&self.chaddr.0);
        bytes.extend(core::iter::repeat(0).take(16 - 6));
        
        // sname (64) + file (128)
        bytes.extend(core::iter::repeat(0).take(192));
        
        // Magic Cookie
        bytes.extend_from_slice(&[0x63, 0x82, 0x53, 0x63]);
        
        // Options
        bytes.extend_from_slice(&self.options);
        bytes.push(0xFF); // End Option
        
        // Padding to min size (BOOTP 300 bytes)
        while bytes.len() < 300 {
            bytes.push(0);
        }
        
        bytes
    }
    
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 240 { return None; }
        
        let op = data[0];
        let htype = data[1];
        let hlen = data[2];
        let hops = data[3];
        let xid = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let secs = u16::from_be_bytes([data[8], data[9]]);
        let flags = u16::from_be_bytes([data[10], data[11]]);
        
        let ciaddr = Ipv4Address::new(data[12], data[13], data[14], data[15]);
        let yiaddr = Ipv4Address::new(data[16], data[17], data[18], data[19]);
        let siaddr = Ipv4Address::new(data[20], data[21], data[22], data[23]);
        let giaddr = Ipv4Address::new(data[24], data[25], data[26], data[27]);
        
        // chaddr
        let mut mac = [0u8; 6];
        mac.copy_from_slice(&data[28..34]);
        let chaddr = MacAddress::new(mac);
        
        // TODO: Parse options
        
        Some(Self {
            op, htype, hlen, hops, xid, secs, flags,
            ciaddr, yiaddr, siaddr, giaddr, chaddr,
            options: Vec::new(), // Options parsing simplifié
        })
    }
    
    pub fn get_option<'a>(&'a self, code: u8, raw_packet: &'a [u8]) -> Option<&'a [u8]> {
        // Parsing "on-demand" depuis le raw_packet car self.options est pas fully parsed
        let mut i = 240; // Après magic cookie
        while i < raw_packet.len() {
            let opt = raw_packet[i];
            if opt == 0xFF { break; }
            if opt == 0 { i+=1; continue; }
            
            if i+1 >= raw_packet.len() { break; }
            let len = raw_packet[i+1] as usize;
            
            if opt == code {
                if i+2+len <= raw_packet.len() {
                    return Some(&raw_packet[i+2..i+2+len]);
                }
            }
            i += 2 + len;
        }
        None
    }
}

/// Machine à états DHCP
pub enum DhcpState {
    Init,
    DiscoverSent,
    RequestSent,
    Bound,
}

pub struct DhcpClient {
    state: DhcpState,
    xid: u32,
    server_id: Option<Ipv4Address>,
    offered_ip: Option<Ipv4Address>,
}

impl DhcpClient {
    pub fn new() -> Self {
        Self {
            state: DhcpState::Init,
            xid: 0,
            server_id: None,
            offered_ip: None,
        }
    }
    
    pub fn start(&mut self) -> Result<(), ()> {
        use super::socket::{SocketDomain, SocketType, SOCKET_TABLE};
        
        // Récupérer la MAC adresse
        let mac = if let Some(ref iface) = *NETWORK_INTERFACE.lock() {
             iface.mac_address
        } else {
             return Err(());
        };
        
        // Ouvrir socket UDP
        let mut table = SOCKET_TABLE.lock();
        let socket_id = table.socket(SocketDomain::Inet, SocketType::Datagram).map_err(|_| ())?;
        
        // Bind 0.0.0.0:68
        let local_addr = SocketAddr::new(Ipv4Address::new(0,0,0,0), 68);
        table.bind(socket_id, local_addr).map_err(|_| ())?;
        
        // Connect Broadcast:67
        let remote_addr = SocketAddr::new(Ipv4Address::new(255,255,255,255), 67);
        table.connect(socket_id, remote_addr).map_err(|_| ())?;
        
        drop(table);
        
        // 1. Send DISCOVER
        let discover = DhcpPacket::new_discover(mac);
        self.xid = discover.xid;
        let bytes = discover.serialize();
        
        let mut table = SOCKET_TABLE.lock();
        table.send(socket_id, &bytes).map_err(|_| ())?;
        drop(table);
        
        self.state = DhcpState::DiscoverSent;
        
        // 2. Wait OFFER
        // Boucle de réception... (simplifié ici, devrait être async ou state machine polluée)
        // ...
        
        Ok(())
    }
    
    // Suite de la logique à implémenter...
}
