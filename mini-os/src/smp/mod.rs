
pub mod percpu;
pub mod trampoline;

use crate::acpi;
use crate::interrupts::apic::LocalApic;
use x86_64::registers::control::Cr3;
use core::ptr::{copy_nonoverlapping, write_volatile};

extern crate alloc;

const TRAMPOLINE_ADDR: u64 = 0x8000;

pub fn init() {
    // Detect & Boot CPUs
    if let Some(rsdp) = acpi::find_rsdp() {
        if let Some(madt) = acpi::find_madt(&rsdp) {
             let lapic_addr = madt.local_apic_address as u64;
             let mut bootstrap_lapic = LocalApic::new(lapic_addr);
             bootstrap_lapic.enable();
             
             percpu::register_cpu(bootstrap_lapic.id());
             
             // Copy trampoline code
             let code = trampoline::get_trampoline_code();
             if code.len() > 4096 {
                 panic!("Trampoline code too large!");
             }
             
             unsafe {
                 copy_nonoverlapping(code.as_ptr(), TRAMPOLINE_ADDR as *mut u8, code.len());
             }

             let madt_ptr = &madt as *const acpi::madt::Madt;
             let processors = acpi::madt::parse_madt(madt_ptr);
             
             let bsp_id = bootstrap_lapic.id();
             
             for cpu in processors {
                 if cpu.apic_id == bsp_id as u8 {
                     continue;
                 }
                 
                 crate::serial_println!("Booting CPU {} (APIC {})", cpu.processor_id, cpu.apic_id);
                 boot_ap(&mut bootstrap_lapic, cpu.apic_id, TRAMPOLINE_ADDR);
             }
        }
    }
}

fn boot_ap(lapic: &mut LocalApic, apic_id: u8, trampoline_addr: u64) {
    // 1. Prepare Data in Trampoline
    let start_offset = unsafe { &trampoline::trampoline_start as *const _ as u64 };
    let pml4_offset = unsafe { &trampoline::pml4_ptr as *const _ as u64 } - start_offset;
    let stack_offset = unsafe { &trampoline::stack_ptr as *const _ as u64 } - start_offset;
    let entry_offset = unsafe { &trampoline::entry_ptr as *const _ as u64 } - start_offset;
    
    let (pml4_frame, _) = Cr3::read();
    let pml4_addr = pml4_frame.start_address().as_u64();
    
    // Allocate stack (Quick hack: using array or vec leak)
    let stack_size = 4096 * 4;
    let stack = alloc::vec![0u8; stack_size];
    let stack_ptr = unsafe { stack.as_ptr().add(stack_size) as u64 };
    core::mem::forget(stack); // Leak stack so it lives forever
    
    unsafe {
        write_volatile((trampoline_addr + pml4_offset) as *mut u32, pml4_addr as u32); // Lower 32 bits
        write_volatile((trampoline_addr + stack_offset) as *mut u64, stack_ptr);
        write_volatile((trampoline_addr + entry_offset) as *mut u64, ap_entry as *const () as u64);
    }
    
    // 2. Clear APIC errors
    lapic.clear_error();
    
    // 3. Send INIT
    lapic.send_init(apic_id as u32);
    // Wait 10ms
    spin_wait(1000000);
    
    // 4. Send SIPI
    let vector = (trampoline_addr >> 12) as u8;
    lapic.send_sipi(apic_id as u32, vector);
     // Wait 200us
    spin_wait(10000);
}

#[no_mangle]
pub extern "C" fn ap_entry() -> ! {
    // Initialiser les segments, GDT, IDT pour ce CPU
    crate::interrupts::init_idt();
    // crate::gdt::init(); // TODO: Need per-cpu GDT or shared?
    
    // Enable LAPIC
    let lapic = LocalApic::new(0xFEE00000);
    lapic.enable();
    
    let id = lapic.id();
    percpu::register_cpu(id);
    
    crate::serial_println!("Hello from CPU APIC ID: {}", id);
    
    crate::serial_println!("Hello from CPU APIC ID: {}", id);
    
    // Start scheduling on this AP
    crate::scheduler::SCHEDULER.run();
}

fn spin_wait(count: u64) {
    for _ in 0..count {
        core::hint::spin_loop();
    }
}
