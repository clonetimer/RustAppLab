//! 逐级提示：优先给观察方向，最后一级才显示具体命令。
pub const HELP: &str = concat!(
    "LabQuest ",
    env!("CARGO_PKG_VERSION"),
    r#" · Rust AppLab 的案例式学习控制台

完全零基础：请先打开 START_HERE.html，Windows 用户可双击 START_WINDOWS.cmd。

从 RustAppLab 根目录运行：
  cargo run --offline -p labquest -- advanced start my-workbench    进入进阶 A
  cargo run --offline -p labquest -- advanced check my-workbench 1  累计检查1/2/3
  cargo run --offline -p labquest -- advanced lesson 1              读取进阶讲义
  cargo run --offline -p labquest -- advanced-b start my-taskpocket-pro  进入多 crate 与 Serde 课程
  cargo run --offline -p labquest -- advanced-b add-json <旧工程>     显式添加 B4 JSON 扩展
  cargo run --offline -p labquest -- advanced-b prepare-json <工程>  显式联网准备 B4 固定依赖
  cargo run --offline -p labquest -- advanced-b check <工程> <1|2|3|4|all>  累计验收；all=4
  cargo run --offline -p labquest -- advanced-c2 start <新工程>    C2 Axum工单服务
  cargo run --offline -p labquest -- advanced-c2 prepare <工程>   显式准备固定依赖
  cargo run --offline -p labquest -- advanced-c2 check <工程> [1|2|3|all]  累计验收
  cargo run --offline -p labquest -- advanced-c1 start <新工程>    C1真实异步下载
  cargo run --offline -p labquest -- advanced-c1 prepare <工程>   显式准备Rust1.85固定依赖
  cargo run --offline -p labquest -- advanced-c1 check <工程> [1|2|3|all]  累计验收
  cargo run --offline -p labquest -- learn reading [工程]   Book 对照阅读（只读概念导航）
  cargo run --offline -p labquest -- learn route          查看推荐的分层学习路线
  cargo run --offline -p labquest -- learn lesson <工程> [步骤]           当前微任务卡
  cargo run --offline -p labquest -- learn hint <工程> <1|2|3> [步骤]     概念/思路/自测三级提示
  cargo run --offline -p labquest -- learn check <工程> [步骤|all]        真正编译检查；进阶 B 有4阶段
  cargo run --offline -p labquest -- learn next my-unitlab  不运行程序，给出当前工程下一步
  cargo run --offline -p labquest -- learn diagnose error.txt 解释第一条常见 Rust 编译报错
  cargo run --offline -p labquest -- ecosystem plan      维护者：查看网络依赖准备（不下载）
  cargo run --offline -p labquest -- ecosystem prepare network-offline-kit  明确联网准备与本机探针

  cargo run -p labquest -- studio catalog                十二款分步作品目录
  cargo run -p labquest -- studio start unitlab my-unitlab  创建三个 TODO 的独立工程
  cargo run -p labquest -- studio check my-unitlab 1      只检查第一阶段；可选 2/3/all
  cargo run -p labquest -- studio lesson unitlab          显示完整讲义
  cargo run -p labquest -- studio hint unitlab            查看分层提示

  cargo run -p labquest -- beginner start my-first-card   创建只改一行的独立 Rust 小工程
  cargo run -p labquest -- beginner check my-first-card   真实编译并用四种输入验收
  cargo run -p labquest -- beginner hint <1|2|3>         分层提示（最后才提供整行示例）
  cargo run -p labquest -- start <新案例目录>       建立案例及可运行的 Rust 小项目
  cargo run -p labquest -- status <案例目录>      按真实产出检查六项工作
  cargo run -p labquest -- next <案例目录>        给出当前下一步、Rust 知识与行动命令（只读）
  cargo run -p labquest -- practice              查看三道边界预测练习
  cargo run -p labquest -- predict <阈值> <数量>   先预测 ERROR 记录数，再核对反馈（不计分）
  cargo run -p labquest -- simulate <案例目录>    根据配置重跑确定性模拟（更新 current.log）
  cargo run -p labquest -- test-app <案例目录>    编译运行你编辑的独立 Rust 项目，验证多个阈值下的 SUMMARY
  cargo run -p labquest -- board <案例目录>       生成可离线打开的 HTML 任务进度板
  cargo run -p labquest -- hint <1-6> [1-3]     按层级领取提示，避免直接看答案
  cargo run -p labquest -- finish <案例目录>      六步验收全绿后生成调查交付报告
  cargo run -p labquest -- app init <新目录>    创建独立 TaskPocket 软件工程（从零实现）
  cargo run -p labquest -- app check <项目目录>  编译并以 9 组真实跨进程测试验证作品
  cargo run -p labquest -- app hint <1|2|3>     由浅入深获取 TaskPocket 开发提示
  app checkpoint <taskpocket|pagemint> <目录> <1|2|3>  独立作品累计检查点（3=原完整合同）
  cargo run -p labquest -- app catalog              显示两款独立实战作品与教学目标
  cargo run -p labquest -- app init pagemint <目录>  创建离线笔记网页生成器骨架
  cargo run -p labquest -- app check pagemint <目录> 编译并检查真实 HTML 与异常输入
  cargo run -p labquest -- app hint pagemint <1|2|3> 获取不含完整答案的分级提示
  # 兼容原 app init/check <目录>，默认仍为 TaskPocket
  cargo run -p labquest -- help                 显示本帮助

基础教学可以离线运行；B4 首次依赖准备与 ecosystem prepare 会显式联网。start 只创建全新目录，simulate 和 board 只更新案例自己的输出。
test-app 和 app check 会显式编译并执行你编辑的 Rust 代码，仅对可信的自建练习案例使用；它们不是沙箱。
新案例使用 v2 多阈值迁移验收；旧版 v1 案例继续保持单配置验证，建议新学员使用新案例。
"#
);

