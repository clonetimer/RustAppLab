//! 泛型、trait bound 和借用生命周期练习。

use std::fmt::Display;

/// 拥有标签和值的泛型学习卡。 `impl<T>` 为实现块声明类型参数。
///
/// 结构体或函数中多次使用同一个 `T`，要求各处的类型一致。
///
/// ```compile_fail,E0308
/// fn pair<T>(left: T, right: T) -> (T, T) {
///     (left, right)
/// }
/// let _ = pair(7, "Rust");
/// ```
///
/// 独立的类型参数允许异构值。方法也可以声明自己的 `U`，如 `Card::map`。
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::Card;
///
/// fn pair<T, U>(left: T, right: U) -> (T, U) {
///     (left, right)
/// }
/// assert_eq!(pair(7, "Rust"), (7, "Rust"));
///
/// let card = Card::new(String::from("字节数"), String::from("中文"));
/// let count: Card<usize> = card.map(|text| text.len());
/// assert_eq!(count.describe(), "字节数: 6");
/// ```
///
/// 同一类型参数放在结构体的两个字段中，也会要求字段类型一致。
///
/// ```compile_fail,E0308
/// struct Pair<T> {
///     left: T,
///     right: T,
/// }
///
/// let _ = Pair { left: 7, right: "Rust" };
/// ```
///
/// ```
/// struct Pair<T, U> {
///     left: T,
///     right: U,
/// }
///
/// let pair = Pair { left: 7, right: "Rust" };
/// assert_eq!((pair.left, pair.right), (7, "Rust"));
/// ```
///
/// 泛型枚举可为不同变体指定不同的载荷类型。
///
/// ```
/// enum Input<T, U> {
///     Score(T),
///     Note(U),
/// }
///
/// let inputs: [Input<i32, &str>; 2] = [Input::Score(7), Input::Note("Rust")];
/// let labels = inputs.map(|input| match input {
///     Input::Score(score) => format!("分数：{score}"),
///     Input::Note(note) => format!("笔记：{note}"),
/// });
/// assert_eq!(labels, ["分数：7", "笔记：Rust"]);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Card<T> {
    pub label: String,
    pub value: T,
}

impl<T> Card<T> {
    pub fn new(label: String, value: T) -> Self {
        Self { label, value }
    }

    /// 消费原卡，将标签移动到存放新值的卡片。
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Card<U> {
        Card::new(self.label, f(self.value))
    }
}

impl<T: Display> Card<T> {
    /// 值满足 `Display` 时，卡片可以格式化为一行。
    ///
    /// ```compile_fail,E0599
    /// use book_d6_foundation_lab::generic_lifetimes::Card;
    ///
    /// let card = Card::new(String::from("列表"), vec![1, 2]);
    /// let _ = card.describe();
    /// ```
    ///
    /// ```
    /// use book_d6_foundation_lab::generic_lifetimes::Card;
    ///
    /// let card = Card::new(String::from("列表"), vec![1, 2]);
    /// assert_eq!(card.map(|values| values.len()).describe(), "列表: 2");
    /// ```
    pub fn describe(&self) -> String {
        format!("{}: {}", self.label, self.value)
    }
}

impl Card<i32> {
    /// 判断整数卡片的值是否大于零。该方法专用于 `Card<i32>`。
    ///
    /// ```compile_fail,E0599
    /// use book_d6_foundation_lab::generic_lifetimes::Card;
    ///
    /// let card = Card::new(String::from("长度"), 7_usize);
    /// let _ = card.is_positive();
    /// ```
    ///
    /// ```
    /// use book_d6_foundation_lab::generic_lifetimes::Card;
    ///
    /// assert!(Card::new(String::from("分数"), 7_i32).is_positive());
    /// ```
    pub fn is_positive(&self) -> bool {
        self.value > 0
    }
}

/// 实现者提供标签，也可以覆盖默认摘要。
pub trait Summary {
    fn label(&self) -> &str;

    fn summarize(&self) -> String {
        format!("学习卡：{}", self.label())
    }
}

impl<T> Summary for Card<T> {
    fn label(&self) -> &str {
        &self.label
    }
}

