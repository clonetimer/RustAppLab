use book_d6_foundation_lab::summarize_scores;

#[test]
fn summary_returns_total_then_highest_score() {
    assert_eq!(summarize_scores([72, 95, 81]), (248, 95));
}

#[test]
fn highest_score_can_appear_in_any_attempt() {
    for scores in [[95, 72, 81], [72, 95, 81], [72, 81, 95]] {
        assert_eq!(summarize_scores(scores), (248, 95));
    }
}

#[test]
fn repeated_arrays_include_all_three_attempts() {
    assert_eq!(summarize_scores([80; 3]), (240, 80));
    assert_eq!(summarize_scores([0; 3]), (0, 0));
}

#[test]
fn total_preserves_large_scores_without_overflow() {
    assert_eq!(summarize_scores([u32::MAX; 3]), (12_884_901_885, u32::MAX));
}
