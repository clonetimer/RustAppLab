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
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
    // LESSON 1 END
}
impl svg::Element for Badge {
    fn render(&self) -> String {
        // LESSON 2 BEGIN
        format!(
            r##"<g><rect x="12" y="12" width="616" height="216" rx="20" fill="#14273d"/><rect x="36" y="40" width="7" height="160" rx="3" fill="#63dfc1"/><text x="62" y="66" font-size="13" fill="#63dfc1">RUST APPLAB / MY CREATION</text><text x="62" y="125" font-size="30" fill="#ffffff">{}</text><text x="62" y="170" font-size="17" fill="#cad8e8">{}</text></g>"##,
            escape_xml(&self.title),
            escape_xml(&self.subtitle)
        )
        // LESSON 2 END
    }
}
pub fn document<T: svg::Element>(element: &T) -> String {
    // LESSON 3 BEGIN
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 640 240\" role=\"img\" aria-label=\"作品卡\" font-family=\"sans-serif\">{}</svg>\n",element.render())
    // LESSON 3 END
}
