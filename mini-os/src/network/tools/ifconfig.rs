use crate::network::{IpAddr, Netmask, NetworkError};
use crate::vga_buffer::WRITER;

/// Information d'interface réseau
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub name: &'static str,
    pub mac_address: [u8; 6],
    pub ip_addr: IpAddr,
    pub netmask: Netmask,
    pub mtu: u16,
    pub rx_packets: u64,
    pub rx_bytes: u64,
    pub rx_errors: u64,
    pub tx_packets: u64,
    pub tx_bytes: u64,
    pub tx_errors: u64,
}

impl InterfaceInfo {
    pub fn new(name: &'static str, mac: [u8; 6], ip: IpAddr, mask: Netmask) -> Self {
        Self {
            name,
            mac_address: mac,
            ip_addr: ip,
            netmask: mask,
            mtu: 1500,
            rx_packets: 0,
            rx_bytes: 0,
            rx_errors: 0,
            tx_packets: 0,
            tx_bytes: 0,
            tx_errors: 0,
        }
    }

    pub fn get_broadcast(&self) -> IpAddr {
        let mut broadcast = [0u8; 4];
        for i in 0..4 {
            broadcast[i] = self.ip_addr.octets[i] | (!self.netmask.octets[i]);
        }
        IpAddr::from_bytes(&broadcast)
    }
}

/// Affiche les informations des interfaces réseau
pub fn ifconfig() -> Result<(), NetworkError> {
    // Créer des interfaces de test
    let eth0 = InterfaceInfo::new(
        "eth0",
        [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
        IpAddr::new(192, 168, 1, 100),
        Netmask::from_prefix(24),
    );

    let lo = InterfaceInfo::new(
        "lo",
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        IpAddr::new(127, 0, 0, 1),
        Netmask::from_prefix(8),
    );

    display_interface(&lo)?;
    display_interface(&eth0)?;

    Ok(())
}

/// Affiche les informations d'une interface
pub fn display_interface(iface: &InterfaceInfo) -> Result<(), NetworkError> {
    WRITER.lock().write_string(&format!(
        "{}: flags=UP,BROADCAST,RUNNING,MULTICAST  mtu {}\n",
        iface.name, iface.mtu
    ));

    WRITER.lock().write_string(&format!(
        "    inet {}.{}.{}.{}  netmask {}.{}.{}.{}  broadcast {}.{}.{}.{}\n",
        iface.ip_addr.octets[0], iface.ip_addr.octets[1],
        iface.ip_addr.octets[2], iface.ip_addr.octets[3],
        iface.netmask.octets[0], iface.netmask.octets[1],
        iface.netmask.octets[2], iface.netmask.octets[3],
        {
            let bc = iface.get_broadcast();
            format!("{}.{}.{}.{}", bc.octets[0], bc.octets[1], bc.octets[2], bc.octets[3])
        },
        {
            let bc = iface.get_broadcast();
            bc.octets[0]
        },
        {
            let bc = iface.get_broadcast();
            bc.octets[1]
        },
        {
            let bc = iface.get_broadcast();
            bc.octets[2]
        }
    ));

    WRITER.lock().write_string(&format!(
        "    ether {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}  txqueuelen 1000\n",
        iface.mac_address[0], iface.mac_address[1], iface.mac_address[2],
        iface.mac_address[3], iface.mac_address[4], iface.mac_address[5]
    ));

    WRITER.lock().write_string(&format!(
        "    RX packets {}  bytes {} ({} KiB)\n",
        iface.rx_packets, iface.rx_bytes, iface.rx_bytes / 1024
    ));

    WRITER.lock().write_string(&format!(
        "    RX errors {}  dropped 0  overruns 0  frame 0\n",
        iface.rx_errors
    ));

    WRITER.lock().write_string(&format!(
        "    TX packets {}  bytes {} ({} KiB)\n",
        iface.tx_packets, iface.tx_bytes, iface.tx_bytes / 1024
    ));

    WRITER.lock().write_string(&format!(
        "    TX errors {}  dropped 0 overruns 0  carrier 0  collisions 0\n\n",
        iface.tx_errors
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_interface_creation() {
        let iface = InterfaceInfo::new(
            "eth0",
            [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            IpAddr::new(192, 168, 1, 100),
            Netmask::from_prefix(24),
        );
        assert_eq!(iface.name, "eth0");
        assert_eq!(iface.mtu, 1500);
    }

    #[test_case]
    fn test_interface_broadcast() {
        let iface = InterfaceInfo::new(
            "eth0",
            [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            IpAddr::new(192, 168, 1, 100),
            Netmask::from_prefix(24),
        );
        let bc = iface.get_broadcast();
        assert_eq!(bc.octets[3], 255);
    }

    #[test_case]
    fn test_interface_stats() {
        let mut iface = InterfaceInfo::new(
            "eth0",
            [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            IpAddr::new(192, 168, 1, 100),
            Netmask::from_prefix(24),
        );
        iface.rx_packets = 1000;
        iface.rx_bytes = 500000;
        assert_eq!(iface.rx_packets, 1000);
        assert_eq!(iface.rx_bytes / 1024, 488);
    }

    #[test_case]
    fn test_loopback_interface() {
        let lo = InterfaceInfo::new(
            "lo",
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            IpAddr::new(127, 0, 0, 1),
            Netmask::from_prefix(8),
        );
        assert_eq!(lo.name, "lo");
        assert!(lo.ip_addr.is_localhost());
    }

    #[test_case]
    fn test_ifconfig_execution() {
        assert!(ifconfig().is_ok());
    }
}
