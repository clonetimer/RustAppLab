//! 教学任务以真实软件特性和验收结果组织，而不是按照语法条目堆叠。

const MISSIONS: &[(&str, &str, &str, &str)] = &[
    (
        "00",
        "第一张地图",
        "生成演示工程，查看离线报告",
        "course/00_first_five_minutes.md",
    ),
    (
        "01",
        "让命令行听懂你",
        "理解 match 与 enum，改善命令体验",
        "course/01_cli_contract.md",
    ),
    (
        "02",
        "深入文件森林",
        "使用 PathBuf 与标准库遍历文件",
        "course/02_filesystem.md",
    ),
    (
        "03",
        "容错的扫描仪",
        "借助 Result 处理失败路径",
        "course/03_error_handling.md",
    ),
    (
        "04",
        "数据分析小队",
        "结构体、Vec、BTreeMap 与聚合",
        "course/04_collections.md",
    ),
    (
        "05",
        "失踪文件追踪",
        "借用、迭代器与名称检索",
        "course/05_ownership_and_search.md",
    ),
    (
        "06",
        "把结果变成产品",
        "HTML 报告、转义与自动化测试",
        "course/06_report_and_tests.md",
    ),
    (
        "07",
        "打包交付",
        "发布构建、CLI 验收与兼容性",
        "course/07_ship.md",
    ),
];

pub fn show(id: Option<&str>) -> Result<(), String> {
    if let Some(id) = id {
        let lesson = MISSIONS
            .iter()
            .find(|lesson| lesson.0 == id)
            .ok_or_else(|| format!("任务 {id} 不存在；运行 missions 查看有效编号"))?;
        println!("任务 {} · {}", lesson.0, lesson.1);
        println!("目标：{}", lesson.2);
        println!("讲义：{}", lesson.3);
        println!("验收：按照讲义执行实际命令，并通过 cargo test。\n");
    } else {
        println!("FileQuest 任务地图 · 每次完成一个实际软件能力\n");
        for (id, title, outcome, _) in MISSIONS {
            println!("{id}  {title:<14}  {outcome}");
        }
        println!("\n打开对应 course/ 教程；例如：cargo run -- missions 02");
    }
    Ok(())
}
