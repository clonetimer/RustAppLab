//! Independent observable contracts for the editable appendix examples.
//! The workbench embeds this file unchanged, so deleting in-source tests cannot
//! turn an empty learner library into a successful experiment check.
use book_d6_appendix_lab::{
    apply_mask, parse_priority, precedence_example, range_total, task_priorities, Point, Priority,
    Task,
};
use std::collections::HashMap;

#[test]
fn raw_keyword_function_uses_both_inputs_and_preserves_case() {
    assert!(book_d6_appendix_lab::r#match("rust", "learn rust"));
    assert!(!book_d6_appendix_lab::r#match("Rust", "learn rust"));
    assert!(book_d6_appendix_lab::r#match("所有权", "练习所有权与借用"));
    assert!(!book_d6_appendix_lab::r#match("borrow", "learn rust"));
}

#[test]
fn arithmetic_range_and_mask_keep_their_distinct_operator_contracts() {
    assert_eq!(precedence_example(), 14);
    assert_eq!(range_total(1, 5), 10);
    assert_eq!(range_total(3, 3), 0);
    assert_eq!(range_total(4, 7), 15);
    assert_eq!(apply_mask(0b1010, 0b1100), 0b1000);
    assert_eq!(apply_mask(0xff, 0), 0);
    assert_eq!(apply_mask(0b0101, 0xff), 0b0101);
}

#[test]
fn priority_parser_retains_values_and_propagates_parse_errors() {
    assert_eq!(parse_priority("8").unwrap(), Priority::new(8));
    assert_eq!(parse_priority("255").unwrap().value(), 255);
    for invalid in ["high", "256", "-1", ""] {
        assert!(parse_priority(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn priorities_are_copyable_totally_ordered_hash_keys_with_a_zero_default() {
    let original = Priority::new(3);
    let copied = original;
    assert_eq!(original.value(), copied.value());
    let mut values = [Priority::new(8), copied, Priority::default()];
    values.sort();
    assert_eq!(
        values,
        [Priority::new(0), Priority::new(3), Priority::new(8)]
    );
    let priorities = HashMap::from([(original, "normal")]);
    assert_eq!(priorities.get(&copied), Some(&"normal"));
}

#[test]
fn task_clone_owns_its_title_and_display_is_a_user_facing_choice() {
    let original = Task::new(7, "记录观察");
    let mut cloned = original.clone();
    assert_eq!(cloned, original);
    cloned.title.push('!');
    assert_eq!(original.title, "记录观察");
    assert_ne!(cloned, original);
    assert_eq!(original.to_string(), "7: 记录观察");
    assert!(format!("{original:?}").contains("Task"));
    assert_eq!(Task::default(), Task::new(0, ""));
}

#[test]
fn task_hash_keys_use_both_identity_and_title() {
    let values = task_priorities();
    assert_eq!(
        values.get(&Task::new(7, "document")),
        Some(&Priority::new(2))
    );
    assert_eq!(values.get(&Task::new(8, "document")), None);
    assert_eq!(values.get(&Task::new(7, "other")), None);
}

#[test]
fn overloaded_add_combines_each_coordinate() {
    assert_eq!(
        Point { x: 2, y: -3 } + Point { x: 5, y: 7 },
        Point { x: 7, y: 4 }
    );
    assert_eq!(
        Point::default() + Point { x: -2, y: 9 },
        Point { x: -2, y: 9 }
    );
}
