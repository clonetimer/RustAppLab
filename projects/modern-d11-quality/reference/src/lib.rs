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
    if payload.len() > MAX_PAYLOAD {
        return Err(FrameError::TooLong);
    }
    let mut frame = Vec::with_capacity(payload.len() + 3);
    frame.extend([MAGIC, payload.len() as u8]);
    frame.extend_from_slice(payload);
    frame.push(checksum(payload));
    Ok(frame)
}

/// Borrow the payload from a complete frame. Never allocate to decode.
pub fn decode(frame: &[u8]) -> Result<&[u8], FrameError> {
    if frame.len() < 3 {
        return Err(FrameError::TooShort);
    }
    if frame[0] != MAGIC {
        return Err(FrameError::BadMagic);
    }
    let length = usize::from(frame[1]);
    if length > MAX_PAYLOAD {
        return Err(FrameError::TooLong);
    }
    if frame.len() != length + 3 {
        return Err(FrameError::Length);
    }
    let payload = &frame[2..2 + length];
    if frame[2 + length] != checksum(payload) {
        return Err(FrameError::Checksum);
    }
    Ok(payload)
}

pub fn count_loop(values: &[u8], threshold: u8) -> usize {
    let mut count = 0;
    for &value in values {
        if value >= threshold {
            count += 1;
        }
    }
    count
}

pub fn count_iter(values: &[u8], threshold: u8) -> usize {
    values.iter().filter(|&&value| value >= threshold).count()
}
