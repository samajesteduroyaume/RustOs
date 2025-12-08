#![no_std]
#![no_main]

use core::panic::PanicInfo;
pub extern "C" fn _start() -> ! {
    // Initialize UART 16550 at 0x3F8 (COM1)
    unsafe {
        // Create port
        let port_base: u16 = 0x3F8;
        
        // Helper function to write to port
        fn outb(port: u16, value: u8) {
            unsafe {
                core::arch::asm!(
                    "out dx, al",
                    in("dx") port,
                    in("al") value,
                    options(nomem, nostack, preserves_flags)
                );
            }
        }
        
        // Initialize UART
        outb(port_base + 1, 0x00);    // Disable all interrupts
        outb(port_base + 3, 0x80);    // Enable DLAB (set baud rate divisor)
        outb(port_base + 0, 0x03);    // Set divisor to 3 (lo byte) 38400 baud
        outb(port_base + 1, 0x00);    //                  (hi byte)
        outb(port_base + 3, 0x03);    // 8 bits, no parity, one stop bit
        outb(port_base + 2, 0xC7);    // Enable FIFO, clear them, with 14-byte threshold
        outb(port_base + 4, 0x0B);    // IRQs enabled, RTS/DSR set
        
        // Write message
        let message = b"*** KERNEL ALIVE ***\n";
        for &byte in message {
            // Wait for transmit to be ready
            loop {
                let mut status: u8;
                core::arch::asm!(
                    "in al, dx",
                    in("dx") port_base + 5,
                    out("al") status,
                    options(nomem, nostack, preserves_flags)
                );
                if (status & 0x20) != 0 {
                    break;
                }
            }
            
            // Write byte
            outb(port_base, byte);
        }
    }
    
    // Halt forever
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
