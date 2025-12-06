# 🔌 Détection Automatique des Périphériques - RustOS v1.1.0

## 📋 Vue d'ensemble

Ce document décrit le plan d'implémentation pour la détection automatique des périphériques réseau et matériels dans RustOS v1.1.0.

---

## 🎯 Objectifs

### Détection Réseau
- ✅ Détection automatique des interfaces Ethernet
- ✅ Détection automatique des interfaces Wi-Fi
- ✅ Configuration automatique des adresses IP (DHCP)
- ✅ Gestion des multiples interfaces réseau

### Détection Matérielle
- ✅ Détection automatique des disques USB
- ✅ Détection automatique des périphériques Bluetooth
- ✅ Détection automatique des périphériques audio/vidéo
- ✅ Énumération PCI/PCIe complète

### Gestion des Périphériques
- ✅ Hotplug (insertion/retrait à chaud)
- ✅ Gestion des drivers
- ✅ Allocation des ressources (IRQ, DMA, mémoire)
- ✅ Gestion des événements

---

## 🏗️ Architecture

### Couches de Détection

```
┌─────────────────────────────────────┐
│      Applications                   │
├─────────────────────────────────────┤
│      Device Manager                 │
│  (Détection & Configuration)        │
├─────────────────────────────────────┤
│      Bus Enumerators                │
│  (PCI, USB, Bluetooth, etc.)        │
├─────────────────────────────────────┤
│      Hardware Abstraction Layer     │
│  (Accès aux registres, I/O)         │
├─────────────────────────────────────┤
│      Matériel (x86-64)              │
└─────────────────────────────────────┘
```

---

## 📡 1. Détection Réseau

### 1.1 Interfaces Ethernet

#### Détection
```rust
pub struct EthernetInterface {
    pub name: String,
    pub mac_address: [u8; 6],
    pub speed: u32,              // Mbps
    pub duplex: Duplex,          // Full/Half
    pub status: InterfaceStatus, // Up/Down
    pub driver: String,
}

pub enum Duplex {
    Half,
    Full,
    Unknown,
}

pub enum InterfaceStatus {
    Up,
    Down,
    Unknown,
}
```

#### Implémentation
- Énumération des périphériques PCI (classe 0x02)
- Lecture des registres MAC
- Détection de la vitesse de liaison
- Gestion des événements de liaison

#### Exemple
```rust
let mut device_manager = DeviceManager::new();
let ethernet_ifaces = device_manager.detect_ethernet_interfaces()?;

for iface in ethernet_ifaces {
    println!("Interface: {}", iface.name);
    println!("MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        iface.mac_address[0], iface.mac_address[1], 
        iface.mac_address[2], iface.mac_address[3],
        iface.mac_address[4], iface.mac_address[5]);
    println!("Speed: {} Mbps", iface.speed);
}
```

### 1.2 Interfaces Wi-Fi

#### Détection
```rust
pub struct WifiInterface {
    pub name: String,
    pub mac_address: [u8; 6],
    pub standard: WifiStandard,  // 802.11a/b/g/n/ac/ax
    pub bands: Vec<WifiBand>,
    pub channels: Vec<u8>,
    pub power: u8,               // dBm
    pub status: InterfaceStatus,
    pub driver: String,
}

pub enum WifiStandard {
    A,
    B,
    G,
    N,
    AC,
    AX,
    Unknown,
}

pub struct WifiBand {
    pub frequency: u32,          // MHz
    pub channels: Vec<u8>,
}
```

#### Implémentation
- Énumération des périphériques PCI (classe 0x02, sous-classe 0x80)
- Détection du standard Wi-Fi (802.11a/b/g/n/ac/ax)
- Énumération des canaux disponibles
- Gestion des événements de scan

#### Exemple
```rust
let wifi_ifaces = device_manager.detect_wifi_interfaces()?;

for iface in wifi_ifaces {
    println!("Wi-Fi Interface: {}", iface.name);
    println!("Standard: {:?}", iface.standard);
    println!("Bands: {:?}", iface.bands);
}
```

### 1.3 Configuration Automatique

