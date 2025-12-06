use super::{Device, DeviceType, DeviceError};
use alloc::vec::Vec;
use alloc::string::String;
use crate::vga_buffer::WRITER;

/// Type de périphérique vidéo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoType {
    Monitor,
    Projector,
    TV,
    Webcam,
    HDMI,
    DisplayPort,
    VGA,
    DVI,
    Unknown,
}

/// Résolution vidéo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32, refresh_rate: u32) -> Self {
        Self {
            width,
            height,
            refresh_rate,
        }
    }

    pub fn get_aspect_ratio(&self) -> (u32, u32) {
        let gcd = Self::gcd(self.width, self.height);
        (self.width / gcd, self.height / gcd)
    }

    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    pub fn get_pixels(&self) -> u64 {
        (self.width as u64) * (self.height as u64)
    }
}

/// Périphérique vidéo
#[derive(Debug, Clone)]
pub struct VideoDevice {
    pub name: String,
    pub device_type: VideoType,
    pub resolutions: Vec<Resolution>,
    pub current_resolution: Resolution,
    pub color_depth: u8,
    pub driver: String,
    pub connected: bool,
    pub powered: bool,
}

impl VideoDevice {
    pub fn new(name: &str, device_type: VideoType) -> Self {
        Self {
            name: name.into(),
            device_type,
            resolutions: Vec::new(),
            current_resolution: Resolution::new(1920, 1080, 60),
            color_depth: 24,
            driver: "nouveau".into(),
            connected: false,
            powered: false,
        }
    }

    pub fn add_resolution(&mut self, resolution: Resolution) {
        self.resolutions.push(resolution);
    }

    pub fn set_resolution(&mut self, resolution: Resolution) -> Result<(), DeviceError> {
        if self.resolutions.contains(&resolution) {
            self.current_resolution = resolution;
            WRITER.lock().write_string(&format!(
                "Résolution changée à {}x{}@{} Hz\n",
                resolution.width, resolution.height, resolution.refresh_rate
            ));
            Ok(())
        } else {
            Err(DeviceError::NotSupported)
        }
    }

    pub fn get_max_resolution(&self) -> Option<Resolution> {
        self.resolutions.iter().max_by_key(|r| r.get_pixels()).copied()
    }

    pub fn power_on(&mut self) {
        self.powered = true;
        self.connected = true;
    }

    pub fn power_off(&mut self) {
        self.powered = false;
    }
}

impl Device for VideoDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Video
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Initialisation Vidéo: {} ({}x{}@{} Hz, {} bits)\n",
            self.name,
            self.current_resolution.width,
            self.current_resolution.height,
            self.current_resolution.refresh_rate,
            self.color_depth
        ));
        self.power_on();
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!("Arrêt Vidéo: {}\n", self.name));
        self.power_off();
        Ok(())
    }
}

/// Adaptateur vidéo
#[derive(Debug, Clone)]
pub struct VideoAdapter {
    pub name: String,
    pub devices: Vec<VideoDevice>,
    pub vram: u64,
}

impl VideoAdapter {
    pub fn new(name: &str, vram: u64) -> Self {
        Self {
            name: name.into(),
            devices: Vec::new(),
            vram,
        }
    }

    pub fn add_device(&mut self, device: VideoDevice) {
        self.devices.push(device);
    }

    pub fn get_connected_devices(&self) -> Vec<&VideoDevice> {
        self.devices.iter().filter(|d| d.connected).collect()
    }

    pub fn get_powered_devices(&self) -> Vec<&VideoDevice> {
        self.devices.iter().filter(|d| d.powered).collect()
    }
}

/// Énumérateur vidéo
pub struct VideoEnumerator;

impl VideoEnumerator {
    pub fn enumerate() -> Result<Vec<VideoAdapter>, DeviceError> {
        let mut adapters = Vec::new();

        // Créer un adaptateur vidéo
        let mut adapter = VideoAdapter::new("NVIDIA GeForce RTX 3060", 12 * 1024 * 1024 * 1024); // 12 GB VRAM

        // Créer un moniteur
        let mut monitor = VideoDevice::new("HDMI-1", VideoType::Monitor);
        monitor.add_resolution(Resolution::new(1920, 1080, 60));
        monitor.add_resolution(Resolution::new(1920, 1080, 144));
        monitor.add_resolution(Resolution::new(2560, 1440, 60));
        monitor.add_resolution(Resolution::new(3840, 2160, 30));
        monitor.connected = true;
        adapter.add_device(monitor);

        // Créer un second moniteur
        let mut monitor2 = VideoDevice::new("DisplayPort-1", VideoType::Monitor);
        monitor2.add_resolution(Resolution::new(1920, 1080, 60));
        monitor2.add_resolution(Resolution::new(2560, 1440, 144));
        monitor2.add_resolution(Resolution::new(3840, 2160, 60));
        monitor2.connected = false;
        adapter.add_device(monitor2);

        adapters.push(adapter);
        Ok(adapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_resolution_creation() {
        let res = Resolution::new(1920, 1080, 60);
        assert_eq!(res.width, 1920);
        assert_eq!(res.height, 1080);
    }

    #[test_case]
    fn test_resolution_aspect_ratio() {
        let res = Resolution::new(1920, 1080, 60);
        let (w, h) = res.get_aspect_ratio();
        assert_eq!(w, 16);
        assert_eq!(h, 9);
    }

    #[test_case]
    fn test_video_device_creation() {
        let device = VideoDevice::new("HDMI-1", VideoType::Monitor);
        assert_eq!(device.name, "HDMI-1");
        assert_eq!(device.device_type, VideoType::Monitor);
    }

    #[test_case]
    fn test_video_enumerator() {
        let adapters = VideoEnumerator::enumerate().unwrap();
        assert!(adapters.len() > 0);
        assert!(adapters[0].devices.len() > 0);
    }
}

