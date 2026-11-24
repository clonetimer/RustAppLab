//! Executable repair chains for the 31 frozen IDs in the six-chapter audit.
//!
//! Each section has a successful observation and a rejected program or a
//! deliberately wrong assumption. The accompanying Chinese lesson supplies
//! prediction, repair, and transfer questions. Passing these reference examples
//! is evidence about the material, not a learner's understanding.
//!
//! ## 4.1-03
//!
//! A literal is borrowed immutable UTF-8 data in the program; `String::from`
//! creates an owned growable buffer. Making a binding mutable does not change
//! the type or turn a shared `&str` into owned storage.
//!
//! ```
//! let literal: &str = "Rust";
//! let mut owned = String::from(literal);
//! owned.push_str(" Book");
//! assert_eq!(literal, "Rust");
//! assert_eq!(owned, "Rust Book");
//! ```
//!
//! ```compile_fail,E0599
//! let mut literal: &str = "Rust";
//! literal.push_str(" Book");
//! ```
//!
//! ## 4.1-06
//!
//! A tuple is `Copy` only when all its fields are `Copy`. A `String` tuple
//! moves on assignment and on a by-value call. Explicit cloning creates an
//! independently owned string; copying a tuple of scalars needs no clone.
//!
//! ```
//! fn take_pair(pair: (u32, bool)) -> (u32, bool) { pair }
//! fn take_title(pair: (String, bool)) -> (String, bool) { pair }
//! let scalar = (7_u32, true);
//! let copied = scalar;
//! assert_eq!(take_pair(scalar), copied);
//! assert_eq!(scalar, (7, true));
//! let owned = (String::from("Rust"), true);
//! let mut separate = take_title(owned.clone());
//! separate.0.push('!');
//! assert_eq!(owned.0, "Rust");
//! assert_eq!(separate.0, "Rust!");
//! ```
//!
//! ```compile_fail,E0382
//! let original = (String::from("Rust"), true);
//! let moved = original;
//! println!("{} {}", original.0, moved.0);
//! ```
//!
//! ## 4.1-07
//!
//! Ordinary function parameters follow the same move/copy rule as assignment.
//! Returning an owned value transfers it to the caller; it does not revive a
//! moved binding unless the caller binds the returned value again.
//!
//! ```
//! fn number(value: u32) -> u32 { value + 1 }
//! fn title(mut value: String) -> String { value.push('!'); value }
//! let score = 7;
//! assert_eq!(number(score), 8);
//! assert_eq!(score, 7);
//! let note = String::from("Rust");
//! let note = title(note);
//! assert_eq!(note, "Rust!");
//! ```
//!
//! ```compile_fail,E0382
//! fn consume(value: String) { println!("{value}"); }
//! let title = String::from("Rust");
//! consume(title);
//! println!("{title}");
//! ```
//!
//! ## 4.2-07
//!
//! Two shared reads can coexist because neither changes the data. An exclusive
//! borrow conflicts while either shared reference is still used. Last use,
//! rather than just the enclosing braces, determines this overlap.
//!
//! ```
//! let mut text = String::from("Rust");
//! let first = &text;
//! let second = &text;
//! assert_eq!(first.len() + second.len(), 8);
//! let exclusive = &mut text; // both shared references have had their last use
//! exclusive.push('!');
//! assert_eq!(text, "Rust!");
//! ```
//!
//! ```compile_fail,E0502
//! let mut text = String::from("Rust");
//! let first = &text;
//! let second = &text;
//! let exclusive = &mut text;
//! exclusive.push('!');
//! println!("{first} {second}");
//! ```
//!
//! ## 4.3-05
//!
//! A string literal has type `&'static str`: its underlying bytes remain valid
//! for the program lifetime. `mut` can let the binding point at another literal;
//! it does not permit mutation through the shared reference.
//!
//! ```
//! let mut literal: &'static str = "Rust";
//! assert_eq!(literal, "Rust");
//! literal = "Book";
//! let mut owned = literal.to_string();
//! owned.make_ascii_uppercase();
//! assert_eq!(literal, "Book");
//! assert_eq!(owned, "BOOK");
//! ```
//!
//! ```compile_fail,E0596
//! let mut literal: &str = "Rust";
//! literal.make_ascii_uppercase();
//! ```
//!
//! ## 5.1-02
//!
//! Directly assigning an ordinary field requires a mutable instance binding.
//! There is no `mut` field declaration for permitting just one field to change.
//! Interior mutability is a separate mechanism, outside this example.
//!
//! ```
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let mut note = ReadingNote::new("Rust".into(), 20);
//! note.minutes = 30;
//! assert_eq!(note.minutes, 30);
//! assert_eq!(note.title, "Rust");
//! ```
//!
//! ```compile_fail,E0594
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let note = ReadingNote::new("Rust".into(), 20);
//! note.minutes = 30;
//! ```
//!
//! ## 5.1-03
//!
//! `title` in a struct initializer is shorthand for `title: title` when the
//! local variable has that exact name. A differently named local requires an
//! explicit field-to-expression mapping; neither spelling changes ownership.
//!
//! ```
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let title = String::from("Rust");
//! let minutes = 20;
//! let short = ReadingNote { title, minutes, completed: false };
//! let heading = String::from("Rust");
//! let long = ReadingNote { title: heading, minutes: minutes, completed: false };
//! assert_eq!(short.title, long.title);
//! assert_eq!(short.minutes, long.minutes);
//! ```
//!
//! ```compile_fail,E0425
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let heading = String::from("Rust");
//! let note = ReadingNote { title, minutes: 20, completed: false };
//! ```
//!
//! ## 5.1-06
//!
//! A unit-like struct has no fields but is its own named type and can carry
//! methods/trait implementations. It is not the unit type `()`.
//!
//! ```
//! use book_d6_foundation_lab::concept_repairs::Marker;
//! let marker = Marker;
//! assert_eq!(marker.label(), "ready");
//! assert_eq!(std::mem::size_of_val(&marker), 0);
//! assert_eq!(std::mem::size_of_val(&()), 0);
//! // Equal size does not mean identical type.
//! ```
//!
//! ```compile_fail,E0308
//! use book_d6_foundation_lab::concept_repairs::Marker;
//! let marker: Marker = ();
//! ```
//!
//! ## 5.2-01
//!
//! Independent arguments and tuple positions require remembering what each
//! number means. Named fields keep related data together and expose that
//! meaning at a call site. Names improve communication; they do not prove
//! that same-typed values were assigned to the correct fields.
//!
//! ```
//! fn loose(total: u32, done: u32) -> String { format!("{done}/{total}") }
//! fn tuple(progress: (u32, u32)) -> String { loose(progress.0, progress.1) }
//! struct Progress { total: u32, done: u32 }
//! fn named(progress: &Progress) -> String { loose(progress.total, progress.done) }
//! let progress = Progress { total: 10, done: 2 };
//! assert_eq!(loose(10, 2), "2/10");
//! assert_eq!(tuple((10, 2)), named(&progress));
//! assert_ne!(tuple((2, 10)), named(&progress)); // compiles, but reverses the meaning
//! assert_eq!(tuple((progress.total, progress.done)), "2/10"); // repair
//! ```
//!
//! ## 5.3-04
//!
//! `takes_longer_than(&self, other: &ReadingNote)` borrows both notes, so the
//! caller can use both after comparison. A by-value `other` would consume a
//! non-Copy note even though `self` is still borrowed.
//!
//! ```
//! use book_d6_foundation_lab::ownership::ReadingNote;
//! let first = ReadingNote::new("Rust".into(), 30);
//! let second = ReadingNote::new("Book".into(), 20);
//! assert!(first.takes_longer_than(&second));
//! assert_eq!((first.minutes, second.minutes), (30, 20));
//! assert_eq!((first.title(), second.title()), ("Rust", "Book"));
//! ```
//!
//! ```compile_fail,E0382
//! struct Note { minutes: u32 }
//! impl Note { fn longer(&self, other: Note) -> bool { self.minutes > other.minutes } }
//! let first = Note { minutes: 30 };
//! let second = Note { minutes: 20 };
//! assert!(first.longer(second));
//! println!("{}", second.minutes);
//! ```
//!
//! ## 6.1-03
//!
//! An enum's `impl` can contain instance methods. `minutes(&self)` inspects
//! one value's variant and payload, whereas `new()` has no receiver and is an
//! associated constructor called through the type.
//!
//! ```
//! use book_d6_foundation_lab::concept_repairs::LessonState;
//! let draft = LessonState::new();
//! let reviewed = LessonState::Reviewed { minutes: 20 };
//! assert_eq!(draft.minutes(), None);
//! assert_eq!(reviewed.minutes(), Some(20));
//! assert_eq!(LessonState::minutes(&reviewed), Some(20));
//! ```
//!
//! ```compile_fail,E0599
//! use book_d6_foundation_lab::concept_repairs::LessonState;
//! let draft = LessonState::new();
//! let another = draft.new();
//! ```
//!
//! ## 6.3-03
//!
//! `if let Some(3)` selects exactly that payload, not every `Some`. It does
//! not require an account of the other cases. Adding an enum variant can leave
//! an `if let` compiling while an exhaustive `match` must be updated.
//!
//! ```
//! fn is_three(value: Option<u32>) -> bool {
//!     if let Some(3) = value { true } else { false }
//! }
//! assert_eq!([Some(3), Some(4), None].map(is_three), [true, false, false]);
//! enum State { Draft, Done, Archived }
//! fn is_done(state: State) -> bool { if let State::Done = state { true } else { false } }
//! assert!(!is_done(State::Draft));
//! assert!(is_done(State::Done));
//! assert!(!is_done(State::Archived)); // no new branch was required
//! fn label(state: State) -> &'static str {
//!     match state { State::Draft => "draft", State::Done => "done", State::Archived => "archived" }
//! }
//! assert_eq!(label(State::Archived), "archived");
//! ```
//!
//! ```compile_fail,E0004
//! enum State { Draft, Done, Archived }
//! fn label(state: State) -> &'static str {
//!     match state { State::Draft => "draft", State::Done => "done" }
//! }
//! ```
//!
//! ## book-8-1-vec-drop-elements
//!
//! A Vec owns its elements. Dropping it drops elements still in it and releases
//! its allocation. An element removed with `pop` belongs to the caller and is
//! dropped separately. Do not assume a promised element destruction order.
//!
//! ```
//! use std::cell::Cell;
//! struct Tracked<'a>(&'a Cell<usize>);
//! impl Drop for Tracked<'_> { fn drop(&mut self) { self.0.set(self.0.get() + 1); } }
//! let dropped = Cell::new(0);
//! let removed;
//! {
//!     let mut items = vec![Tracked(&dropped), Tracked(&dropped), Tracked(&dropped)];
//!     removed = items.pop().unwrap();
//!     assert_eq!(dropped.get(), 0);
//! } // Vec and its two remaining owned elements leave scope
//! assert_eq!(dropped.get(), 2);
//! assert_ne!(dropped.get(), 3); // pop transferred ownership; it did not drop that value
//! drop(removed);
//! assert_eq!(dropped.get(), 3);
//! ```
//!
//! ## book-8-2-string-construction
//!
//! `String::new()` starts empty. `String::from(text)` and `text.to_string()`
//! both create owned strings from this `&str`; choose the spelling that makes
//! the conversion clear. Appending the same text to `new()` reaches the same
//! content, without implying an identical allocation history.
//!
//! ```
//! let text = "中 Rust";
//! let mut grown = String::new();
//! assert!(grown.is_empty());
//! grown.push_str(text);
//! assert_eq!(grown, String::from(text));
//! assert_eq!(grown, text.to_string());
//! assert_eq!(String::new(), String::from(""));
//! ```
//!
//! ```compile_fail,E0061
//! let text = String::new("Rust");
//! ```
//!
//! ## book-8-2-string-integer-index
//!
//! UTF-8 uses variable numbers of bytes per Unicode scalar. An integer string
//! index would be ambiguous, and finding the nth scalar from the start requires
//! scanning/decoding rather than the O(1) expectation of direct indexing.
//! `chars()` yields scalars, which are not necessarily displayed graphemes.
//!
//! ```
//! let text = "A中🦀";
//! assert_eq!(text.len(), 8);
//! assert_eq!(text.chars().count(), 3);
//! let mut visited = 0;
//! let third = text.chars().inspect(|_| visited += 1).nth(2);
//! assert_eq!(third, Some('🦀'));
//! assert_eq!(visited, 3);
//! assert_eq!(text.as_bytes()[0], b'A');
//! assert_eq!(text.get(1..4), Some("中"));
//! assert_eq!(text.get(1..2), None);
//! ```
//!
//! ```compile_fail,E0277
//! let text = String::from("A中🦀");
//! let character = text[1];
//! ```
//!
//! ## book-8-3-hashmap-construction
//!
//! In one `HashMap<K, V>`, every key has the same K and every value the same V;
//! K and V need not be the same type as each other. Type annotation or the first
//! insert fixes those types; later inserts cannot silently change them.
//!
//! ```
//! use std::collections::HashMap;
//! let mut scores: HashMap<String, u32> = HashMap::new();
//! assert_eq!(scores.insert("Ada".into(), 7), None);
//! assert_eq!(scores.insert("Ada".into(), 8), Some(7));
//! assert_eq!(scores.get("Ada"), Some(&8));
//! ```
//!
//! ```compile_fail,E0308
//! use std::collections::HashMap;
//! let mut scores: HashMap<String, u32> = HashMap::new();
//! scores.insert(7, 8);
//! ```
//!
//! ```compile_fail,E0308
//! use std::collections::HashMap;
//! let mut scores: HashMap<String, u32> = HashMap::new();
//! scores.insert("Ada".into(), "eight");
//! ```
//!
//! ## book-8-3-hashmap-collect
//!
//! `zip` yields pairs; `collect` builds the requested target collection.
//! An explicit `HashMap<_, _>` supplies otherwise missing destination context.
//! zip ends with the shorter iterator; duplicate keys replace earlier values.
//!
//! ```
//! use std::collections::HashMap;
//! let map: HashMap<_, _> = ["Ada", "Lin"].into_iter().zip([3, 5]).collect();
//! assert_eq!(map.get("Ada"), Some(&3));
//! assert_eq!(map.get("Lin"), Some(&5));
//! let shorter = ["Ada", "Lin"].into_iter().zip([3]).collect::<HashMap<_, _>>();
//! assert_eq!(shorter.len(), 1);
//! assert_eq!(shorter.get("Lin"), None);
//! ```
//!
//! ```compile_fail,E0282
//! let map = ["Ada", "Lin"].into_iter().zip([3, 5]).collect();
//! let found = map.get("Lin");
//! ```
//!
//! ## book-8-3-hashmap-ownership
//!
//! `insert` receives owned K and V. Copy keys/values remain usable in the caller;
//! String keys/values move unless cloned. A map of references borrows its
//! referents, so those owners must remain valid wherever the map is used.
//!
//! ```
//! use std::collections::HashMap;
//! let key = 7_u32;
//! let score = 8_u32;
//! let mut copied = HashMap::new();
//! copied.insert(key, score);
//! assert_eq!((key, score), (7, 8));
//! let name = String::from("Ada");
//! let mut owned = HashMap::new();
//! owned.insert(name.clone(), score);
//! assert_eq!(name, "Ada");
//! let mut borrowed = HashMap::new();
//! borrowed.insert(name.as_str(), &score);
//! assert_eq!(borrowed.get("Ada"), Some(&&8));
//! ```
//!
//! ```compile_fail,E0382
//! use std::collections::HashMap;
//! let name = String::from("Ada");
//! let mut map = HashMap::new();
//! map.insert(name, 8);
//! println!("{name}");
//! ```
//!
//! ```compile_fail,E0597
//! use std::collections::HashMap;
//! let mut map = HashMap::new();
//! {
//!     let name = String::from("Ada");
//!     map.insert(name.as_str(), 8);
//! }
//! println!("{map:?}");
//! ```
//!
//! ## book-8-3-hashmap-hasher
//!
//! The default RandomState uses randomized seeding to help resist deliberate
//! collision attacks. Specialized hashing can trade that protection for speed
//! on particular workloads; this example measures no speed advantage. The
//! extension point is `BuildHasher`, which builds per-key `Hasher` instances.
//! `BuildHasherDefault<DefaultHasher>` demonstrates the interface without the
//! default randomized seeding. Neither hash values nor iteration order are a
//! stable cross-version contract. Keep the default for this lesson's untrusted
//! inputs; do not infer production suitability from equal lookup results.
//!
//! ```
//! use std::collections::{hash_map::DefaultHasher, HashMap};
//! use std::hash::BuildHasherDefault;
//! let mut standard = HashMap::new();
//! let mut explicit = HashMap::with_hasher(BuildHasherDefault::<DefaultHasher>::default());
//! for (key, value) in [("Ada", 3), ("Lin", 5)] {
//!     standard.insert(key, value);
//!     explicit.insert(key, value);
//! }
//! assert_eq!(standard.get("Lin"), explicit.get("Lin"));
//! assert_eq!(standard.get("absent"), explicit.get("absent"));
//! ```
//!
//! ```compile_fail,E0599
//! use std::collections::HashMap;
//! struct NotABuilder;
//! let mut map = HashMap::with_hasher(NotABuilder);
//! map.insert("Ada", 3);
//! ```
//!
//! ## book-9-1-panic-unwind-abort
//!
//! With unwind, panicking unwinds Rust frames and runs their live destructors
//! before a compatible catch boundary. With abort the process terminates and
//! those destructors do not run; catch_unwind cannot recover it. Cargo's
//! `[profile.release] panic = "abort"` selects a build policy, not a runtime
//! toggle. The integration test compiles the same independent probe with
//! `rustc -C panic=unwind` and `-C panic=abort`, then checks an actual Drop file.
//! It never changes the test harness's own panic strategy.
//!
//! ```
//! let result = std::panic::catch_unwind(|| panic!("expected unwind"));
//! assert!(result.is_err());
//! // This observes unwind only. The separate process test proves abort behavior.
//! ```
//!
//! ## book-9-1-panic-location
//!
//! Both an explicit panic and an out-of-range index report their source
//! location. `explicit_and_index_panics_both_locate_own_source` runs both modes
//! of the probe; read the file:line:column next to the message, then open that
//! expression. A library panic frame is not necessarily the caller's bug.
//!
//! ```
//! fn at(values: &[u32], index: usize) -> Option<&u32> { values.get(index) }
//! assert_eq!(at(&[7], 0), Some(&7));
//! assert_eq!(at(&[7], 1), None); // repair when the index is external input
//! ```
//!
//! ## book-9-1-panic-backtrace
//!
//! Run the same probe in fresh child processes with RUST_BACKTRACE=0 and =1.
//! The latter should show probe_inner, probe_middle, probe_outer and their
//! own-source locations in order from the panic outward. Standard-library
//! frames provide context; identify the first relevant application frame.
//! Line numbers and platform paths are observations, not fixed expected text.
//!
//! ```
//! fn checked_step(value: usize) -> Result<usize, &'static str> {
//!     value.checked_add(1).ok_or("step overflow")
//! }
//! assert_eq!(checked_step(3), Ok(4));
//! assert_eq!(checked_step(usize::MAX), Err("step overflow"));
//! // A recoverable Result removes the need to panic on this invalid input.
//! ```
//!
//! ## book-9-2-result-error-kind
//!
//! This API deliberately treats a missing optional note as empty. It propagates
//! other I/O errors, preserving their kind. A required file would need a
//! different NotFound policy. The deterministic PermissionDenied value tests
//! the branch; it does not claim that this machine denied a filesystem call.
//!
//! ```
//! use book_d6_foundation_lab::concept_repairs::optional_note;
//! use std::io::{self, ErrorKind};
//! assert_eq!(optional_note(Ok("Rust".into())).unwrap(), "Rust");
//! assert_eq!(optional_note(Err(io::Error::from(ErrorKind::NotFound))).unwrap(), "");
//! let denied = optional_note(Err(io::Error::from(ErrorKind::PermissionDenied)));
//! assert_eq!(denied.unwrap_err().kind(), ErrorKind::PermissionDenied);
//! ```
//!
//! ## book-9-2-result-unwrap-expect
//!
//! unwrap/expect extract the success value from Some/Ok, and panic on None/Err.
//! expect adds the caller's explanation of the violated assumption; it does
//! not turn failure into a Result or make recovery happen automatically.
//!
//! ```
//! use book_d6_foundation_lab::concept_repairs::panic_message;
//! assert_eq!(Some(7).unwrap(), 7);
//! assert_eq!(Some(8).expect("known optional score"), 8);
//! assert_eq!(Ok::<_, &str>(7).unwrap(), 7);
//! assert_eq!(Ok::<_, &str>(8).expect("known score"), 8);
//! let none = std::panic::catch_unwind(|| None::<u32>.unwrap()).unwrap_err();
//! assert!(panic_message(none).contains("None"));
//! let none = std::panic::catch_unwind(|| None::<u32>.expect("optional fixture missing")).unwrap_err();
//! assert!(panic_message(none).contains("optional fixture missing"));
//! let err = std::panic::catch_unwind(|| Err::<u32, _>("bad input").unwrap()).unwrap_err();
//! assert!(panic_message(err).contains("bad input"));
//! let err = std::panic::catch_unwind(|| Err::<u32, _>("bad input").expect("valid fixture required")).unwrap_err();
//! let message = panic_message(err);
//! assert!(message.contains("valid fixture required") && message.contains("bad input"));
//! ```
//!
//! ## book-9-3-panic-prototype-assumption
//!
//! A fixed, reviewed literal or a test fixture can justify expect with an
//! explicit reason; a prototype can use it temporarily to expose assumptions.
//! External input has no such guarantee, so this public parser returns Result.
//! Test success is evidence for the chosen inputs, not all future input.
//!
//! ```
//! fn parse_input(text: &str) -> Result<u32, std::num::ParseIntError> { text.parse() }
//! let fixture = "42".parse::<u32>().expect("the reviewed decimal literal 42 fits u32");
//! assert_eq!(fixture, 42);
//! assert_eq!(parse_input("43"), Ok(43));
//! assert!(parse_input("forty-two").is_err());
//! let wrong_assumption = std::panic::catch_unwind(|| "forty-two".parse::<u32>().expect("prototype assumption"));
//! assert!(wrong_assumption.is_err());
//! ```
//!
//! ## book-9-3-validated-types
//!
//! Private fields prevent callers from bypassing a checked constructor in safe
//! code. `ValidatedScore::try_new` accepts only 0..=100; every public operation
//! preserves that invariant. Parsing/validation happens at the API boundary,
//! allowing consumers to accept the validated type instead of rechecking u32.
//!
//! ```
//! use book_d6_foundation_lab::concept_repairs::ValidatedScore;
//! assert_eq!(ValidatedScore::try_new(0).unwrap().value(), 0);
//! assert_eq!(ValidatedScore::try_new(100).unwrap().value(), 100);
//! assert!(ValidatedScore::try_new(-1).is_err());
//! assert!(ValidatedScore::try_new(101).is_err());
//! ```
//!
//! ```compile_fail,E0451
//! use book_d6_foundation_lab::concept_repairs::ValidatedScore;
//! let invalid = ValidatedScore { value: 255 };
//! ```
//!
//! ## book-10-1-generic-monomorphization
//!
//! A generic function is instantiated for concrete types used by the program.
//! These calls resolve statically to u32 and String operations. Monomorphization
//! avoids a runtime type choice for this call; it is not a promise that every
//! generic program is faster or smaller (extra instances can increase code).
//!
//! ```
//! fn identity<T>(value: T) -> T { value }
//! let number: u32 = identity::<u32>(7);
//! let text: String = identity::<String>("Rust".into());
//! assert_eq!(number, 7);
//! assert_eq!(text, "Rust");
//! assert!(identity::<bool>(true));
//! ```
//!
//! ```compile_fail,E0308
//! fn identity<T>(value: T) -> T { value }
//! let number: fn(u32) -> u32 = identity::<String>;
//! ```
//!
//! ## book-10-2-trait-coherence
//!
//! For these non-generic examples an impl needs a local trait or a local type.
//! Display and Vec are both external, so implementing Display directly for
//! Vec<String> is rejected. A local newtype owns the implementation choice;
//! type aliases do not create a new local type. More general generic impls
//! also have ordering/coverage restrictions under the orphan rules.
//!
//! ```
//! use std::fmt;
//! struct Labels(Vec<String>);
//! impl fmt::Display for Labels {
//!     fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(out, "[{}]", self.0.join(", "))
//!     }
//! }
//! assert_eq!(Labels(vec!["Rust".into(), "Book".into()]).to_string(), "[Rust, Book]");
//! ```
//!
//! ```compile_fail,E0117
//! use std::fmt;
//! impl fmt::Display for Vec<String> {
//!     fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result { write!(out, "labels") }
//! }
//! ```
//!
//! ## book-10-2-trait-copy-clone-borrow
//!
//! PartialOrd can report None for incomparable values, for example NaN. The
//! existing Ord helpers deliberately demand a total order. Our partial-order
//! variant rejects an incomparable scan with None. Copy/Clone constrain how
//! the result is owned; returning a reference borrows instead of duplicating
//! it. String implements Ord and Clone but not Copy.
//!
//! ```
//! use book_d6_foundation_lab::concept_repairs::largest_partial;
//! use book_d6_foundation_lab::generic_lifetimes::largest;
//! fn largest_copy<T: Ord + Copy>(values: &[T]) -> Option<T> { largest(values).copied() }
//! fn largest_clone<T: Ord + Clone>(values: &[T]) -> Option<T> { largest(values).cloned() }
//! assert_eq!(1.0_f64.partial_cmp(&2.0), Some(std::cmp::Ordering::Less));
//! assert_eq!(f64::NAN.partial_cmp(&2.0), None);
//! assert_eq!(largest_partial(&[1.0, 3.0, 2.0]), Some(&3.0));
//! assert_eq!(largest_partial(&[1.0, f64::NAN]), None);
//! assert_eq!(largest_partial(&[f64::NAN]), None);
//! assert_eq!(largest_copy(&[1, 3, 2]), Some(3));
//! let labels = [String::from("Ada"), String::from("Lin")];
//! assert_eq!(largest_clone(&labels), Some(String::from("Lin")));
//! let borrowed = largest(&labels).unwrap();
//! assert!(std::ptr::eq(borrowed, &labels[1]));
//! assert_eq!(largest_partial::<f64>(&[]), None);
//! ```
//!
//! ```compile_fail,E0277
//! use book_d6_foundation_lab::generic_lifetimes::largest;
//! let found = largest(&[1.0_f64, f64::NAN]);
//! ```
//!
//! ## book-10-2-trait-blanket-impl
//!
//! `impl<T: Display> Tagged for T` gives every matching T the local trait
//! implementation, including types declared later. It is not a manually
//! enumerated list. A second implementation for an already-covered concrete
//! type overlaps and is rejected.
//!
//! ```
//! use std::fmt;
//! trait Tagged { fn tag(&self) -> String; }
//! impl<T: fmt::Display> Tagged for T { fn tag(&self) -> String { format!("[{self}]") } }
//! struct Ticket(u32);
//! impl fmt::Display for Ticket {
//!     fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result { write!(out, "T{}", self.0) }
//! }
//! assert_eq!(7.tag(), "[7]");
//! assert_eq!(String::from("Rust").tag(), "[Rust]");
//! assert_eq!(Ticket(9).tag(), "[T9]");
//! ```
//!
//! ```compile_fail,E0119
//! trait Tagged {}
//! impl<T: std::fmt::Display> Tagged for T {}
//! impl Tagged for u32 {}
//! ```
//!
//! ## book-10-3-lifetime-method-impl
//!
//! In `impl<'a> Excerpt<'a>`, the first lifetime declares a name and the second
//! applies it to the struct. Existing `impl Excerpt<'_>` leaves that name
//! anonymous when the implementation need not refer to it explicitly. Neither
//! changes the stored reference's lifetime. These two method spellings return
//! data borrowed for the call's self borrow; a temporary prefix is independent.
//!
//! ```
//! use book_d6_foundation_lab::generic_lifetimes::Excerpt;
//! let source = String::from("prefix:Rust");
//! let excerpt = Excerpt { text: &source };
//! let first;
//! let second;
//! {
//!     let prefix = String::from("prefix:");
//!     first = excerpt.without_prefix(&prefix);
//!     second = excerpt.named_without_prefix(&prefix);
//! }
//! assert_eq!(first, "Rust");
//! assert_eq!(second, first);
//! assert_eq!(first.as_ptr(), second.as_ptr());
//! ```
//!
//! ```compile_fail,E0597
//! use book_d6_foundation_lab::generic_lifetimes::Excerpt;
//! let source = String::from("prefix:Rust");
//! let result;
//! {
//!     let excerpt = Excerpt { text: &source };
//!     result = excerpt.named_without_prefix("prefix:");
//! }
//! println!("{result}");
//! ```