/// 返回两份摘要。两个参数位置上的 `impl Summary` 各自引入独立的类型参数。
///
/// ```compile_fail,E0308
/// use book_d6_foundation_lab::generic_lifetimes::{Card, Summary};
///
/// fn same_kind<T: Summary>(left: &T, right: &T) -> (String, String) {
///     (left.summarize(), right.summarize())
/// }
///
/// let number = Card::new(String::from("分数"), 7);
/// let text = Card::new(String::from("章节"), "Rust");
/// let _ = same_kind(&number, &text);
/// ```
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::{summaries, Card, Summary};
///
/// struct Notice;
/// impl Summary for Notice {
///     fn label(&self) -> &str { "提醒" }
///     fn summarize(&self) -> String { String::from("请先预测") }
/// }
///
/// let card = Card::new(String::from("借用"), ());
/// assert_eq!(summaries(&card, &Notice), ("学习卡：借用".into(), "请先预测".into()));
/// ```
///
/// `where` 可以把多个类型参数的约束集中放在签名末尾。
///
/// ```
/// use std::fmt::Debug;
/// use book_d6_foundation_lab::generic_lifetimes::{Card, Summary};
///
/// fn format_inline<T: Summary + Debug, U: Summary>(left: &T, right: &U) -> String {
///     format!("{} / {} / {:?}", left.summarize(), right.summarize(), left)
/// }
///
/// fn format_where<T, U>(left: &T, right: &U) -> String
/// where
///     T: Summary + Debug,
///     U: Summary,
/// {
///     format!("{} / {} / {:?}", left.summarize(), right.summarize(), left)
/// }
///
/// let number = Card::new(String::from("分数"), 7);
/// let text = Card::new(String::from("章节"), "Rust");
/// let inline = format_inline(&number, &text);
/// assert_eq!(inline, format_where(&number, &text));
/// assert!(inline.starts_with("学习卡：分数 / 学习卡：章节"));
/// ```
///
/// 返回位置的 `impl Summary` 隐藏一个具体类型。所有返回分支必须使用该类型。
///
/// ```compile_fail,E0308
/// use book_d6_foundation_lab::generic_lifetimes::{Card, Summary};
///
/// fn make_card(numeric: bool) -> impl Summary {
///     if numeric {
///         Card::new(String::from("分数"), 7)
///     } else {
///         Card::new(String::from("章节"), "Rust")
///     }
/// }
/// ```
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::{Card, Summary};
///
/// fn make_card(numeric: bool) -> impl Summary {
///     if numeric {
///         Card::new(String::from("分数"), String::from("7"))
///     } else {
///         Card::new(String::from("章节"), String::from("Rust"))
///     }
/// }
///
/// assert_eq!(make_card(false).summarize(), "学习卡：章节");
/// ```
pub fn summaries(left: &impl Summary, right: &impl Summary) -> (String, String) {
    (left.summarize(), right.summarize())
}

/// 借用切片中的最大值，空切片返回 `None`，并列时保留最先出现的值。
///
/// 无约束的 `T` 无法使用比较运算符。
///
/// ```compile_fail,E0369
/// fn greater<T>(left: &T, right: &T) -> bool {
///     left > right
/// }
/// ```
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::largest;
///
/// let labels = [String::from("a"), String::from("c"), String::from("b")];
/// assert!(std::ptr::eq(largest(&labels).unwrap(), &labels[1]));
/// assert_eq!(largest::<String>(&[]), None);
/// ```
///
/// 在挑选最高分的任务中，三种签名可以共用同一套查找逻辑。`Copy` 和
/// `Clone` 允许带走拥有的结果。`Clone` 可能分配内存，例如克隆非空
/// `String` 时需要独立的缓冲区。借用版本只需要 `Ord`，其结果引用原元素。
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::largest;
///
/// fn largest_copy<T: Ord + Copy>(values: &[T]) -> Option<T> {
///     largest(values).copied()
/// }
///
/// fn largest_clone<T: Ord + Clone>(values: &[T]) -> Option<T> {
///     largest(values).cloned()
/// }
///
/// let scores = [2, 9, 4];
/// assert_eq!(largest_copy(&scores), Some(9));
/// assert_eq!(largest_clone(&scores), Some(9));
/// assert_eq!(largest(&scores), Some(&9));
///
/// let winner = {
///     let entries = [(2, String::from("甲")), (9, String::from("乙"))];
///     let borrowed = largest(&entries).unwrap();
///     assert!(std::ptr::eq(borrowed, &entries[1]));
///     largest_clone(&entries).unwrap()
/// };
/// assert_eq!(winner, (9, String::from("乙")));
///
/// #[derive(PartialEq, Eq, PartialOrd, Ord)]
/// struct Entry(i32, String);
/// let entries = [Entry(2, "甲".into()), Entry(9, "乙".into())];
/// assert!(std::ptr::eq(largest(&entries).unwrap(), &entries[1]));
///
/// let empty: [i32; 0] = [];
/// assert_eq!(largest_copy(&empty), None);
/// assert_eq!(largest_clone(&empty), None);
/// assert_eq!(largest(&empty), None);
/// ```
pub fn largest<T: Ord>(values: &[T]) -> Option<&T> {
    let (mut largest, rest) = values.split_first()?;
    for value in rest {
        if value > largest {
            largest = value;
        }
    }
    Some(largest)
}

