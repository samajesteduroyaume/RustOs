use spin::Mutex;
use crate::vga_buffer::WRITER;

// Simple RAM Filesystem
pub struct RamFs {
    files: [Option<&'static [u8]>; 16],
    names: [&'static str; 16],
}

static RAM_FS: Mutex<RamFs> = Mutex::new(RamFs {
    files: [None; 16],
    names: [""; 16],
});

impl RamFs {
    pub fn init() {
        WRITER.lock().write_string("Initializing RAM filesystem...\n");
    }

    pub fn create_file(&mut self, name: &'static str, data: &'static [u8]) -> bool {
        for i in 0..self.files.len() {
            if self.files[i].is_none() {
                self.files[i] = Some(data);
                self.names[i] = name;
                return true;
            }
        }
        false
    }

    pub fn read_file(&self, name: &str) -> Option<&'static [u8]> {
        for i in 0..self.names.len() {
            if self.names[i] == name {
                return self.files[i];
            }
        }
        None
    }
}

pub fn init() {
    RamFs::init();
}
