//! Ownership, method receivers, and UTF-8 slices.
//!
//! A move transfers ownership of a value. Moving a `String` field leaves the
//! remaining fields accessible, but a method that borrows the whole struct
//! needs every field.
//!
//! ```compile_fail,E0382
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let note = ReadingNote::new(String::from("Rust"), 20);
//! let title = note.title;
//! assert_eq!(note.title(), title);
//! ```
//!
//! ```
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let note = ReadingNote::new(String::from("Rust"), 20);
//! let title = note.title;
//! assert_eq!(title, "Rust");
//! assert_eq!(note.minutes, 20);
//! ```
//!
//! Struct update copies unlisted `Copy` fields and moves unlisted owned
//! fields such as `String`.
//!
//! ```compile_fail,E0382
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let base = ReadingNote::new(String::from("Rust"), 20);
//! let next = ReadingNote { minutes: 30, ..base };
//! assert_eq!(base.title, next.title);
//! ```
//!
//! Provide a cloned title explicitly to keep both notes intact. A scalar
//! assignment copies a `u32`, while cloning the title creates a separate
//! owned string. Method syntax automatically borrows the receiver as needed.
//!
//! ```
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let base = ReadingNote::new(String::from("Rust"), 20);
//! let mut next = ReadingNote { title: base.title.clone(), ..base };
//! let mut copied_minutes = next.minutes;
//! assert_eq!(copied_minutes, 20);
//! copied_minutes += 10;
//! assert_eq!(next.minutes, 20);
//! assert_eq!(copied_minutes, 30);
//! next.append_to_title(" Book");
//! let nested_borrow = &&base;
//! assert_eq!(nested_borrow.title(), "Rust");
//! assert_eq!(next.into_title(), "Rust Book");
//! ```
//!
//! Receiver lookup automatically dereferences `&&ReadingNote` for `title()`.
//! A shared reference still cannot provide a mutable receiver.
//!
//! Observe the heap buffer of a `String` before and after a move. `len()`
//! counts the current bytes, while `capacity()` counts the bytes that fit
//! without a reallocation. The requested capacity is a lower bound. The
//! example compares raw pointers without dereferencing them and makes no
//! assumptions about Rust's memory layout or ABI.
//!
//! ```
//! let mut original = String::with_capacity(16);
//! original.push_str("中🦀");
//! let buffer = original.as_ptr();
//! let length = original.len();
//! assert_eq!(length, 7);
//! assert!(original.capacity() >= 16);
//! let moved = original;
//! assert_eq!(moved.as_ptr(), buffer);
//! assert_eq!(moved.len(), length);
//! assert!(moved.capacity() >= 16);
//! ```
//!
//! A function can take ownership and return it with a result. A borrowed
//! parameter lets the caller keep ownership throughout the call.
//!
//! ```
//! fn consume(text: String) -> (String, usize) {
//!     let length = text.len();
//!     (text, length)
//! }
//! fn bytes(text: &str) -> usize {
//!     text.len()
//! }
//! let title = String::from("Rust");
//! let (title, length) = consume(title);
//! assert_eq!(length, 4);
//! assert_eq!(bytes(&title), length);
//! assert_eq!(title, "Rust");
//! ```
//!
//! A shared reference cannot call a method that requires a mutable reference.
//!
//! ```compile_fail,E0596
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let mut note = ReadingNote::new(String::from("Rust"), 20);
//! let shared = &note;
//! shared.append_to_title(" Book");
//! ```
//!
//! ```
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let mut note = ReadingNote::new(String::from("Rust"), 20);
//! let exclusive = &mut note;
//! exclusive.append_to_title(" Book");
//! assert_eq!(exclusive.title(), "Rust Book");
//! ```
//!
//! Two mutable references cannot both be active for the same value.
//!
//! ```compile_fail,E0499
//! let mut text = String::from("Rust");
//! let first = &mut text;
//! let second = &mut text;
//! second.push_str(" Book");
//! first.push('!');
//! ```
//!
//! Non-lexical lifetimes allow a new borrow once the previous reference's
//! last use has passed, even in the same block.
//!
//! ```
//! let mut text = String::from("Rust");
//! let first = &mut text;
//! first.push('!');
//! let second = &mut text;
//! second.push_str(" Book");
//! assert_eq!(text, "Rust! Book");
//! ```
//!
//! A shared slice also keeps the underlying string borrowed until its last use.
//!
//! ```compile_fail,E0502
//! use book_d6_foundation_lab::ownership::first_word;
//! let mut text = String::from("Rust Book");
//! let word = first_word(&text);
//! text.clear();
//! assert_eq!(word, "Rust");
//! ```
//!
//! ```
//! use book_d6_foundation_lab::ownership::first_word;
//! let mut text = String::from("Rust Book");
//! let word = first_word(&text);
//! assert_eq!(word, "Rust");
//! text.clear();
//! assert!(text.is_empty());
//! ```
//!
//! A reference cannot outlive a local owner. An explicit lifetime on the
//! return type cannot keep that local value alive.
//!
//! ```compile_fail,E0515
//! fn make_title() -> &'static str {
//!     let title = String::from("Rust");
//!     &title
//! }
//! let _title = make_title();
//! ```
//!
//! ```
//! fn make_title() -> String {
//!     let title = String::from("Rust");
//!     title
//! }
//! assert_eq!(make_title(), "Rust");
//! ```
//!
//! A saved byte offset has no borrow relationship with its source. It can
//! become stale after an edit. A checked slice still verifies the current
//! byte boundaries, and `first_word` borrows the word from the current text.
//!
//! ```
//! use book_d6_foundation_lab::ownership::{first_word, preview};
//! let mut text = String::from("Rust Book");
//! let end = first_word(&text).len();
//! assert_eq!(&text[5..9], "Book");
//! assert_eq!(&text[..end], "Rust");
//! assert_eq!(&text[5..], "Book");
//! assert_eq!(&text[..], "Rust Book");
//! text.replace_range(.., "中🦀 Book");
//! assert_eq!(preview(&text, 0, end), None);
//! assert_eq!(first_word(&text), "中🦀");
//! ```
//!
//! `None` belongs to an `Option<T>`. Supply `T` when the surrounding code
//! does not determine it.
//!
//! ```compile_fail,E0282
//! let missing = None;
//! assert!(missing.is_none());
//! ```
//!
//! ```
//! use book_d6_foundation_lab::ownership::preview;
//! let missing: Option<&str> = None;
//! assert_eq!(preview("中", 0, 1), missing);
//! ```
//!
//! `Option<usize>` can carry a `usize` or be empty. Use `match` to decide
//! which `usize` result to use in each case.
//!
//! ```compile_fail,E0308
//! let maybe_length: Option<usize> = Some(4);
//! let length: usize = maybe_length;
//! assert_eq!(length, 4);
//! ```
//!
//! ```
//! for (maybe_length, expected) in [(Some(4_usize), 4), (None, 0)] {
//!     let length: usize = match maybe_length {
//!         Some(length) => length,
//!         None => 0,
//!     };
//!     assert_eq!(length, expected);
//! }
//! ```
//!
//! A match on `Option` must cover both variants.
//!
//! ```compile_fail,E0004
//! use book_d6_foundation_lab::ownership::preview;
//! let label = match preview("中", 0, 3) {
//!     Some(word) => word,
//! };
//! assert_eq!(label, "中");
//! ```
//!
//! ```
//! use book_d6_foundation_lab::ownership::preview;
//! let label = match preview("中", 0, 3) {
//!     Some(word) => word,
//!     None => "invalid range",
//! };
//! assert_eq!(label, "中");
//! ```

/// A reading note with an owned title and copyable progress fields.
#[derive(Clone)]
pub struct ReadingNote {
    pub title: String,
    pub minutes: u32,
    pub completed: bool,
}

impl ReadingNote {
    /// Creates an incomplete reading note.
    pub fn new(title: String, minutes: u32) -> Self {
        Self {
            title,
            minutes,
            completed: false,
        }
    }
}

impl ReadingNote {
    /// Borrows the title for reading.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Appends to the title through an exclusive borrow.
    pub fn append_to_title(&mut self, suffix: &str) {
        self.title.push_str(suffix);
    }

    /// Consumes the note and transfers ownership of its title.
    pub fn into_title(self) -> String {
        self.title
    }
}

/// Returns the first Unicode-whitespace-delimited word, or an empty slice.
pub fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or("")
}

/// Returns the slice from byte `start` inclusive to byte `end` exclusive.
///
/// Returns `None` for a reversed or out-of-bounds range or if either endpoint
/// falls inside a UTF-8 code point. Equal endpoints at a boundary yield an
/// empty slice.
pub fn preview(text: &str, start: usize, end: usize) -> Option<&str> {
    text.get(start..end)
}
