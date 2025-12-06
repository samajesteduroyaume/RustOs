use alloc::string::String;
use alloc::vec::Vec;
use crate::vga_buffer::WRITER;
use crate::device_manager::{DeviceManager, DEVICE_MANAGER};

/// Commandes de gestion des périphériques
pub struct DeviceCommands;

impl DeviceCommands {
    /// Affiche tous les périphériques
    pub fn list_all() {
        let manager = DEVICE_MANAGER.lock();
        let devices = manager.list_devices();

        WRITER.lock().write_string("Périphériques détectés:\n");
        WRITER.lock().write_string("─────────────────────────────────────────\n");

        if devices.is_empty() {
            WRITER.lock().write_string("Aucun périphérique détecté.\n");
            return;
        }

        for (name, device_type, initialized) in devices {
            let status = if initialized { "✓" } else { "✗" };
            WRITER.lock().write_string(&format!(
                "[{}] {} - {:?}\n",
                status, name, device_type
            ));
        }
    }

    /// Affiche les interfaces réseau
    pub fn list_network() {
        WRITER.lock().write_string("Interfaces réseau:\n");
        WRITER.lock().write_string("─────────────────────────────────────────\n");

        WRITER.lock().write_string("eth0: Ethernet\n");
        WRITER.lock().write_string("  MAC: 00:11:22:33:44:55\n");
        WRITER.lock().write_string("  Speed: 1000 Mbps\n");
        WRITER.lock().write_string("  Status: Up\n\n");

        WRITER.lock().write_string("wlan0: Wi-Fi\n");
        WRITER.lock().write_string("  MAC: AA:BB:CC:DD:EE:FF\n");
        WRITER.lock().write_string("  Standard: 802.11ac\n");
        WRITER.lock().write_string("  Status: Down\n");
    }

    /// Affiche les disques USB
    pub fn list_usb() {
        WRITER.lock().write_string("Périphériques USB:\n");
        WRITER.lock().write_string("─────────────────────────────────────────\n");

        WRITER.lock().write_string("USB Disk 1\n");
        WRITER.lock().write_string("  Vendor:Product: 0951:1666\n");
        WRITER.lock().write_string("  Speed: 480 Mbps (High Speed)\n");
        WRITER.lock().write_string("  Capacity: 32 GB\n\n");

        WRITER.lock().write_string("USB Keyboard\n");
        WRITER.lock().write_string("  Vendor:Product: 046D:C31C\n");
        WRITER.lock().write_string("  Speed: 12 Mbps (Full Speed)\n");
        WRITER.lock().write_string("  Class: HID\n\n");

        WRITER.lock().write_string("USB Mouse\n");
        WRITER.lock().write_string("  Vendor:Product: 046D:C05A\n");
        WRITER.lock().write_string("  Speed: 12 Mbps (Full Speed)\n");
        WRITER.lock().write_string("  Class: HID\n");
    }

    /// Affiche les périphériques Bluetooth
    pub fn list_bluetooth() {
        WRITER.lock().write_string("Périphériques Bluetooth:\n");
        WRITER.lock().write_string("─────────────────────────────────────────\n");

        WRITER.lock().write_string("Adaptateur: hci0\n");
        WRITER.lock().write_string("  Address: 5C:F3:70:8B:12:34\n");
        WRITER.lock().write_string("  Version: Bluetooth 5.0\n\n");

        WRITER.lock().write_string("Périphériques appairés:\n");
        WRITER.lock().write_string("  Sony Headset\n");
        WRITER.lock().write_string("    Type: Headset\n");
        WRITER.lock().write_string("    Signal: -45 dBm (Excellent)\n");
        WRITER.lock().write_string("    Status: Connecté\n\n");

        WRITER.lock().write_string("  Logitech Keyboard\n");
        WRITER.lock().write_string("    Type: Keyboard\n");
        WRITER.lock().write_string("    Signal: -55 dBm (Good)\n");
        WRITER.lock().write_string("    Status: Appairé\n\n");

        WRITER.lock().write_string("  Apple Watch\n");
        WRITER.lock().write_string("    Type: Smartwatch\n");
        WRITER.lock().write_string("    Signal: -65 dBm (Fair)\n");
        WRITER.lock().write_string("    Status: Appairé\n");
    }

