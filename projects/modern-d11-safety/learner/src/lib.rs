//! Implement safe interfaces. Prefer safe APIs unless practicing a documented invariant.
pub fn checked_read(bytes: &[u8], index: usize) -> Option<u8> {
    // TODO(1): validate index before reading; return None for invalid input.
    let _ = (bytes, index);
    None
}

pub fn initialized_prefix<const N: usize>(values: &[u32]) -> Option<[u32; N]> {
    // TODO(2): require N elements and initialize every slot; N=0 is valid.
    let _ = values;
    None
}
