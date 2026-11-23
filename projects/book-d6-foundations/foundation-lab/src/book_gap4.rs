//! Four formerly unmapped Rust Book concepts, with positive and failure probes.
//!
//! `HashMap` can be built from zipped pairs. A type annotation is
//! required when the destination cannot be inferred:
//!
//! ```
//! use std::collections::HashMap;
//! let map: HashMap<_, _> = ["Ada", "Lin"].into_iter().zip([3, 5]).collect();
//! assert_eq!(map.get("Lin"), Some(&5));
//! ```
//!
//! Omit the needed type context in a generic/ambiguous expression and the
//! compiler cannot infer the concrete collection:
//!
//! ```compile_fail
//! let map = ["Ada", "Lin"].into_iter().zip([3, 5]).collect();
//! let _ = map.get("Lin");
//! ```
//!
//! A panic is not a substitute for `Result`. The backtrace probe in
//! `examples/panic_backtrace_probe.rs` runs out of process. Test harnesses
//! must never switch their own panic strategy to abort.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

/// Map two parallel lists. Extra elements are dropped by zip; explain that boundary.
pub fn zipped_map<'a>(keys: &'a [&'a str], values: &[u32]) -> HashMap<&'a str, u32> {
    keys.iter().copied().zip(values.iter().copied()).collect()
}

/// Demonstrate explicit hasher selection. Hash implementation details and
/// ordering are *not* a cross-version, cryptographic or deterministic contract.
pub fn custom_hasher_map() -> HashMap<String, u32, BuildHasherDefault<DefaultHasher>> {
    let mut map = HashMap::with_hasher(BuildHasherDefault::<DefaultHasher>::default());
    map.insert("notes".to_string(), 7);
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_collect_selects_the_requested_map_type() {
        let map = zipped_map(&["Ada", "Lin"], &[3, 5]);
        assert_eq!(map.get("Ada"), Some(&3));
        assert_eq!(map.get("Lin"), Some(&5));
        assert_eq!(zipped_map(&["Ada"], &[3, 5]).len(), 1);
    }

    #[test]
    fn build_hasher_changes_policy_not_map_lookup_contract() {
        let default_map = HashMap::from([("notes".to_string(), 7)]);
        let custom_map = custom_hasher_map();
        assert_eq!(default_map.get("notes"), custom_map.get("notes"));
        assert!(custom_map.contains_key("notes"));
    }

    #[test]
    fn unwind_can_be_caught_but_is_not_a_user_input_error() {
        let result = std::panic::catch_unwind(|| panic!("isolated teaching panic"));
        assert!(result.is_err());
    }
}
