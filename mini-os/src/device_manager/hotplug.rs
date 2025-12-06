use crate::vga_buffer::WRITER;

#[derive(Debug, Clone)]
pub enum HotplugEvent {
    DeviceAdded(alloc::string::String),
    DeviceRemoved(alloc::string::String),
}

pub struct HotplugManager {
    events: alloc::vec::Vec<HotplugEvent>,
}

impl HotplugManager {
    pub fn new() -> Self {
        Self {
            events: alloc::vec::Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: HotplugEvent) {
        match &event {
            HotplugEvent::DeviceAdded(name) => {
                WRITER.lock().write_string(&format!("Hotplug: Périphérique ajouté: {}\n", name));
            }
            HotplugEvent::DeviceRemoved(name) => {
                WRITER.lock().write_string(&format!("Hotplug: Périphérique retiré: {}\n", name));
            }
        }
        
        self.events.push(event);
    }

    pub fn get_events(&self) -> &alloc::vec::Vec<HotplugEvent> {
        &self.events
    }
}
