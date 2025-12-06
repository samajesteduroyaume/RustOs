pub mod ipv4;
pub mod icmp;
pub mod udp;
pub mod tcp;
pub mod dns;
pub mod tools;

pub use ipv4::*;
pub use icmp::*;
pub use udp::*;
pub use tcp::*;
pub use dns::*;
pub use tools::*;

/// Erreurs réseau
#[derive(Debug, Clone, Copy)]
pub enum NetworkError {
    InvalidPacket,
    InvalidChecksum,
    NoRoute,
    HostUnreachable,
    PortUnreachable,
    Timeout,
    BufferTooSmall,
}

/// Adresse IP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpAddr {
    pub octets: [u8; 4],
}

impl IpAddr {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }

    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        Self {
            octets: *bytes,
        }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.octets
    }

    pub fn is_localhost(&self) -> bool {
        self.octets[0] == 127
    }

    pub fn is_broadcast(&self) -> bool {
        self.octets == [255, 255, 255, 255]
    }

    pub fn is_multicast(&self) -> bool {
        self.octets[0] >= 224 && self.octets[0] <= 239
    }
}

/// Masque de sous-réseau
#[derive(Debug, Clone, Copy)]
pub struct Netmask {
    pub octets: [u8; 4],
}

impl Netmask {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }

    pub fn from_prefix(prefix: u8) -> Self {
        let mut octets = [0u8; 4];
        let mut bits = prefix;
        
        for i in 0..4 {
            if bits >= 8 {
                octets[i] = 0xFF;
                bits -= 8;
            } else if bits > 0 {
                octets[i] = (0xFF << (8 - bits)) as u8;
                bits = 0;
            }
        }
        
        Self { octets }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.octets
    }
}

/// Configuration réseau
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub ip_addr: IpAddr,
    pub netmask: Netmask,
    pub gateway: IpAddr,
    pub dns_servers: [IpAddr; 2],
}

impl NetworkConfig {
    pub fn new(ip: IpAddr, netmask: Netmask, gateway: IpAddr) -> Self {
        Self {
            ip_addr: ip,
            netmask,
            gateway,
            dns_servers: [
                IpAddr::new(8, 8, 8, 8),     // Google DNS
                IpAddr::new(8, 8, 4, 4),     // Google DNS
            ],
        }
    }

    pub fn get_network_addr(&self) -> IpAddr {
        let mut network = [0u8; 4];
        for i in 0..4 {
            network[i] = self.ip_addr.octets[i] & self.netmask.octets[i];
        }
        IpAddr::from_bytes(&network)
    }

    pub fn get_broadcast_addr(&self) -> IpAddr {
        let mut broadcast = [0u8; 4];
        for i in 0..4 {
            broadcast[i] = self.ip_addr.octets[i] | (!self.netmask.octets[i]);
        }
        IpAddr::from_bytes(&broadcast)
    }

    pub fn is_on_network(&self, ip: IpAddr) -> bool {
        for i in 0..4 {
            if (ip.octets[i] & self.netmask.octets[i]) != 
               (self.ip_addr.octets[i] & self.netmask.octets[i]) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_ip_addr_creation() {
        let ip = IpAddr::new(192, 168, 1, 1);
        assert_eq!(ip.octets[0], 192);
        assert_eq!(ip.octets[3], 1);
    }

    #[test_case]
    fn test_ip_addr_localhost() {
        let ip = IpAddr::new(127, 0, 0, 1);
        assert!(ip.is_localhost());
    }

    #[test_case]
    fn test_netmask_from_prefix() {
        let mask = Netmask::from_prefix(24);
        assert_eq!(mask.octets[0], 255);
        assert_eq!(mask.octets[1], 255);
        assert_eq!(mask.octets[2], 255);
        assert_eq!(mask.octets[3], 0);
    }

    #[test_case]
    fn test_network_config() {
        let ip = IpAddr::new(192, 168, 1, 100);
        let mask = Netmask::from_prefix(24);
        let gw = IpAddr::new(192, 168, 1, 1);
        let config = NetworkConfig::new(ip, mask, gw);
        
        let network = config.get_network_addr();
        assert_eq!(network.octets[0], 192);
        assert_eq!(network.octets[1], 168);
        assert_eq!(network.octets[2], 1);
        assert_eq!(network.octets[3], 0);
    }
}
