
use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::Mutex;
use crate::process::thread::Thread;
use x86_64::registers::model_specific::GsBase;
use x86_64::VirtAddr;

#[derive(Debug)]
pub struct PerCpuData {
    pub lapic_id: u32,
    pub current_thread: Option<Arc<Mutex<Thread>>>,
}

impl PerCpuData {
    pub fn new(lapic_id: u32) -> Self {
        Self {
            lapic_id,
            current_thread: None,
        }
    }
}

use alloc::vec::Vec;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PER_CPU_DATA: Mutex<Vec<Box<PerCpuData>>> = Mutex::new(Vec::new());
}

pub fn register_cpu(lapic_id: u32) {
    let cpu_data = Box::new(PerCpuData::new(lapic_id));
    let cpu_ptr = &*cpu_data as *const PerCpuData as u64;
    
    // Set GS Base to point to this structure
    unsafe {
        GsBase::write(VirtAddr::new(cpu_ptr));
    }
    
    PER_CPU_DATA.lock().push(cpu_data);
}

pub fn get_current_cpu_id() -> u32 {
    let cpu_ptr = GsBase::read().as_u64();
    if cpu_ptr == 0 {
        return 0; // Should not happen if registered
    }
    unsafe { (*(cpu_ptr as *const PerCpuData)).lapic_id }
}

pub fn set_current_thread(thread: Option<Arc<Mutex<Thread>>>) {
    let cpu_ptr = GsBase::read().as_u64();
    if cpu_ptr != 0 {
        unsafe { (*(cpu_ptr as *mut PerCpuData)).current_thread = thread; }
    }
}

pub fn get_current_thread() -> Option<Arc<Mutex<Thread>>> {
    let cpu_ptr = GsBase::read().as_u64();
    if cpu_ptr != 0 {
        unsafe { (*(cpu_ptr as *const PerCpuData)).current_thread.clone() }
    } else {
        None
    }
}
