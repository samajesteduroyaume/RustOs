#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_unsafe)]

#[macro_use]
extern crate alloc;

mod vga_buffer;
mod interrupts;
mod keyboard;
mod mouse;
mod memory;
mod hardware;
mod pci;
mod storage;
mod ethernet;
mod process;
mod scheduler;
mod syscall;
mod sync;
mod fs;
mod shell;
mod terminal;
mod libc;
mod drivers;
mod network;
mod device_manager;

use core::panic::PanicInfo;
use core::alloc::Layout;
use alloc::sync::Arc;
use spin::Mutex;
use crate::vga_buffer::WRITER;
use crate::process::{ProcessManager, test_process};
use crate::scheduler::{Scheduler, SchedulerPolicy};

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

/// Point d'entrée du noyau
#[no_mangle]
pub extern "C" fn _start(multiboot_address: usize) -> ! {
    // Initialiser l'écran
    WRITER.lock().write_string("Mini OS Rust démarré!\n");
    
    // Détection du matériel
    hardware::detect_cpu();
    hardware::detect_memory(multiboot_address);
    hardware::scan_pci();

    // Initialiser le tas (heap)
    const HEAP_START: usize = 0x_4444_0000;
    const HEAP_SIZE: usize = 100 * 1024; // 100 KB
    
    unsafe {
        memory::ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    
    WRITER.lock().write_string("Tas initialisé\n");

    // Initialiser les interruptions
    interrupts::init_idt();
    WRITER.lock().write_string("IDT initialisée\n");
    
    // Activer les interruptions
    unsafe { x86_64::instructions::interrupts::enable(); }
    WRITER.lock().write_string("Interruptions activées\n");

    // Initialiser le gestionnaire de processus
    let mut process_manager = ProcessManager::new();
    
    // Créer le processus initial
    match process_manager.create_process("init", init_process, 1) {
        Ok(pid) => WRITER.lock().write_string(&format!("Processus init créé avec PID: {}\n", pid)),
        Err(e) => WRITER.lock().write_string(&format!("Erreur création processus: {}\n", e)),
    }
    
    // Initialiser le planificateur
    let process_manager = Arc::new(Mutex::new(process_manager));
    let mut scheduler = Scheduler::new(process_manager.clone(), SchedulerPolicy::RoundRobin);
    scheduler.set_quantum(100); // 100 ticks par processus
    
    WRITER.lock().write_string("Planificateur initialisé\n");
    
    // Initialiser le gestionnaire de périphériques
    WRITER.lock().write_string("Initialisation du gestionnaire de périphériques...\n");
    let mut device_manager = device_manager::DEVICE_MANAGER.lock();
    
    // Détecter tous les périphériques
    match device_manager.detect_all_devices() {
        Ok(_) => WRITER.lock().write_string("Détection des périphériques complétée\n"),
        Err(e) => WRITER.lock().write_string(&format!("Erreur détection périphériques: {:?}\n", e)),
    }
    
    // Initialiser tous les périphériques
    match device_manager.init_all_devices() {
        Ok(_) => WRITER.lock().write_string("Initialisation des périphériques complétée\n"),
        Err(e) => WRITER.lock().write_string(&format!("Erreur initialisation périphériques: {:?}\n", e)),
    }
    
    // Afficher les périphériques détectés
    let devices = device_manager.list_devices();
    WRITER.lock().write_string(&format!("Périphériques détectés: {}\n", devices.len()));
    
    drop(device_manager); // Libérer le verrou
    
    WRITER.lock().write_string("Démarrage du multitâche...\n");
    
    // Démarrer le planificateur (cette fonction ne retourne jamais)
    scheduler.run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    
    let mut writer = WRITER.lock();
    writer.write_string("\n\x1b[31mPANIC!\x1b[0m\n");
    writeln!(writer, "{}", info).unwrap();
    
    loop {
        x86_64::instructions::hlt();
    }
}

/// Processus d'initialisation
fn init_process() -> ! {
    WRITER.lock().write_string("Processus init démarré\n");
    
    loop {
        WRITER.lock().write_string(".");
        unsafe { x86_64::instructions::hlt(); }
    }
}