/// 按 UTF-8 字节数选出较长的输入，字节数相等时取左侧。
///
/// 两个输入都要能在输出借用的使用范围内保持有效。生命周期标注描述借用
/// 之间的关系，即使实际会选中左侧，也无法用它延长右侧值的生存时间。
///
/// ```compile_fail,E0597
/// use book_d6_foundation_lab::generic_lifetimes::choose_longer;
///
/// let left = String::from("longer");
/// let chosen;
/// {
///     let right = String::from("x");
///     chosen = choose_longer(&left, &right);
/// }
/// assert_eq!(chosen, "longer");
/// ```
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::choose_longer;
///
/// let left = String::from("中文");
/// let right = String::from("Rust");
/// assert_eq!(choose_longer(&left, &right), "中文");
/// ```
///
/// 两个独立输入的自由函数仍需显式说明返回值借自哪个输入。
///
/// ```compile_fail,E0106
/// fn first(left: &str, _right: &str) -> &str {
///     left
/// }
/// ```
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::first_label;
///
/// assert_eq!(first_label("left", "right"), "left");
/// ```
pub fn choose_longer<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

/// 返回第一个输入的视图。第二个输入有独立的生命周期。
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::first_label;
///
/// let first = String::from("持续可用");
/// let result;
/// {
///     let second = String::from("短作用域");
///     result = first_label(&first, &second);
/// }
/// assert_eq!(result, "持续可用");
/// ```
pub fn first_label<'a>(first: &'a str, _second: &str) -> &'a str {
    first
}

/// 借用原文的摘录。使用摘录中引用的字段时，原文必须仍然有效。
///
/// ```compile_fail,E0597
/// use book_d6_foundation_lab::generic_lifetimes::Excerpt;
///
/// let excerpt;
/// {
///     let source = String::from("Rust 中文");
///     excerpt = Excerpt { text: &source };
/// }
/// assert_eq!(excerpt.first_word(), "Rust");
/// ```
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::Excerpt;
///
/// let source = String::from("Rust 中文");
/// let excerpt = Excerpt { text: &source };
/// assert_eq!(excerpt.first_word(), "Rust");
/// ```
///
/// `'static` 也无法让函数返回对本地 `String` 的引用。
///
/// ```compile_fail,E0515
/// fn heading() -> &'static str {
///     let text = String::from("Rust");
///     &text
/// }
/// ```
///
/// ```
/// fn literal_heading() -> &'static str {
///     "Rust"
/// }
///
/// fn owned_heading() -> String {
///     String::from("Rust")
/// }
///
/// assert_eq!(literal_heading(), owned_heading());
/// ```
///
/// 函数签名可以同时约束类型参数和借用。
///
/// ```
/// use book_d6_foundation_lab::generic_lifetimes::{Card, Summary};
///
/// fn longer_with_announcement<'a, T: Summary>(left: &'a str, right: &'a str, item: &T) -> &'a str {
///     println!("{}", item.summarize());
///     book_d6_foundation_lab::generic_lifetimes::choose_longer(left, right)
/// }
///
/// let left = String::from("中文");
/// let right = String::from("Rust");
/// let result;
/// {
///     let card = Card::new(String::from("比较字节数"), ());
///     result = longer_with_announcement(&left, &right, &card);
/// }
/// assert_eq!(result, "中文");
/// ```
pub struct Excerpt<'a> {
    pub text: &'a str,
}

impl Excerpt<'_> {
    /// 按 Unicode 空白切分并返回首词，没有词时返回空字符串。
    pub fn first_word(&self) -> &str {
        self.text.split_whitespace().next().unwrap_or("")
    }

    /// 去掉匹配的前缀，匹配失败时返回原文。输出借自 `self`。
    ///
    /// ```
    /// use book_d6_foundation_lab::generic_lifetimes::Excerpt;
    ///
    /// let excerpt = Excerpt { text: "前缀：Rust" };
    /// let result;
    /// {
    ///     let prefix = String::from("前缀：");
    ///     result = excerpt.without_prefix(&prefix);
    /// }
    /// assert_eq!(result, "Rust");
    /// ```
    pub fn without_prefix(&self, prefix: &str) -> &str {
        self.text.strip_prefix(prefix).unwrap_or(self.text)
    }
}
