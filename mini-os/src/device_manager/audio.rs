use super::{Device, DeviceType, DeviceError};
use alloc::vec::Vec;
use alloc::string::String;
use crate::vga_buffer::WRITER;

/// Type de périphérique audio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioType {
    Microphone,
    Speaker,
    Headset,
    LineIn,
    LineOut,
    SPDIF,
    HDMI,
    USB,
    Bluetooth,
    Unknown,
}

/// Format audio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    PCM,
    AC3,
    DTS,
    MPEG,
    AAC,
    FLAC,
    Vorbis,
    Unknown,
}

/// Périphérique audio
#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub device_type: AudioType,
    pub channels: u8,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub format: AudioFormat,
    pub volume: u8,
    pub muted: bool,
    pub driver: String,
}

impl AudioDevice {
    pub fn new(name: &str, device_type: AudioType) -> Self {
        Self {
            name: name.into(),
            device_type,
            channels: 2,
            sample_rate: 48000,
            bit_depth: 16,
            format: AudioFormat::PCM,
            volume: 100,
            muted: false,
            driver: "alsa".into(),
        }
    }

    pub fn set_volume(&mut self, volume: u8) -> Result<(), DeviceError> {
        if volume > 100 {
            return Err(DeviceError::InvalidArgument);
        }
        self.volume = volume;
        Ok(())
    }

    pub fn mute(&mut self) {
        self.muted = true;
    }

    pub fn unmute(&mut self) {
        self.muted = false;
    }

    pub fn get_bitrate(&self) -> u32 {
        self.sample_rate * self.channels as u32 * self.bit_depth as u32
    }
}

impl Device for AudioDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Audio
    }

    fn init(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!(
            "Initialisation Audio: {} ({} Hz, {} bits, {} canaux)\n",
            self.name, self.sample_rate, self.bit_depth, self.channels
        ));
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DeviceError> {
        WRITER.lock().write_string(&format!("Arrêt Audio: {}\n", self.name));
        Ok(())
    }
}

/// Adaptateur audio
#[derive(Debug, Clone)]
pub struct AudioAdapter {
    pub name: String,
    pub devices: Vec<AudioDevice>,
    pub default_input: Option<String>,
    pub default_output: Option<String>,
}

impl AudioAdapter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            devices: Vec::new(),
            default_input: None,
            default_output: None,
        }
    }

    pub fn add_device(&mut self, device: AudioDevice) {
        self.devices.push(device);
    }

    pub fn get_input_devices(&self) -> Vec<&AudioDevice> {
        self.devices
            .iter()
            .filter(|d| matches!(d.device_type, AudioType::Microphone | AudioType::LineIn | AudioType::HDMI | AudioType::USB | AudioType::Bluetooth))
            .collect()
    }

    pub fn get_output_devices(&self) -> Vec<&AudioDevice> {
        self.devices
            .iter()
            .filter(|d| matches!(d.device_type, AudioType::Speaker | AudioType::Headset | AudioType::LineOut | AudioType::SPDIF | AudioType::HDMI | AudioType::USB | AudioType::Bluetooth))
            .collect()
    }

    pub fn set_default_input(&mut self, name: &str) -> Result<(), DeviceError> {
        if self.devices.iter().any(|d| d.name == name) {
            self.default_input = Some(name.into());
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn set_default_output(&mut self, name: &str) -> Result<(), DeviceError> {
        if self.devices.iter().any(|d| d.name == name) {
            self.default_output = Some(name.into());
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }
}

/// Énumérateur audio
pub struct AudioEnumerator;

impl AudioEnumerator {
    pub fn enumerate() -> Result<Vec<AudioAdapter>, DeviceError> {
        let mut adapters = Vec::new();

        // Créer un adaptateur audio
        let mut adapter = AudioAdapter::new("HDA Intel");

        // Ajouter des périphériques d'exemple
        let mut speaker = AudioDevice::new("Speaker", AudioType::Speaker);
        speaker.channels = 2;
        speaker.sample_rate = 48000;
        speaker.bit_depth = 16;
        adapter.add_device(speaker);

        let mut microphone = AudioDevice::new("Microphone", AudioType::Microphone);
        microphone.channels = 1;
        microphone.sample_rate = 16000;
        microphone.bit_depth = 16;
        adapter.add_device(microphone);

        let mut headset = AudioDevice::new("Headset", AudioType::Headset);
        headset.channels = 2;
        headset.sample_rate = 44100;
        headset.bit_depth = 24;
        adapter.add_device(headset);

        adapter.set_default_output("Speaker").ok();
        adapter.set_default_input("Microphone").ok();

        adapters.push(adapter);
        Ok(adapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_audio_device_creation() {
        let device = AudioDevice::new("Speaker", AudioType::Speaker);
        assert_eq!(device.name, "Speaker");
        assert_eq!(device.device_type, AudioType::Speaker);
    }

    #[test_case]
    fn test_audio_device_volume() {
        let mut device = AudioDevice::new("Speaker", AudioType::Speaker);
        assert!(device.set_volume(50).is_ok());
        assert_eq!(device.volume, 50);
        assert!(device.set_volume(150).is_err());
    }

    #[test_case]
    fn test_audio_device_bitrate() {
        let device = AudioDevice::new("Speaker", AudioType::Speaker);
        let bitrate = device.get_bitrate();
        assert_eq!(bitrate, 48000 * 2 * 16);
    }

    #[test_case]
    fn test_audio_enumerator() {
        let adapters = AudioEnumerator::enumerate().unwrap();
        assert!(adapters.len() > 0);
        assert!(adapters[0].devices.len() > 0);
    }
}

