use crate::network::{IpAddr, icmp::*, NetworkError};
use crate::vga_buffer::WRITER;
use alloc::string::String;

/// Statistiques de ping
#[derive(Debug, Clone)]
pub struct PingStats {
    pub packets_sent: u32,
    pub packets_received: u32,
    pub min_time: u32,
    pub max_time: u32,
    pub avg_time: u32,
    pub total_time: u64,
}

impl PingStats {
    pub fn new() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            min_time: u32::MAX,
            max_time: 0,
            avg_time: 0,
            total_time: 0,
        }
    }

    pub fn add_response(&mut self, time: u32) {
        self.packets_received += 1;
        self.total_time += time as u64;
        
        if time < self.min_time {
            self.min_time = time;
        }
        if time > self.max_time {
            self.max_time = time;
        }
        
        if self.packets_received > 0 {
            self.avg_time = (self.total_time / self.packets_received as u64) as u32;
        }
    }

    pub fn get_loss_percent(&self) -> f32 {
        if self.packets_sent == 0 {
            return 0.0;
        }
        
        let lost = self.packets_sent - self.packets_received;
        (lost as f32 / self.packets_sent as f32) * 100.0
    }
}

/// Effectue un ping vers une adresse IP
pub fn ping(target: &str, count: u32) -> Result<PingStats, NetworkError> {
    // Parser l'adresse IP
    let parts: alloc::vec::Vec<&str> = target.split('.').collect();
    if parts.len() != 4 {
        return Err(NetworkError::InvalidPacket);
    }

    let octets = [
        parts[0].parse::<u8>().unwrap_or(0),
        parts[1].parse::<u8>().unwrap_or(0),
        parts[2].parse::<u8>().unwrap_or(0),
        parts[3].parse::<u8>().unwrap_or(0),
    ];

    let target_ip = IpAddr::from_bytes(&octets);
    let mut stats = PingStats::new();

    WRITER.lock().write_string(&format!(
        "PING {} ({}.{}.{}.{}) 56(84) bytes of data.\n",
        target, octets[0], octets[1], octets[2], octets[3]
    ));

    for seq in 0..count {
        stats.packets_sent += 1;

        // Créer une requête ICMP echo
        let packet = IcmpPacket::echo_request(1, seq as u16, alloc::vec![0; 56]);
        let serialized = packet.serialize();

        // TODO: Envoyer le paquet et attendre la réponse
        // Pour l'instant, simuler une réponse
        let response_time = 10 + (seq as u32 * 2) % 5; // Simulation
        stats.add_response(response_time);

        WRITER.lock().write_string(&format!(
            "64 bytes from {}: icmp_seq={} time={} ms\n",
            target, seq + 1, response_time
        ));
    }

    // Afficher les statistiques
    WRITER.lock().write_string(&format!(
        "\n--- {} statistics ---\n",
        target
    ));

    WRITER.lock().write_string(&format!(
        "{} packets transmitted, {} received, {:.1}% packet loss\n",
        stats.packets_sent,
        stats.packets_received,
        stats.get_loss_percent()
    ));

    if stats.packets_received > 0 {
        WRITER.lock().write_string(&format!(
            "rtt min/avg/max/mdev = {}/{}/{}/{} ms\n",
            stats.min_time,
            stats.avg_time,
            stats.max_time,
            if stats.max_time > stats.min_time {
                (stats.max_time - stats.min_time) / 2
            } else {
                0
            }
        ));
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_ping_stats_creation() {
        let stats = PingStats::new();
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.packets_received, 0);
    }

    #[test_case]
    fn test_ping_stats_add_response() {
        let mut stats = PingStats::new();
        stats.packets_sent = 1;
        stats.add_response(10);
        assert_eq!(stats.packets_received, 1);
        assert_eq!(stats.min_time, 10);
        assert_eq!(stats.max_time, 10);
    }

    #[test_case]
    fn test_ping_stats_loss() {
        let mut stats = PingStats::new();
        stats.packets_sent = 4;
        stats.packets_received = 3;
        let loss = stats.get_loss_percent();
        assert!(loss > 24.0 && loss < 26.0);
    }

    #[test_case]
    fn test_ping_stats_avg() {
        let mut stats = PingStats::new();
        stats.packets_sent = 3;
        stats.add_response(10);
        stats.add_response(20);
        stats.add_response(30);
        assert_eq!(stats.avg_time, 20);
    }

    #[test_case]
    fn test_ping_stats_minmax() {
        let mut stats = PingStats::new();
        stats.packets_sent = 3;
        stats.add_response(15);
        stats.add_response(5);
        stats.add_response(25);
        assert_eq!(stats.min_time, 5);
        assert_eq!(stats.max_time, 25);
    }
}
