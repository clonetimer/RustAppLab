//! 挑战入口：已有基础功能与学员需亲手实现的增强功能分开验收。
const MISSIONS: &[(&str, &str, &str)] = &[
    ("12", "点亮控制台", "运行演示、读取第一条真实 HTTP 响应"),
    (
        "13",
        "文件筛选关卡",
        "通过 ?ext=rs 参数在服务端按扩展名筛选",
    ),
    ("14", "日志分级关卡", "支持 ?min_level=ERROR 统计高等级日志"),
    (
        "15",
        "服务器身份关卡",
        "支持 --name 参数并在 /api/health 返回自定义名称",
    ),
];

pub fn show(id: Option<&str>) -> Result<(), String> {
    if let Some(id) = id {
        let (id, title, goal) = MISSIONS
            .iter()
            .find(|entry| entry.0 == id)
            .ok_or_else(|| "关卡编号应为 12、13、14 或 15".to_string())?;
        println!("LocalHub 第 {id} 关 · {title}\n目标：{goal}\n教材：course/{id}_*.md");
    } else {
        println!("LocalHub · 12–15 关：");
        for (id, title, goal) in MISSIONS {
            println!("  {id} {title} · {goal}");
        }
        println!("课程索引：course/README.md");
    }
    Ok(())
}
