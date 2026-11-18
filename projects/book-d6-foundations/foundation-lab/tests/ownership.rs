use book_d6_foundation_lab::ownership::{first_word, preview, ReadingNote};

#[test]
fn cloning_a_note_copies_its_owned_title_independently() {
    let original = ReadingNote::new(String::from("Rust"), 20);
    let mut copy = original.clone();
    copy.append_to_title(" Book");
    copy.completed = true;

    assert_eq!(original.title(), "Rust");
    assert_eq!(original.minutes, 20);
    assert!(!original.completed);
    assert_eq!(copy.into_title(), "Rust Book");
}

#[test]
fn struct_update_moves_the_title_and_copies_scalar_fields() {
    let original = ReadingNote::new(String::from("Ownership"), 15);
    let updated = ReadingNote {
        minutes: 25,
        ..original
    };

    assert_eq!(original.minutes, 15);
    assert!(!original.completed);
    assert_eq!(updated.minutes, 25);
    assert_eq!(updated.into_title(), "Ownership");
}

#[test]
fn moving_one_field_leaves_the_other_fields_accessible() {
    let note = ReadingNote::new(String::from("Borrowing"), 10);
    let title = note.title;
    let minutes = note.minutes;

    assert_eq!(title, "Borrowing");
    assert_eq!(minutes, 10);
    assert_eq!(note.minutes, 10);
    assert!(!note.completed);
}

#[test]
fn first_word_skips_leading_unicode_whitespace() {
    assert_eq!(first_word("   Rust book"), "Rust");
    assert_eq!(first_word("\t\u{3000}所有权\u{2003}切片"), "所有权");
    assert_eq!(first_word("🦀"), "🦀");
    assert_eq!(first_word(""), "");
    assert_eq!(first_word("\t\u{3000}"), "");
}

#[test]
fn preview_uses_checked_utf8_byte_ranges() {
    let text = "a中🦀";
    assert_eq!(preview(text, 0, 8), Some(text));
    assert_eq!(preview(text, 1, 4), Some("中"));
    assert_eq!(preview(text, 4, 8), Some("🦀"));
    assert_eq!(preview(text, 4, 4), Some(""));
    assert_eq!(preview(text, 8, 8), Some(""));
    assert_eq!(preview(text, 3, 3), None);
    assert_eq!(preview(text, 2, 4), None);
    assert_eq!(preview(text, 1, 6), None);
    assert_eq!(preview(text, 4, 1), None);
    assert_eq!(preview(text, 8, 9), None);
    assert_eq!(preview(text, usize::MAX, usize::MAX), None);
    assert_eq!(preview("", 0, 0), Some(""));
    assert_eq!(preview("", 0, 1), None);
}
