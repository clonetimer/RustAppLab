//! Executable reference examples for Rust Book advanced features, chapter 19.
//!
//! Unsafe operations stay in small, documented blocks. The examples never
//! construct invalid references or dereference invalid pointers.
//!
//! Calling an unsafe API without an unsafe block is rejected:
//!
//! ~~~compile_fail
//! use rustapplab_book_d4_lab::add_to_raw_pointer;
//! let mut value = 1;
//! add_to_raw_pointer(&mut value, 2);
//! ~~~
//!
//! Newtypes are distinct types even when their inner representation is equal:
//!
//! ~~~compile_fail
//! use rustapplab_book_d4_lab::Meters;
//! let _wrong: Meters = 5;
//! ~~~

#![deny(unsafe_op_in_unsafe_fn)]

extern crate self as rustapplab_book_d4_lab;

use std::fmt::{self, Display};
use std::ops::Add;

pub use rustapplab_book_d4_macros::{audit_tag, token_count, Describe};

/// A trait whose implementors provide a static description.
pub trait Describe {
    /// Returns the type's stable teaching label.
    fn describe(&self) -> &'static str;
}

/// A declarative macro that generates a public record with named fields.
#[macro_export]
macro_rules! make_record {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $(pub $field: $ty),*
        }
    };
}

/// A declarative macro that constructs a pair expression.
#[macro_export]
macro_rules! make_pair {
    ($left:expr, $right:expr $(,)?) => {
        ($left, $right)
    };
}

make_record!(GeneratedReading {
    source: String,
    pages: usize,
});

/// Sums a slice by walking its raw pointer inside a safe abstraction.
pub fn sum_via_raw_pointer(values: &[i32]) -> i32 {
    let pointer = values.as_ptr();
    let mut total = 0;
    let mut offset = 0;
    while offset < values.len() {
        // SAFETY: pointer comes from values; offset is strictly less than
        // the slice length, so it is in-bounds and initialized.
        total += unsafe { *pointer.add(offset) };
        offset += 1;
    }
    total
}

/// Adds delta to a caller-provided raw pointer.
///
/// # Safety
///
/// pointer must be non-null, aligned for i32, valid for reads and writes,
/// initialized, and exclusively accessible for the duration of this call.
pub unsafe fn add_to_raw_pointer(pointer: *mut i32, delta: i32) {
    // SAFETY: the caller guarantees pointer validity, alignment, initialization,
    // and exclusive access for the complete read-modify-write operation.
    unsafe {
        *pointer = (*pointer).wrapping_add(delta);
    }
}

/// A union demonstrates that reading a field requires an external active-field invariant.
#[repr(C)]
#[derive(Clone, Copy)]
pub union NumberUnion {
    /// Integer representation.
    pub integer: u32,
    /// Floating-point representation.
    pub float: f32,
}

/// Reads the integer field of a union.
///
/// # Safety
///
/// The integer field must be the field most recently written to value.
pub unsafe fn read_union_integer(value: NumberUnion) -> u32 {
    // SAFETY: the caller promises that the integer field is the active field.
    unsafe { value.integer }
}

/// A UTF-8 view contract that implementors must uphold.
///
/// # Safety
///
/// Implementors must guarantee that every returned string slice is valid UTF-8
/// and remains backed by the same valid bytes for the lifetime of the borrow.
pub unsafe trait ValidUtf8 {
    /// Returns the valid UTF-8 contents for the borrow's lifetime.
    fn as_str(&self) -> &str;
}

/// A fixed-size ASCII label with private storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiLabel {
    bytes: [u8; 8],
    length: usize,
}

impl AsciiLabel {
    /// Creates a label containing at most eight ASCII bytes.
    pub fn new(value: &str) -> Result<Self, &'static str> {
        if !value.is_ascii() {
            return Err("label must contain ASCII characters");
        }
        if value.len() > 8 {
            return Err("label must contain at most eight bytes");
        }
        let mut bytes = [0; 8];
        bytes[..value.len()].copy_from_slice(value.as_bytes());
        Ok(Self {
            bytes,
            length: value.len(),
        })
    }
}

// SAFETY: the only constructor accepts ASCII strings, the byte array is private,
// and the stored length never exceeds the fixed array capacity.
unsafe impl ValidUtf8 for AsciiLabel {
    fn as_str(&self) -> &str {
        // SAFETY: ASCII is valid UTF-8 and the private length is bounded by 8.
        unsafe { std::str::from_utf8_unchecked(&self.bytes[..self.length]) }
    }
}

