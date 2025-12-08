/// En-tête Multiboot2 pour GRUB
/// 
/// Ce module définit l'en-tête Multiboot2 requis par GRUB

use core::arch::global_asm;

// Constantes Multiboot2
const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
const ARCHITECTURE: u32 = 0; // i386
const HEADER_LENGTH: u32 = 24;
const CHECKSUM: u32 = 0u32.wrapping_sub(MULTIBOOT2_MAGIC + ARCHITECTURE + HEADER_LENGTH);

// En-tête Multiboot2 en assembly
global_asm!(
    ".section .multiboot_header",
    ".align 8",
    "multiboot_header_start:",
    "    .long 0xE85250D6",           // magic
    "    .long 0",                     // architecture (i386)
    "    .long multiboot_header_end - multiboot_header_start", // header_length
    "    .long -(0xE85250D6 + 0 + (multiboot_header_end - multiboot_header_start))", // checksum
    "",
    "    // End tag",
    "    .short 0",    // type
    "    .short 0",    // flags
    "    .long 8",     // size
    "multiboot_header_end:",
);
