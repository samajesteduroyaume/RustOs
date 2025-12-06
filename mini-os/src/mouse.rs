use spin::Mutex;
use x86_64::instructions::port::Port;
use crate::vga_buffer::WRITER;

static MOUSE_BYTE: Mutex<u8> = Mutex::new(0);

pub fn init_mouse() {
    WRITER.lock().write_string("Initializing PS/2 mouse...\n");
    // TODO: Implémenter l'initialisation de la souris
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: x86_64::structures::idt::InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let byte = unsafe { port.read() };
    *MOUSE_BYTE.lock() = byte;
}
