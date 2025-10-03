#[cfg(target_arch = "x86_64")]
use core::arch::x86_64;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64;

#[cfg(all(feature = "simd", target_arch = "aarch64"))]
pub(crate) type Chunk = aarch64::uint64x2x4_t;

/// Interface for working with chunks of bytes at a time, providing the
/// operations needed for the functionality in str_utils.
pub(crate) trait ByteChunk: Copy + Clone {
    /// Size of the chunk in bytes.
    const SIZE: usize;

    /// Creates a new chunk with all bytes set to zero.
    fn zero() -> Self;
}

#[cfg(all(feature = "simd", target_arch = "aarch64"))]
impl ByteChunk for Chunk {
    const SIZE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        todo!()
    }
}
