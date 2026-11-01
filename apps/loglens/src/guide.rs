//! 每关均指向真实应用行为与有边界的扩展挑战。
const MISSIONS: &[(&str, &str, &str)] = &[
    ("08", "逐行读日志", "course/08_first_log.md"),
    ("09", "异常与解析契约", "course/09_parsing_and_errors.md"),
    ("10", "版本前后比较", "course/10_compare_and_metrics.md"),
    ("11", "提交可审查的报告", "course/11_log_report.md"),
];

pub fn show(id: Option<&str>) -> Result<(), String> {
    match id {
        None => {
            println!("LogLens · 教学关卡 08–11");
            for (number, name, path) in MISSIONS {
                println!("  {number} {name} · {path}");
            }
            Ok(())
        }
        Some(key) => {
            let (_, name, path) = MISSIONS
                .iter()
                .find(|item| item.0 == key)
                .ok_or_else(|| format!("不存在的关卡：{key}"))?;
            println!("关卡 {key}：{name}\n教程：{path}\n先运行功能，再阅读、修改、验收。");
            Ok(())
        }
    }
}
