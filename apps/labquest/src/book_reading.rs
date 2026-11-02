//! Book references are a read-only companion to existing projects, never a checker.
use std::{fs, path::Path};

const VERSIONS: &str = "版本说明：rust.bootcss.com 中文站基于 Rust 1.37 / Edition 2018；本项目使用 Edition 2021 / MSRV 1.74。当前官方 Book 的版本和示例配置可能更新，实际配置以本工程 Cargo.toml 为准，不自动迁移 edition、依赖或工具链。";
const READ_ONLY: &str = "对照阅读只读：不编译或执行学员代码，不写进度或报告，不自动打开网页；读过参考资料不代表通过检查。";

#[derive(Clone, Copy)]
struct Topic {
    title: &'static str,
    chinese: &'static str,
    official: &'static str,
}
const CARGO: Topic = Topic {
    title: "Cargo 与工程入口",
    chinese: "https://rust.bootcss.com/ch01-03-hello-cargo.html",
    official: "https://doc.rust-lang.org/book/ch01-03-hello-cargo.html",
};
const MATCH: Topic = Topic {
    title: "枚举与 match",
    chinese: "https://rust.bootcss.com/ch06-02-match.html",
    official: "https://doc.rust-lang.org/book/ch06-02-match.html",
};
const STRING: Topic = Topic {
    title: "字符串与 UTF-8",
    chinese: "https://rust.bootcss.com/ch08-02-strings.html",
    official: "https://doc.rust-lang.org/book/ch08-02-strings.html",
};
const BORROW: Topic = Topic {
    title: "引用 & 借用",
    chinese: "https://rust.bootcss.com/ch04-02-references-and-borrowing.html",
    official: "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html",
};
const RESULT: Topic = Topic {
    title: "Result 与错误传播",
    chinese: "https://rust.bootcss.com/ch09-02-recoverable-errors-with-result.html",
    official: "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html",
};
const TEST: Topic = Topic {
    title: "测试组织与公开行为",
    chinese: "https://rust.bootcss.com/ch11-03-test-organization.html",
    official: "https://doc.rust-lang.org/book/ch11-03-test-organization.html",
};
const THREAD: Topic = Topic {
    title: "线程与消息传递",
    chinese: "https://rust.bootcss.com/ch16-02-message-passing.html",
    official: "https://doc.rust-lang.org/book/ch16-02-message-passing.html",
};
const TRAIT: Topic = Topic {
    title: "trait 与共享行为",
    chinese: "https://rust.bootcss.com/ch10-02-traits.html",
    official: "https://doc.rust-lang.org/book/ch10-02-traits.html",
};
const SMART_POINTER: Topic = Topic {
    title: "Rc / RefCell 与引用环",
    chinese: "https://rust.bootcss.com/ch15-06-reference-cycles.html",
    official: "https://doc.rust-lang.org/book/ch15-06-reference-cycles.html",
};
const WORKSPACE: Topic = Topic {
    title: "Cargo 工作空间",
    chinese: "https://rust.bootcss.com/ch14-03-cargo-workspaces.html",
    official: "https://doc.rust-lang.org/cargo/reference/workspaces.html",
};
// The old translation has no async chapter: label the difference rather than invent a URL.
const ASYNC: Topic = Topic {
    title: "并发与 async（中文为线程基础；async 阅读当前官方章节）",
    chinese: "https://rust.bootcss.com/ch16-00-concurrency.html",
    official: "https://doc.rust-lang.org/book/ch17-00-async-await.html",
};

