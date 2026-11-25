#![no_main]
use d11_quality_lab::{MAX_PAYLOAD, decode, encode};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // All byte inputs may be rejected, but accepted bytes must be canonical.
    if let Ok(payload) = decode(data) {
        assert_eq!(encode(payload).unwrap(), data);
    }
    if data.len() <= MAX_PAYLOAD {
        let mut frame = encode(data).unwrap();
        assert_eq!(decode(&frame).unwrap(), data);
        let last = frame.len() - 1;
        frame[last] ^= 1;
        assert!(decode(&frame).is_err(), "single-bit checksum mutation accepted");
    } else {
        assert!(encode(data).is_err());
    }
});
