
use std::collections::BTreeMap;
use crate::{ 
    cpd::*,
    man::*,
    ext::*,
    huffman::*,
};

/// Representing a CSME11 module.
pub struct Module {
    /// Filename for this module.
    pub name: String,
    /// Module attributes.
    pub attr: ModAttrExt,
    /// Set of extensions associated with this module.
    pub ext: Vec<ManifestExtension>,
    /// Decompressed contents of this module.
    pub data: Vec<u8>,
    /// Original contents of the module file.
    pub raw_data: Vec<u8>,
}

/// Representing a code partition (containing CSME modules).
pub struct CodePartition {
    /// Directory of files in this partition
    pub cpd: CodePartitionDirectory,
    /// Manifest for this partition
    pub man: CodePartitionManifest,
    /// Map from module names to copies of module data (and metadata)
    pub modules: BTreeMap<String, Module>,
    /// Copy of the raw data for this partition
    _raw_data: Vec<u8>,
}
impl CodePartition {
    pub fn new(data: &[u8]) -> Self {
        let mut modules: BTreeMap<String, Module> = BTreeMap::new();
        let part_data = data.to_vec();

        let cpd = CodePartitionDirectory::new(&part_data);

        // NOTE: The manifest is typically the first entry in the directory.
        let man = if cpd.entries[0].filename().ends_with(".man") {
            let off = cpd.entries[0].attrs.address() as usize;
            let len = cpd.entries[0].length as usize;
            CodePartitionManifest::new(&part_data[off..off+len])
        } else {
            panic!("No manifest for code partition");
        };

        // Use the metadata files in this partition to make a map of modules
        for e in cpd.entries.iter().filter(|x| x.filename().ends_with(".met")) {
            assert_eq!(e.attrs.compress_flag(), false);
            let met_data = &part_data[e.offset()..e.offset() + e.len()];
            let module_name = std::str::from_utf8(&e.name)
                .expect("Couldn't interpret CPD entry name as UTF-8")
                .trim_end_matches(char::from(0)).trim_end_matches(".met")
                .to_owned();
            let mut module_attr = None;
            let extensions: Vec<ManifestExtension> = {
                let mut cur: usize = 0;
                let mut res = Vec::new();
                while cur < met_data.len() {
                    let ext_data = &met_data[cur..];
                    let ext = ManifestExtension::new(ext_data);
                    if let ExtensionData::ModuleAttrs { data } = ext.data {
                        module_attr = Some(data);
                    }
                    cur += ext.hdr.length as usize;
                    res.push(ext);
                }
                res
            };
            modules.insert(module_name.clone(), 
                Module { 
                    name: module_name,
                    attr: module_attr.unwrap(),
                    ext: extensions, 
                    data: Vec::new(),
                    raw_data: Vec::new(),
                }
            );
        }

        // Decompress the contents of each module
        for (name, module) in &mut modules {
            let raw_data: Vec<&[u8]> = cpd.entries.iter().filter_map(|x| {
                if &x.filename() == name {
                    Some(&part_data[x.offset()..x.offset() + x.len()])
                } else {
                    None
                }
            }).collect();
            assert_eq!(raw_data.len(), 1);
            let mod_data: Vec<u8> = match module.attr.compression_type() {
                CompressionType::None => raw_data[0].to_vec(),
                CompressionType::Lzma => {
                    let mut buf = Vec::new();
                    buf.extend_from_slice(&raw_data[0][..0xe]);
                    buf.extend_from_slice(&raw_data[0][0x11..]);
                    match lzma::decompress(&buf) {
                        Ok(res) => res,
                        Err(e) => panic!("{}", e),
                    }
                },
                CompressionType::Huff => 
                    decompress_huff(&raw_data[0], &module.attr),
            };
            assert_eq!(mod_data.len(), module.attr.uncompressed_size());
            module.data = mod_data;
            module.raw_data = raw_data[0].to_vec();
        }

        Self { cpd, man, modules, _raw_data: part_data }
    }
}


