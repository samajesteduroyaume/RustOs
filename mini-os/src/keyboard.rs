use x86_64::structures::idt::InterruptStackFrame;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use spin::Mutex;
use lazy_static::lazy_static;
use crate::vga_buffer::WRITER;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(c) => {
                    WRITER.lock().write_byte(c as u8);
                }
                DecodedKey::RawKey(code) => {
                    match code {
                        // KeyCode::F11 => mini_os::power::reboot(),
                        // KeyCode::F12 => mini_os::power::shutdown(),
                        _ => {}
                    }
                }
            }
        }
    }

    // EOI pour le LAPIC
    crate::interrupts::apic::signal_eoi();
}
