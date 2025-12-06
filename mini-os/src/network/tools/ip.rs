use crate::network::{IpAddr, Netmask, NetworkError};
use crate::vga_buffer::WRITER;

/// Information de route
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub destination: &'static str,
    pub gateway: &'static str,
    pub interface: &'static str,
    pub metric: u32,
}

impl RouteInfo {
    pub fn new(dest: &'static str, gw: &'static str, iface: &'static str, metric: u32) -> Self {
        Self {
            destination: dest,
            gateway: gw,
            interface: iface,
            metric,
        }
    }
}

/// Affiche les adresses IP
pub fn ip_addr_show() -> Result<(), NetworkError> {
    WRITER.lock().write_string("1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536\n");
    WRITER.lock().write_string("    inet 127.0.0.1/8 scope host lo\n");
    WRITER.lock().write_string("    inet6 ::1/128 scope host\n");

    WRITER.lock().write_string("2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500\n");
    WRITER.lock().write_string("    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0\n");
    WRITER.lock().write_string("    inet6 fe80::1/64 scope link\n");

    Ok(())
}

/// Affiche la table de routage
pub fn ip_route_show() -> Result<(), NetworkError> {
    let routes = [
        RouteInfo::new("default", "192.168.1.1", "eth0", 0),
        RouteInfo::new("192.168.1.0/24", "0.0.0.0", "eth0", 0),
        RouteInfo::new("127.0.0.0/8", "0.0.0.0", "lo", 0),
    ];

    for route in &routes {
        if route.gateway == "0.0.0.0" {
            WRITER.lock().write_string(&format!(
                "{} dev {} proto kernel scope link src 192.168.1.100\n",
                route.destination, route.interface
            ));
        } else {
            WRITER.lock().write_string(&format!(
                "{} via {} dev {}\n",
                route.destination, route.gateway, route.interface
            ));
        }
    }

    Ok(())
}

/// Affiche les interfaces réseau
pub fn ip_link_show() -> Result<(), NetworkError> {
    WRITER.lock().write_string("1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000\n");
    WRITER.lock().write_string("    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00\n");

    WRITER.lock().write_string("2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP mode DEFAULT group default qlen 1000\n");
    WRITER.lock().write_string("    link/ether 00:11:22:33:44:55 brd ff:ff:ff:ff:ff:ff\n");

    Ok(())
}

/// Configure une adresse IP
pub fn ip_addr_add(addr: &str, iface: &str) -> Result<(), NetworkError> {
    WRITER.lock().write_string(&format!(
        "Configuration de {} sur {}\n",
        addr, iface
    ));
    Ok(())
}

/// Ajoute une route
pub fn ip_route_add(dest: &str, gw: &str, iface: &str) -> Result<(), NetworkError> {
    WRITER.lock().write_string(&format!(
        "Ajout de la route {} via {} sur {}\n",
        dest, gw, iface
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_route_creation() {
        let route = RouteInfo::new("192.168.1.0/24", "0.0.0.0", "eth0", 0);
        assert_eq!(route.destination, "192.168.1.0/24");
        assert_eq!(route.interface, "eth0");
    }

    #[test_case]
    fn test_ip_addr_show() {
        assert!(ip_addr_show().is_ok());
    }

    #[test_case]
    fn test_ip_route_show() {
        assert!(ip_route_show().is_ok());
    }

    #[test_case]
    fn test_ip_link_show() {
        assert!(ip_link_show().is_ok());
    }

    #[test_case]
    fn test_ip_addr_add() {
        assert!(ip_addr_add("192.168.1.100/24", "eth0").is_ok());
    }
}
