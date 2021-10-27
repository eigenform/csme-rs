
use std::env;
use std::str::from_utf8;
use csme_rs::{ FromBytes, fpt::*, part::*, ext::* };

/// Read bytes from a file.
fn read_file(filename: &str) -> Vec<u8> {
    use std::fs::File;
    use std::io::{ Read };

    let mut file = File::open(filename).expect("Couldn't open file");
    let file_len = std::fs::metadata(filename).unwrap().len() as usize;
    let mut res = vec![0; file_len];
    file.read(&mut res).unwrap();
    res
}

pub fn main() -> Result<(), &'static str> {
    use hex::encode;
    use sha2::{ Sha256, Digest };

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <CSME image>", args[0]);
        return Err("Invalid arguments");
    }

    let file_buf = read_file(&args[1]);

    let fpt = FlashPartitionTable::from_bytes(&file_buf[0x10..0x1000]);

    for entry in fpt.entries.iter() {
        if !entry.attrs.entry_valid() { 
            continue
        }
        if entry.attrs.kind() != PartitionType::Code {
            println!("[!] Skipped data partition {}", 
                from_utf8(&entry.name).unwrap());
            continue;
        }

        println!("[*] Parsing partition {}", from_utf8(&entry.name).unwrap());
        let off = entry.offset as usize;
        let len = entry.length as usize;
        assert!(off + len <= file_buf.len());
        assert!(off < file_buf.len());
        let part = CodePartition::new(&file_buf[off..off+len]);

        println!("[*] Directory for {}", from_utf8(&entry.name).unwrap());
        for f in part.cpd.entries.iter() {
            println!("  - {}", f.filename());
        }

        for (name, m) in part.modules.iter() {

            // Compute the SHA256 digest for this module.
            // NOTE: I think I'm doing something wrong here
            let computed_digest = {
                let mut hasher = Sha256::new();
                match m.attr.compression_type() {
                    CompressionType::None => unimplemented!(),
                    CompressionType::Lzma => hasher.update(&m.raw_data),
                    CompressionType::Huff => hasher.update(&m.raw_data),
                }
                hasher.finalize()
            };

            println!("  => Found module '{}'", name);
            println!("     | Compression:     {:?}", m.attr.compression_type());
            //println!("     | Module ID:       {:04x}", m.attr.ven_module_id);
            //println!("     | {:02x?}", m.attr.sha256_digest);
            //println!("     | {:02x?}", computed_digest);
        }

    }

    Ok(())
}


