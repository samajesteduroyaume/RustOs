OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
  /* Start at 1MB */
  . = 1M;
  
  /* Multiboot2 header - MUST be in a LOAD segment */
  .boot : ALIGN(8) {
    KEEP(*(.multiboot_header))
    *(.text._start)
  }

  /* Main text section */
  .text : ALIGN(4K) {
    *(.text*)
  }

  /* Read-only data */
  .rodata : ALIGN(4K) {
    *(.rodata*)
  }

  /* Data */
  .data : ALIGN(4K) {
    *(.data*)
  }

  /* BSS */
  .bss : ALIGN(4K) {
    *(.bss*)
    *(COMMON)
  }

  /* Discard unwanted sections */
  /DISCARD/ : {
    *(.comment)
    *(.eh_frame)
    *(.note*)
  }
}
