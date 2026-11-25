//! D11-A: a bounded teaching frame codec; XOR detects single-bit faults only.
pub const MAGIC: u8 = 0xa5;
pub const MAX_PAYLOAD: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub enum FrameError {
    TooShort,
    BadMagic,
    TooLong,
    Length,
    Checksum,
}

/// XOR includes the length byte. Not a CRC, MAC or cryptographic guarantee.
pub fn checksum(payload: &[u8]) -> u8 {
    payload.iter().fold(payload.len() as u8, |acc, &value| acc ^ value)
}

/// Exact wire format: magic, length, payload, checksum.
pub fn encode(payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    // TODO(1): enforce size, emit header, payload and checksum.
    let _ = payload;
    Err(FrameError::TooLong)
}

/// Borrow the payload from a complete frame. Never allocate to decode.
pub fn decode(frame: &[u8]) -> Result<&[u8], FrameError> {
    // TODO(2): validate length before indexing, then magic and checksum.
    let _ = frame;
    Err(FrameError::TooShort)
}

pub fn count_loop(values: &[u8], threshold: u8) -> usize {
    // TODO(3): count values >= threshold with a loop.
    let _ = (values, threshold);
    0
}

pub fn count_iter(values: &[u8], threshold: u8) -> usize {
    // TODO(4): same contract with iterator/filter/count.
    let _ = (values, threshold);
    0
}
