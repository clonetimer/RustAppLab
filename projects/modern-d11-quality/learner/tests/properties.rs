//! Reproducible, bounded generated-input checks. No claim of exhaustive fuzzing.
use d11_quality_lab::*;

struct Generator(u64);

impl Generator {
    fn byte(&mut self) -> u8 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 24) as u8
    }

    fn bytes(&mut self, length: usize) -> Vec<u8> {
        (0..length).map(|_| self.byte()).collect()
    }
}

#[test]
fn roundtrip_and_single_bit_faults() {
    let mut generator = Generator(0x2c85_06d4_950c_1231);
    for case in 0..4096 {
        let length = case % (MAX_PAYLOAD + 1);
        let payload = generator.bytes(length);
        let frame = encode(&payload).unwrap();
        assert_eq!(decode(&frame).unwrap(), payload, "seed case {case}");
        // All single-bit positions in a representative frame for each length.
        if case < MAX_PAYLOAD + 1 {
            for index in 0..frame.len() {
                for bit in 0..8 {
                    let mut damaged = frame.clone();
                    damaged[index] ^= 1 << bit;
                    assert!(decode(&damaged).is_err(), "case={case} byte={index} bit={bit}");
                }
            }
        }
        let mut trailing = frame;
        trailing.push(0);
        assert!(decode(&trailing).is_err(), "trailing bytes accepted, case {case}");
    }
}

#[test]
fn arbitrary_bytes_never_panic_and_accepted_frames_are_canonical() {
    let mut generator = Generator(0x7e18_9cf2_a10b_4d35);
    for case in 0..8192 {
        let mut bytes = generator.bytes(case % 100);
        // Increase meaningful coverage of the header and length checks.
        if case % 3 == 0 && bytes.len() >= 3 {
            bytes[0] = MAGIC;
            bytes[1] = (bytes.len() - 3) as u8;
        }
        if let Ok(payload) = decode(&bytes) {
            assert_eq!(encode(payload).unwrap(), bytes, "noncanonical accepted case {case}");
        }
    }
    for byte in 0..=u8::MAX {
        assert!(decode(&[byte]).is_err());
    }
}

#[test]
fn implementations_agree_with_histogram_oracle() {
    let mut generator = Generator(0x174c_7ad2_8e09_a6d1);
    for case in 0..1024 {
        let values = generator.bytes(case % 129);
        let mut histogram = [0usize; 256];
        for &value in &values {
            histogram[value as usize] += 1;
        }
        for threshold in [0, generator.byte(), 127, 255] {
            let expected: usize = histogram[usize::from(threshold)..].iter().sum();
            assert_eq!(count_loop(&values, threshold), expected, "loop case {case}");
            assert_eq!(count_iter(&values, threshold), expected, "iterator case {case}");
        }
    }
}