struct Reading {
    title: &'static str,
    topics: &'static [Topic],
    question: &'static str,
    check: &'static str,
}
fn mapping(id: &str) -> Option<Reading> {
    let (title, topics, question, check): (_, &[Topic], _, _) = match id {
        "firstcard" => ("FirstCard", &[CARGO, STRING][..], "换成自己的姓名和任务（包含中文或空格），先写下预期输出；哪个字符串改变，哪个工程入口保持不变？", "learn check"),
        "unitlab" => ("UnitLab / unitlab", &[MATCH, RESULT][..], "用自己的温度和非法单位输入预测 Ok / Err，边界值应返回什么？", "learn check"),
        "quizdeck" => ("QuizDeck / quizdeck", &[MATCH, STRING, BORROW][..], "用自己设计的正确、错误和空白答案预测得分；比较答案时借用了哪个 String，题目 Vec 的所有权是否改变？", "learn check"),
        "textscope" => ("TextScope / textscope", &[STRING, BORROW][..], "输入你自己的一段中英混合文本，先预测字节数、字符数和词数；读取文本是否需要拿走所有权？", "learn check"),
        "ledgerlite" => ("LedgerLite / ledgerlite", &[RESULT, TEST][..], "给出自己的合法账目和格式错误账目，先预测哪条记录会被拒绝以及总额是否改变。", "learn check"),
        "badgeforge" => ("BadgeForge / badgeforge", &[STRING, TRAIT, TEST][..], "在自己的标题里加入 <、& 和引号，预测转义后的 SVG；自己的 Element 实现如何通过泛型 document 生成同一份产物？", "learn check"),
        "termgarden" => ("TermGarden / termgarden", &[MATCH, RESULT, TEST][..], "用自己的任务名连续新增、完成、查询；哪个枚举变体改变状态，错误命令是否保留旧状态？", "learn check"),
        "batchmeter" => ("BatchMeter / batchmeter", &[THREAD, RESULT][..], "用自己准备的两个文本文件与一个不存在路径，预测各线程返回什么，以及汇总是否依赖完成顺序。", "learn check"),
        "minikv" => ("MiniKV / minikv", &[BORROW, TRAIT, RESULT][..], "用自己的键和值查询同一前缀；迭代器借用的数据来自哪里，活跃借用期间能否修改仓库？", "learn check"),
        "netfetch" => ("NetFetch / netfetch", &[RESULT, TEST][..], "用自己的本机响应正文，预测成功与无效响应的结果；失败时输出文件应是什么状态？", "learn check"),
        "servicedesk" => ("ServiceDesk / servicedesk", &[MATCH, TEST][..], "用自己的工单正文请求已知与未知路由，预测状态码和响应内容，再验证公开接口。", "learn check"),
        "storedesk" => ("StoreDesk / storedesk", &[RESULT, TEST][..], "用自己的一组库存变更，让中间一条失败；预测事务结束后全部库存，而不只看最后一行。", "learn check"),
        "asyncboard" => ("AsyncBoard / asyncboard", &[ASYNC, RESULT][..], "用自己的提醒内容和等待条件，预测创建 future 与 await 完成的区别；漏掉 await 会少哪项结果？", "learn check"),
        "taskpocket" => ("TaskPocket", &[MATCH, RESULT, TEST][..], "用自己的任务执行新增、完成、重启读取，并加入一条无效命令；先为持久化和错误路径各写一个预期。", "learn check"),
        "pagemint" => ("PageMint", &[STRING, RESULT, TEST][..], "用自己的中文标题、< 标签和 & 符号生成页面；先写下预期转义文本及缺少输入文件时的行为。", "learn check"),
        "advanced-a" => ("进阶 A · TermGarden 工作区", &[TRAIT, BORROW, SMART_POINTER][..], "用自己的任务集合观察共享与修改：哪些 Rc 保持节点存活，RefCell 何时检查借用，使用 Weak 能怎样避免引用环？", "advanced check"),
        "advanced-b" => ("进阶 B · 多 crate Workspace", &[WORKSPACE, RESULT, TEST][..], "用自己的任务数据同时驱动 CLI 与报告；业务库的同一错误应如何跨 crate 传播并被公开行为测试发现？", "advanced-b check"),
        _ => return None,
    };
    Some(Reading {
        title,
        topics,
        question,
        check,
    })
}