use crate::generic_lifetimes::Excerpt;
use crate::ownership::ReadingNote;
use std::any::Any;
use std::cmp::Ordering;
use std::io;

/// A fieldless named type, distinct from `()`.
pub struct Marker;

impl Marker {
    pub fn label(&self) -> &'static str {
        "ready"
    }
}

impl ReadingNote {
    /// Borrows both non-Copy notes; neither caller-owned value is consumed.
    pub fn takes_longer_than(&self, other: &ReadingNote) -> bool {
        self.minutes > other.minutes
    }
}

/// A constructor and a receiver method on an enum.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum LessonState {
    #[default]
    Draft,
    Reviewed {
        minutes: u32,
    },
}

impl LessonState {
    pub fn new() -> Self {
        Self::Draft
    }

    pub fn minutes(&self) -> Option<u32> {
        match self {
            Self::Draft => None,
            Self::Reviewed { minutes } => Some(*minutes),
        }
    }
}

/// Missing optional notes have a default; other I/O errors remain observable.
pub fn optional_note(result: io::Result<String>) -> io::Result<String> {
    match result {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error),
    }
}

/// Inspect standard string panic payloads inside isolated teaching experiments.
pub fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else {
        "non-string panic payload".to_string()
    }
}

/// Safe callers can construct only a score in the inclusive 0..=100 range.
#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedScore {
    value: u8,
}

impl ValidatedScore {
    pub fn try_new(value: i32) -> Result<Self, &'static str> {
        if (0..=100).contains(&value) {
            Ok(Self { value: value as u8 })
        } else {
            Err("score must be in 0..=100")
        }
    }

    pub fn value(&self) -> u8 {
        self.value
    }
}

/// Borrow the first maximum; return None for empty input or an incomparable scan.
pub fn largest_partial<T: PartialOrd>(values: &[T]) -> Option<&T> {
    let mut largest = values.first()?;
    for value in values {
        if value.partial_cmp(largest)? == Ordering::Greater {
            largest = value;
        }
    }
    Some(largest)
}

// The named parameter is the teaching target contrasted with impl Excerpt<'_>.
#[allow(clippy::needless_lifetimes)]
impl<'a> Excerpt<'a> {
    /// Named-impl counterpart to the existing placeholder-impl method.
    pub fn named_without_prefix(&self, prefix: &str) -> &str {
        self.text.strip_prefix(prefix).unwrap_or(self.text)
    }
}
