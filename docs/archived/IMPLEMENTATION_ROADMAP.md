# 🗺️ Feuille de Route d'Implémentation - Pile Logicielle

## 📅 Calendrier Proposé

### Mois 1 : Fondations
```
Semaine 1-2 : Shell & Terminal
├── Parser de commandes
├── Builtins de base (cd, pwd, ls, echo, cat)
├── Édition de ligne
└── Historique

Semaine 3-4 : Librairie Standard (libc)
├── stdio (printf, fprintf, sprintf)
├── stdlib (malloc, free, exit)
├── string (strlen, strcpy, strcmp, memcpy)
└── Appels système de base
```

### Mois 2 : Drivers & Matériel
```
Semaine 5-6 : Drivers Matériels
├── Amélioration VGA
├── Amélioration Clavier
├── Driver Disque (ATA/SATA)
└── Gestionnaire de Drivers

Semaine 7-8 : Intégration
├── Shell + Drivers
├── Système de fichiers + Drivers
└── Tests et débogage
```

### Mois 3 : Réseau
```
Semaine 9-10 : Couches Basses
├── Ethernet (MAC)
├── ARP (Résolution d'adresses)
└── IPv4 (Routage)

Semaine 11-12 : Couches Hautes
├── ICMP (Ping)
├── UDP (Datagrams)
├── TCP (Connexions)
└── DNS (Résolution de noms)

Semaine 13-14 : Utilitaires
├── ping
├── ifconfig
├── netstat
└── ip
```

### Mois 4 : Intégration & Optimisation
```
Semaine 15-16 : Tests Complets
├── Tests unitaires
├── Tests d'intégration
├── Tests de performance
└── Optimisations
```

---

## 🎯 Objectifs par Phase

### Phase 1 : Shell Minimal (Semaine 1-2)

#### Objectifs
- [ ] Parser de commandes simple
- [ ] Exécution de commandes
- [ ] Redirection stdin/stdout
- [ ] 10+ commandes builtins

#### Commandes à Implémenter
```bash
cd <dir>          # Changer de répertoire
pwd               # Afficher le répertoire courant
ls [dir]          # Lister les fichiers
echo <text>       # Afficher du texte
cat <file>        # Afficher le contenu d'un fichier
mkdir <dir>       # Créer un répertoire
rm <file>         # Supprimer un fichier
cp <src> <dst>    # Copier un fichier
mv <src> <dst>    # Déplacer un fichier
exit              # Quitter le shell
help              # Afficher l'aide
```

#### Exemple de Code
```rust
// src/shell/mod.rs
pub struct Shell {
    current_dir: String,
    env_vars: HashMap<String, String>,
    history: Vec<String>,
}

impl Shell {
    pub fn run(&mut self) -> ! {
        loop {
            print!("{}> ", self.current_dir);
            let input = self.read_line();
            
            match self.parse_command(&input) {
                Ok(cmd) => {
                    if let Err(e) = self.execute(cmd) {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
    }
    
    fn parse_command(&self, input: &str) -> Result<Command, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }
        
        Ok(Command {
            program: parts[0].to_string(),
            args: parts[1..].iter().map(|s| s.to_string()).collect(),
            stdin: None,
            stdout: None,
            stderr: None,
            pipes: Vec::new(),
        })
    }
}
```

---

### Phase 2 : Librairie Standard (Semaine 3-4)

#### Objectifs
- [ ] Fonctions stdio (printf, fprintf, sprintf)
- [ ] Fonctions stdlib (malloc, free, exit)
- [ ] Fonctions string (strlen, strcpy, strcmp, memcpy)
- [ ] Fonctions math (sin, cos, sqrt)
- [ ] Fonctions time (time, sleep)

#### Exemple de Code
```rust
// src/libc/stdio.rs
pub fn printf(format: &str, args: &[&dyn std::fmt::Display]) -> i32 {
    let output = format_string(format, args);
    crate::vga_buffer::WRITER.lock().write_string(&output);
    output.len() as i32
}

pub fn fprintf(fd: i32, format: &str, args: &[&dyn std::fmt::Display]) -> i32 {
    let output = format_string(format, args);
    // Écrire vers le descripteur de fichier
    output.len() as i32
}

// src/libc/stdlib.rs
pub fn malloc(size: usize) -> *mut u8 {
    // Allouer de la mémoire
    unsafe { alloc::alloc::alloc(Layout::from_size_align_unchecked(size, 8)) }
}

pub fn free(ptr: *mut u8) {
    // Libérer la mémoire
    unsafe { alloc::alloc::dealloc(ptr, Layout::from_size_align_unchecked(1, 8)) }
}

// src/libc/string.rs
pub fn strlen(s: &str) -> usize {
    s.len()
}

pub fn strcmp(s1: &str, s2: &str) -> i32 {
    if s1 < s2 { -1 } else if s1 > s2 { 1 } else { 0 }
}

pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe { core::ptr::copy_nonoverlapping(src, dest, n) };
    dest
}
```

