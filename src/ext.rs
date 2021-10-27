//! Metadata extensions.

use crate::FromBytes;

/// A container for different kinds of extensions.
pub struct ManifestExtension {
    pub hdr: ExtensionHeader,
    pub data: ExtensionData,
}
impl ManifestExtension {
    pub fn new(x: &[u8]) -> Self {
        let hdr = ExtensionHeader::from_bytes(&x);
        let ext_bytes = &x[std::mem::size_of::<ExtensionHeader>()..];
        let data = ExtensionData::new(&hdr, ext_bytes);
        ManifestExtension { hdr, data }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ExtensionHeader { pub id: u32, pub length: u32 }
impl crate::FromBytes for ExtensionHeader {}

/// Variable length extension data.
#[derive(Debug)]
pub enum ExtensionData {
    /// Extension ID 0x0000_0000
    SystemInfo { data: SystemInfoExt, entries: Vec<IndependentPartitionEntry> },
    /// Extension ID 0x0000_0001
    InitScript { data: InitScriptExt, entries: Vec<InitScriptEntry> },
    /// Extension ID 0x0000_0002
    FeaturePermissions { data: FeaturePermissionsExt,
        entries: Vec<FeaturePermissionsEntry>,
    },
    /// Extension ID 0x0000_0003
    PartitionInfo { data: ManifestPartitionInfoExt,
        entries: Vec<ManifestModuleInfoExt>,
    },
    /// Extension ID 0x0000_0004
    SharedLibrary { data: SharedLibExt },
    /// Extension ID 0x0000_0005
    ProcessAttrs { data: ManProcessExt, entries: Vec<ProcessGroupId> },
    /// Extension ID 0x0000_0006
    ThreadAttrs { entries: Vec<Thread> },
    /// Extension ID 0x0000_0007
    DeviceIds { entries: Vec<Device> },
    /// Extension ID 0x0000_0008
    MmioRanges { entries: Vec<MmioRange> },
    /// Extension ID 0x0000_0009
    SpecialFiles { data: SpecialFileProducerExt, entries: Vec<SpecialFileDef> },
    /// Extension ID 0x0000_000a
    ModuleAttrs { data: ModAttrExt },
    /// Extension ID 0x0000_000b
    LockedRanges { entries: Vec<LockedRange> },
    /// Extension ID 0x0000_000c
    ClientSystemInfo { data: ClientSystemInfoExt },
    /// Extension ID 0x0000_000d
    UserInfo { entries: Vec<UserInfoEntry> },
}
impl ExtensionData {
    pub fn new(hdr: &ExtensionHeader, x: &[u8]) -> Self {
        macro_rules! parse_ext_ent {
            ($enum:ident, $hdr:ident, $ent:ident) => {{
                let data = $hdr::from_bytes(x);
                let entry_off = std::mem::size_of::<$hdr>();
                let entry_data = &x[entry_off..];
                let num_entries = entry_data.len() / std::mem::size_of::<$ent>();
                let mut entries = vec![$ent::default(); num_entries];
                unsafe {
                    entries.copy_from_slice(std::slice::from_raw_parts(
                        entry_data.as_ptr() as *const $ent, num_entries));
                }
                Self::$enum { data, entries }
            }}
        }
        macro_rules! parse_ent {
            ($enum:ident, $ent:ident) => {{
                let entry_data = &x;
                let num_entries = entry_data.len() / std::mem::size_of::<$ent>();
                let mut entries = vec![$ent::default(); num_entries];
                unsafe {
                    entries.copy_from_slice(std::slice::from_raw_parts(
                        entry_data.as_ptr() as *const $ent, num_entries));
                }
                Self::$enum { entries }
            }}
        }

        match hdr.id {
            0x0 => parse_ext_ent!(SystemInfo, SystemInfoExt, 
                                  IndependentPartitionEntry),
            0x1 => parse_ext_ent!(InitScript, InitScriptExt, InitScriptEntry),
            0x2 => parse_ext_ent!(FeaturePermissions, FeaturePermissionsExt, 
                                  FeaturePermissionsEntry),
            0x3 => parse_ext_ent!(PartitionInfo, ManifestPartitionInfoExt, 
                                  ManifestModuleInfoExt),
            0x4 => Self::SharedLibrary { 
                data: SharedLibExt::from_bytes(x)
            },
            0x5 => parse_ext_ent!(ProcessAttrs, ManProcessExt, ProcessGroupId),
            0x6 => parse_ent!(ThreadAttrs, Thread),
            0x7 => parse_ent!(DeviceIds, Device),
            0x8 => parse_ent!(MmioRanges, MmioRange),
            0x9 => parse_ext_ent!(SpecialFiles, SpecialFileProducerExt, 
                                  SpecialFileDef),
            0xa => Self::ModuleAttrs { 
                data: ModAttrExt::from_bytes(x)
            },
            0xb => parse_ent!(LockedRanges, LockedRange),
            0xc => Self::ClientSystemInfo { 
                data: ClientSystemInfoExt::from_bytes(x) 
            },
            0xd => parse_ent!(UserInfo, UserInfoEntry),
            _ => panic!("Unimplemented extension ID {:08x}", hdr.id),
        }
    }
}

/// Extension ID 0x0000_0000
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SystemInfoExt {
    min_uma_size: u32,
    chipset_version: u32,
    default_sha256_digest: [u8; 32],
    pageable_uma_size: u32,
    reserved_0: u64,
    reserved_1: u32,
}
impl crate::FromBytes for SystemInfoExt {}

/// Extension ID 0x0000_0001
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InitScriptExt { reserved: u32, num_modules: u32 }
impl crate::FromBytes for InitScriptExt {}

/// Extension ID 0x0000_0002
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FeaturePermissionsExt { num_modules: u32 }
impl crate::FromBytes for FeaturePermissionsExt {}

/// Extension ID 0x0000_0003
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ManifestPartitionInfoExt {
    part_name: u32,
    part_len: u32,
    part_sha256_digest: [u8; 32],
    version_control_number: u32,
    part_version: u32,
    format_version: u32,
    instance_id: u32,
    flags: u32,
    reserved: [u8; 20],
}
impl crate::FromBytes for ManifestPartitionInfoExt {}

/// Extension ID 0x0000_0004
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SharedLibExt {
    context_size: u32,
    total_alloc_virtual_space: u32,
    code_base_address: u32,
    tls_size: u32,
    reserved: u32,
}
impl crate::FromBytes for SharedLibExt {}

/// Extension ID 0x0000_0005
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ManProcessExt {
    flags: ManProcessExtFlags,
    main_thread_id: u32,
    code_base_address: u32,
    uncompressed_code_size: u32,
    cm0_heap_size: u32,
    bss_size: u32,
    default_heap_size: u32,
    main_thread_entry: u32,
    allowed_syscalls: [u8; 12],
    user_id: u16,
    reserved_0: u32,
    reserved_1: u16,
    reserved_2: u64,
}
impl crate::FromBytes for ManProcessExt {}

/// Extension ID 0x0000_0009
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SpecialFileProducerExt { dev_major_id: u16, flags: u16 }
impl crate::FromBytes for SpecialFileProducerExt {}

/// Extension ID 0x0000_000a
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModAttrExt {
    /// 0 - Uncompressed 1 - Huffman Compressed 2 - LZMA Compressed
    pub compression_type: u8,
    pub reserved0: u8,
    pub reserved1: u8,
    pub reserved2: u8,
    pub uncompressed_size: u32,
    pub compressed_size: u32,
    pub ven_module_id: u16,
    pub ven_id: u16,
    pub sha256_digest: [u8; 32],
    //pub sha256_digest: [u32; 8],
}
impl crate::FromBytes for ModAttrExt {}
impl ModAttrExt {
    pub fn compression_type(&self) -> CompressionType {
        CompressionType::from(self.compression_type)
    }
    pub fn compressed_size(&self) -> usize {
        self.compressed_size as usize
    }
    pub fn uncompressed_size(&self) -> usize {
        self.uncompressed_size as usize
    }

}

#[derive(Debug)]
#[repr(u8)]
pub enum CompressionType { None = 0, Huff = 1, Lzma = 2 }
impl From<u8> for CompressionType {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::None, 1 => Self::Huff, 2 => Self::Lzma,
            _ => unreachable!(),
        }
    }
}