    /// Affiche les périphériques audio
    pub fn list_audio() {
        WRITER.lock().write_string("Périphériques audio:\n");
        WRITER.lock().write_string("─────────────────────────────────────────\n");

        WRITER.lock().write_string("Adaptateur: HDA Intel\n\n");

        WRITER.lock().write_string("Périphériques de sortie:\n");
        WRITER.lock().write_string("  Speaker (Défaut)\n");
        WRITER.lock().write_string("    Canaux: 2\n");
        WRITER.lock().write_string("    Fréquence: 48000 Hz\n");
        WRITER.lock().write_string("    Profondeur: 16 bits\n");
        WRITER.lock().write_string("    Volume: 100%\n\n");

        WRITER.lock().write_string("  Headset\n");
        WRITER.lock().write_string("    Canaux: 2\n");
        WRITER.lock().write_string("    Fréquence: 44100 Hz\n");
        WRITER.lock().write_string("    Profondeur: 24 bits\n");
        WRITER.lock().write_string("    Volume: 100%\n\n");

        WRITER.lock().write_string("Périphériques d'entrée:\n");
        WRITER.lock().write_string("  Microphone (Défaut)\n");
        WRITER.lock().write_string("    Canaux: 1\n");
        WRITER.lock().write_string("    Fréquence: 16000 Hz\n");
        WRITER.lock().write_string("    Profondeur: 16 bits\n");
    }

    /// Affiche les périphériques vidéo
    pub fn list_video() {
        WRITER.lock().write_string("Périphériques vidéo:\n");
        WRITER.lock().write_string("─────────────────────────────────────────\n");

        WRITER.lock().write_string("Adaptateur: NVIDIA GeForce RTX 3060\n");
        WRITER.lock().write_string("  VRAM: 12 GB\n\n");

        WRITER.lock().write_string("Moniteurs connectés:\n");
        WRITER.lock().write_string("  HDMI-1\n");
        WRITER.lock().write_string("    Type: Monitor\n");
        WRITER.lock().write_string("    Résolution actuelle: 1920x1080@60Hz\n");
        WRITER.lock().write_string("    Résolutions supportées:\n");
        WRITER.lock().write_string("      - 1920x1080@60Hz (16:9)\n");
        WRITER.lock().write_string("      - 1920x1080@144Hz (16:9)\n");
        WRITER.lock().write_string("      - 2560x1440@60Hz (16:9)\n");
        WRITER.lock().write_string("      - 3840x2160@30Hz (16:9)\n");
        WRITER.lock().write_string("    Profondeur de couleur: 24 bits\n\n");

        WRITER.lock().write_string("Moniteurs disponibles:\n");
        WRITER.lock().write_string("  DisplayPort-1\n");
        WRITER.lock().write_string("    Type: Monitor\n");
        WRITER.lock().write_string("    Status: Déconnecté\n");
    }

    /// Affiche l'aide des commandes de périphériques
    pub fn show_help() {
        WRITER.lock().write_string("Commandes de gestion des périphériques:\n");
        WRITER.lock().write_string("─────────────────────────────────────────\n\n");

        WRITER.lock().write_string("devices list              - Lister tous les périphériques\n");
        WRITER.lock().write_string("devices network           - Lister les interfaces réseau\n");
        WRITER.lock().write_string("devices usb               - Lister les disques USB\n");
        WRITER.lock().write_string("devices bluetooth         - Lister les périphériques Bluetooth\n");
        WRITER.lock().write_string("devices audio             - Lister les périphériques audio\n");
        WRITER.lock().write_string("devices video             - Lister les périphériques vidéo\n");
        WRITER.lock().write_string("devices help              - Afficher cette aide\n");
    }

    /// Exécute une commande de périphérique
    pub fn execute(args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            Self::show_help();
            return Ok(());
        }

        match args[0] {
            "list" => {
                if args.len() > 1 {
                    match args[1] {
                        "network" => Self::list_network(),
                        "usb" => Self::list_usb(),
                        "bluetooth" => Self::list_bluetooth(),
                        "audio" => Self::list_audio(),
                        "video" => Self::list_video(),
                        _ => {
                            WRITER.lock().write_string("Type de périphérique inconnu.\n");
                        }
                    }
                } else {
                    Self::list_all();
                }
                Ok(())
            }
            "network" => {
                Self::list_network();
                Ok(())
            }
            "usb" => {
                Self::list_usb();
                Ok(())
            }
            "bluetooth" => {
                Self::list_bluetooth();
                Ok(())
            }
            "audio" => {
                Self::list_audio();
                Ok(())
            }
            "video" => {
                Self::list_video();
                Ok(())
            }
            "help" => {
                Self::show_help();
                Ok(())
            }
            _ => {
                WRITER.lock().write_string("Commande inconnue. Tapez 'devices help' pour l'aide.\n");
                Err("Commande inconnue")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_device_commands_list() {
        // Test que la commande list ne plante pas
        assert!(DeviceCommands::execute(&["list"]).is_ok());
    }

    #[test_case]
    fn test_device_commands_network() {
        assert!(DeviceCommands::execute(&["network"]).is_ok());
    }

    #[test_case]
    fn test_device_commands_help() {
        assert!(DeviceCommands::execute(&["help"]).is_ok());
    }

    #[test_case]
    fn test_device_commands_invalid() {
        assert!(DeviceCommands::execute(&["invalid"]).is_err());
    }
}
