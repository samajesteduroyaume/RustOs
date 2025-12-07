/// Module d'Interface Réseau
/// 
/// Gère l'interface entre le matériel (driver) et la stack réseau (sockets).

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;

use super::ethernet::{EthernetFrame, MacAddress, EtherType};
use super::ipv4::{Ipv4Packet, IpProtocol};
use super::arp::{ArpCache, ARP_CACHE, Ipv4Address, ArpPacket};
use super::socket::{SOCKET_TABLE, SocketType, SocketDomain};
use super::udp::UdpDatagram;
use super::tcp::TcpSegment;

/// Structure représentant une interface réseau
pub struct NetworkInterface {
    /// Adresse MAC de l'interface
    pub mac_address: MacAddress,
    /// Adresse IP de l'interface
    pub ip_address: Ipv4Address,
}

impl NetworkInterface {
    /// Crée une nouvelle interface
    pub fn new(mac_address: MacAddress, ip_address: Ipv4Address) -> Self {
        Self {
            mac_address,
            ip_address,
        }
    }

    /// Traite une frame Ethernet reçue
    pub fn handle_ethernet_frame(&self, frame: &EthernetFrame) {
        // Vérifier si la frame nous est destinée (ou broadcast)
        if frame.dst != self.mac_address && !frame.dst.is_broadcast() {
            return;
        }

        match frame.ether_type {
            EtherType::IPv4 => {
                if let Ok(packet) = Ipv4Packet::parse(&frame.payload) {
                    self.handle_ipv4_packet(&packet);
                }
            }
            EtherType::ARP => {
                // TODO: Gérer les paquets ARP
                if let Ok(arp_packet) = ArpPacket::parse(&frame.payload) {
                    let mut arp_cache = ARP_CACHE.lock();
                    arp_cache.insert(arp_packet.sender_ip, arp_packet.sender_mac);
                }
            }
            _ => {}
        }
    }

    /// Traite un paquet IPv4
    fn handle_ipv4_packet(&self, packet: &Ipv4Packet) {
        // Vérifier si le paquet nous est destiné
        if packet.dst != self.ip_address {
             // TODO: Forwarding si routeur? Pour l'instant on ignore.
             return;
        }

        match packet.protocol {
            IpProtocol::UDP => {
                if let Ok(dgram) = UdpDatagram::parse(&packet.payload) {
                    self.handle_udp_datagram(&dgram, packet.src);
                }
            }
            IpProtocol::TCP => {
                 if let Ok(segment) = TcpSegment::parse(&packet.payload) {
                     // TODO: Dispatch TCP
                 }
            }
            _ => {}
        }
    }

    /// Traite un datagram UDP
    fn handle_udp_datagram(&self, dgram: &UdpDatagram, src_ip: Ipv4Address) {
        let mut socket_table = SOCKET_TABLE.lock();
        
        // Chercher un socket lié à ce port
        // TODO: Optimiser la recherche (hashmap par port?)
        for (_, socket) in socket_table.sockets.iter_mut() {
            if socket.socket_type == SocketType::Datagram && socket.domain == SocketDomain::Inet {
                if let Some(local_addr) = socket.local_addr {
                    if local_addr.port == dgram.dst_port {
                        // Socket trouvé !
                        // Vérifier si connecté et si l'adresse source correspond (optionnel pour UDP mais bon pour la sécu)
                        // Pour UDP standard, on accepte tout si bound au port
                        
                        socket.udp_recv_buffer.push_back(dgram.payload.clone());
                        return;
                    }
                }
            }
        }
    }
}

// Instance globale de l'interface (pour l'exemple, normalement géré par le kernel)
lazy_static! {
    pub static ref NETWORK_INTERFACE: Mutex<Option<NetworkInterface>> = Mutex::new(None);
}

/// Initialise l'interface réseau
pub fn init(mac: MacAddress, ip: Ipv4Address) {
    let mut interface = NETWORK_INTERFACE.lock();
    *interface = Some(NetworkInterface::new(mac, ip));
}

/// Point d'entrée pour le driver réseau lors de la réception d'un paquet
pub fn on_receive(data: &[u8]) {
    if let Ok(frame) = EthernetFrame::parse(data) {
        if let Some(interface) = NETWORK_INTERFACE.lock().as_ref() {
            interface.handle_ethernet_frame(&frame);
        }
    }
}