#### DHCP
```rust
pub struct DhcpClient {
    pub interface: String,
    pub ip_addr: IpAddr,
    pub netmask: Netmask,
    pub gateway: IpAddr,
    pub dns_servers: [IpAddr; 2],
    pub lease_time: u32,
}

impl DhcpClient {
    pub fn discover(&mut self) -> Result<(), NetworkError>;
    pub fn request(&mut self) -> Result<(), NetworkError>;
    pub fn release(&mut self) -> Result<(), NetworkError>;
}
```

#### Exemple
```rust
let mut dhcp = DhcpClient::new("eth0");
dhcp.discover()?;
dhcp.request()?;

println!("IP: {}", dhcp.ip_addr);
println!("Gateway: {}", dhcp.gateway);
```

---

## 💾 2. Détection Matérielle

### 2.1 Disques USB

#### Détection
```rust
pub struct UsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: String,
    pub product_name: String,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub bus_number: u8,
    pub device_number: u8,
    pub port_number: u8,
    pub speed: UsbSpeed,
}

pub enum UsbSpeed {
    LowSpeed,      // 1.5 Mbps
    FullSpeed,     // 12 Mbps
    HighSpeed,     // 480 Mbps
    SuperSpeed,    // 5 Gbps
    SuperSpeedPlus, // 10 Gbps
}

pub struct UsbDisk {
    pub device: UsbDevice,
    pub capacity: u64,
    pub block_size: u32,
    pub partitions: Vec<Partition>,
}
```

#### Implémentation
- Énumération des contrôleurs USB (classe 0x0C, sous-classe 0x03)
- Énumération des périphériques USB
- Détection des disques (classe 0x08)
- Lecture de la capacité et des partitions

#### Exemple
```rust
let usb_disks = device_manager.detect_usb_disks()?;

for disk in usb_disks {
    println!("USB Disk: {}", disk.device.product_name);
    println!("Capacity: {} GB", disk.capacity / (1024 * 1024 * 1024));
    println!("Speed: {:?}", disk.device.speed);
}
```

### 2.2 Périphériques Bluetooth

#### Détection
```rust
pub struct BluetoothDevice {
    pub address: [u8; 6],
    pub name: String,
    pub device_class: u32,
    pub rssi: i8,                // Signal strength
    pub tx_power: i8,
    pub device_type: BluetoothType,
    pub paired: bool,
    pub connected: bool,
}

pub enum BluetoothType {
    Headset,
    Keyboard,
    Mouse,
    Speaker,
    Printer,
    Phone,
    Tablet,
    Unknown,
}

pub struct BluetoothAdapter {
    pub address: [u8; 6],
    pub name: String,
    pub version: u8,
    pub manufacturer: u16,
    pub devices: Vec<BluetoothDevice>,
}
```

#### Implémentation
- Énumération des adaptateurs Bluetooth
- Scan des périphériques disponibles
- Détection du type de périphérique
- Gestion de l'appairage

#### Exemple
```rust
let bt_adapters = device_manager.detect_bluetooth_adapters()?;

for adapter in bt_adapters {
    println!("Bluetooth Adapter: {}", adapter.name);
    
    let devices = adapter.scan_devices()?;
    for device in devices {
        println!("  Device: {}", device.name);
        println!("  Type: {:?}", device.device_type);
        println!("  Signal: {} dBm", device.rssi);
    }
}
```

### 2.3 Périphériques Audio/Vidéo

#### Détection Audio
```rust
pub struct AudioDevice {
    pub name: String,
    pub device_type: AudioType,
    pub channels: u8,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub driver: String,
}

pub enum AudioType {
    Microphone,
    Speaker,
    Headset,
    LineIn,
    LineOut,
    SPDIF,
    HDMI,
}

pub struct AudioAdapter {
    pub name: String,
    pub devices: Vec<AudioDevice>,
    pub default_input: Option<String>,
    pub default_output: Option<String>,
}
```

#### Détection Vidéo
```rust
pub struct VideoDevice {
    pub name: String,
    pub device_type: VideoType,
    pub resolutions: Vec<Resolution>,
    pub refresh_rates: Vec<u32>,
    pub color_depth: u8,
    pub driver: String,
}

pub enum VideoType {
    Monitor,
    Projector,
    TV,
    Webcam,
    HDMI,
    DisplayPort,
}

pub struct Resolution {
    pub width: u32,
    pub height: u32,
}
```

