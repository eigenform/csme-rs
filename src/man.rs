
use std::convert::TryInto;
use crate::{ cpd, ext };
use std::io::BufRead;

use crate::FromBytes;

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BCDTimestamp(pub u32);

/// Partition manifest header.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ManifestHeader {
    manifest_type: u32,
    header_length_words: u32,
    version: u32,
    flags: u32,
    vendor: u32,
    date: BCDTimestamp,
    manifest_length_words: u32,
    marker: [u8; 4],
    reserved0: u32,
    version_major: u16,
    version_minor: u16,
    version_hotfix: u16,
    version_build: u16,
    secure_version_number: u32,
    reserved1: u64,
    reserved2: [u8; 64],
    modulus_len_words: u32,
    exponent_size_words: u32,
}
impl crate::FromBytes for ManifestHeader {
    fn validate(&self) -> Result<(), &'static str> {
        assert_eq!(self.marker, Self::MARKER_MN2);
        assert_eq!(self.vendor, 0x8086);
        assert_eq!(self.header_length_words, 0xa1);
        Ok(())
    }
}
impl ManifestHeader {
    const MARKER_MN2: [u8; 4] = *b"$MN2";
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CryptoBlock {
    public_key: [u8; 256],
    exponent: u32,
    rsa_signature: [u8; 256],
}
impl crate::FromBytes for CryptoBlock {}


pub struct CodePartitionManifest {
    header: ManifestHeader,
    crypto: CryptoBlock,
    extensions: Vec<ext::ManifestExtension>,
}
impl CodePartitionManifest {
    pub fn new(x: &[u8]) -> Self {
        let hdr_len = std::mem::size_of::<ManifestHeader>();
        let cry_len = std::mem::size_of::<CryptoBlock>();
        let header = ManifestHeader::from_bytes(&x[0x00..hdr_len]);
        let crypto = CryptoBlock::from_bytes(&x[hdr_len..hdr_len+cry_len]);
        let mut extensions = Vec::new();

        let mut cursor: usize = hdr_len + cry_len;
        while cursor < x.len() {
            let ext_data = &x[cursor..];
            let extension = ext::ManifestExtension::new(ext_data);
            cursor += extension.hdr.length as usize;
            extensions.push(extension);
        }
        CodePartitionManifest { header, crypto, extensions }
    }
}

