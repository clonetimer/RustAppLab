//! 用 trait 和泛型把数据变成可独立打开的矢量图。
pub mod svg {
    pub trait Element {
        fn render(&self) -> String;
    }
}
#[derive(Debug)]
pub struct Badge {
    pub title: String,
    pub subtitle: String,
}
pub fn escape_xml(text: &str) -> String {
    // LESSON 1 BEGIN
    // TODO(1): 正确转义文本而不破坏中文。先读 LESSON.md 对应步骤。
    let _ = text;
    String::new()
    // LESSON 1 END
}
impl svg::Element for Badge {
    fn render(&self) -> String {
        // LESSON 2 BEGIN
        // TODO(2): 为作品卡实现 Element trait。先读 LESSON.md 对应步骤。
        let _ = self;
        String::new()
        // LESSON 2 END
    }
}
pub fn document<T: svg::Element>(element: &T) -> String {
    // LESSON 3 BEGIN
    // TODO(3): 用泛型组合独立 SVG 文档。先读 LESSON.md 对应步骤。
    let _ = element;
    String::new()
    // LESSON 3 END
}
