use alloc::vec::Vec;
use core::mem::size_of;

// Type de fichier
pub const ET_NONE: u16 = 0;
pub const ET_REL: u16 = 1;
pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16 = 3;
pub const ET_CORE: u16 = 4;

// Machine
pub const EM_X86_64: u16 = 62;

// Type de segment
pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_SHLIB: u32 = 5;
pub const PT_PHDR: u32 = 6;

// Flags de segment
pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2;
pub const PF_R: u32 = 4;

/// En-tête ELF 64-bits
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl Elf64Header {
    pub const MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.e_ident[0..4] != Self::MAGIC {
            return Err("Invalid ELF Magic");
        }
        if self.e_ident[4] != 2 { // EI_CLASS = 64-bit
            return Err("Not a 64-bit ELF");
        }
        if self.e_ident[5] != 1 { // EI_DATA = Little Endian
            return Err("Not Little Endian");
        }
        if self.e_type != ET_EXEC && self.e_type != ET_DYN {
            return Err("Not an executable");
        }
        if self.e_machine != EM_X86_64 {
            return Err("Not x86-64");
        }
        Ok(())
    }
}

/// En-tête de programme ELF 64-bits
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub struct ElfFile<'a> {
    data: &'a [u8],
    pub header: Elf64Header,
}

impl<'a> ElfFile<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        if data.len() < size_of::<Elf64Header>() {
            return Err("File too small");
        }

        let ptr = data.as_ptr() as *const Elf64Header;
        
        // Use read_unaligned to safely copy packed struct
        let header_copy = unsafe { core::ptr::read_unaligned(ptr) };
        
        Ok(ElfFile {
            data,
            header: header_copy,
        })
    }

    pub fn entry_point(&self) -> u64 {
        self.header.e_entry
    }

    pub fn program_headers(&self) -> ProgramHeaderIter<'a> {
        ProgramHeaderIter {
            data: self.data,
            offset: self.header.e_phoff as usize,
            count: self.header.e_phnum as usize,
            size: self.header.e_phentsize as usize,
            current: 0,
        }
    }
}

pub struct ProgramHeaderIter<'a> {
    data: &'a [u8],
    offset: usize,
    count: usize,
    size: usize,
    current: usize,
}

impl<'a> Iterator for ProgramHeaderIter<'a> {
    type Item = Elf64ProgramHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }

        let offset = self.offset + self.current * self.size;
        if offset + size_of::<Elf64ProgramHeader>() > self.data.len() {
            return None;
        }

        let ptr = self.data[offset..].as_ptr() as *const Elf64ProgramHeader;
        let header = unsafe { core::ptr::read_unaligned(ptr) };

        self.current += 1;
        Some(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_elf_header_validation() {
        let mut data = [0u8; 64]; // Taille min Header
        data[0] = 0x7f;
        data[1] = b'E';
        data[2] = b'L';
        data[3] = b'F';
        data[4] = 2; // 64-bit
        data[5] = 1; // Little Endian
        data[16] = 2; // ET_EXEC
        data[17] = 0;
        data[18] = 62; // x86-64
        data[19] = 0;

        let elf = ElfFile::new(&data).expect("Should parse");
        assert!(elf.header.validate().is_ok());
    }

    #[test_case]
    fn test_elf_magic_fail() {
        let data = [0u8; 64];
        let elf = ElfFile::new(&data).expect("Should parse");
        assert!(elf.header.validate().is_err());
    }
}
