use crate::network::NetworkError;
use crate::vga_buffer::WRITER;

/// Information de connexion
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub protocol: &'static str,
    pub local_addr: &'static str,
    pub remote_addr: &'static str,
    pub state: &'static str,
    pub pid: u32,
}

impl ConnectionInfo {
    pub fn new(
        protocol: &'static str,
        local: &'static str,
        remote: &'static str,
        state: &'static str,
        pid: u32,
    ) -> Self {
        Self {
            protocol,
            local_addr: local,
            remote_addr: remote,
            state,
            pid,
        }
    }
}

/// Affiche les connexions réseau
pub fn netstat() -> Result<(), NetworkError> {
    WRITER.lock().write_string("Active Internet connections (w/o servers)\n");
    WRITER.lock().write_string("Proto Recv-Q Send-Q Local Address           Foreign Address         State       PID/Program name\n");

    // Afficher les connexions TCP
    let connections = [
        ConnectionInfo::new("tcp", "192.168.1.100:22", "192.168.1.50:54321", "ESTABLISHED", 1234),
        ConnectionInfo::new("tcp", "192.168.1.100:80", "192.168.1.51:12345", "ESTABLISHED", 5678),
        ConnectionInfo::new("tcp", "0.0.0.0:22", "0.0.0.0:*", "LISTEN", 1234),
        ConnectionInfo::new("tcp", "0.0.0.0:80", "0.0.0.0:*", "LISTEN", 5678),
        ConnectionInfo::new("udp", "192.168.1.100:53", "0.0.0.0:*", "", 9012),
    ];

    for conn in &connections {
        WRITER.lock().write_string(&format!(
            "{:<5} {:<8} {:<8} {:<23} {:<23} {:<11} {}/\n",
            conn.protocol, 0, 0, conn.local_addr, conn.remote_addr, conn.state, conn.pid
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_connection_creation() {
        let conn = ConnectionInfo::new("tcp", "127.0.0.1:8080", "127.0.0.1:54321", "ESTABLISHED", 1234);
        assert_eq!(conn.protocol, "tcp");
        assert_eq!(conn.pid, 1234);
    }

    #[test_case]
    fn test_connection_listen() {
        let conn = ConnectionInfo::new("tcp", "0.0.0.0:80", "0.0.0.0:*", "LISTEN", 5678);
        assert_eq!(conn.state, "LISTEN");
    }

    #[test_case]
    fn test_connection_established() {
        let conn = ConnectionInfo::new("tcp", "192.168.1.100:22", "192.168.1.50:54321", "ESTABLISHED", 1234);
        assert_eq!(conn.state, "ESTABLISHED");
    }

    #[test_case]
    fn test_netstat_execution() {
        assert!(netstat().is_ok());
    }

    #[test_case]
    fn test_udp_connection() {
        let conn = ConnectionInfo::new("udp", "192.168.1.100:53", "0.0.0.0:*", "", 9012);
        assert_eq!(conn.protocol, "udp");
    }
}
