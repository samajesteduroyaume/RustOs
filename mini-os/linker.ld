OUTPUT_FORMAT("elf64-x86-64")
ENTRY(_start)

SECTIONS
{
  . = 0x100000; /* charge le noyau à 1 MiB */

  .text : {
    *(.text._start)
    *(.text*)
  }

  .rodata : { *(.rodata*) }
  .data : { *(.data*) }
  .bss : { *(.bss*) }
}
