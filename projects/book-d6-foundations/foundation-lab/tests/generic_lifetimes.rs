use book_d6_foundation_lab::generic_lifetimes::{
    choose_longer, first_label, largest, summaries, Card, Excerpt, Summary,
};

#[test]
fn mapping_consumes_the_value_and_preserves_the_owned_label() {
    let card = Card::new(String::from("字节"), String::from("Rust 中"));
    let mapped = card.map(String::into_bytes);
    assert_eq!(mapped.label, "字节");
    assert_eq!(mapped.value, [82, 117, 115, 116, 32, 228, 184, 173]);
}

#[test]
fn generic_consumer_respects_default_and_overridden_summaries() {
    struct Notice;
    impl Summary for Notice {
        fn label(&self) -> &str {
            "提醒"
        }
        fn summarize(&self) -> String {
            String::from("请先预测结果")
        }
    }
    let card = Card::new(String::from("借用"), ());
    assert_eq!(
        summaries(&card, &Notice),
        (String::from("学习卡：借用"), String::from("请先预测结果"))
    );
}

#[test]
fn conditional_display_and_integer_specific_behavior_are_observable() {
    let positive = Card::new(String::from("分数"), 7_i32);
    assert_eq!(positive.describe(), "分数: 7");
    assert!(positive.is_positive());
    assert!(!Card::new(String::new(), 0_i32).is_positive());
    assert!(!Card::new(String::new(), -1_i32).is_positive());
    assert_eq!(
        Card::new(String::from("文本"), "中文").describe(),
        "文本: 中文"
    );
}

#[test]
fn largest_borrows_non_clone_values_and_returns_none_for_empty_input() {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Token(u32, String);
    let values = [
        Token(2, "a".into()),
        Token(9, "b".into()),
        Token(4, "c".into()),
    ];
    assert!(std::ptr::eq(largest(&values).unwrap(), &values[1]));
    let empty: [Token; 0] = [];
    assert!(largest(&empty).is_none());
    let tied = [String::from("same"), String::from("same")];
    assert!(std::ptr::eq(largest(&tied).unwrap(), &tied[0]));
}

#[test]
fn longer_uses_utf8_bytes_and_preserves_the_left_input_on_a_tie() {
    assert_eq!(choose_longer("", "Rust"), "Rust");
    assert_eq!(choose_longer("Rust", ""), "Rust");
    assert_eq!(choose_longer("", ""), "");
    assert_eq!(choose_longer("中", "ab"), "中");
    let left = String::from("ab");
    let right = String::from("cd");
    assert_eq!(choose_longer(&left, &right).as_ptr(), left.as_ptr());
}

#[test]
fn first_label_does_not_borrow_the_second_argument_for_its_result() {
    let first = String::from("持续可用");
    let result;
    {
        let second = String::from("短作用域");
        result = first_label(&first, &second);
    }
    assert_eq!(result, "持续可用");
    assert_eq!(result.as_ptr(), first.as_ptr());
}

#[test]
fn excerpt_output_borrows_itself_and_not_a_temporary_prefix() {
    let source = String::from("前缀：Rust 中文");
    let excerpt = Excerpt { text: &source };
    let result;
    {
        let prefix = String::from("前缀：");
        result = excerpt.without_prefix(&prefix);
    }
    assert_eq!(result, "Rust 中文");
    assert_eq!(result.as_ptr(), source[9..].as_ptr());
    assert_eq!(excerpt.without_prefix("不存在"), source);
    assert_eq!(excerpt.without_prefix(""), source);
    assert_eq!(
        Excerpt {
            text: "\t中文\u{2003}Rust"
        }
        .first_word(),
        "中文"
    );
    assert_eq!(Excerpt { text: " \t" }.first_word(), "");
}
