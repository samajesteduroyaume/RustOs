/* Linker script pour test-kernel avec support Multiboot2 */

ENTRY(_start)

SECTIONS {
    /* Charger le kernel à 1MB (adresse standard pour multiboot) */
    . = 1M;

    /* Section multiboot2 header - DOIT être au début */
    .multiboot2 ALIGN(8) : {
        KEEP(*(.multiboot2))
    }

    /* Section text (code) */
    .text ALIGN(4K) : {
        *(.text .text.*)
    }

    /* Section rodata (données en lecture seule) */
    .rodata ALIGN(4K) : {
        *(.rodata .rodata.*)
    }

    /* Section data (données initialisées) */
    .data ALIGN(4K) : {
        *(.data .data.*)
    }

    /* Section bss (données non initialisées) */
    .bss ALIGN(4K) : {
        *(.bss .bss.*)
        *(COMMON)
    }

    /* Symboles de fin */
    _end = .;
}
