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
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate alloc;

use mini_os::test_runner;

mod vga_buffer;
mod interrupts;
mod keyboard;
mod mouse;
// mod memory; // Use from lib
mod hardware;
mod pci;
mod storage;
mod ethernet;
// mod process; // Use from lib
// mod scheduler; // Use from lib
// mod syscall; // Use from lib
mod sync;
// mod fs; // Use from lib
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

// Use modules from lib
use alloc::vec::Vec;
use alloc::string::ToString;
use mini_os::memory;
use mini_os::process::{self, ProcessManager, test_process};
use mini_os::scheduler::{self, Scheduler};
use mini_os::syscall;
use mini_os::fs;

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
        mini_os::memory::HYBRID_ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
    
    WRITER.lock().write_string("Tas initialisé (Hybrid: SLAB + Buddy)\n");

    // Initialiser les interruptions
    interrupts::init_idt();
    WRITER.lock().write_string("IDT initialisée\n");
    
    // Activer les interruptions
    unsafe { x86_64::instructions::interrupts::enable(); }
    WRITER.lock().write_string("Interruptions activées\n");

    // Initialiser le système de fichiers (VFS RAMFS par défaut)
    WRITER.lock().write_string("Initialisation du système de fichiers...\n");
    match mini_os::fs::init_vfs() {
        Ok(_) => {
            WRITER.lock().write_string("VFS initialisé avec succès\n");
            // Créer quelques fichiers de test
            let _ = mini_os::fs::vfs_mkdir("/home");
            let _ = mini_os::fs::vfs_write_file("/home/README.txt", b"Bienvenue sur RustOS!\nCe fichier est stocke en RAM.\n");
        },
        Err(e) => WRITER.lock().write_string(&format!("Erreur initialisation VFS: {:?}\n", e)),
    }

    // Initialiser le driver disque ATA
    WRITER.lock().write_string("Initialisation du driver disque ATA...\n");
    let mut disk = mini_os::drivers::disk::DiskDriver::new("sda", true); // Primary Master
    
    // Initialisation du disque et détection GPT
    use mini_os::drivers::Driver;
    use mini_os::gpt::parse_gpt;
    
    match disk.init() {
        Ok(_) => {
            WRITER.lock().write_string("Disque ATA initialisé.\n");
            
            // Tentative de parsing GPT
            match parse_gpt(&mut disk) {
                Ok(partitions) => {
                    WRITER.lock().write_string(&format!("Table GPT analysee. {} partitions trouvees.\n", partitions.len()));
                    
                    for (i, p) in partitions.iter().enumerate() {
                        WRITER.lock().write_string(&format!("Partition {}: LBA {} - {} ({} secteurs)\n", 
                            i, p.start_lba, p.end_lba, p.size_sectors));
                    }
                    
                    if let Some(first_partition) = partitions.first() {
                         WRITER.lock().write_string("Tentative de montage de la premiere partition (EXT2)...\n");
                         
                         // Initialiser EXT2 sur cette partition
                         // Note: EXT2::new prend possession du disque
                         match mini_os::ext2::Ext2::new(disk) {
                            Ok(fs) => {
                                WRITER.lock().write_string("Système de fichiers EXT2 initialisé avec succès!\n");
                                
                                // Lister le répertoire racine
                                // Note: EXT2 n'a pas de méthode read_dir publique dans l'implémentation actuelle
                                // On affiche juste le succès de l'initialisation
                                WRITER.lock().write_string("EXT2 monté. Opérations de fichiers disponibles.\n");
                            },
                            Err(e) => WRITER.lock().write_string(&format!("Echec init EXT2: {:?}\n", e)),
                         }
                         
                         // Note: `disk` est déplacé dans `fs`, on ne peut plus l'utiliser ici. 
                         // C'est attendu pour ce test.
                    } else {
                        WRITER.lock().write_string("Aucune partition valide trouvée.\n");
                    }
                },
                Err(e) => WRITER.lock().write_string(&format!("Erreur analyse GPT: {:?}\n", e)),
            }
        },
        Err(e) => WRITER.lock().write_string(&format!("Erreur init Disque: {:?}\n", e)),
    }

    // Initialiser le gestionnaire de processus
    // Note: Utilisation de l'instance globale
    {
        let mut process_manager = process::PROCESS_MANAGER.lock();
        
        // Créer le processus initial
        match process_manager.create_process("init", init_process, process::ProcessPriority::Normal) {
            Ok(pid) => WRITER.lock().write_string(&format!("Processus init créé avec PID: {}\n", pid)),
            Err(e) => WRITER.lock().write_string(&format!("Erreur création processus: {}\n", e)),
        }
    }
    
    WRITER.lock().write_string("Planificateur initialisé (Global)\n");
    
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
    
    // ACPI & SMP Init
    mini_os::smp::init();

    WRITER.lock().write_string("Démarrage du multitâche...\n");
    
    // Démarrer le planificateur (cette fonction ne retourne jamais)
    mini_os::scheduler::SCHEDULER.run();
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
