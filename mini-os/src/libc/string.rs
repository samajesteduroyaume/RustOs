use alloc::string::String;

/// Retourne la longueur d'une chaîne
pub fn strlen(s: &str) -> usize {
    s.len()
}

/// Copie une chaîne source vers une destination
pub fn strcpy(dest: &mut [u8], src: &str) -> *mut u8 {
    let src_bytes = src.as_bytes();
    let copy_len = src_bytes.len().min(dest.len() - 1);
    
    dest[..copy_len].copy_from_slice(&src_bytes[..copy_len]);
    dest[copy_len] = 0; // Null terminator
    
    dest.as_mut_ptr()
}

/// Copie une chaîne source vers une destination avec limite de taille
pub fn strncpy(dest: &mut [u8], src: &str, n: usize) -> *mut u8 {
    let src_bytes = src.as_bytes();
    let copy_len = src_bytes.len().min(n).min(dest.len() - 1);
    
    dest[..copy_len].copy_from_slice(&src_bytes[..copy_len]);
    if copy_len < dest.len() {
        dest[copy_len] = 0; // Null terminator
    }
    
    dest.as_mut_ptr()
}

/// Concatène deux chaînes
pub fn strcat(dest: &mut String, src: &str) -> *mut u8 {
    dest.push_str(src);
    dest.as_mut_ptr()
}

/// Concatène deux chaînes avec limite de taille
pub fn strncat(dest: &mut String, src: &str, n: usize) -> *mut u8 {
    let copy_len = src.len().min(n);
    dest.push_str(&src[..copy_len]);
    dest.as_mut_ptr()
}

/// Compare deux chaînes
pub fn strcmp(s1: &str, s2: &str) -> i32 {
    if s1 < s2 {
        -1
    } else if s1 > s2 {
        1
    } else {
        0
    }
}

/// Compare deux chaînes avec limite de taille
pub fn strncmp(s1: &str, s2: &str, n: usize) -> i32 {
    let s1_bytes = s1.as_bytes();
    let s2_bytes = s2.as_bytes();
    let cmp_len = n.min(s1_bytes.len()).min(s2_bytes.len());
    
    if &s1_bytes[..cmp_len] < &s2_bytes[..cmp_len] {
        -1
    } else if &s1_bytes[..cmp_len] > &s2_bytes[..cmp_len] {
        1
    } else {
        0
    }
}

/// Trouve la première occurrence d'un caractère dans une chaîne
pub fn strchr(s: &str, c: char) -> Option<usize> {
    s.find(c)
}

/// Trouve la dernière occurrence d'un caractère dans une chaîne
pub fn strrchr(s: &str, c: char) -> Option<usize> {
    s.rfind(c)
}

/// Trouve la première occurrence d'une sous-chaîne
pub fn strstr(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}

/// Copie de la mémoire
pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, n);
    }
    dest
}

/// Déplace de la mémoire (gère les chevauchements)
pub fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::copy(src, dest, n);
    }
    dest
}

/// Remplit de la mémoire avec une valeur
pub fn memset(s: *mut u8, c: u8, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::write_bytes(s, c, n);
    }
    s
}

/// Compare deux zones de mémoire
pub fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let b1 = *s1.add(i);
            let b2 = *s2.add(i);
            
            if b1 < b2 {
                return -1;
            } else if b1 > b2 {
                return 1;
            }
        }
    }
    0
}

/// Trouve un caractère dans une zone de mémoire
pub fn memchr(s: *const u8, c: u8, n: usize) -> *const u8 {
    unsafe {
        for i in 0..n {
            if *s.add(i) == c {
                return s.add(i);
            }
        }
    }
    core::ptr::null()
}

/// Convertit une chaîne en minuscules
pub fn strtolower(s: &str) -> String {
    s.to_lowercase()
}

/// Convertit une chaîne en majuscules
pub fn strtoupper(s: &str) -> String {
    s.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_strlen() {
        assert_eq!(strlen("hello"), 5);
        assert_eq!(strlen(""), 0);
    }

    #[test_case]
    fn test_strcmp() {
        assert_eq!(strcmp("abc", "abc"), 0);
        assert_eq!(strcmp("abc", "def"), -1);
        assert_eq!(strcmp("def", "abc"), 1);
    }

    #[test_case]
    fn test_strchr() {
        assert_eq!(strchr("hello", 'l'), Some(2));
        assert_eq!(strchr("hello", 'x'), None);
    }

    #[test_case]
    fn test_strstr() {
        assert_eq!(strstr("hello world", "world"), Some(6));
        assert_eq!(strstr("hello world", "xyz"), None);
    }

    #[test_case]
    fn test_strtolower() {
        assert_eq!(strtolower("HELLO"), "hello");
        assert_eq!(strtolower("HeLLo"), "hello");
    }
}
