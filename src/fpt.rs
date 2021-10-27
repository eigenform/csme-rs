
use std::fmt;
use crate::FromBytes;

/// A flash partition table describing partitions in some CSME image.
pub struct FlashPartitionTable {
    pub header: FptHeader,
    pub entries: Vec<FptEntry>,
}
impl FlashPartitionTable {
    pub fn from_bytes(data: &[u8]) -> Self {
        let header = FptHeader::from_bytes(&data[0x00..0x20]);
        let num_entries = header.num_fpt_entries as usize;
        assert!(num_entries <= 127);
        let mut entries = vec![FptEntry::default(); num_entries];
        unsafe {
            entries.copy_from_slice(
                std::slice::from_raw_parts(
                    data[0x20..].as_ptr() as *const FptEntry, 
                    num_entries
                ),
            );
        }
        FlashPartitionTable { header, entries }
    }
}


/// Flash partition table header.
#[repr(C)]
#[derive(Debug)]
pub struct FptHeader {
    pub marker: [u8; 4],
    pub num_fpt_entries: u32,
    pub header_version: u8,
    pub entry_version: u8,
    pub header_length: u8,
    pub header_checksum: u8,
    pub ticks_to_add: u16,
    pub tokens_to_add: u16,
    pub reserved: u32,
    pub flash_layout: u32,
    pub fitc_major_ver: u16,
    pub fitc_minor_ver: u16,
    pub fitc_hotfix_ver: u16,
    pub fitc_build_ver: u16,
}
impl FptHeader {
    const MARKER_FPT: [u8; 4] = *b"$FPT";
}
impl crate::FromBytes for FptHeader {
    fn validate(&self) -> Result<(), &'static str> {
        assert_eq!(self.marker, Self::MARKER_FPT);
        assert_eq!(self.header_length, 0x20);
        Ok(())
    }
}

/// An entry in the flash partition table.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct FptEntry {
    pub name: [u8; 4],
    pub reserved: u32,
    pub offset: u32,
    pub length: u32,
    pub reserved1: u32,
    pub reserved2: u32,
    pub reserved3: u32,
    pub attrs: FptEntryAttributes,
}
impl fmt::Debug for FptEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let name = std::str::from_utf8(&self.name)
            .expect("Couldn't interpret partition name as UTF-8")
            .trim_end_matches(char::from(0));
        f.debug_struct("FptEntry")
            .field("name", &name)
            .field("kind", &self.attrs.kind())
            .field("valid", &self.attrs.entry_valid())
            .field("offset", &self.offset)
            .field("length", &self.length)
            .finish()
    }
}

/// Attributes bitfield for an FPT entry.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct FptEntryAttributes(pub u32);
impl FptEntryAttributes {
    /// Return the particular type of this partition.
    pub fn kind(&self) -> PartitionType { 
        PartitionType::from(self.0 & 0x0000_003f)
    }
    // Return whether or not this partition is valid.
    pub fn entry_valid(&self) -> bool { 
        if ((self.0 & 0xff00_0000) == 0xff00_0000) { false } else { true } 
    }
}

/// Representing different types of partitions represented in a table.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum PartitionType {
    Code = 0x00,
    Data = 0x01,
}
impl From<u32> for PartitionType {
    fn from(x: u32) -> Self {
        match x { 
            0 => PartitionType::Code, 1 => PartitionType::Data,
            _ => panic!("Unimplemented partition type {}", x),
        }
    }
}
