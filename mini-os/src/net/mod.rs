/// Module réseau pour RustOS
/// 
/// Stack TCP/IP complète

pub mod ethernet;
pub mod arp;
pub mod ipv4;
pub mod icmp;
pub mod udp;
pub mod tcp;
pub mod socket;
pub mod interface;
pub mod dns;
pub mod dhcp;
pub mod http;

pub use ethernet::{EthernetFrame, MacAddress, EtherType};
pub use arp::{ArpPacket, ArpCache, Ipv4Address, ARP_CACHE};
pub use ipv4::{Ipv4Packet, IpProtocol};
pub use icmp::{IcmpMessage, IcmpType};
pub use udp::{UdpDatagram, Port};
pub use tcp::{TcpSegment, TcpConnection, TcpState, TcpFlags};
pub use socket::{Socket, SocketTable, SocketAddr, SocketType, SocketDomain, SOCKET_TABLE};
