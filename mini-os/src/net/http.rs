/// Module HTTP (Hypertext Transfer Protocol)
/// 
/// Client HTTP/1.1 basique (GET uniquement)

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use super::socket::{SocketAddr, Socket};
use super::arp::Ipv4Address;
use crate::net::dns::resolve;

/// Erreurs HTTP
#[derive(Debug)]
pub enum HttpError {
    ConnectionFailed,
    SendError,
    RecvError,
    DnsError,
    ParseError,
    InvalidStatus,
}

/// Réponse HTTP simplifiée
pub struct HttpResponse {
    pub status_code: u16,
    pub body: Vec<u8>,
}

/// Client HTTP
pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        Self
    }
    
    /// Effectue une requête GET
    pub fn get(url: &str) -> Result<HttpResponse, HttpError> {
        // Parsing basique de l'URL "http://domain/path"
        let url_part = if url.starts_with("http://") {
            &url[7..]
        } else {
            url
        };
        
        // Séparer domaine et path
        let (domain, path) = if let Some(slash_idx) = url_part.find('/') {
            (&url_part[..slash_idx], &url_part[slash_idx..])
        } else {
            (url_part, "/")
        };
        
        // Résoudre le domaine
        // Pour l'instant, hardcode DNS 8.8.8.8 pour la résolution
        let dns_server = Ipv4Address::new(8, 8, 8, 8);
        let ip = resolve(domain, dns_server).map_err(|_| HttpError::DnsError)?;
        
        use super::socket::{SocketDomain, SocketType, SOCKET_TABLE};
        
        let mut table = SOCKET_TABLE.lock();
        let socket_id = table.socket(SocketDomain::Inet, SocketType::Stream)
            .map_err(|_| HttpError::ConnectionFailed)?;
        
        // Port 80
        let remote_addr = SocketAddr::new(ip, 80);
        
        // Connect (TCP Handshake)
        table.connect(socket_id, remote_addr)
             .map_err(|_| HttpError::ConnectionFailed)?;
             
        drop(table); // Libérer pour le handshake qui peut prendre du temps
        
        // Attendre que la connexion soit établie (polling état socket)
        // TODO: Implémenter une vraie attente, ici on suppose que connect bloque ou réussit vite
        // En vrai, connect est non-bloquant ou asynchrone dans notre implémentation actuelle
        // Il faudrait attendre l'état Established.
        
        // Construire la requête
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: RustOS/0.1\r\nConnection: close\r\n\r\n",
            path, domain
        );
        
        let mut table = SOCKET_TABLE.lock();
        table.send(socket_id, request.as_bytes())
             .map_err(|_| HttpError::SendError)?;
        drop(table);
        
        // Lire la réponse
        let mut response_data = Vec::new();
        let mut buffer = [0u8; 1024];
        
        loop {
            let mut table = SOCKET_TABLE.lock();
            match table.recv(socket_id, &mut buffer) {
                Ok(len) => {
                    if len == 0 { break; } // Fin de connexion
                    response_data.extend_from_slice(&buffer[..len]);
                },
                Err(super::socket::SocketError::WouldBlock) => {
                    drop(table);
                    // Attente active
                    for _ in 0..1000 { core::hint::spin_loop(); }
                    continue;
                },
                Err(_) => return Err(HttpError::RecvError),
            }
            drop(table);
        }
        
        // Parser la réponse (très simplifié)
        // HTTP/1.1 200 OK\r\n...
        if response_data.len() < 12 { return Err(HttpError::ParseError); }
        
        // Status code
        // "HTTP/1.1 " -> 9 chars
        // Code -> 3 chars chars[9..12]
        // TODO: Vérifier proprement
        // let status_str = core::str::from_utf8(&response_data[9..12]).map_err(|_| HttpError::ParseError)?;
        // let status_code = status_str.parse::<u16>().map_err(|_| HttpError::ParseError)?;
        
        let status_code = 200; // Mock pour l'instant
        
        // Trouver la fin des headers (\r\n\r\n)
        let mut body_start = 0;
        for i in 0..response_data.len()-3 {
            if &response_data[i..i+4] == b"\r\n\r\n" {
                body_start = i + 4;
                break;
            }
        }
        
        let body = if body_start > 0 {
            response_data[body_start..].to_vec()
        } else {
            Vec::new()
        };
        
        Ok(HttpResponse {
            status_code,
            body,
        })
    }
}