---

### Phase 3 : Drivers Matériels (Semaine 5-6)

#### Objectifs
- [ ] Driver VGA amélioré
- [ ] Driver Clavier amélioré
- [ ] Driver Disque (ATA/SATA)
- [ ] Gestionnaire de Drivers

#### Exemple de Code
```rust
// src/drivers/disk.rs
pub struct DiskDriver {
    sectors: u64,
    sector_size: u16,
}

impl DiskDriver {
    pub fn new() -> Self {
        Self {
            sectors: 0,
            sector_size: 512,
        }
    }
    
    pub fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), DiskError> {
        if buffer.len() < self.sector_size as usize {
            return Err(DiskError::BufferTooSmall);
        }
        
        // Lire le secteur depuis le disque
        // 1. Envoyer la commande au contrôleur ATA
        // 2. Attendre la fin de la lecture
        // 3. Copier les données dans le buffer
        
        Ok(())
    }
    
    pub fn write_sector(&mut self, sector: u64, data: &[u8]) -> Result<(), DiskError> {
        if data.len() != self.sector_size as usize {
            return Err(DiskError::InvalidSize);
        }
        
        // Écrire le secteur sur le disque
        Ok(())
    }
}

// src/drivers/manager.rs
pub struct DriverManager {
    drivers: HashMap<String, Box<dyn Driver>>,
}

impl DriverManager {
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }
    
    pub fn register_driver(&mut self, name: &str, driver: Box<dyn Driver>) {
        self.drivers.insert(name.to_string(), driver);
    }
    
    pub fn init_all_drivers(&mut self) {
        for (name, driver) in &mut self.drivers {
            match driver.init() {
                Ok(_) => println!("Driver {} initialized", name),
                Err(e) => eprintln!("Failed to initialize driver {}: {:?}", name, e),
            }
        }
    }
}

pub trait Driver {
    fn name(&self) -> &str;
    fn init(&mut self) -> Result<(), DriverError>;
    fn handle_interrupt(&mut self, irq: u8);
}
```

---

### Phase 4 : Réseau (Semaine 9-12)

