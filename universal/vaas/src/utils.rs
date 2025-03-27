pub use ruint::aliases::U256;

/// Simple keccak256 hash with configurable backend.
#[inline]
pub fn keccak256(buf: impl AsRef<[u8]>) -> [u8; 32] {
    #[cfg(feature = "alloy")]
    alloy_primitives::keccak256(buf).0;
    #[cfg(not(feature = "alloy"))]
    <sha3::Keccak256 as sha3::Digest>::digest(buf).into()
}

/// Return the number of guardians to reach quorum.
#[inline]
pub fn quorum(n: usize) -> usize {
    (n * 2) / 3 + 1
}
