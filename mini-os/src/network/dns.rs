use super::{NetworkError, IpAddr};
use alloc::string::String;
use alloc::vec::Vec;

/// Requête DNS
#[derive(Debug, Clone)]
pub struct DnsQuery {
    pub name: String,
    pub query_type: u16,
    pub query_class: u16,
}

impl DnsQuery {
    pub fn new(name: &str, query_type: u16) -> Self {
        Self {
            name: name.into(),
            query_type,
            query_class: 1, // IN (Internet)
        }
    }
}

/// Réponse DNS
#[derive(Debug, Clone)]
pub struct DnsAnswer {
    pub name: String,
    pub answer_type: u16,
    pub answer_class: u16,
    pub ttl: u32,
    pub data: Vec<u8>,
}

impl DnsAnswer {
    pub fn new(name: &str, answer_type: u16, ttl: u32, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            answer_type,
            answer_class: 1,
            ttl,
            data,
        }
    }
}

/// Types de requête DNS
pub mod dns_type {
    pub const A: u16 = 1;      // IPv4
    pub const AAAA: u16 = 28;  // IPv6
    pub const CNAME: u16 = 5;  // Canonical Name
    pub const MX: u16 = 15;    // Mail Exchange
    pub const NS: u16 = 2;     // Name Server
    pub const PTR: u16 = 12;   // Pointer
    pub const SOA: u16 = 6;    // Start of Authority
    pub const SRV: u16 = 33;   // Service
    pub const TXT: u16 = 16;   // Text
}

/// Résolveur DNS
pub struct DnsResolver {
    pub dns_servers: [IpAddr; 2],
    pub cache: alloc::collections::BTreeMap<String, IpAddr>,
}

impl DnsResolver {
    pub fn new() -> Self {
        Self {
            dns_servers: [
                IpAddr::new(8, 8, 8, 8),     // Google DNS
                IpAddr::new(8, 8, 4, 4),     // Google DNS
            ],
            cache: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn set_dns_servers(&mut self, primary: IpAddr, secondary: IpAddr) {
        self.dns_servers[0] = primary;
        self.dns_servers[1] = secondary;
    }

    pub fn resolve(&mut self, hostname: &str) -> Result<IpAddr, NetworkError> {
        // Vérifier le cache
        if let Some(&ip) = self.cache.get(hostname) {
            return Ok(ip);
        }

        // TODO: Implémenter la résolution DNS
        // 1. Créer une requête DNS
        // 2. Envoyer la requête au serveur DNS
        // 3. Attendre la réponse
        // 4. Parser la réponse
        // 5. Mettre en cache le résultat

        Err(NetworkError::HostUnreachable)
    }

    pub fn reverse_resolve(&self, ip: IpAddr) -> Result<String, NetworkError> {
        // TODO: Implémenter la résolution inverse DNS
        Err(NetworkError::HostUnreachable)
    }

    pub fn add_to_cache(&mut self, hostname: &str, ip: IpAddr) {
        self.cache.insert(hostname.into(), ip);
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_dns_query_creation() {
        let query = DnsQuery::new("example.com", dns_type::A);
        assert_eq!(query.name, "example.com");
        assert_eq!(query.query_type, dns_type::A);
    }

    #[test_case]
    fn test_dns_resolver_creation() {
        let resolver = DnsResolver::new();
        assert_eq!(resolver.dns_servers[0].octets[0], 8);
    }

    #[test_case]
    fn test_dns_cache() {
        let mut resolver = DnsResolver::new();
        let ip = IpAddr::new(93, 184, 216, 34);
        resolver.add_to_cache("example.com", ip);
        assert!(resolver.resolve("example.com").is_ok());
    }
}
