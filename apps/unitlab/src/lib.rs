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
    celsius * 9.0 / 5.0 + 32.0
    // LESSON 1 END
}
pub fn convert(value: f64, from: Unit, to: Unit) -> Result<f64, String> {
    // LESSON 2 BEGIN
    if !value.is_finite() {
        return Err("数值必须是有限数，不能是 NaN 或无穷大".into());
    }
    let result = match (from, to) {
        (a, b) if a == b => value,
        (Unit::Celsius, Unit::Fahrenheit) => celsius_to_fahrenheit(value),
        (Unit::Fahrenheit, Unit::Celsius) => (value - 32.0) * 5.0 / 9.0,
        (Unit::Meter, Unit::Kilometer) => value / 1000.0,
        (Unit::Kilometer, Unit::Meter) => value * 1000.0,
        _ => return Err("温度和长度不能互相换算".into()),
    };
    if !result.is_finite() {
        return Err("计算结果超出表示范围".into());
    }
    Ok(result)
    // LESSON 2 END
}
pub fn describe(value: &str, from: &str, to: &str) -> Result<String, String> {
    // LESSON 3 BEGIN
    let number = value
        .parse::<f64>()
        .map_err(|_| "请输入数字，例如 25 或 -40".to_string())?;
    let result = convert(number, Unit::parse(from)?, Unit::parse(to)?)?;
    Ok(format!("{number:.2} {from} = {result:.2} {to}"))
    // LESSON 3 END
}
