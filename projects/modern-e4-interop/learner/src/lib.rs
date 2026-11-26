//! Shared integer signal analysis. C owns its buffers; JS uses scalar exports.
pub const LIMIT: i32 = 100_000;
pub const MAX_SAMPLES: usize = 4096;
pub const BAD_INPUT: i32 = -1;
pub const BAD_LENGTH: i32 = -2;
pub const BAD_POINTER: i32 = -3;
pub const NOT_IMPLEMENTED: i32 = -9;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub count: u32,
    pub min: i32,
    pub max: i32,
    pub sum: i32,
    pub high_count: u32,
}

/// TODO(1): classify a bounded sample; reject malformed thresholds and values.
pub fn classify(_value: i32, _low: i32, _high: i32) -> Result<i32, i32> {
    Err(NOT_IMPLEMENTED)
}

/// TODO(2): summarize the bounded slice with the checked classifier.
pub fn summarize(_samples: &[i32], _low: i32, _high: i32) -> Result<Stats, i32> {
    Err(NOT_IMPLEMENTED)
}

#[unsafe(no_mangle)]
pub extern "C" fn e4_abi_version() -> u32 { 1 }

/// Scalar ABI shared by C and WebAssembly. No pointers or allocation.
#[unsafe(no_mangle)]
pub extern "C" fn e4_classify(value: i32, low: i32, high: i32) -> i32 {
    classify(value, low, high).unwrap_or_else(|error| error)
}

/// Native caller-owned memory interface. Returns 0 only after a successful write.
///
/// # Safety
/// For non-null, aligned pointers and a length in 1..=4096, samples must
/// designate that many initialized, readable i32 values in one live allocation.
/// out must designate writable, aligned Stats storage, disjoint from samples.
/// Both must remain valid for the call; no concurrent mutation is permitted.
/// Null/alignment/length checks cannot establish allocation validity.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn e4_summarize(
    samples: *const i32, len: u32, low: i32, high: i32, out: *mut Stats,
) -> i32 {
    // TODO(3): validate length/null/alignment before forming a slice; write
    // only after successful computation. The caller still owns memory.
    let _ = (samples, len, low, high, out);
    NOT_IMPLEMENTED
}
