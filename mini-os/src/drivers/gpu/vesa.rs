/// Module Driver VESA - Mode Graphique
/// 
/// Gestion du Framebuffer VESA (Linear Frame Buffer)

use alloc::vec::Vec;
use spin::Mutex;

/// Information VESA Mode
#[derive(Debug, Clone, Copy)]
pub struct VesaModeInfo {
    pub width: u16,
    pub height: u16,
    pub pitch: u16,
    pub bpp: u8,        // Bits Per Pixel
    pub framebuffer: u64,
}

/// Pixel format (ARGB)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    pub fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Driver VESA
pub struct VesaDriver {
    pub mode_info: Option<VesaModeInfo>,
    buffer: Option<&'static mut [u8]>,
    back_buffer: Option<Vec<u8>>,
    double_buffering: bool,
}

impl VesaDriver {
    pub const fn new() -> Self {
        Self {
            mode_info: None,
            buffer: None,
            back_buffer: None,
            double_buffering: false,
        }
    }
    
    /// Initialise le driver avec les infos mode (fournies par Multiboot/UEFI)
    pub unsafe fn init(&mut self, info: VesaModeInfo) {
        self.mode_info = Some(info);
        
        // Mapper le framebuffer linéaire (Attention: nécessite paging configuré)
        let len = (info.pitch as usize) * (info.height as usize);
        self.buffer = Some(core::slice::from_raw_parts_mut(info.framebuffer as *mut u8, len));
        
        // Initialiser back buffer pour double buffering
        if self.double_buffering {
            let mut vec = Vec::with_capacity(len);
            vec.resize(len, 0);
            self.back_buffer = Some(vec);
        }
    }
    
    /// Active/Désactive le double buffering
    pub fn set_double_buffering(&mut self, enabled: bool) {
        self.double_buffering = enabled;
        if enabled && self.back_buffer.is_none() {
            if let Some(info) = self.mode_info {
                let len = (info.pitch as usize) * (info.height as usize);
                let mut vec = Vec::with_capacity(len);
                vec.resize(len, 0);
                self.back_buffer = Some(vec);
            }
        }
    }
    
    /// Dessine un pixel
    #[inline]
    pub fn put_pixel(&mut self, x: u16, y: u16, color: Color) {
        let info = if let Some(i) = self.mode_info { i } else { return };
        
        if x >= info.width || y >= info.height {
            return;
        }
        
        let offset = (y as usize) * (info.pitch as usize) + (x as usize) * ((info.bpp / 8) as usize);
        
        let target_buffer = if self.double_buffering {
            if let Some(ref mut bb) = self.back_buffer {
                bb.as_mut_slice()
            } else {
                return
            }
        } else {
            if let Some(ref mut fb) = self.buffer {
                fb
            } else {
                return
            }
        };
        
        // Support seulement 32bpp (ARGB) pour simplifier l'exemple
        if info.bpp == 32 {
            target_buffer[offset] = color.b;
            target_buffer[offset + 1] = color.g;
            target_buffer[offset + 2] = color.r;
            target_buffer[offset + 3] = color.a;
        }
    }
    
    /// Récupère un pixel
    pub fn get_pixel(&self, x: u16, y: u16) -> Color {
        // TODO: Implémenter lecture pixel
        Color::BLACK
    }
    
    /// Efface l'écran avec une couleur
    pub fn clear(&mut self, color: Color) {
        let info = if let Some(i) = self.mode_info { i } else { return };
        
        for y in 0..info.height {
            for x in 0..info.width {
                self.put_pixel(x, y, color);
            }
        }
    }
    
    /// Échange les buffers (Flip)
    pub fn swap_buffers(&mut self) {
        if !self.double_buffering {
            return;
        }
        
        if let (Some(fb), Some(bb)) = (&mut self.buffer, &self.back_buffer) {
            fb.copy_from_slice(bb);
        }
    }
    
    /// Largeur de l'écran
    pub fn width(&self) -> u16 {
        self.mode_info.map(|i| i.width).unwrap_or(0)
    }
    
    /// Hauteur de l'écran
    pub fn height(&self) -> u16 {
        self.mode_info.map(|i| i.height).unwrap_or(0)
    }
}

/// Instance globale
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VESA_DRIVER: Mutex<VesaDriver> = Mutex::new(VesaDriver::new());
}
