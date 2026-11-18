use std::error::Error;
use std::io::{Cursor, ErrorKind};
use std::num::IntErrorKind;

use book_d6_foundation_lab::collections::{
    annotate_labels, read_score, read_score_explicit, scores_by_name, select_label, ScoreError,
};

#[test]
fn label_lookup_reports_missing_indices() {
    let labels = vec![String::from("分数"), String::from("等级")];
    assert_eq!(select_label(&labels, 1), Some("等级"));
    assert_eq!(select_label(&labels, labels.len()), None);
    assert_eq!(select_label(&[], 0), None);
}

#[test]
fn annotation_appends_a_unicode_suffix_in_place() {
    let mut labels = vec![String::from("初"), String::from("終")];
    annotate_labels(&mut labels[..1], "✓");
    assert_eq!(labels, ["初✓", "終"]);
}

#[test]
fn repeated_names_keep_the_last_score() {
    let name = String::from("小明");
    let scores = scores_by_name(&[(name.as_str(), 80), ("Lin", 91), (name.as_str(), 73)]);
    drop(name);
    assert_eq!(scores.len(), 2);
    assert_eq!(scores.get("小明"), Some(&73));
    assert_eq!(scores.get("Lin"), Some(&91));
}

#[test]
fn score_readers_trim_and_consume_only_the_first_line() {
    let input = "\u{3000}42 \r\n7";
    let mut short_reader = Cursor::new(input);
    let mut explicit_reader = Cursor::new(input);
    assert_eq!(read_score(&mut short_reader).unwrap(), 42);
    assert_eq!(read_score_explicit(&mut explicit_reader).unwrap(), 42);
    assert_eq!(read_score(short_reader).unwrap(), 7);
    assert_eq!(read_score_explicit(explicit_reader).unwrap(), 7);
}

#[test]
fn malformed_first_lines_return_parse_errors() {
    let input = b"invalid\n42";
    for result in [read_score(&input[..]), read_score_explicit(&input[..])] {
        let error = result.unwrap_err();
        assert!(error.source().is_some());
        assert!(error.to_string().contains("parse"));
        assert!(
            matches!(error, ScoreError::Parse(source) if *source.kind() == IntErrorKind::InvalidDigit)
        );
    }
}

#[test]
fn end_of_input_returns_an_empty_parse_error() {
    for result in [read_score(&b""[..]), read_score_explicit(&b""[..])] {
        assert!(
            matches!(result, Err(ScoreError::Parse(source)) if *source.kind() == IntErrorKind::Empty)
        );
    }
}

#[test]
fn invalid_utf8_preserves_the_read_error() {
    let input = b"42\xff\n7";
    for result in [read_score(&input[..]), read_score_explicit(&input[..])] {
        let error = result.unwrap_err();
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap();
        assert_eq!(source.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("read"));
        assert!(matches!(error, ScoreError::Io(source) if source.kind() == ErrorKind::InvalidData));
    }
}