const HINTS: [[&str; 3]; 6] = [
    [
        "先找到 project/ 内的源码和配置，不必一开始逐行阅读。你需要一份可搜索的文件地图。",
        "FileQuest 的 report 可以把目录扫描结果写为 HTML。交付路径是 deliverables/inventory.html。",
        "cargo run -p filequest -- report <案例>/project -o <案例>/deliverables/inventory.html （已有报告且内容变化时，加 --force）。",
    ],
    [
        "不要凭直觉判断事故原因。比较 logs/baseline.log 和 logs/incident.log 的 ERROR 数量。",
        "LogLens 的 compare 可显示两个版本的错误记录占比；report 则生成可交付 Markdown。",
        "cargo run -p loglens -- compare <案例>/logs/baseline.log <案例>/logs/incident.log；再使用 report <案例>/logs/incident.log -o <案例>/deliverables/incident.md。",
    ],
    [
        "查看事故日志中的 latency_ms、timeout_ms。哪些请求被判定为 ERROR？",
        "检查 project/config/service.conf；在这个模拟中，当 latency_ms > timeout_ms 才报 ERROR。修复后一定要 simulate。",
        "样例 12 条请求中最长 750ms：将 timeout_ms 设为 800（仅适用于教学样例），执行 cargo run -p labquest -- simulate <案例>。",
    ],
    [
        "不要只看配置文件已经改变；还需要分析新日志来证明改善。",
        "LogLens 的 report 可为 current.log 生成与当前数据一致的 Markdown；与 incident.md 并排比较。",
        "cargo run -p loglens -- report <案例>/logs/current.log -o <案例>/deliverables/current.md；已存在时先确认再加 --force。",
    ],
    [
        "你需要真正修改并编译 project/src/main.rs，而不是只修改日志或配置；固定写死 errors=0 不能通过新案例。",
        "运行 labquest practice，先预测 150/400/750ms 的错误数；再看迭代器 iter().filter().count() 如何根据当前 timeout 计算。",
        "在 for 循环后新增：let errors = LATENCIES.iter().filter(|&&ms| ms > timeout).count(); 然后 println!(\"SUMMARY requests=12 errors={errors}\"); 再运行 cargo run -p labquest -- test-app <案例>。",
    ],
    [
        "代码和配置是最终要交付的对象；仅有 Markdown 报告不算完成现场封存。",
        "SafeSync 的 backup + verify 组合可以保护 project/ 的字节内容。备份必须对应最新配置。",
        "cargo run -p safesync -- backup <案例>/project <案例>/backups/project-snapshot；再执行 verify <案例>/backups/project-snapshot。已有旧快照不要直接覆盖，请使用全新的案例或经确认后的新备份路径。",
    ],
];

pub fn hint(step: usize, tier: usize) -> Result<&'static str, String> {
    if !(1..=6).contains(&step) || !(1..=3).contains(&tier) {
        return Err("用法：hint <步骤1-6> [层级1-3]".into());
    }
    Ok(HINTS[step - 1][tier - 1])
}