/// Extension ID 0x0000_000c
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClientSystemInfoExt {
    sku_cap: u32,
    sku_cap_reserved: [u8; 28],
    sku_attrs: u64,
}
impl crate::FromBytes for ClientSystemInfoExt {}



#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IndependentPartitionEntry {
    name: [u8; 4],
    version: u32,
    user_id: u16,
    reserved: u16,
}
impl crate::FromBytes for IndependentPartitionEntry {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InitScriptEntry {
    partition_name: [u8; 4],
    name: [u8; 12],
    init_flags: u32,
    boot_type: u32,
}
impl crate::FromBytes for InitScriptEntry {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FeaturePermissionsEntry { user_id: u16, reserved: u16 }
impl crate::FromBytes for FeaturePermissionsEntry {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ManifestModuleInfoExt {
    name: [u8; 12],
    kind: u8,
    reserved0: u8,
    reserved1: u16,
    metadata_size: u32,
    metadata_sha256_digest: [u8; 32],
}
impl crate::FromBytes for ManifestModuleInfoExt {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessGroupId { group_id: u16, }
impl crate::FromBytes for ProcessGroupId {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ManProcessExtFlags(pub u32);
impl ManProcessExtFlags {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Thread {
    stack_size: u32,
    flags: u32,
    scheduling_policy: u32,
    reserved: u32,
}
impl crate::FromBytes for Thread {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LockedRange { range_base: u32, range_size: u32 }
impl crate::FromBytes for LockedRange {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Device { device_id: u32, reserved: u32 }
impl crate::FromBytes for Device {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MmioRange { base: u32, size: u32, flags: u32 }
impl crate::FromBytes for MmioRange {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SpecialFileDef {
    name: [u8; 12],
    access_mode: u16,
    uid: u16,
    gid: u16,
    dev_minor_id: u8,
    reserved0: u8,
    reserved1: u32,
}
impl crate::FromBytes for SpecialFileDef {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UserInfoEntry {
    id: u16,
    reserved: u16,
    nvram_storage_quota: u32,
    ram_storage_quota: u32,
    wop_quota: u32,
    working_dir: [u8; 36],
}
impl crate::FromBytes for UserInfoEntry {}
impl Default for UserInfoEntry {
    fn default() -> Self {
        Self {
            id: 0, reserved: 0, nvram_storage_quota: 0,
            ram_storage_quota: 0, wop_quota: 0, working_dir: [0; 36],
        }
    }
}

