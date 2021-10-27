pub mod fpt;
pub mod cpd;
pub mod man;
pub mod ext;
pub mod part;
pub mod huffman;

/// Trait implemented for types that can be cast from a byte-array.
///
/// NOTE: Types implementing this probably need to be `#[repr(C)]`.
pub trait FromBytes {
    fn validate(&self) -> Result<(), &'static str> { Ok(()) }
    fn from_bytes(x: &[u8]) -> Self where Self: Sized {
        assert!(x.len() >= std::mem::size_of::<Self>());
        let res = unsafe { std::ptr::read(x.as_ptr() as *const _) };
        match Self::validate(&res) {
            Ok(_) => {},
            Err(e) => panic!("{:?}", e),
        }
        res
    }
}