mod foreign {
    extern "C" {
        pub fn d4_ffi_identity(value: i32) -> i32;
    }
}

/// A Rust function exported with the C ABI for the FFI example.
#[no_mangle]
pub extern "C" fn d4_ffi_identity(value: i32) -> i32 {
    value
}

/// Calls the matching C-ABI symbol declared in the foreign module.
pub fn call_ffi_identity(value: i32) -> i32 {
    // SAFETY: the symbol is defined in this crate with the same C ABI and
    // signature; no pointers, ownership, or unwinding cross the ABI boundary.
    unsafe { foreign::d4_ffi_identity(value) }
}

static mut SINGLE_THREAD_COUNTER: u32 = 0;

/// Adds to a deliberately single-threaded mutable static counter.
///
/// # Safety
///
/// The caller must guarantee that no other thread or call reads or writes this
/// static at the same time. Prefer atomics or a mutex in real applications.
pub unsafe fn increment_single_thread_counter(delta: u32) -> u32 {
    let pointer = std::ptr::addr_of_mut!(SINGLE_THREAD_COUNTER);
    // SAFETY: the caller upholds exclusive access; raw-pointer access avoids
    // creating a reference to a mutable static.
    unsafe {
        *pointer = (*pointer).wrapping_add(delta);
        *pointer
    }
}

/// A trait with an associated output type.
pub trait Measurement {
    /// The unit or representation returned by measure.
    type Unit;

    /// Returns the measurement.
    fn measure(&self) -> Self::Unit;
}

/// A pulse counter used by the associated-type example.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pulse(pub u32);

impl Measurement for Pulse {
    type Unit = u32;

    fn measure(&self) -> Self::Unit {
        self.0
    }
}

/// A point whose default coordinate type is i32.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point<T = i32> {
    /// Horizontal coordinate.
    pub x: T,
    /// Vertical coordinate.
    pub y: T,
}

impl Add for Point<i32> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Millimeters demonstrate the default Add Rhs = Self parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Millimeters(pub u32);

/// Meters are a distinct newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Meters(pub u32);

impl Add for Millimeters {
    type Output = Millimeters;

    fn add(self, other: Self) -> Self::Output {
        Millimeters(self.0 + other.0)
    }
}

impl Add<Meters> for Millimeters {
    type Output = Millimeters;

    fn add(self, other: Meters) -> Self::Output {
        Millimeters(self.0 + other.0 * 1_000)
    }
}

/// A trait with an associated function that can be fully qualified.
pub trait Pilot {
    /// Returns the trait's baby-name example.
    fn baby_name() -> &'static str;
}

/// A type with an inherent and trait-associated function of the same name.
pub struct Human;

impl Human {
    /// Returns the inherent implementation.
    pub fn baby_name() -> &'static str {
        "inherent name"
    }
}

impl Pilot for Human {
    fn baby_name() -> &'static str {
        "trait name"
    }
}

/// Calls the trait-associated function using fully qualified syntax.
pub fn fully_qualified_baby_name() -> &'static str {
    <Human as Pilot>::baby_name()
}

/// A supertrait example requiring Display.
pub trait OutlinePrint: Display {
    /// Renders the value inside a text outline.
    fn outline_print(&self) -> String {
        let output = self.to_string();
        let border = "*".repeat(output.len() + 4);
        format!("{border}\n* {output} *\n{border}")
    }
}

/// A displayable point that also implements the supertrait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPoint {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

impl Display for DisplayPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "({}, {})", self.x, self.y)
    }
}

impl OutlinePrint for DisplayPoint {}

/// A wrapper type that can implement a foreign trait for a foreign container.
pub struct Wrapper(pub Vec<String>);

impl Display for Wrapper {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0.join(", "))
    }
}

/// Parses meters while preserving a short domain-specific error type.
pub type UnitResult<T> = Result<T, &'static str>;

/// Parses a non-negative meter count.
pub fn parse_meters(value: &str) -> UnitResult<Meters> {
    let parsed = value.parse::<u32>().map_err(|_| "invalid meter count")?;
    if parsed > 1_000_000 {
        return Err("meter count exceeds the teaching limit");
    }
    Ok(Meters(parsed))
}

/// A short alias for an owned, thread-safe deferred computation.
pub type Thunk = Box<dyn Fn() + Send + 'static>;

/// A diverging function demonstrates the never type.
pub fn fail_fast() -> ! {
    panic!("D4 fail-fast path")
}

