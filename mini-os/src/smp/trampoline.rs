
use core::arch::global_asm;

// Trampoline code for AP startup
// This code is 16-bit real mode code that transitions to 64-bit long mode.
// It will be copied to a low memory address (TRAMPOLINE_ADDR).

// Variables patched by the BSP:
// - CR3 (Page Table) at offset +X
// - Stack Pointer at offset +Y
// - Entry Point at offset +Z

global_asm!(r#"
.section .trampoline, "awx"
.global trampoline_start
.global trampoline_end
.code16

trampoline_start:
    cli
    
    // 1. Setup segments
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    
    // 2. Load GDTR
    // wrapper for lgdt expects 6 bytes (2 limit + 4 base)
    // defined at bottom
    lgdt [gdt_desc - trampoline_start + 0x8000]
    
    // 3. Enable Protected Mode (CR0.PE)
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    
    // 4. Far Jump to 32-bit code (flush pipeline)
    // 0x08 is the Code Segment selector in our GDT
    ljmp $0x08, $(prot_mode - trampoline_start + 0x8000)

.code32
prot_mode:
    // 5. Setup segments for 32-bit
    mov ax, 0x10 // Data Segment
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    
    // 6. Enable PAE and PGE (CR4)
    mov eax, cr4
    or eax, 0xA0 // PAE | PGE
    mov cr4, eax
    
    // 7. Load CR3 (PML4)
    // Value patched at offset (pml4_ptr - trampoline_start)
    mov eax, [pml4_ptr - trampoline_start + 0x8000]
    mov cr3, eax
    
    // 8. Enable Long Mode (EFER MSR)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100 // LME
    wrmsr
    
    // 9. Enable Paging (CR0)
    mov eax, cr0
    or eax, 0x80000000 // PG
    mov cr0, eax
    
    // 10. Far Jump to 64-bit code
    // 0x08 is Code Segment (Long Mode) - we reuse same selector index if configured
    ljmp $0x08, $(long_mode - trampoline_start + 0x8000)

.code64
long_mode:
    // 11. Setup Stack
    // Value patched
    mov rsp, [stack_ptr - trampoline_start + 0x8000]
    
    // 12. Jump to Rust AP Entry
    // Value patched
    mov rax, [entry_ptr - trampoline_start + 0x8000]
    call rax
    
    // Should not reach
    hlt

// Data Area (aligned)
.align 8
gdt_start:
    .quad 0x0000000000000000 // Null
    .quad 0x00209A0000000000 // Code 64-bit (Exec/Read)
    .quad 0x0000920000000000 // Data 64-bit (Read/Write)
gdt_end:

gdt_desc:
    .word gdt_end - gdt_start - 1
    .long gdt_start - trampoline_start + 0x8000

.align 8
.global pml4_ptr
.global stack_ptr
.global entry_ptr

pml4_ptr:
    .long 0
    .long 0    // Padding for alignment if needed

stack_ptr:
    .quad 0

entry_ptr:
    .quad 0

trampoline_end:
"#);

extern "C" {
    pub static trampoline_start: u8;
    pub static trampoline_end: u8;
    pub static pml4_ptr: u8;
    pub static stack_ptr: u8;
    pub static entry_ptr: u8;
}

pub fn get_trampoline_code() -> &'static [u8] {
    unsafe {
        let start = &trampoline_start as *const u8;
        let end = &trampoline_end as *const u8;
        let len = (end as usize) - (start as usize);
        core::slice::from_raw_parts(start, len)
    }
}