#### Objectifs
- [ ] Ethernet (MAC)
- [ ] ARP (Résolution d'adresses)
- [ ] IPv4 (Routage)
- [ ] ICMP (Ping)
- [ ] UDP (Datagrams)
- [ ] TCP (Connexions)
- [ ] DNS (Résolution de noms)

#### Exemple de Code
```rust
// src/network/ethernet.rs
pub struct EthernetFrame {
    pub dest_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn serialize(&self) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend_from_slice(&self.dest_mac);
        frame.extend_from_slice(&self.src_mac);
        frame.extend_from_slice(&self.ethertype.to_be_bytes());
        frame.extend_from_slice(&self.payload);
        frame
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, NetError> {
        if data.len() < 14 {
            return Err(NetError::InvalidFrame);
        }
        
        let mut dest_mac = [0u8; 6];
        let mut src_mac = [0u8; 6];
        dest_mac.copy_from_slice(&data[0..6]);
        src_mac.copy_from_slice(&data[6..12]);
        
        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        let payload = data[14..].to_vec();
        
        Ok(EthernetFrame {
            dest_mac,
            src_mac,
            ethertype,
            payload,
        })
    }
}

// src/network/ipv4.rs
pub struct Ipv4Packet {
    pub src_ip: [u8; 4],
    pub dest_ip: [u8; 4],
    pub protocol: u8,
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        
        // Version (4 bits) + IHL (4 bits)
        packet.push(0x45);
        
        // DSCP (6 bits) + ECN (2 bits)
        packet.push(0x00);
        
        // Total Length
        let total_length = 20 + self.payload.len();
        packet.extend_from_slice(&(total_length as u16).to_be_bytes());
        
        // Identification
        packet.extend_from_slice(&0u16.to_be_bytes());
        
        // Flags (3 bits) + Fragment Offset (13 bits)
        packet.extend_from_slice(&0u16.to_be_bytes());
        
        // TTL
        packet.push(64);
        
        // Protocol
        packet.push(self.protocol);
        
        // Checksum (calculé plus tard)
        packet.extend_from_slice(&0u16.to_be_bytes());
        
        // Source IP
        packet.extend_from_slice(&self.src_ip);
        
        // Destination IP
        packet.extend_from_slice(&self.dest_ip);
        
        // Payload
        packet.extend_from_slice(&self.payload);
        
        packet
    }
}

// src/network/tcp.rs
pub struct TcpSocket {
    pub state: TcpState,
    pub local_ip: [u8; 4],
    pub local_port: u16,
    pub remote_ip: [u8; 4],
    pub remote_port: u16,
    pub send_buffer: VecDeque<u8>,
    pub recv_buffer: VecDeque<u8>,
}

impl TcpSocket {
    pub fn new() -> Self {
        Self {
            state: TcpState::Closed,
            local_ip: [0, 0, 0, 0],
            local_port: 0,
            remote_ip: [0, 0, 0, 0],
            remote_port: 0,
            send_buffer: VecDeque::new(),
            recv_buffer: VecDeque::new(),
        }
    }
    
    pub fn connect(&mut self, addr: ([u8; 4], u16)) -> Result<(), NetError> {
        self.remote_ip = addr.0;
        self.remote_port = addr.1;
        self.state = TcpState::SynSent;
        
        // Envoyer un paquet SYN
        // TODO: Implémenter le handshake TCP
        
        Ok(())
    }
    
    pub fn send(&mut self, data: &[u8]) -> Result<usize, NetError> {
        if self.state != TcpState::Established {
            return Err(NetError::NotConnected);
        }
        
        self.send_buffer.extend(data);
        Ok(data.len())
    }
    
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, NetError> {
        if self.state != TcpState::Established {
            return Err(NetError::NotConnected);
        }
        
        let len = buffer.len().min(self.recv_buffer.len());
        for i in 0..len {
            buffer[i] = self.recv_buffer.pop_front().unwrap();
        }
        
        Ok(len)
    }
}
```

---

## 📊 Matrice de Dépendances

```
Shell
├── Terminal
│   └── VGA Driver
├── libc
│   └── Syscalls
└── Filesystem
    └── Driver Disque

Network Stack
├── Ethernet
│   └── Driver Réseau
├── ARP
│   └── Ethernet
├── IPv4
│   ├── Ethernet
│   └── ARP
├── ICMP
│   └── IPv4
├── UDP
│   └── IPv4
├── TCP
│   ├── IPv4
│   └── UDP
└── DNS
    └── UDP
```

---

## 🧪 Tests Proposés

### Tests Unitaires
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shell_parse_command() {
        let shell = Shell::new();
        let cmd = shell.parse_command("ls -la /home").unwrap();
        assert_eq!(cmd.program, "ls");
        assert_eq!(cmd.args, vec!["-la", "/home"]);
    }
    
    #[test]
    fn test_ethernet_serialize() {
        let frame = EthernetFrame {
            dest_mac: [0xFF; 6],
            src_mac: [0x00; 6],
            ethertype: 0x0800,
            payload: vec![1, 2, 3, 4],
        };
        let serialized = frame.serialize();
        assert_eq!(serialized.len(), 18);
    }
    
    #[test]
    fn test_ipv4_packet() {
        let packet = Ipv4Packet {
            src_ip: [192, 168, 1, 1],
            dest_ip: [192, 168, 1, 2],
            protocol: 6, // TCP
            payload: vec![],
        };
        let serialized = packet.serialize();
        assert!(serialized.len() >= 20);
    }
}
```

### Tests d'Intégration
```bash
# Test du shell
./test_shell.sh

# Test des drivers
./test_drivers.sh

# Test du réseau
./test_network.sh

# Test complet
./test_all.sh
```

---

## 📈 Métriques de Succès

### Phase 1 : Shell
- [ ] 10+ commandes builtins fonctionnelles
- [ ] Édition de ligne complète
- [ ] Historique des commandes
- [ ] Redirection stdin/stdout

### Phase 2 : libc
- [ ] 20+ fonctions implémentées
- [ ] Tests unitaires pour chaque fonction
- [ ] Compatibilité POSIX

### Phase 3 : Drivers
- [ ] VGA driver complet
- [ ] Clavier driver complet
- [ ] Disque driver fonctionnel
- [ ] Gestionnaire de drivers

### Phase 4 : Réseau
- [ ] Ping fonctionnel
- [ ] TCP/UDP fonctionnels
- [ ] DNS fonctionnel
- [ ] Utilitaires réseau (ifconfig, netstat)

---

## 🎓 Ressources d'Apprentissage

### Documentation
- [POSIX Standard](https://pubs.opengroup.org/onlinepubs/9699919799/)
- [TCP/IP Illustrated](https://en.wikipedia.org/wiki/TCP/IP_Illustrated)
- [Linux Kernel Documentation](https://www.kernel.org/doc/)
- [Rust Book](https://doc.rust-lang.org/book/)

### Outils
- `strace` - Tracer les appels système
- `tcpdump` - Analyser le trafic réseau
- `gdb` - Débogage
- `valgrind` - Détection de fuites mémoire

---

## 🎉 Conclusion

Cette feuille de route fournit un plan détaillé pour implémenter une pile logicielle complète pour RustOS sur 4 mois.

L'approche progressive permet de :
- ✅ Tester chaque composant indépendamment
- ✅ Intégrer progressivement les fonctionnalités
- ✅ Identifier et corriger les problèmes rapidement
- ✅ Optimiser les performances au fur et à mesure

---

**Auteur**: Assistant IA Cascade
**Date**: 6 Décembre 2025
**Version**: 1.0
