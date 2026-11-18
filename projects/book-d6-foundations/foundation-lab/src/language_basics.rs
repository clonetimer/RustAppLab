//! Variables and modules through a small study-plan example.
//!
//! `mut` permits assignment to the same binding; another `let` creates a new
//! binding and can change its type. A constant has an explicit type.
//!
//! ```
//! const MINUTES_PER_ROUND: u32 = 5;
//! let mut minutes = 0;
//! minutes += MINUTES_PER_ROUND;
//! let input = " 25 ";
//! let input = input.trim();
//! let input: u32 = input.parse().unwrap();
//! assert_eq!((minutes, input), (5, 25));
//! ```
//!
//! An immutable binding cannot be assigned a second value:
//!
//! ```compile_fail,E0384
//! let minutes = 25;
//! minutes = 30;
//! println!("{minutes}");
//! ```
//!
//! `mut` does not permit changing the binding's type:
//!
//! ```compile_fail,E0308
//! let mut input = "25";
//! input = 25_u32;
//! ```
//!
//! A constant is not a mutable storage location:
//!
//! ```compile_fail,E0070
//! use book_d6_foundation_lab::language_basics::MAX_MINUTES;
//! MAX_MINUTES = 30;
//! ```
//!
//! Public reexports let callers use the types without knowing the private file
//! layout. `as` gives an imported item a local name; both public enum variants
//! remain available to the caller.
//!
//! ```
//! use book_d6_foundation_lab::language_basics::{total_minutes, Activity};
//! use book_d6_foundation_lab::language_basics::StudyCard as Card;
//! let cards = [
//!     Card::new("Ownership", "25", Activity::Read).unwrap(),
//!     Card::new("Modules", "40", Activity::Practice).unwrap(),
//! ];
//! assert_eq!(total_minutes(&cards), 65);
//! assert_eq!(cards[1].summary(), "practice | Modules | 40 min");
//! ```
//!
//! The public API does not expose the private `catalog` module:
//!
//! ```compile_fail,E0603
//! use book_d6_foundation_lab::language_basics::catalog::StudyCard;
//! ```
//!
//! Private fields prevent callers from bypassing the constructor's checks:
//!
//! ```compile_fail,E0616
//! use book_d6_foundation_lab::language_basics::{Activity, StudyCard};
//! let card = StudyCard::new("Rust", "25", Activity::Read).unwrap();
//! let _minutes = card.minutes;
//! ```

// `mod` loads language_basics/catalog.rs as part of this same crate.
mod catalog;

// `crate::` starts at the crate root; `as` only changes the local import name.
use crate::language_basics::catalog::StudyCard as Card;
// `self::` starts at this module; `pub use` exposes selected public items.
pub use self::catalog::{Activity, StudyCard};

/// The maximum duration accepted for one study card, in minutes.
pub const MAX_MINUTES: u32 = 120;

/// Parse a duration from 1 through [`MAX_MINUTES`], allowing outer whitespace.
pub fn parse_minutes(input: &str) -> Result<u32, &'static str> {
    let input = input.trim();
    // Shadowing creates a new u32 binding; the earlier binding was an &str.
    let input: u32 = input.parse().map_err(|_| "minutes must be an integer")?;
    if !(1..=MAX_MINUTES).contains(&input) {
        return Err("minutes must be between 1 and 120");
    }
    Ok(input)
}

/// Sum the durations of all cards without changing the plan.
pub fn total_minutes(cards: &[Card]) -> u64 {
    let mut total = 0_u64;
    for card in cards {
        total += u64::from(card.minutes());
    }
    total
}
