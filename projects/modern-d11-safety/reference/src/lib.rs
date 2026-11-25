//! Small safe interfaces with narrowly scoped unsafe internals.
//! Prefer the ordinary safe slice/array APIs in real code where available.
use std::mem::MaybeUninit;

/// Read only a valid in-bounds byte.
pub fn checked_read(bytes: &[u8], index: usize) -> Option<u8> {
    if index >= bytes.len() {
        return None;
    }
    // SAFETY: index < bytes.len(); the shared slice supplies initialized u8s.
    Some(unsafe { *bytes.get_unchecked(index) })
}

/// Copy a prefix into a fixed-size array. This is an initialization exercise.
pub fn initialized_prefix<const N: usize>(values: &[u32]) -> Option<[u32; N]> {
    let prefix = values.get(..N)?;
    let mut slots: [MaybeUninit<u32>; N] = [const { MaybeUninit::uninit() }; N];
    for (slot, value) in slots.iter_mut().zip(prefix) {
        slot.write(*value);
    }
    // SAFETY: prefix has exactly N initialized u32s and every slot was written.
    // u32 is Copy and has no destructor; no generic drop guard is implied.
    Some(slots.map(|slot| unsafe { slot.assume_init() }))
}
