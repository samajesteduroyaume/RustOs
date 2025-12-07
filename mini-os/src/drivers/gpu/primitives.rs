/// Module Primitives Graphiques
/// 
/// Bibliothèque de dessin 2D basique

use super::vesa::{VesaDriver, Color};

/// Trait pour contexte graphique
pub trait GraphicsContext {
    fn draw_pixel(&mut self, x: u16, y: u16, color: Color);
    fn width(&self) -> u16;
    fn height(&self) -> u16;
}

impl GraphicsContext for VesaDriver {
    fn draw_pixel(&mut self, x: u16, y: u16, color: Color) {
        self.put_pixel(x, y, color);
    }
    
    fn width(&self) -> u16 {
        self.width()
    }
    
    fn height(&self) -> u16 {
        self.height()
    }
}

/// Bibliothèque de dessin
pub struct Canvas<'a, G: GraphicsContext> {
    context: &'a mut G,
}

impl<'a, G: GraphicsContext> Canvas<'a, G> {
    pub fn new(context: &'a mut G) -> Self {
        Self { context }
    }
    
    /// Dessine une ligne (Algorithme de Bresenham)
    pub fn draw_line(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
        let mut x = x0 as i32;
        let mut y = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;
        
        let dx = (x1 - x).abs();
        let dy = -(y1 - y).abs();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        
        loop {
            if x >= 0 && y >= 0 {
                self.context.draw_pixel(x as u16, y as u16, color);
            }
            
            if x == x1 && y == y1 { break; }
            
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Dessine un rectangle (contour)
    pub fn draw_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color) {
        self.draw_line(x, y, x + w, y, color);
        self.draw_line(x + w, y, x + w, y + h, color);
        self.draw_line(x + w, y + h, x, y + h, color);
        self.draw_line(x, y + h, x, y, color);
    }
    
    /// Dessine un rectangle plein
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color) {
        for i in 0..h {
            self.draw_line(x, y + i, x + w - 1, y + i, color);
        }
    }
    
    /// Dessine un cercle (Algorithme de Bresenham)
    pub fn draw_circle(&mut self, center_x: u16, center_y: u16, radius: u16, color: Color) {
        let mut x = 0;
        let mut y = radius as i32;
        let mut d = 3 - 2 * radius as i32;
        
        self.draw_circle_points(center_x, center_y, x, y, color);
        
        while y >= x {
            x += 1;
            if d > 0 {
                y -= 1;
                d = d + 4 * (x - y) + 10;
            } else {
                d = d + 4 * x + 6;
            }
            self.draw_circle_points(center_x, center_y, x, y, color);
        }
    }
    
    fn draw_circle_points(&mut self, cx: u16, cy: u16, x: i32, y: i32, color: Color) {
        let cx = cx as i32;
        let cy = cy as i32;
        
        // 8 octants symmetry
        self.safe_pixel(cx + x, cy + y, color);
        self.safe_pixel(cx - x, cy + y, color);
        self.safe_pixel(cx + x, cy - y, color);
        self.safe_pixel(cx - x, cy - y, color);
        self.safe_pixel(cx + y, cy + x, color);
        self.safe_pixel(cx - y, cy + x, color);
        self.safe_pixel(cx + y, cy - x, color);
        self.safe_pixel(cx - y, cy - x, color);
    }
    
    fn safe_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 {
            self.context.draw_pixel(x as u16, y as u16, color);
        }
    }
}
