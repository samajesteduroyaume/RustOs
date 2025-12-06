use spin::Mutex;
use x86_64::instructions::port::Port;
use crate::vga_buffer::WRITER;

pub struct EthernetController {
    io_base: u16,
    mac_address: [u8; 6],
}

impl EthernetController {
    pub fn new(io_base: u16) -> Self {
        WRITER.lock().write_string(&format!("Initializing Ethernet controller at {:#x}\n", io_base));
        EthernetController {
            io_base,
            mac_address: [0; 6],
        }
    }

    pub fn read_mac_address(&mut self) {
        // Implémentation simplifiée pour la lecture de l'adresse MAC
        for i in 0..6 {
            let mut port = Port::new(self.io_base + i as u16);
            self.mac_address[i] = unsafe { port.read() };
        }
    }
}

static ETHERNET: Mutex<Option<EthernetController>> = Mutex::new(None);

pub fn init() {
    // Détection et initialisation du contrôleur Ethernet
    *ETHERNET.lock() = Some(EthernetController::new(0x1000)); // Adresse d'E/S exemple
}
