/* Linker script pour test-kernel avec support Multiboot2 */

ENTRY(_start)

SECTIONS
{
  . = 0x100000; /* charge le noyau à 1 MiB */

  . = ALIGN(0x1000);
  /* Limine requests section */
  .requests : {
    KEEP(*(.requests_start_marker))
    KEEP(*(.requests))
    KEEP(*(.requests_end_marker))
  }

  .text : {
    *(.text._start)
    *(.text*)
  }
  . = ALIGN(0x1000);

  . = ALIGN(0x1000);
  .trampoline : {
    KEEP(*(.trampoline))
  }

  . = ALIGN(0x1000);
  .rodata : { *(.rodata*) }

  . = ALIGN(0x1000);
  .data : { *(.data*) }

  . = ALIGN(0x1000);
  .bss : { *(.bss*) }
}
