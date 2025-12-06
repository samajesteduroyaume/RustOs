use alloc::string::String;
use alloc::vec::Vec;
use crate::vga_buffer::WRITER;

/// Couleurs disponibles
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

/// Éditeur de ligne pour le terminal
pub struct LineEditor {
    buffer: Vec<char>,
    cursor_pos: usize,
    history: Vec<String>,
    history_index: usize,
}

impl LineEditor {
    /// Crée un nouvel éditeur de ligne
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor_pos: 0,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Ajoute un caractère à la position du curseur
    pub fn insert_char(&mut self, c: char) {
        if self.cursor_pos <= self.buffer.len() {
            self.buffer.insert(self.cursor_pos, c);
            self.cursor_pos += 1;
        }
    }

    /// Supprime le caractère avant le curseur (backspace)
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.buffer.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    /// Supprime le caractère à la position du curseur (delete)
    pub fn delete(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.buffer.remove(self.cursor_pos);
        }
    }

    /// Déplace le curseur vers la gauche
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Déplace le curseur vers la droite
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.cursor_pos += 1;
        }
    }

    /// Déplace le curseur au début de la ligne
    pub fn move_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Déplace le curseur à la fin de la ligne
    pub fn move_end(&mut self) {
        self.cursor_pos = self.buffer.len();
    }

    /// Efface toute la ligne
    pub fn clear_line(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
    }

    /// Ajoute une ligne à l'historique
    pub fn add_to_history(&mut self, line: &str) {
        self.history.push(line.into());
        self.history_index = self.history.len();
    }

    /// Récupère la ligne précédente de l'historique
    pub fn history_prev(&mut self) -> Option<String> {
        if self.history_index > 0 {
            self.history_index -= 1;
            Some(self.history[self.history_index].clone())
        } else {
            None
        }
    }

    /// Récupère la ligne suivante de l'historique
    pub fn history_next(&mut self) -> Option<String> {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            Some(self.history[self.history_index].clone())
        } else if self.history_index == self.history.len() - 1 {
            self.history_index += 1;
            Some(String::new())
        } else {
            None
        }
    }

    /// Retourne le contenu du buffer sous forme de String
    pub fn get_line(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Affiche le buffer avec le curseur
    pub fn display(&self, prompt: &str) {
        WRITER.lock().write_string(prompt);
        
        for (i, &c) in self.buffer.iter().enumerate() {
            if i == self.cursor_pos {
                WRITER.lock().write_string("█");
            }
            WRITER.lock().write_string(&format!("{}", c));
        }
        
        if self.cursor_pos == self.buffer.len() {
            WRITER.lock().write_string("█");
        }
    }

    /// Redessine la ligne
    pub fn redraw(&self, prompt: &str) {
        // Effacer la ligne actuelle
        WRITER.lock().write_string("\r");
        
        // Afficher le prompt
        WRITER.lock().write_string(prompt);
        
        // Afficher le buffer
        for c in &self.buffer {
            WRITER.lock().write_string(&format!("{}", c));
        }
    }
}

/// Terminal principal
pub struct Terminal {
    width: usize,
    height: usize,
    current_color: Color,
    line_editor: LineEditor,
}

impl Terminal {
    /// Crée un nouveau terminal
    pub fn new() -> Self {
        Self {
            width: 80,
            height: 25,
            current_color: Color::White,
            line_editor: LineEditor::new(),
        }
    }

    /// Écrit une chaîne de caractères
    pub fn write_string(&self, s: &str) {
        WRITER.lock().write_string(s);
    }

    /// Écrit une chaîne de caractères avec une couleur
    pub fn write_colored(&self, s: &str, _color: Color) {
        // TODO: Implémenter la coloration
        WRITER.lock().write_string(s);
    }

    /// Efface l'écran
    pub fn clear_screen(&self) {
        WRITER.lock().write_string("\x1b[2J\x1b[H");
    }

    /// Définit la couleur courante
    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    /// Lit une ligne depuis l'entrée utilisateur
    pub fn read_line(&mut self, prompt: &str) -> String {
        self.line_editor.clear_line();
        self.write_string(prompt);
        
        // TODO: Implémenter la lecture interactive
        // Pour l'instant, retourner une ligne vide
        String::new()
    }

    /// Affiche une ligne avec un saut à la ligne
    pub fn println(&self, s: &str) {
        WRITER.lock().write_string(s);
        WRITER.lock().write_string("\n");
    }

    /// Affiche une ligne d'erreur
    pub fn print_error(&self, s: &str) {
        WRITER.lock().write_string("\x1b[31m");
        WRITER.lock().write_string("Erreur: ");
        WRITER.lock().write_string(s);
        WRITER.lock().write_string("\x1b[0m\n");
    }

    /// Affiche une ligne d'avertissement
    pub fn print_warning(&self, s: &str) {
        WRITER.lock().write_string("\x1b[33m");
        WRITER.lock().write_string("Avertissement: ");
        WRITER.lock().write_string(s);
        WRITER.lock().write_string("\x1b[0m\n");
    }

    /// Affiche une ligne d'information
    pub fn print_info(&self, s: &str) {
        WRITER.lock().write_string("\x1b[32m");
        WRITER.lock().write_string("Info: ");
        WRITER.lock().write_string(s);
        WRITER.lock().write_string("\x1b[0m\n");
    }

    /// Obtient la largeur du terminal
    pub fn width(&self) -> usize {
        self.width
    }

    /// Obtient la hauteur du terminal
    pub fn height(&self) -> usize {
        self.height
    }

    /// Obtient l'éditeur de ligne
    pub fn line_editor(&mut self) -> &mut LineEditor {
        &mut self.line_editor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_line_editor_creation() {
        let editor = LineEditor::new();
        assert_eq!(editor.get_line(), "");
        assert_eq!(editor.cursor_pos, 0);
    }

    #[test_case]
    fn test_insert_char() {
        let mut editor = LineEditor::new();
        editor.insert_char('a');
        editor.insert_char('b');
        editor.insert_char('c');
        assert_eq!(editor.get_line(), "abc");
        assert_eq!(editor.cursor_pos, 3);
    }

    #[test_case]
    fn test_backspace() {
        let mut editor = LineEditor::new();
        editor.insert_char('a');
        editor.insert_char('b');
        editor.backspace();
        assert_eq!(editor.get_line(), "a");
        assert_eq!(editor.cursor_pos, 1);
    }

    #[test_case]
    fn test_terminal_creation() {
        let terminal = Terminal::new();
        assert_eq!(terminal.width(), 80);
        assert_eq!(terminal.height(), 25);
    }
}
