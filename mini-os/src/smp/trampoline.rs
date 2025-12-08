
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
.intel_syntax noprefix
.code16

trampoline_start:
    cli
    
    // 1. Setup segments
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    
    // 2. Fixup GDT Descriptor Base
    // GDT Base = 0x8000 + (gdt_start - trampoline_start)
    // Use 32-bit registers for math to supported 32-bit relocations
    mov eax, 0x8000
    add eax, offset gdt_start
    sub eax, offset trampoline_start
    
    // Address of gdt_desc
    mov edi, 0x8000
    add edi, offset gdt_desc
    sub edi, offset trampoline_start
    
    // Store Base (Low 32 bits) into gdt_desc + 2
    // di is valid because edi < 64KB
    mov [di+2], eax
    
    // Load GDTR
    lgdt [di]
    
    // 3. Enable Protected Mode (CR0.PE)
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    
    // 4. Far Jump to 32-bit code via Stack (retf)
    mov ebx, 0x8000
    add ebx, offset prot_mode
    sub ebx, offset trampoline_start
    
    push 0x08  // CS
    push bx    // IP (16-bit from ebx)
    retf

.code32
prot_mode:
    // 5. Setup segments for 32-bit
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    
    // 6. Enable PAE and PGE (CR4)
    mov eax, cr4
    or eax, 0xA0
    mov cr4, eax
    
    // 7. Load CR3 (PML4)
    mov esi, 0x8000
    add esi, offset pml4_ptr
    sub esi, offset trampoline_start
    mov eax, [esi]
    mov cr3, eax
    
    // 8. Enable Long Mode (EFER MSR)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100
    wrmsr
    
    // 9. Enable Paging (CR0)
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
    
    // 10. Far Jump to 64-bit via Stack (retf)
    mov esi, 0x8000
    add esi, offset long_mode
    sub esi, offset trampoline_start
    
    push 0x08 // CS (32-bit push)
    push esi  // EIP (32-bit push)
    retf

.code64
long_mode:
    // 11. Setup Stack
    mov rsi, 0x8000
    add rsi, offset stack_ptr
    sub rsi, offset trampoline_start
    mov rsp, [rsi]
    
    // 12. Jump to FreeRust AP Entry
    mov rsi, 0x8000
    add rsi, offset entry_ptr
    sub rsi, offset trampoline_start
    mov rax, [rsi]
    call rax
    
    hlt

// Data Area
.align 8
gdt_start:
    .quad 0x0000000000000000
    .quad 0x00209A0000000000
    .quad 0x0000920000000000
gdt_end:

gdt_desc:
    .word gdt_end - gdt_start - 1
    .long 0 // Patched at runtime

.align 8
.global pml4_ptr
.global stack_ptr
.global entry_ptr

pml4_ptr:
    .long 0
    .long 0

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
