/// Module Driver VGA (Video Graphics Array) - Mode Texte
/// 
/// Gestion avancée du mode texte VGA 80x25

use volatile::Volatile;
use core::fmt;
use spin::Mutex;
use x86_64::instructions::port::Port;

/// Hauteur écran (lignes)
const SCREEN_HEIGHT: usize = 25;
/// Largeur écran (colonnes)
const SCREEN_WIDTH: usize = 80;

/// Adresse mémoire VGA
const VGA_BUFFER_ADDR: usize = 0xb8000;

/// Couleurs VGA
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Code couleur complet (avant-plan | arrière-plan)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// Caractère à l'écran
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// Buffer VGA
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

/// Écrivain VGA
pub struct VgaWriter {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    crtc_addr_port: Port<u8>,
    crtc_data_port: Port<u8>,
}

impl VgaWriter {
    pub fn new() -> Self {
        Self {
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) },
            crtc_addr_port: Port::new(0x3D4),
            crtc_data_port: Port::new(0x3D5),
        }
    }

    /// Écrit un octet
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= SCREEN_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
                self.update_cursor();
            }
        }
    }

    /// Écrit une chaîne
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Caractères imprimables ASCII ou newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Autres caractères -> carré blanc
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Nouvelle ligne avec scrolling si nécessaire
    fn new_line(&mut self) {
        if self.row_position < SCREEN_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scrolling
            for row in 1..SCREEN_HEIGHT {
                for col in 0..SCREEN_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(SCREEN_HEIGHT - 1);
        }
        self.column_position = 0;
        self.update_cursor();
    }

    /// Nettoie une ligne
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..SCREEN_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Change la couleur courante
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    /// Efface l'écran
    pub fn clear_screen(&mut self) {
        for row in 0..SCREEN_HEIGHT {
            self.clear_row(row);
        }
        self.row_position = 0;
        self.column_position = 0;
        self.update_cursor();
    }

    /// Met à jour la position du curseur matériel
    fn update_cursor(&mut self) {
        let pos = (self.row_position * SCREEN_WIDTH + self.column_position) as u16;

        unsafe {
            // Octet de poids faible
            self.crtc_addr_port.write(0x0F);
            self.crtc_data_port.write((pos & 0xFF) as u8);
            
            // Octet de poids fort
            self.crtc_addr_port.write(0x0E);
            self.crtc_data_port.write(((pos >> 8) & 0xFF) as u8);
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Instance globale protégée par Mutex
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VGA_WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new());
}

/// Macro print standard
#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => ($crate::drivers::gpu::vga::_print(format_args!($($arg)*)));
}

/// Macro println standard
#[macro_export]
macro_rules! vga_println {
    () => ($crate::vga_print!("\n"));
    ($($arg:tt)*) => ($crate::vga_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Désactiver les interruptions pendant l'écriture pour éviter les deadlocks
    interrupts::without_interrupts(|| {
        VGA_WRITER.lock().write_fmt(args).unwrap();
    });
}
