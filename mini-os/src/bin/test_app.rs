#![no_std]
#![no_main]

extern crate alloc;
use mini_os::{
    net::{
        dns,
        dhcp,
        http,
        arp::Ipv4Address,
    },
};

// Point d'entrée pour le test kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_dns_compilation();
    test_dhcp_compilation();
    test_http_compilation();
    
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn test_dns_compilation() {
    // Vérifier juste que les types et fonctions sont accessibles
    // On ne lance pas vraiment la résolution car pas de réseau
    let _packet = dns::DnsPacket::new("example.com");
    // let _ = dns::resolve("example.com", Ipv4Address::new(8,8,8,8));
}

fn test_dhcp_compilation() {
    let _client = dhcp::DhcpClient::new();
}

fn test_http_compilation() {
    let _client = http::HttpClient::new();
    // let _ = http::HttpClient::get("http://example.com");
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
