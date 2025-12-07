/// Module GPU - Drivers Graphiques
/// 
/// Drivers VGA/VESA et primitives de dessin

pub mod vga;
pub mod vesa;
pub mod primitives;

pub use vga::{VGA_WRITER, VgaWriter, Color as VgaColor};
pub use vesa::{VESA_DRIVER, VesaDriver, VesaModeInfo, Color as GRAPHICS_COLOR};
pub use primitives::{Canvas, GraphicsContext};
