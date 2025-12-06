use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;

/// Alloue de la mémoire
/// Similaire à malloc en C
pub fn malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    unsafe {
        let layout = Layout::from_size_align_unchecked(size, 8);
        alloc(layout)
    }
}

/// Alloue de la mémoire et l'initialise à zéro
/// Similaire à calloc en C
pub fn calloc(count: usize, size: usize) -> *mut u8 {
    let total_size = count.saturating_mul(size);
    
    if total_size == 0 {
        return core::ptr::null_mut();
    }

    let ptr = malloc(total_size);
    
    if !ptr.is_null() {
        unsafe {
            core::ptr::write_bytes(ptr, 0, total_size);
        }
    }
    
    ptr
}

/// Libère la mémoire allouée
/// Similaire à free en C
pub fn free(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size > 0 {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, 8);
            dealloc(ptr, layout);
        }
    }
}

/// Retourne un nombre aléatoire
pub fn rand() -> u32 {
    // Générateur linéaire congruentiel simple
    static mut SEED: u32 = 1;
    
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        (SEED / 65536) % 32768
    }
}

/// Initialise le générateur de nombres aléatoires
pub fn srand(seed: u32) {
    unsafe {
        static mut SEED: u32 = 1;
        SEED = seed;
    }
}

/// Retourne la valeur absolue d'un entier
pub fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

/// Retourne la valeur absolue d'un entier long
pub fn labs(x: i64) -> i64 {
    if x < 0 { -x } else { x }
}

/// Convertit une chaîne en entier
pub fn atoi(s: &str) -> i32 {
    let mut result: i32 = 0;
    let mut negative = false;
    let mut started = false;

    for c in s.chars() {
        if !started {
            if c == '-' {
                negative = true;
                started = true;
            } else if c == '+' {
                started = true;
            } else if c.is_numeric() {
                result = (c as i32) - ('0' as i32);
                started = true;
            }
        } else if c.is_numeric() {
            result = result.saturating_mul(10);
            result = result.saturating_add((c as i32) - ('0' as i32));
        } else {
            break;
        }
    }

    if negative { -result } else { result }
}

/// Convertit une chaîne en entier long
pub fn atol(s: &str) -> i64 {
    atoi(s) as i64
}

/// Convertit une chaîne en nombre flottant
pub fn atof(s: &str) -> f64 {
    let mut result: f64 = 0.0;
    let mut decimal_places_count = 0;
    let mut in_decimal = false;
    let mut negative = false;
    let mut started = false;

    for c in s.chars() {
        if !started {
            if c == '-' {
                negative = true;
                started = true;
            } else if c == '+' {
                started = true;
            } else if c.is_numeric() {
                result = (c as i32 - '0' as i32) as f64;
                started = true;
            } else if c == '.' {
                in_decimal = true;
                started = true;
            }
        } else if c.is_numeric() {
            if in_decimal {
                decimal_places_count += 1;
                let mut divisor = 1.0_f64;
                for _ in 0..decimal_places_count {
                    divisor *= 10.0;
                }
                result += ((c as i32 - '0' as i32) as f64) / divisor;
            } else {
                result = result * 10.0 + ((c as i32 - '0' as i32) as f64);
            }
        } else if c == '.' && !in_decimal {
            in_decimal = true;
        } else {
            break;
        }
    }

    if negative { -result } else { result }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_malloc() {
        let ptr = malloc(1024);
        assert!(!ptr.is_null());
        free(ptr, 1024);
    }

    #[test_case]
    fn test_calloc() {
        let ptr = calloc(10, 100);
        assert!(!ptr.is_null());
        free(ptr, 1000);
    }

    #[test_case]
    fn test_abs() {
        assert_eq!(abs(-5), 5);
        assert_eq!(abs(5), 5);
        assert_eq!(abs(0), 0);
    }

    #[test_case]
    fn test_atoi() {
        assert_eq!(atoi("123"), 123);
        assert_eq!(atoi("-456"), -456);
        assert_eq!(atoi("0"), 0);
    }
}