pub(crate) fn overview() -> String {
    format!("Rust AppLab · 对照阅读\n\n本项目以现有作品练习，Book 用于理解当前行为：\nFirstCard → Cargo 与字符串；12 款 Studio → match、Result、借用、测试与并发；TaskPocket / PageMint → 独立作品；进阶 A / B → trait、工作空间与跨模块验证。\n按作品查看：cargo run --offline -p labquest -- learn reading <学员工程目录>\n只选当前作品的 1–3 个概念，预测自己的输入，再回到 learn check 或 advanced / advanced-b check 验证。\n中文参考：https://rust.bootcss.com/\n官方参考：https://doc.rust-lang.org/book/\n\n{VERSIONS}\n{READ_ONLY}\n")
}

/// Reuse the learning navigator's bounded marker readers and established detection rules.
fn identify(root: &Path) -> Result<String, String> {
    crate::learning::learning_root(root)?;
    for (marker, expected, id) in [
        (
            ".applab-advanced-b",
            "RustAppLab Advanced B v1\n",
            "advanced-b",
        ),
        (
            ".applab-advanced",
            "RustAppLab Advanced A v1\n",
            "advanced-a",
        ),
    ] {
        if fs::symlink_metadata(root.join(marker)).is_ok() {
            if crate::learning::require_regular(&root.join(marker), 128)? != expected {
                return Err("进阶工程标记格式有误，不会猜测工程类型".into());
            }
            return Ok(id.into());
        }
    }
    if let Some(id) = crate::learning::studio_id(root)? {
        return Ok(id);
    }
    if crate::learning::first_card(root) {
        return Ok("firstcard".into());
    }
    if let Some(id) = crate::independent::identify(root)? {
        return Ok(id.into());
    }
    Err("未知学习工程目录：仅支持 FirstCard、12 款 Studio、TaskPocket、PageMint 与进阶 A/B；不会修改目录".into())
}

pub(crate) fn for_project(root: &Path) -> Result<String, String> {
    let id = identify(root)?;
    let reading = mapping(&id).ok_or("未配置该工程的对照阅读")?;
    let mut text = format!("{} · 对照阅读\n\n", reading.title);
    for topic in reading.topics {
        text.push_str(&format!(
            "概念：{}\n  中文：{}\n  官方：{}\n",
            topic.title, topic.chinese, topic.official
        ));
    }
    text.push_str(&format!("\n用自己的输入思考/实验：{}\n回到实际验证：cargo run --offline -p labquest -- {} \"{}\"{}\n\n{VERSIONS}\n{READ_ONLY}\n", reading.question, reading.check, root.display(), if reading.check == "learn check" { "" } else { " 1" }));
    Ok(text)
}

pub(crate) fn lesson_reminder(root: &Path) -> String {
    format!("\n对照阅读：cargo run --offline -p labquest -- learn reading \"{}\"（只读概念导航；阅读后仍需实际 check）\n", root.display())
}

pub(crate) fn markdown(id: &str) -> Option<String> {
    let reading = mapping(id)?;
    let mut text = String::from("\n## 对照阅读\n\n");
    for t in reading.topics {
        text.push_str(&format!(
            "- {}：[中文参考]({}) · [当前官方]({})\n",
            t.title, t.chinese, t.official
        ));
    }
    text.push_str(&format!("\n用自己的输入思考/实验：{}\n\n回到平台运行 `learn reading <本学员工程目录>` 查看导航，再用 `learn check <本学员工程目录>` 实际验证。\n\n{VERSIONS}\n\n{READ_ONLY}\n", reading.question));
    Some(text)
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
pub(crate) fn html(id: &str) -> Option<String> {
    let reading = mapping(id)?;
    let mut text = String::from("<section><h2>对照阅读</h2><ul>");
    for t in reading.topics {
        text.push_str(&format!(
            "<li>{}：<a href=\"{}\">中文参考</a> · <a href=\"{}\">当前官方</a></li>",
            escape(t.title),
            escape(t.chinese),
            escape(t.official)
        ));
    }
    text.push_str(&format!("</ul><p>用自己的输入思考/实验：{}</p><p>回到平台运行 <code>learn reading &lt;本学员工程目录&gt;</code> 查看导航，再用 <code>learn check &lt;本学员工程目录&gt;</code> 实际验证。</p><p>{}</p><p>{}</p></section>", escape(reading.question), escape(VERSIONS), escape(READ_ONLY)));
    Some(text)
}
