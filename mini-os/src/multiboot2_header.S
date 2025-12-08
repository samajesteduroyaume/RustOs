.section .multiboot_header
.align 8
multiboot_header_start:
    .long 0xE85250D6                # magic
    .long 0                          # architecture (i386)
    .long multiboot_header_end - multiboot_header_start  # header_length
    .long -(0xE85250D6 + 0 + (multiboot_header_end - multiboot_header_start))  # checksum
    
    # End tag
    .short 0    # type
    .short 0    # flags  
    .long 8     # size
multiboot_header_end:
