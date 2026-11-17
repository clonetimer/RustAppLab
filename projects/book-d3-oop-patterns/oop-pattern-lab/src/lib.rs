//! Executable reference examples for Rust Book chapters 17 and 18.
//!
//! The module contrasts encapsulation, trait-object dispatch and state transitions,
//! then explores literal, range, destructuring, guard, @ and refutable patterns.
//!
//! Refutable patterns cannot be used as an ordinary `let` binding:
//!
//! ```compile_fail
//! let Some(value) = None::<i32>; // This pattern may fail to match.
//! println!("{value}");
//! ```

/// 18.1–18.3: messages provide examples of enum and struct destructuring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

pub struct User {
    pub active: bool,
    pub username: String,
    pub email: String,
    pub sign_in_count: u64,
}

/// Literal, multiple, range, and wildcard patterns.
pub fn classify_number(value: i32) -> &'static str {
    match value {
        0 => "zero",
        1 | 2 => "one-or-two",
        3..=9 => "small-positive",
        -9..=-1 => "small-negative",
        _ => "other",
    }
}

/// Tuple destructuring distinguishes origin, axes, and general coordinates.
pub fn describe_point((x, y): (i32, i32)) -> &'static str {
    match (x, y) {
        (0, 0) => "origin",
        (0, _) => "y-axis",
        (_, 0) => "x-axis",
        (_, _) => "plane",
    }
}

pub fn describe_message(message: &Message) -> String {
    match message {
        Message::Quit => "quit".into(),
        Message::Move { x, y } => format!("move:{x},{y}"),
        Message::Write(text) => format!("write:{text}"),
        Message::ChangeColor(r, g, b) => format!("color:{r},{g},{b}"),
    }
}

/// Struct destructuring with ignored fields and the rest pattern.
pub fn describe_user(user: &User) -> String {
    let User {
        active, username, ..
    } = user;
    format!("{username}:active={active}")
}

/// The @ operator both tests a range and binds the original value.
pub fn capture_id(id: i32) -> String {
    match id {
        value @ 1..=100 => format!("valid-id:{value}"),
        _ => "invalid-id".into(),
    }
}

/// Match guards refine a pattern without changing the pattern itself.
pub fn classify_optional(value: Option<i32>, threshold: i32) -> &'static str {
    match value {
        Some(number) if number >= threshold => "at-or-above",
        Some(_) => "below",
        None => "missing",
    }
}

/// A for loop uses an irrefutable binding for each item.
pub fn first_even(values: &[i32]) -> Option<i32> {
    for &value in values {
        if value % 2 == 0 {
            return Some(value);
        }
    }
    None
}

/// while let repeatedly handles the Some variant and stops at None.
pub fn take_until_negative(values: &[i32]) -> Vec<i32> {
    let mut iter = values.iter();
    let mut taken = Vec::new();
    while let Some(&value) = iter.next() {
        if value < 0 {
            break;
        }
        taken.push(value);
    }
    taken
}

/// let-else handles the refutable case and keeps the success path unindented.
pub fn require_username(name: Option<&str>) -> Result<String, &'static str> {
    let Some(name) = name else {
        return Err("username is required");
    };
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("username cannot be empty");
    }
    Ok(trimmed.to_owned())
}

/// 17.1: private state protects an invariant; callers use methods, not fields.
#[derive(Debug, Default, PartialEq)]
pub struct AveragedCollection {
    values: Vec<i32>,
    sum: i64,
}

impl AveragedCollection {
    pub fn add(&mut self, value: i32) {
        self.values.push(value);
        self.sum += i64::from(value);
    }

    pub fn remove(&mut self) -> Option<i32> {
        let value = self.values.pop()?;
        self.sum -= i64::from(value);
        Some(value)
    }

    pub fn average(&self) -> Option<f64> {
        (!self.values.is_empty())
            .then(|| self.sum as f64 / self.values.len() as f64)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// 17.2: behavior is described by a trait and dispatched through trait objects.
pub trait Draw {
    fn draw(&self) -> String;
}

pub struct Button {
    pub label: String,
}

impl Draw for Button {
    fn draw(&self) -> String {
        format!("[button:{}]", self.label)
    }
}

pub struct Text {
    pub value: String,
}

impl Draw for Text {
    fn draw(&self) -> String {
        format!("[text:{}]", self.value)
    }
}

pub struct Screen {
    components: Vec<Box<dyn Draw>>,
}

impl Screen {
    pub fn new(components: Vec<Box<dyn Draw>>) -> Self {
        Self { components }
    }

    pub fn run(&self) -> Vec<String> {
        self.components.iter().map(|component| component.draw()).collect()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

/// 17.3: classic state pattern. State transitions consume the previous state.
trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;
    fn content<'a>(&self, _post: &'a Post) -> &'a str {
        ""
    }
    fn is_published(&self) -> bool {
        false
    }
}

struct Draft;
struct PendingReview;
struct Published;

impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview)
    }

    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }
}

impl State for PendingReview {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn approve(self: Box<Self>) -> Box<dyn State> {
        Box::new(Published)
    }
}

impl State for Published {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.content
    }

    fn is_published(&self) -> bool {
        true
    }
}

pub struct Post {
    content: String,
    state: Box<dyn State>,
}

impl Post {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            state: Box::new(Draft),
        }
    }

    pub fn content(&self) -> &str {
        self.state.content(self)
    }

    pub fn request_review(&mut self) {
        let state = std::mem::replace(&mut self.state, Box::new(Draft));
        self.state = state.request_review();
    }

    pub fn approve(&mut self) {
        let state = std::mem::replace(&mut self.state, Box::new(Draft));
        self.state = state.approve();
    }

    pub fn is_published(&self) -> bool {
        self.state.is_published()
    }
}

/// Stable command output for the tutorial and CLI tests.
pub fn demo(topic: &str) -> Result<Vec<String>, &'static str> {
    match topic {
        "patterns" => Ok(vec![
            classify_number(0).into(),
            classify_number(7).into(),
            describe_point((0, 4)).into(),
            describe_message(&Message::Move { x: 2, y: 5 }),
            capture_id(42),
            classify_optional(Some(9), 8).into(),
        ]),
        "oop" => {
            let mut values = AveragedCollection::default();
            values.add(10);
            values.add(20);
            let screen = Screen::new(vec![
                Box::new(Button {
                    label: "save".into(),
                }),
                Box::new(Text {
                    value: "ready".into(),
                }),
            ]);
            Ok(vec![
                format!("average={:.1}", values.average().unwrap_or_default()),
                format!("components={}", screen.len()),
                screen.run().join("|"),
            ])
        }
        "state" => {
            let mut post = Post::new("A Rust state-pattern example.");
            let draft = format!("draft-visible={}", post.content().is_empty());
            post.request_review();
            let review = format!("review-visible={}", post.content().is_empty());
            post.approve();
            Ok(vec![
                draft,
                review,
                format!("published={}", post.is_published()),
                format!("content={}", post.content()),
            ])
        }
        _ => Err("主题只能是 patterns、oop 或 state"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_pattern_matches_zero() {
        assert_eq!(classify_number(0), "zero");
    }

    #[test]
    fn or_pattern_matches_either_value() {
        assert_eq!(classify_number(1), "one-or-two");
        assert_eq!(classify_number(2), "one-or-two");
    }

    #[test]
    fn inclusive_range_matches_small_positive_values() {
        assert_eq!(classify_number(3), "small-positive");
        assert_eq!(classify_number(9), "small-positive");
    }

    #[test]
    fn negative_range_matches_small_negative_values() {
        assert_eq!(classify_number(-9), "small-negative");
        assert_eq!(classify_number(-1), "small-negative");
    }

    #[test]
    fn wildcard_handles_values_outside_explicit_patterns() {
        assert_eq!(classify_number(10), "other");
        assert_eq!(classify_number(-10), "other");
    }

    #[test]
    fn tuple_pattern_distinguishes_origin() {
        assert_eq!(describe_point((0, 0)), "origin");
    }

    #[test]
    fn tuple_pattern_distinguishes_axes() {
        assert_eq!(describe_point((0, 4)), "y-axis");
        assert_eq!(describe_point((4, 0)), "x-axis");
    }

    #[test]
    fn tuple_pattern_matches_general_plane() {
        assert_eq!(describe_point((4, 7)), "plane");
    }

    #[test]
    fn enum_unit_variant_is_destructured() {
        assert_eq!(describe_message(&Message::Quit), "quit");
    }

    #[test]
    fn enum_struct_variant_binds_named_fields() {
        assert_eq!(describe_message(&Message::Move { x: 2, y: 5 }), "move:2,5");
    }

    #[test]
    fn enum_tuple_variant_binds_owned_payload() {
        assert_eq!(
            describe_message(&Message::Write("hello".into())),
            "write:hello"
        );
    }

    #[test]
    fn enum_tuple_variant_destructures_multiple_fields() {
        assert_eq!(
            describe_message(&Message::ChangeColor(1, 2, 3)),
            "color:1,2,3"
        );
    }

    #[test]
    fn struct_pattern_uses_rest_to_ignore_unneeded_fields() {
        let user = User {
            active: true,
            username: "ada".into(),
            email: "ada@example.test".into(),
            sign_in_count: 4,
        };
        assert_eq!(describe_user(&user), "ada:active=true");
    }

    #[test]
    fn at_pattern_binds_the_value_that_passed_a_range() {
        assert_eq!(capture_id(42), "valid-id:42");
        assert_eq!(capture_id(0), "invalid-id");
        assert_eq!(capture_id(101), "invalid-id");
    }

    #[test]
    fn match_guard_checks_the_threshold() {
        assert_eq!(classify_optional(Some(9), 8), "at-or-above");
        assert_eq!(classify_optional(Some(8), 8), "at-or-above");
    }

    #[test]
    fn match_guard_routes_values_below_the_threshold() {
        assert_eq!(classify_optional(Some(7), 8), "below");
    }

    #[test]
    fn none_pattern_handles_missing_values() {
        assert_eq!(classify_optional(None, 8), "missing");
    }

    #[test]
    fn for_loop_pattern_finds_the_first_even_value() {
        assert_eq!(first_even(&[1, 3, 6, 8]), Some(6));
    }

    #[test]
    fn for_loop_pattern_handles_empty_or_odd_slices() {
        assert_eq!(first_even(&[]), None);
        assert_eq!(first_even(&[1, 3, 5]), None);
    }

    #[test]
    fn while_let_stops_at_the_first_negative_value() {
        assert_eq!(take_until_negative(&[1, 2, -1, 3]), vec![1, 2]);
    }

    #[test]
    fn while_let_handles_empty_input() {
        assert!(take_until_negative(&[]).is_empty());
    }

    #[test]
    fn let_else_returns_the_bound_value() {
        assert_eq!(require_username(Some(" Ada ")), Ok("Ada".into()));
    }

    #[test]
    fn let_else_handles_missing_and_blank_values() {
        assert_eq!(require_username(None), Err("username is required"));
        assert_eq!(require_username(Some("  ")), Err("username cannot be empty"));
    }

    #[test]
    fn averaged_collection_starts_empty() {
        let values = AveragedCollection::default();
        assert!(values.is_empty());
        assert_eq!(values.average(), None);
    }

    #[test]
    fn averaged_collection_add_maintains_sum_and_average() {
        let mut values = AveragedCollection::default();
        values.add(10);
        values.add(20);
        assert_eq!(values.len(), 2);
        assert_eq!(values.average(), Some(15.0));
    }

    #[test]
    fn averaged_collection_remove_maintains_invariant() {
        let mut values = AveragedCollection::default();
        values.add(10);
        values.add(20);
        assert_eq!(values.remove(), Some(20));
        assert_eq!(values.average(), Some(10.0));
        assert_eq!(values.remove(), Some(10));
        assert_eq!(values.remove(), None);
        assert_eq!(values.average(), None);
    }

    #[test]
    fn trait_object_screen_dispatches_to_multiple_concrete_types() {
        let screen = Screen::new(vec![
            Box::new(Button {
                label: "save".into(),
            }),
            Box::new(Text {
                value: "ready".into(),
            }),
        ]);
        assert_eq!(screen.run(), vec!["[button:save]", "[text:ready]"]);
    }

    #[test]
    fn trait_object_screen_handles_no_components() {
        let screen = Screen::new(Vec::new());
        assert!(screen.is_empty());
        assert!(screen.run().is_empty());
    }

    #[test]
    fn draft_content_is_hidden() {
        let post = Post::new("private draft");
        assert_eq!(post.content(), "");
        assert!(!post.is_published());
    }

    #[test]
    fn requesting_review_does_not_publish_content() {
        let mut post = Post::new("review this");
        post.request_review();
        assert_eq!(post.content(), "");
        assert!(!post.is_published());
    }

    #[test]
    fn approval_before_review_does_not_publish() {
        let mut post = Post::new("review this");
        post.approve();
        assert_eq!(post.content(), "");
        assert!(!post.is_published());
    }

    #[test]
    fn approval_after_review_publishes_content() {
        let mut post = Post::new("approved");
        post.request_review();
        post.approve();
        assert_eq!(post.content(), "approved");
        assert!(post.is_published());
    }

    #[test]
    fn repeated_review_request_does_not_skip_state() {
        let mut post = Post::new("review this");
        post.request_review();
        post.request_review();
        assert_eq!(post.content(), "");
        post.approve();
        assert_eq!(post.content(), "review this");
    }

    #[test]
    fn repeated_approval_does_not_revert_published_state() {
        let mut post = Post::new("published");
        post.request_review();
        post.approve();
        post.approve();
        assert_eq!(post.content(), "published");
    }
}
