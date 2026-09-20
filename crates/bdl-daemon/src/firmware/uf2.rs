//! The image a Raspberry Pi Pico takes over its own bootloader: the
//! firmware's flash contents, 256 bytes to a 512-byte block, in the UF2
//! container (<https://github.com/microsoft/uf2>).  A pure function of the
//! linked ELF: the loadable segments whose load address lies in the
//! board's flash are laid into pages; nothing is relocated, linked or
//! checked beyond what the container needs.

use std::collections::BTreeMap;

/// Where a family's firmware lives and how its blocks are tagged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Family {
    /// The UF2 family id the bootloader accepts (RP2040: `0xe48bff56`).
    pub id: u32,
    /// The first flash address and the size of flash the image may cover.
    pub flash_start: u32,
    pub flash_len: u32,
}

/// The RP2040: 16 MiB of XIP flash at `0x1000_0000` (the target entry
/// carries the same numbers; this is the tests' copy).
#[cfg(test)]
pub const RP2040: Family = Family {
    id: 0xe48b_ff56,
    flash_start: 0x1000_0000,
    flash_len: 16 * 1024 * 1024,
};

const PAGE: usize = 256;
const BLOCK: usize = 512;
const MAGIC_START0: u32 = 0x0A32_4655;
const MAGIC_START1: u32 = 0x9E5D_5157;
const MAGIC_END: u32 = 0x0AB1_6F30;
const FLAG_FAMILY_ID_PRESENT: u32 = 0x0000_2000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Uf2Error {
    /// Not a 32-bit little-endian ELF.
    NotElf32Le,
    /// The file ends before a header or a segment it declares.
    Truncated(&'static str),
    /// No loadable segment lies in the family's flash.
    NothingInFlash,
    /// A segment extends past the family's flash.
    OutsideFlash { start: u32, len: u32 },
}

impl std::fmt::Display for Uf2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Uf2Error::NotElf32Le => write!(f, "not a 32-bit little-endian ELF"),
            Uf2Error::Truncated(what) => write!(f, "the ELF ends inside its {what}"),
            Uf2Error::NothingInFlash => write!(f, "no loadable segment lies in flash"),
            Uf2Error::OutsideFlash { start, len } => write!(
                f,
                "a segment at {start:#x} of {len} bytes extends past the flash"
            ),
        }
    }
}

impl std::error::Error for Uf2Error {}

fn u16_at(b: &[u8], at: usize, what: &'static str) -> Result<u16, Uf2Error> {
    b.get(at..at + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or(Uf2Error::Truncated(what))
}

fn u32_at(b: &[u8], at: usize, what: &'static str) -> Result<u32, Uf2Error> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or(Uf2Error::Truncated(what))
}

/// One loadable segment of the ELF: its load (physical) address and the
/// bytes the file holds for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub paddr: u32,
    pub bytes: Vec<u8>,
}

/// The `PT_LOAD` segments with file contents, in header order.
pub fn load_segments(elf: &[u8]) -> Result<Vec<Segment>, Uf2Error> {
    if elf.len() < 52 || &elf[..4] != b"\x7fELF" || elf[4] != 1 || elf[5] != 1 {
        return Err(Uf2Error::NotElf32Le);
    }
    let phoff = u32_at(elf, 28, "header")? as usize;
    let phentsize = u16_at(elf, 42, "header")? as usize;
    let phnum = u16_at(elf, 44, "header")? as usize;
    let mut out = Vec::new();
    for i in 0..phnum {
        let at = phoff + i * phentsize;
        let p_type = u32_at(elf, at, "program header")?;
        if p_type != 1 {
            continue;
        }
        let offset = u32_at(elf, at + 4, "program header")? as usize;
        let paddr = u32_at(elf, at + 12, "program header")?;
        let filesz = u32_at(elf, at + 16, "program header")? as usize;
        if filesz == 0 {
            continue;
        }
        let bytes = elf
            .get(offset..offset + filesz)
            .ok_or(Uf2Error::Truncated("segment"))?
            .to_vec();
        out.push(Segment { paddr, bytes });
    }
    Ok(out)
}