/// Accepts a possibly dynamically sized displayable value behind a reference.
pub fn display_len<T: Display + ?Sized>(value: &T) -> usize {
    value.to_string().len()
}

/// A function pointer example.
pub fn add_one(value: i32) -> i32 {
    value + 1
}

/// Calls a function pointer twice.
pub fn apply_twice(operation: fn(i32) -> i32, value: i32) -> i32 {
    operation(operation(value))
}

/// An enum constructor can be used as a function pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// A numeric status.
    Value(u32),
    /// A stopped status.
    Stop,
}

/// Converts values using the enum variant constructor as a function pointer.
pub fn statuses(values: &[u32]) -> Vec<Status> {
    values.iter().copied().map(Status::Value).collect()
}

/// Returns a closure that captures its argument by value.
pub fn make_adder(amount: i32) -> impl Fn(i32) -> i32 {
    move |value| value + amount
}

/// Returns one of two different closure types behind a trait object.
pub fn select_transform(add: bool) -> Box<dyn Fn(i32) -> i32> {
    if add {
        Box::new(|value| value + 10)
    } else {
        Box::new(|value| value * 10)
    }
}

/// A packet used to exercise all three procedural-macro forms.
#[audit_tag(telemetry)]
#[derive(Debug, Clone, PartialEq, Describe)]
pub struct Packet {
    /// Monotonic sequence.
    pub sequence: u32,
    /// Small payload.
    pub payload: [u8; 4],
}

