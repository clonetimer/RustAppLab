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

/// Return 0 below low, 1 in [low, high), 2 at or above high.
pub fn classify(value: i32, low: i32, high: i32) -> Result<i32, i32> {
    if !(-LIMIT..=LIMIT).contains(&value)
        || !(-LIMIT..=LIMIT).contains(&low)
        || !(-LIMIT..=LIMIT).contains(&high)
        || low >= high
    {
        return Err(BAD_INPUT);
    }
    Ok(if value < low { 0 } else if value < high { 1 } else { 2 })
}

pub fn summarize(samples: &[i32], low: i32, high: i32) -> Result<Stats, i32> {
    if samples.is_empty() || samples.len() > MAX_SAMPLES {
        return Err(BAD_LENGTH);
    }
    let mut result = Stats {
        count: samples.len() as u32,
        min: samples[0],
        max: samples[0],
        sum: 0,
        high_count: 0,
    };
    for &sample in samples {
        let level = classify(sample, low, high)?;
        result.min = result.min.min(sample);
        result.max = result.max.max(sample);
        // At most 4096 * 100000, safely inside i32.
        result.sum += sample;
        result.high_count += u32::from(level == 2);
    }
    Ok(result)
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
    let n = len as usize;
    if n == 0 || n > MAX_SAMPLES {
        return BAD_LENGTH;
    }
    if samples.is_null() || out.is_null() || !samples.is_aligned() || !out.is_aligned() {
        return BAD_POINTER;
    }
    // SAFETY: the caller guarantees the live, initialized, disjoint allocation.
    let values = unsafe { std::slice::from_raw_parts(samples, n) };
    match summarize(values, low, high) {
        Ok(stats) => {
            // SAFETY: the caller supplies exclusive writable Stats storage.
            unsafe { out.write(stats) };
            0
        }
        Err(code) => code,
    }
}
