#![no_std]
#![no_main]

extern crate alloc;
use mini_os::{
    net::{
        socket::{SocketDomain, SocketType, SocketAddr, SOCKET_TABLE},
        arp::Ipv4Address,
    },
};
use alloc::vec;

// Point d'entrée pour le test kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialisation minimale (heap, etc.) - supposée faite par le bootloader/lib
    // Pour ce test, on suppose que l'allocateur fonctionne
    
    test_udp_socket();
    test_tcp_accept();
    
    // Sortie QEMU success
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn test_udp_socket() {
    let mut table = SOCKET_TABLE.lock();
    let id = table.socket(SocketDomain::Inet, SocketType::Datagram).expect("Failed to create UDP socket");
    
    let local = SocketAddr::new(Ipv4Address::new(127, 0, 0, 1), 8080);
    table.bind(id, local).expect("Failed to bind UDP socket");
    
    let remote = SocketAddr::new(Ipv4Address::new(192, 168, 1, 1), 1234);
    // Connect pour UDP définit juste l'adresse par défaut
    table.connect(id, remote).expect("Failed to connect UDP socket"); 
    
    let data = vec![1, 2, 3, 4];
    // Send doit réussir (encapsulation)
    let len = table.send(id, &data).expect("Failed to send UDP data");
    assert_eq!(len, 4);
}

fn test_tcp_accept() {
    let mut table = SOCKET_TABLE.lock();
    let id = table.socket(SocketDomain::Inet, SocketType::Stream).expect("Failed to create TCP socket");
    
    let local = SocketAddr::new(Ipv4Address::new(127, 0, 0, 1), 80);
    table.bind(id, local).expect("Failed to bind TCP socket");
    
    table.listen(id, 5).expect("Failed to listen TCP socket");
    
    // Accept doit retourner WouldBlock car pas de connexion entrante simulée
    let res = table.accept(id);
    assert!(res.is_err()); // Devrait être WouldBlock
    
    // Simuler une connexion entrante (nécessiterait accès aux champs privés ou mock)
    // Pour ce test basique, on vérifie juste que ça compile et s'exécute sans panique
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