#### Implémentation
- Énumération des cartes son (classe 0x04)
- Énumération des cartes vidéo (classe 0x03)
- Détection des moniteurs (EDID)
- Support ALSA et PulseAudio

#### Exemple
```rust
let audio_adapters = device_manager.detect_audio_devices()?;

for adapter in audio_adapters {
    println!("Audio Adapter: {}", adapter.name);
    for device in &adapter.devices {
        println!("  Device: {}", device.name);
        println!("  Type: {:?}", device.device_type);
        println!("  Channels: {}", device.channels);
        println!("  Sample Rate: {} Hz", device.sample_rate);
    }
}

let video_devices = device_manager.detect_video_devices()?;

for device in video_devices {
    println!("Video Device: {}", device.name);
    println!("Type: {:?}", device.device_type);
    for res in &device.resolutions {
        println!("  {}x{}", res.width, res.height);
    }
}
```

---

## 🔌 3. Gestionnaire de Périphériques

### Architecture
```rust
pub struct DeviceManager {
    devices: HashMap<String, Box<dyn Device>>,
    buses: HashMap<String, Box<dyn BusEnumerator>>,
    hotplug_handlers: Vec<Box<dyn HotplugHandler>>,
}

pub trait Device: Send + Sync {
    fn name(&self) -> &str;
    fn device_type(&self) -> DeviceType;
    fn init(&mut self) -> Result<(), DeviceError>;
    fn shutdown(&mut self) -> Result<(), DeviceError>;
}

pub trait BusEnumerator: Send + Sync {
    fn name(&self) -> &str;
    fn enumerate(&self) -> Result<Vec<Box<dyn Device>>, DeviceError>;
}

pub trait HotplugHandler: Send + Sync {
    fn on_device_added(&mut self, device: &dyn Device) -> Result<(), DeviceError>;
    fn on_device_removed(&mut self, device: &dyn Device) -> Result<(), DeviceError>;
}
```

### Implémentation
```rust
impl DeviceManager {
    pub fn new() -> Self;
    
    pub fn detect_all_devices(&mut self) -> Result<(), DeviceError>;
    pub fn detect_ethernet_interfaces(&self) -> Result<Vec<EthernetInterface>, DeviceError>;
    pub fn detect_wifi_interfaces(&self) -> Result<Vec<WifiInterface>, DeviceError>;
    pub fn detect_usb_disks(&self) -> Result<Vec<UsbDisk>, DeviceError>;
    pub fn detect_bluetooth_adapters(&self) -> Result<Vec<BluetoothAdapter>, DeviceError>;
    pub fn detect_audio_devices(&self) -> Result<Vec<AudioAdapter>, DeviceError>;
    pub fn detect_video_devices(&self) -> Result<Vec<VideoDevice>, DeviceError>;
    
    pub fn register_device(&mut self, device: Box<dyn Device>) -> Result<(), DeviceError>;
    pub fn register_bus_enumerator(&mut self, enumerator: Box<dyn BusEnumerator>) -> Result<(), DeviceError>;
    pub fn register_hotplug_handler(&mut self, handler: Box<dyn HotplugHandler>) -> Result<(), DeviceError>;
    
    pub fn get_device(&self, name: &str) -> Option<&dyn Device>;
    pub fn list_devices(&self) -> Vec<&dyn Device>;
}
```

---

## 🔄 4. Hotplug (Insertion/Retrait à Chaud)

### Événements
```rust
pub enum HotplugEvent {
    DeviceAdded(Box<dyn Device>),
    DeviceRemoved(String),
    DeviceChanged(String),
}

pub struct HotplugManager {
    listeners: Vec<Box<dyn HotplugListener>>,
}

pub trait HotplugListener: Send + Sync {
    fn on_event(&mut self, event: &HotplugEvent) -> Result<(), DeviceError>;
}
```

### Implémentation
- Monitoring des événements USB
- Monitoring des événements Bluetooth
- Monitoring des événements réseau
- Gestion des ressources (allocation/libération)

---

## 📊 5. Énumération PCI/PCIe

### Structure
```rust
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
}

pub struct PciEnumerator;

impl PciEnumerator {
    pub fn enumerate() -> Result<Vec<PciDevice>, DeviceError>;
    pub fn read_config(bus: u8, slot: u8, func: u8, offset: u8) -> u32;
    pub fn write_config(bus: u8, slot: u8, func: u8, offset: u8, value: u32);
}
```

