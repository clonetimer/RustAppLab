//! 单位换算：第一次把输入、计算、错误和输出连接起来。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Celsius,
    Fahrenheit,
    Meter,
    Kilometer,
}
impl Unit {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "c" => Ok(Self::Celsius),
            "f" => Ok(Self::Fahrenheit),
            "m" => Ok(Self::Meter),
            "km" => Ok(Self::Kilometer),
            _ => Err(format!("未知单位 {value}；请用 c/f/m/km")),
        }
    }
}
/// 把摄氏温度转为华氏温度。输入有效性在 convert 里检查。
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    // LESSON 1 BEGIN
    // TODO(1): 写出温度转换公式。先读 LESSON.md 对应步骤。
    let _ = celsius;
    0.0
    // LESSON 1 END
}
pub fn convert(value: f64, from: Unit, to: Unit) -> Result<f64, String> {
    // LESSON 2 BEGIN
    // TODO(2): 用 enum 和 match 区分单位。先读 LESSON.md 对应步骤。
    let _ = (value, from, to);
    Err("请完成第 2 步：转换与校验".into())
    // LESSON 2 END
}
pub fn describe(value: &str, from: &str, to: &str) -> Result<String, String> {
    // LESSON 3 BEGIN
    // TODO(3): 连接 parse、Result 与格式化输出。先读 LESSON.md 对应步骤。
    let _ = (value, from, to);
    Err("请完成第 3 步：输入与输出".into())
    // LESSON 3 END
}
