use alloc::string::String;
use crate::vga_buffer::WRITER;

/// Affiche du texte formaté sur la sortie standard
/// Similaire à printf en C
pub fn printf(format: &str) -> i32 {
    WRITER.lock().write_string(format);
    format.len() as i32
}

/// Affiche du texte formaté avec arguments
pub fn printf_args(format: &str, args: &[&str]) -> i32 {
    let mut output = String::new();
    let mut arg_index = 0;
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next_c) = chars.peek() {
                match next_c {
                    's' => {
                        chars.next();
                        if arg_index < args.len() {
                            output.push_str(args[arg_index]);
                            arg_index += 1;
                        }
                    }
                    'd' => {
                        chars.next();
                        if arg_index < args.len() {
                            output.push_str(args[arg_index]);
                            arg_index += 1;
                        }
                    }
                    '%' => {
                        chars.next();
                        output.push('%');
                    }
                    _ => {
                        output.push(c);
                    }
                }
            } else {
                output.push(c);
            }
        } else if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                match next_c {
                    'n' => {
                        chars.next();
                        output.push('\n');
                    }
                    't' => {
                        chars.next();
                        output.push('\t');
                    }
                    '\\' => {
                        chars.next();
                        output.push('\\');
                    }
                    _ => {
                        output.push(c);
                    }
                }
            } else {
                output.push(c);
            }
        } else {
            output.push(c);
        }
    }

    WRITER.lock().write_string(&output);
    output.len() as i32
}

/// Affiche une chaîne de caractères
pub fn puts(s: &str) -> i32 {
    WRITER.lock().write_string(s);
    WRITER.lock().write_string("\n");
    s.len() as i32 + 1
}

/// Affiche un caractère
pub fn putchar(c: char) -> i32 {
    WRITER.lock().write_string(&format!("{}", c));
    c as i32
}

/// Affiche une chaîne sans saut à la ligne
pub fn fputs(s: &str) -> i32 {
    WRITER.lock().write_string(s);
    s.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_printf() {
        let result = printf("Hello, World!");
        assert_eq!(result, 13);
    }

    #[test_case]
    fn test_puts() {
        let result = puts("Hello");
        assert_eq!(result, 6);
    }

    #[test_case]
    fn test_putchar() {
        let result = putchar('A');
        assert_eq!(result, 65);
    }
}
