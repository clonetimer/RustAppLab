use book_d6_foundation_lab::language_basics::{parse_minutes, total_minutes, Activity, StudyCard};

#[test]
fn minutes_parser_accepts_trimmed_numbers_and_rejects_out_of_range_input() {
    assert_eq!(parse_minutes(" 25 "), Ok(25));
    assert_eq!(parse_minutes("120"), Ok(120));
    for input in ["", "abc", "0", "121", "-1", "4294967296"] {
        assert!(parse_minutes(input).is_err(), "accepted {input:?}");
    }
}

#[test]
fn reading_card_keeps_unicode_title_and_normalizes_outer_space() {
    let card = StudyCard::new("  Rust 所有权  ", " 25 ", Activity::Read).unwrap();
    assert_eq!(card.summary(), "read | Rust 所有权 | 25 min");
}

#[test]
fn practice_variant_is_available_through_the_public_reexport() {
    use book_d6_foundation_lab::language_basics::StudyCard as Card;

    let card = Card::new("模块实验", "40", Activity::Practice).unwrap();
    assert_eq!(card.summary(), "practice | 模块实验 | 40 min");
}

#[test]
fn card_constructor_rejects_blank_title_and_invalid_minutes() {
    assert!(StudyCard::new("  \t ", "20", Activity::Read).is_err());
    assert!(StudyCard::new("Rust", "0", Activity::Read).is_err());
    assert!(StudyCard::new("Rust", "121", Activity::Practice).is_err());
}

#[test]
fn plan_total_includes_every_card_and_handles_an_empty_plan() {
    let cards = [
        StudyCard::new("读书", "25", Activity::Read).unwrap(),
        StudyCard::new("练习", "40", Activity::Practice).unwrap(),
        StudyCard::new("回顾", "5", Activity::Read).unwrap(),
    ];
    assert_eq!(total_minutes(&cards), 70);
    assert_eq!(total_minutes(&[]), 0);
}
