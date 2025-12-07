// Tests d'intégration pour RamFS
// Ce fichier teste le module RamFS indépendamment du harness global

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

// Importer le crate mini-os
use mini_os::fs::ramfs::RamFileSystemRef;
use mini_os::fs::vfs_core::{FileMode, FileType, FileSystemOps};

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    panic!("allocation error")
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Exécuter les tests RamFS
    test_ramfs_file_creation();
    test_ramfs_read_write();
    test_ramfs_mkdir();
    test_ramfs_not_found();
    
    // Succès - tous les tests sont passés
    // Si on arrive ici, tous les tests ont réussi (pas de panic)
    loop {}
}

fn test_ramfs_file_creation() {
    let fs = RamFileSystemRef::new();
    let root = fs.get_inode(1).expect("Should get root inode");
    
    let file_id = root.lock().create("test.txt", FileMode::new(0o644), FileType::Regular)
        .expect("Should create file");
    
    let found_id = root.lock().lookup("test.txt").expect("Should find file");
    assert_eq!(file_id, found_id);
}

fn test_ramfs_read_write() {
    let fs = RamFileSystemRef::new();
    let root = fs.get_inode(1).expect("Should get root inode");
    
    let file_id = root.lock().create("data.bin", FileMode::new(0o644), FileType::Regular)
        .expect("Should create file");
    
    let file_inode = fs.get_inode(file_id).expect("Should get file inode");
    
    let data = b"Hello RamFS";
    let written = file_inode.lock().write(0, data).expect("Should write");
    assert_eq!(written, data.len());
    
    let mut buf = [0u8; 20];
    let read = file_inode.lock().read(0, &mut buf).expect("Should read");
    assert_eq!(read, data.len());
    assert_eq!(&buf[..read], data);
}

fn test_ramfs_mkdir() {
    let fs = RamFileSystemRef::new();
    let root = fs.get_inode(1).expect("Should get root inode");
    
    let dir_id = root.lock().mkdir("subdir", FileMode::new(0o755)).expect("Should mkdir");
    
    let dir_inode = fs.get_inode(dir_id).expect("Should get dir inode");
    assert_eq!(dir_inode.lock().stat().expect("stat").file_type, FileType::Directory);
    
    let found_id = root.lock().lookup("subdir").expect("Should find subdir");
    assert_eq!(found_id, dir_id);
}

fn test_ramfs_not_found() {
    let fs = RamFileSystemRef::new();
    let root = fs.get_inode(1).expect("Should get root inode");
    
    let err = root.lock().lookup("nonexistent");
    assert!(err.is_err());
}