/// The flash pages the segments fill: page address → 256 bytes, gaps
/// inside a page zero.  Segments outside the family's flash (RAM-only
/// data, debug sections) are not part of the image.
pub fn pages(segments: &[Segment], family: Family) -> Result<BTreeMap<u32, [u8; PAGE]>, Uf2Error> {
    let end = family.flash_start as u64 + family.flash_len as u64;
    let mut pages: BTreeMap<u32, [u8; PAGE]> = BTreeMap::new();
    for s in segments {
        let start = s.paddr as u64;
        if start < family.flash_start as u64 || start >= end {
            continue;
        }
        if start + s.bytes.len() as u64 > end {
            return Err(Uf2Error::OutsideFlash {
                start: s.paddr,
                len: s.bytes.len() as u32,
            });
        }
        for (i, b) in s.bytes.iter().enumerate() {
            let addr = s.paddr + i as u32;
            let page = addr & !(PAGE as u32 - 1);
            pages.entry(page).or_insert([0u8; PAGE])[(addr - page) as usize] = *b;
        }
    }
    if pages.is_empty() {
        return Err(Uf2Error::NothingInFlash);
    }
    Ok(pages)
}

/// The UF2 file for the ELF's flash contents.
pub fn from_elf(elf: &[u8], family: Family) -> Result<Vec<u8>, Uf2Error> {
    let pages = pages(&load_segments(elf)?, family)?;
    let total = pages.len() as u32;
    let mut out = Vec::with_capacity(pages.len() * BLOCK);
    for (n, (addr, data)) in pages.iter().enumerate() {
        let mut block = [0u8; BLOCK];
        block[0..4].copy_from_slice(&MAGIC_START0.to_le_bytes());
        block[4..8].copy_from_slice(&MAGIC_START1.to_le_bytes());
        block[8..12].copy_from_slice(&FLAG_FAMILY_ID_PRESENT.to_le_bytes());
        block[12..16].copy_from_slice(&addr.to_le_bytes());
        block[16..20].copy_from_slice(&(PAGE as u32).to_le_bytes());
        block[20..24].copy_from_slice(&(n as u32).to_le_bytes());
        block[24..28].copy_from_slice(&total.to_le_bytes());
        block[28..32].copy_from_slice(&family.id.to_le_bytes());
        block[32..32 + PAGE].copy_from_slice(data);
        block[BLOCK - 4..].copy_from_slice(&MAGIC_END.to_le_bytes());
        out.extend_from_slice(&block);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal ELF32 LE with the given loadable segments (paddr, bytes),
    /// laid out header, program headers, then each segment's bytes.
    fn elf_with(segments: &[(u32, &[u8])]) -> Vec<u8> {
        let phoff = 52u32;
        let phentsize = 32u16;
        let mut data_at = phoff as usize + segments.len() * phentsize as usize;
        let mut header = vec![0u8; 52];
        header[..4].copy_from_slice(b"\x7fELF");
        header[4] = 1; // ELFCLASS32
        header[5] = 1; // little endian
        header[6] = 1;
        header[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC
        header[18..20].copy_from_slice(&40u16.to_le_bytes()); // EM_ARM
        header[20..24].copy_from_slice(&1u32.to_le_bytes());
        header[28..32].copy_from_slice(&phoff.to_le_bytes());
        header[40..42].copy_from_slice(&52u16.to_le_bytes());
        header[42..44].copy_from_slice(&phentsize.to_le_bytes());
        header[44..46].copy_from_slice(&(segments.len() as u16).to_le_bytes());
        let mut phdrs = Vec::new();
        let mut body = Vec::new();
        for (paddr, bytes) in segments {
            let mut ph = [0u8; 32];
            ph[0..4].copy_from_slice(&1u32.to_le_bytes()); // PT_LOAD
            ph[4..8].copy_from_slice(&(data_at as u32).to_le_bytes());
            ph[8..12].copy_from_slice(&paddr.to_le_bytes());
            ph[12..16].copy_from_slice(&paddr.to_le_bytes());
            ph[16..20].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
            ph[20..24].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
            ph[24..28].copy_from_slice(&5u32.to_le_bytes());
            phdrs.extend_from_slice(&ph);
            body.extend_from_slice(bytes);
            data_at += bytes.len();
        }
        [header, phdrs, body].concat()
    }

    #[test]
    fn flash_segments_become_pages_and_ram_segments_are_left_out() {
        let text = [0xAAu8; 300];
        let data = [0x55u8; 4];
        let elf = elf_with(&[
            (RP2040.flash_start, &text),
            (RP2040.flash_start + 0x1000, &data),
            (0x2000_0000, &[1, 2, 3]), // RAM: not in the image
        ]);
        let uf2 = from_elf(&elf, RP2040).unwrap();
        // 300 bytes = two pages at 0x1000_0000, one page at 0x1000_1000
        assert_eq!(uf2.len(), 3 * BLOCK);
        let block = |n: usize| &uf2[n * BLOCK..(n + 1) * BLOCK];
        for n in 0..3 {
            let b = block(n);
            assert_eq!(
                u32::from_le_bytes(b[0..4].try_into().unwrap()),
                MAGIC_START0
            );
            assert_eq!(
                u32::from_le_bytes(b[4..8].try_into().unwrap()),
                MAGIC_START1
            );
            assert_eq!(
                u32::from_le_bytes(b[8..12].try_into().unwrap()),
                FLAG_FAMILY_ID_PRESENT
            );
            assert_eq!(u32::from_le_bytes(b[16..20].try_into().unwrap()), 256);
            assert_eq!(u32::from_le_bytes(b[20..24].try_into().unwrap()), n as u32);
            assert_eq!(u32::from_le_bytes(b[24..28].try_into().unwrap()), 3);
            assert_eq!(
                u32::from_le_bytes(b[28..32].try_into().unwrap()),
                0xe48b_ff56
            );
            assert_eq!(
                u32::from_le_bytes(b[508..512].try_into().unwrap()),
                MAGIC_END
            );
        }
        assert_eq!(
            u32::from_le_bytes(block(0)[12..16].try_into().unwrap()),
            0x1000_0000
        );
        assert_eq!(
            u32::from_le_bytes(block(1)[12..16].try_into().unwrap()),
            0x1000_0100
        );
        assert_eq!(
            u32::from_le_bytes(block(2)[12..16].try_into().unwrap()),
            0x1000_1000
        );
        assert!(block(0)[32..32 + 256].iter().all(|b| *b == 0xAA));
        // the second page: 44 bytes of text, then zero
        assert!(block(1)[32..32 + 44].iter().all(|b| *b == 0xAA));
        assert!(block(1)[32 + 44..32 + 256].iter().all(|b| *b == 0));
        assert_eq!(&block(2)[32..36], &[0x55; 4]);
        assert!(block(2)[36..32 + 256].iter().all(|b| *b == 0));
    }

    #[test]
    fn a_page_is_shared_by_two_segments() {
        let elf = elf_with(&[
            (RP2040.flash_start, &[1u8; 16]),
            (RP2040.flash_start + 16, &[2u8; 16]),
        ]);
        let uf2 = from_elf(&elf, RP2040).unwrap();
        assert_eq!(uf2.len(), BLOCK);
        assert_eq!(&uf2[32..48], &[1u8; 16]);
        assert_eq!(&uf2[48..64], &[2u8; 16]);
    }

    #[test]
    fn refusals_are_values() {
        assert_eq!(from_elf(b"not an elf", RP2040), Err(Uf2Error::NotElf32Le));
        let ram_only = elf_with(&[(0x2000_0000, &[1u8; 8])]);
        assert_eq!(from_elf(&ram_only, RP2040), Err(Uf2Error::NothingInFlash));
        let past_the_end = elf_with(&[(RP2040.flash_start + RP2040.flash_len - 4, &[1u8; 8])]);
        assert!(matches!(
            from_elf(&past_the_end, RP2040),
            Err(Uf2Error::OutsideFlash { .. })
        ));
        let mut truncated = elf_with(&[(RP2040.flash_start, &[1u8; 64])]);
        truncated.truncate(100);
        assert_eq!(
            from_elf(&truncated, RP2040),
            Err(Uf2Error::Truncated("segment"))
        );
    }
}