/// Returns stable command output for the five D4 topics.
pub fn demo(topic: &str) -> Result<Vec<String>, &'static str> {
    match topic {
        "unsafe" => {
            let values = [2, 3, 5];
            let label = AsciiLabel::new("SAT")?;
            let union = NumberUnion { integer: 42 };
            // SAFETY: the integer field is the field initialized above.
            let integer = unsafe { read_union_integer(union) };
            Ok(vec![
                format!("raw-sum={}", sum_via_raw_pointer(&values)),
                format!("ascii={}", label.as_str()),
                format!("union-integer={integer}"),
                format!("ffi-identity={}", call_ffi_identity(17)),
            ])
        }
        "traits" => {
            let point = Point { x: 2, y: 3 } + Point { x: 4, y: 5 };
            let pulse = Pulse(12);
            Ok(vec![
                format!("point={},{}", point.x, point.y),
                format!("measurement={}", pulse.measure()),
                format!("qualified={}", fully_qualified_baby_name()),
            ])
        }
        "types" => {
            let meters = parse_meters("12")?;
            Ok(vec![
                format!("meters={}", meters.0),
                format!("display-len={}", display_len("Rust")),
            ])
        }
        "functions" => Ok(vec![
            format!("function-pointer={}", apply_twice(add_one, 3)),
            format!("returned-closure={}", make_adder(5)(7)),
            format!("boxed-closure={}", select_transform(false)(3)),
        ]),
        "macros" => {
            let packet = Packet {
                sequence: 7,
                payload: [1, 2, 3, 4],
            };
            Ok(vec![
                format!("derive={}", packet.describe()),
                format!("attribute={}", Packet::AUDIT_TAG),
                format!("function-macro={}", token_count!(alpha beta gamma)),
                format!("declarative-macro={:?}", make_pair!(1, "two")),
            ])
        }
        _ => Err("主题只能是 unsafe、traits、types、functions 或 macros"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_pointer_sum_handles_empty_slices() {
        assert_eq!(sum_via_raw_pointer(&[]), 0);
    }

    #[test]
    fn raw_pointer_sum_reads_only_slice_elements() {
        assert_eq!(sum_via_raw_pointer(&[2, 3, 5]), 10);
    }

    #[test]
    fn unsafe_raw_pointer_api_updates_a_valid_exclusive_pointer() {
        let mut value = 5;
        // SAFETY: value is aligned, initialized, live, and exclusively borrowed.
        unsafe { add_to_raw_pointer(std::ptr::addr_of_mut!(value), 4) };
        assert_eq!(value, 9);
    }

    #[test]
    fn ascii_label_exposes_valid_utf8() {
        let label = AsciiLabel::new("SAT").unwrap();
        assert_eq!(label.as_str(), "SAT");
    }

    #[test]
    fn ascii_label_rejects_non_ascii_input() {
        assert_eq!(
            AsciiLabel::new("卫星").unwrap_err(),
            "label must contain ASCII characters"
        );
    }

    #[test]
    fn ascii_label_rejects_values_over_capacity() {
        assert_eq!(
            AsciiLabel::new("123456789").unwrap_err(),
            "label must contain at most eight bytes"
        );
    }

    #[test]
    fn union_reads_the_field_that_was_initialized() {
        let value = NumberUnion { integer: 73 };
        // SAFETY: the integer field is the field initialized above.
        assert_eq!(unsafe { read_union_integer(value) }, 73);
    }

    #[test]
    fn extern_c_function_can_be_called_through_a_foreign_declaration() {
        assert_eq!(call_ffi_identity(17), 17);
    }

    #[test]
    fn mutable_static_example_is_used_only_under_exclusive_single_thread_access() {
        // SAFETY: this test is the only code path that accesses this private static.
        assert_eq!(unsafe { increment_single_thread_counter(2) }, 2);
    }

    #[test]
    fn associated_type_selects_the_measurement_output() {
        assert_eq!(Pulse(17).measure(), 17_u32);
    }

    #[test]
    fn add_uses_the_default_rhs_type_parameter() {
        let result = Point { x: 1, y: 2 } + Point { x: 3, y: 4 };
        assert_eq!(result, Point { x: 4, y: 6 });
    }

    #[test]
    fn add_supports_a_distinct_rhs_type() {
        assert_eq!(Millimeters(500) + Meters(2), Millimeters(2_500));
    }

    #[test]
    fn fully_qualified_syntax_selects_the_trait_function() {
        assert_eq!(Human::baby_name(), "inherent name");
        assert_eq!(fully_qualified_baby_name(), "trait name");
    }

    #[test]
    fn supertrait_default_method_uses_display() {
        let point = DisplayPoint { x: 3, y: 4 };
        assert_eq!(point.outline_print(), "**********\n* (3, 4) *\n**********");
    }

    #[test]
    fn newtype_wrapper_implements_display_for_a_foreign_container() {
        let wrapper = Wrapper(vec!["Rust".into(), "Book".into()]);
        assert_eq!(wrapper.to_string(), "Rust, Book");
    }

    #[test]
    fn unit_parser_accepts_a_valid_value() {
        assert_eq!(parse_meters("42"), Ok(Meters(42)));
    }

    #[test]
    fn unit_parser_rejects_invalid_and_excessive_values() {
        assert_eq!(parse_meters("x"), Err("invalid meter count"));
        assert_eq!(
            parse_meters("1000001"),
            Err("meter count exceeds the teaching limit")
        );
    }

    #[test]
    fn display_len_accepts_unsized_str() {
        assert_eq!(display_len("Rust"), 4);
    }

    #[test]
    fn display_len_accepts_sized_string() {
        assert_eq!(display_len(&String::from("卫星")), "卫星".len());
    }

    #[test]
    fn never_type_function_diverges() {
        let result = std::panic::catch_unwind(fail_fast);
        assert!(result.is_err());
    }

    #[test]
    fn function_pointer_runs_a_named_function_twice() {
        assert_eq!(apply_twice(add_one, 3), 5);
    }

    #[test]
    fn enum_variant_constructor_works_as_a_function_pointer() {
        assert_eq!(statuses(&[1, 2]), vec![Status::Value(1), Status::Value(2)]);
    }

    #[test]
    fn returned_closure_keeps_its_captured_value() {
        let add_five = make_adder(5);
        assert_eq!(add_five(7), 12);
    }

    #[test]
    fn boxed_closure_can_select_different_concrete_closure_types() {
        assert_eq!(select_transform(true)(3), 13);
        assert_eq!(select_transform(false)(3), 30);
    }

    #[test]
    fn declarative_pair_macro_expands_to_a_pair() {
        assert_eq!(make_pair!("left", 9), ("left", 9));
    }

    #[test]
    fn declarative_record_macro_generates_a_usable_type() {
        let reading = GeneratedReading {
            source: "telemetry".into(),
            pages: 3,
        };
        assert_eq!(reading.pages, 3);
        assert_eq!(reading.source, "telemetry");
    }

    #[test]
    fn derive_macro_generates_the_describe_implementation() {
        let packet = Packet {
            sequence: 1,
            payload: [0; 4],
        };
        assert_eq!(packet.describe(), "Packet");
    }

    #[test]
    fn attribute_macro_adds_a_compile_time_tag() {
        assert_eq!(Packet::AUDIT_TAG, "telemetry");
    }

    #[test]
    fn function_like_procedural_macro_counts_top_level_tokens() {
        assert_eq!(token_count!(alpha beta gamma), 3);
    }
}
