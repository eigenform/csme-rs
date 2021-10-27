use std::convert::TryInto;
use std::fmt;
use std::ffi;
use crate::{ FromBytes, man::* };

/// Directory of files contained in some code partition.
pub struct CodePartitionDirectory {
    pub header: CpdHeader,
    pub entries: Vec<CpdEntry>,
}
impl CodePartitionDirectory {
    pub fn new(data: &[u8]) -> Self {
        let header = CpdHeader::from_bytes(&data[0x00..0x10]);
        let num_entries = header.entries as usize;
        let mut entries = vec![CpdEntry::default(); num_entries];
        unsafe {
            entries.copy_from_slice(
                std::slice::from_raw_parts(
                    data[0x10..].as_ptr() as *const CpdEntry,
                    num_entries
                ),
            );
        }
        CodePartitionDirectory { header, entries }
    }
}

/// Header for a code partition directory.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CpdHeader {
    pub marker: [u8; 4],
    pub entries: u32,
    pub header_version: u8,
    pub entry_version: u8,
    pub header_length: u8,
    pub checksum: u8,
    pub partition_name: [u8; 4],
}
impl fmt::Debug for CpdHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let partition_name = std::str::from_utf8(&self.partition_name)
            .expect("Couldn't interpret CPD header partition name as UTF-8")
            .trim_end_matches(char::from(0));
        f.debug_struct("CpdHeader")
            .field("partition_name", &partition_name)
            .field("entries", &self.entries)
            .finish()
    }
}
impl CpdHeader {
    const MARKER_CPD: [u8; 4] = *b"$CPD";
}
impl crate::FromBytes for CpdHeader {
    fn validate(&self) -> Result<(), &'static str> {
        assert_eq!(self.marker, Self::MARKER_CPD);
        assert_eq!(self.header_length, 0x10);
        Ok(())
    }
}

/// An entry in some code partition directory.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CpdEntry {
    pub name: [u8; 12],
    pub attrs: CpdEntryBits,
    pub length: u32,
    pub reserved: u32,
}
impl fmt::Debug for CpdEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let name = std::str::from_utf8(&self.name)
            .expect("Couldn't interpret CPD entry name as UTF-8")
            .trim_end_matches(char::from(0));
        f.debug_struct("CpdEntry")
            .field("name", &name)
            .field("addr", &self.attrs.address())
            .field("length", &self.length)
            .field("compressed", &self.attrs.compress_flag())
            .finish()
    }
}
impl CpdEntry {
    /// Return the filename (as a reference to a UTF-8 string).
    pub fn filename(&self) -> &str { 
        std::str::from_utf8(&self.name)
            .expect("Couldn't interpret CPD entry name as UTF-8")
            .trim_end_matches(char::from(0))
    }
    pub fn len(&self) -> usize { self.length as usize }
    pub fn offset(&self) -> usize { self.attrs.address() as usize }
}

/// Bitfield in [CpdEntry].
#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug)]
pub struct CpdEntryBits(pub u32);
impl CpdEntryBits {
    pub fn address(&self) -> u32 { self.0 & 0x01ff_ffff }
    pub fn compress_flag(&self) -> bool { (self.0 & 0x0200_0000) != 0 }
    pub fn reserved(&self) -> u32 { (self.0 & 0xfc00_0000) >> 26 }
}

