use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use lazy_static::lazy_static;
use crate::keyboard::keyboard_interrupt_handler;
use crate::vga_buffer::WRITER;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // x86_64 0.15 utilise des méthodes directes au lieu de l'indexation
        unsafe {
            idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
            idt.page_fault.set_handler_fn(page_fault_handler);
        }
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    WRITER.lock().write_string("General Protection Fault!\n");
    panic!("GPF");
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    WRITER.lock().write_string("Page fault!\n");
    let cr2 = Cr2::read();
    WRITER.lock().write_string(&format!("Accessed Address: {:?}\n", cr2));
    
    // TODO: Implémenter la gestion CoW
    panic!("Page fault non géré");
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Keyboard = 33,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