### Classes PCI
```
0x00 - Unclassified
0x01 - Mass Storage Controller
0x02 - Network Controller
0x03 - Display Controller
0x04 - Multimedia Device
0x05 - Memory Controller
0x06 - Bridge
0x07 - Simple Communication Controller
0x08 - Base System Peripheral
0x09 - Input Device
0x0A - Docking Station
0x0B - Processor
0x0C - Serial Bus Controller
0x0D - Wireless Controller
0x0E - Intelligent I/O Controller
0x0F - Satellite Communication Controller
0x10 - Encryption/Decryption Controller
0x11 - Data Acquisition and Signal Processing Controller
0x12 - Processing Accelerator
0x13 - Non-Essential Instrumentation
0xFF - Miscellaneous
```

---

## 📈 6. Plan d'Implémentation

### Phase 1 : Fondations (Semaine 1-2)
```
✓ Architecture DeviceManager
✓ Énumération PCI/PCIe
✓ Détection Ethernet
✓ Détection Wi-Fi
```

### Phase 2 : Stockage (Semaine 3-4)
```
✓ Détection USB
✓ Détection des disques
✓ Gestion des partitions
✓ Hotplug USB
```

### Phase 3 : Périphériques (Semaine 5-6)
```
✓ Détection Bluetooth
✓ Détection Audio
✓ Détection Vidéo
✓ Hotplug Bluetooth
```

### Phase 4 : Intégration (Semaine 7-8)
```
✓ Intégration avec le shell
✓ Commandes de gestion
✓ Configuration automatique
✓ Tests complets
```

---

## 🧪 7. Tests

### Tests Unitaires
```rust
#[test]
fn test_pci_enumeration() { ... }

#[test]
fn test_ethernet_detection() { ... }

#[test]
fn test_wifi_detection() { ... }

#[test]
fn test_usb_detection() { ... }

#[test]
fn test_bluetooth_detection() { ... }

#[test]
fn test_audio_detection() { ... }

#[test]
fn test_video_detection() { ... }

#[test]
fn test_hotplug_events() { ... }
```

### Tests d'Intégration
```bash
# Tester la détection
./test_device_detection.sh

# Tester le hotplug
./test_hotplug.sh

# Tester les performances
./test_performance.sh
```

---

## 📊 Statistiques Estimées

### Lignes de Code
```
DeviceManager       : 300 lignes
PCI Enumerator      : 200 lignes
Ethernet Detection  : 250 lignes
Wi-Fi Detection     : 250 lignes
USB Detection       : 300 lignes
Bluetooth Detection : 300 lignes
Audio Detection     : 250 lignes
Video Detection     : 250 lignes
Hotplug Manager     : 200 lignes
─────────────────────────────
TOTAL               : 2300 lignes
```

### Modules
```
device_manager/
├── mod.rs
├── pci.rs
├── ethernet.rs
├── wifi.rs
├── usb.rs
├── bluetooth.rs
├── audio.rs
├── video.rs
└── hotplug.rs
```

---

## 🎯 Commandes Shell

### Détection
```bash
# Lister tous les périphériques
devices list

# Lister les interfaces réseau
devices network

# Lister les disques USB
devices usb

# Lister les périphériques Bluetooth
devices bluetooth

# Lister les périphériques audio
devices audio

# Lister les périphériques vidéo
devices video
```

### Configuration
```bash
# Configurer une interface réseau
network config eth0 dhcp

# Connecter à un réseau Wi-Fi
wifi connect "SSID" "password"

# Monter un disque USB
mount /dev/usb0 /mnt/usb

# Appairer un périphérique Bluetooth
bluetooth pair "device_address"
```

---

## 🔒 Sécurité

- ✅ Validation des IDs de périphérique
- ✅ Vérification des permissions
- ✅ Isolation des ressources
- ✅ Gestion des erreurs robuste

---

## 📝 Conclusion

Ce plan d'implémentation fournit une base solide pour la détection automatique des périphériques réseau et matériels dans RustOS v1.1.0.

**Version**: RustOS v1.1.0 (Planifié)
**Date**: 6 Décembre 2025
**Statut**: 📋 Spécification Complète

