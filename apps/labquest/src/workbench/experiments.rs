//! Isolated, built-in compiler comparisons. These never verify a learner project.
use super::{model::json_string, ApiError, App};
use std::collections::VecDeque;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

struct Example {
    code: &'static str,
    title: &'static str,
    question: &'static str,
    failure: &'static str,
    repair: &'static str,
}

const EXAMPLES: &[Example] = &[
    Example {
        code: "E0382", title: "移动后再次使用", question: "保存标题只需取得所有权一次；把打印移到 push 之前，为什么两处需求都能满足？换成借用参数会怎样？",
        failure: "fn main() {\n    let title = String::from(\"整理笔记\");\n    let tasks = vec![title];\n    println!(\"{title}: {}\", tasks.len());\n}\n",
        repair: "fn main() {\n    let title = String::from(\"整理笔记\");\n    println!(\"{title}\");\n    let tasks = vec![title];\n    assert_eq!(tasks[0], \"整理笔记\");\n}\n",
    },
    Example {
        code: "E0502", title: "共享借用与修改冲突", question: "把读取提前到 push 之前，引用的最后一次使用改变了什么？Vec 扩容为什么影响引用？",
        failure: "fn main() {\n    let mut tasks = vec![String::from(\"写测试\")];\n    let first = &tasks[0];\n    tasks.push(String::from(\"写文档\"));\n    println!(\"{first}\");\n}\n",
        repair: "fn main() {\n    let mut tasks = vec![String::from(\"写测试\")];\n    let first = &tasks[0];\n    assert_eq!(first, \"写测试\");\n    tasks.push(String::from(\"写文档\"));\n    assert_eq!(tasks.len(), 2);\n}\n",
    },
    Example {
        code: "E0499", title: "重叠的可变借用", question: "为什么两个活跃的 &mut 不能指向同一个值？先完成第一段修改再借用，会怎样？",
        failure: "fn main() {\n    let mut score = 1;\n    let first = &mut score;\n    let second = &mut score;\n    *first += 1;\n    *second += 2;\n}\n",
        repair: "fn main() {\n    let mut score = 1;\n    let first = &mut score;\n    *first += 1;\n    let second = &mut score;\n    *second += 2;\n    assert_eq!(score, 4);\n}\n",
    },
    Example {
        code: "E0308", title: "期望的返回类型不匹配", question: "println! 返回什么？把末尾表达式加上分号后，函数返回值还会是 i32 吗？",
        failure: "fn double(value: i32) -> i32 {\n    println!(\"{}\", value * 2);\n}\nfn main() { let _ = double(5); }\n",
        repair: "fn double(value: i32) -> i32 { value * 2 }\nfn main() { assert_eq!(double(5), 10); assert_eq!(double(-2), -4); }\n",
    },
    Example {
        code: "E0277", title: "Trait 约束没有满足", question: "普通结构体为什么不能直接用 {} 格式化？Debug 和 Display 分别适合什么表达？",
        failure: "struct Note { count: u32 }\nfn main() { println!(\"{}\", Note { count: 2 }); }\n",
        repair: "#[derive(Debug)]\nstruct Note { count: u32 }\nfn main() {\n    let note = Note { count: 2 };\n    assert_eq!(note.count, 2);\n    assert_eq!(format!(\"{note:?}\"), \"Note { count: 2 }\");\n}\n",
    },
    Example {
        code: "E0599", title: "方法不可用", question: "方法属于具体类型或可见 trait；在 String 上调用 push 后，怎样取到新增后的长度？",
        failure: "fn main() { let count: u32 = 3; count.push('!'); }\n",
        repair: "fn main() {\n    let mut text = String::from(\"3\");\n    text.push('!');\n    assert_eq!(text, \"3!\");\n}\n",
    },
    Example {
        code: "E0596", title: "绑定不可变", question: "让绑定可变与改变数据类型是同一件事吗？函数接收 &String 时，能仅靠 mut 修改原文吗？",
        failure: "fn main() { let title = String::from(\"Rust\"); title.push('!'); }\n",
        repair: "fn main() {\n    let mut title = String::from(\"Rust\");\n    title.push('!');\n    assert_eq!(title, \"Rust!\");\n}\n",
    },
    Example {
        code: "E0425", title: "名称不在当前作用域", question: "局部绑定在离开代码块后为什么不可见？怎样通过块的返回值把结果带出来？",
        failure: "fn main() {\n    { let count = 3; println!(\"{count}\"); }\n    println!(\"{count}\");\n}\n",
        repair: "fn main() {\n    let count = { let base = 2; base + 1 };\n    assert_eq!(count, 3);\n}\n",
    },
    Example {
        code: "E0432", title: "导入路径不存在", question: "use 不会创建类型；怎样核对模块中真正公开的名称？拼写修复后，检查还会验证什么？",
        failure: "use std::collections::HashMapp;\nfn main() {}\n",
        repair: "use std::collections::HashMap;\nfn main() {\n    let mut scores = HashMap::new();\n    scores.insert(\"A\", 3);\n    assert_eq!(scores.get(\"A\"), Some(&3));\n}\n",
    },
    Example {
        code: "E0433", title: "模块路径无法解析", question: "相对模块路径与 crate 根路径有什么区别？声明模块后，为什么还要让成员 pub？",
        failure: "fn main() { let _ = missing_module::score(); }\n",
        repair: "mod scores { pub fn value() -> u32 { 3 } }\nfn main() { assert_eq!(crate::scores::value(), 3); }\n",
    },
    Example {
        code: "E0597", title: "被借用的值过早释放", question: "为什么 lifetime 标注不能延长局部 String 的生命？将所有权移出来能解决哪个问题？",
        failure: "fn main() {\n    let borrowed: &str;\n    { let message = String::from(\"已完成\"); borrowed = &message; }\n    println!(\"{borrowed}\");\n}\n",
        repair: "fn main() {\n    let owned: String;\n    { let message = String::from(\"已完成\"); owned = message; }\n    assert_eq!(owned, \"已完成\");\n}\n",
    },
];

fn example(code: &str) -> Result<&'static Example, ApiError> {
    EXAMPLES
        .iter()
        .find(|e| e.code == code)
        .ok_or_else(|| ApiError::new(404, "这个错误还没有内置对照实验，请保留原始诊断"))
}

pub(super) fn material(code: &str) -> Result<String, ApiError> {
    let e = example(code)?;
    Ok(format!(
        "{{\"code\":{},\"title\":{},\"question\":{},\"failure\":{},\"repair\":{}}}",
        json_string(e.code),
        json_string(e.title),
        json_string(e.question),
        json_string(e.failure),
        json_string(e.repair)
    ))
}

#[derive(Clone)]
pub(super) struct ExperimentJob {
    id: u64,
    code: String,
    variant: String,
    status: &'static str,
    output: String,
    compiler_exit_code: Option<i32>,
    program_exit_code: Option<i32>,
    matched: Option<bool>,
}

impl ExperimentJob {
    pub(super) fn json(&self) -> String {
        fn number(value: Option<i32>) -> String {
            value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into())
        }
        format!("{{\"id\":{},\"code\":{},\"variant\":{},\"status\":{},\"output\":{},\"compiler_exit_code\":{},\"program_exit_code\":{},\"matched\":{}}}", self.id, json_string(&self.code), json_string(&self.variant), json_string(self.status), json_string(&self.output), number(self.compiler_exit_code), number(self.program_exit_code), self.matched.map(|v| v.to_string()).unwrap_or_else(|| "null".into()))
    }
}

pub(super) struct Store {
    next: u64,
    entries: VecDeque<ExperimentJob>,
}
impl Store {
    pub(super) fn new() -> Self {
        Self {
            next: 1,
            entries: VecDeque::new(),
        }
    }
    fn get(&self, id: u64) -> Result<ExperimentJob, ApiError> {
        self.entries
            .iter()
            .find(|j| j.id == id)
            .cloned()
            .ok_or_else(|| ApiError::new(404, "实验结果已过期，请重新运行"))
    }
}

fn expected_diagnostic(stderr: &str, code: &str) -> bool {
    let diagnostics: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("error["))
        .collect();
    !diagnostics.is_empty()
        && diagnostics
            .iter()
            .all(|line| line.starts_with(&format!("error[{code}]:")))
}

struct Outcome {
    output: String,
    compiler: Option<i32>,
    program: Option<i32>,
    matched: bool,
}

fn execute(e: &Example, variant: &str) -> Result<Outcome, String> {
    let scratch = crate::capstone::TempWorkspace::new_named("compiler-comparison")?;
    let source = scratch.0.join("comparison.rs");
    std::fs::write(
        &source,
        if variant == "failure" {
            e.failure
        } else {
            e.repair
        },
    )
    .map_err(|e| e.to_string())?;
    let binary = scratch
        .0
        .join(format!("comparison{}", std::env::consts::EXE_SUFFIX));
    let mut compiler = Command::new("rustc");
    compiler
        .args([
            "--edition=2021",
            "--error-format=human",
            "--color=never",
            "--crate-name=rustapplab_comparison",
        ])
        .arg(&source)
        .arg("-o")
        .arg(&binary);
    let compilation = crate::process::capture(
        &mut compiler,
        Duration::from_secs(30),
        &scratch.0,
        "对照实验编译",
    )?;
    let stderr = String::from_utf8_lossy(&compilation.stderr);
    let mut output = format!("实验 {} · {}\n命令：rustc --edition=2021 comparison.rs -o comparison\n编译退出码：{:?}\n{}{}", e.code, if variant == "failure" { "观察预期编译错误" } else { "验证修复与断言" }, compilation.status.code(), String::from_utf8_lossy(&compilation.stdout), stderr);
    let mut program_exit_code = None;
    let matched = if variant == "failure" {
        !compilation.status.success() && expected_diagnostic(&stderr, e.code)
    } else if compilation.status.success() {
        let execution = crate::process::capture(
            &mut Command::new(&binary),
            Duration::from_secs(5),
            &scratch.0,
            "修复实验运行",
        )?;
        program_exit_code = execution.status.code();
        output.push_str(&format!(
            "\n运行退出码：{program_exit_code:?}\n{}{}",
            String::from_utf8_lossy(&execution.stdout),
            String::from_utf8_lossy(&execution.stderr)
        ));
        execution.status.success()
    } else {
        false
    };
    output.push_str(if matched {
        "\n实际结果符合本实验预期。\n"
    } else {
        "\n实际结果未符合预期，请查看以上编译器或环境诊断。\n"
    });
    output.push_str("这是内置示例的实验结果，未检查或修改你的项目，也不计为课程通过或理解评分。");
    Ok(Outcome {
        output,
        compiler: compilation.status.code(),
        program: program_exit_code,
        matched,
    })
}

pub(super) fn start(app: &Arc<App>, code: &str, variant: &str) -> Result<ExperimentJob, ApiError> {
    let e = example(code)?;
    if !matches!(variant, "failure" | "repair") {
        return Err(ApiError::new(400, "请选择失败对照或修复对照"));
    }
    let mut store = app
        .experiments
        .lock()
        .map_err(|_| ApiError::new(500, "实验状态不可用"))?;
    if store.entries.iter().any(|j| j.status == "running") {
        return Err(ApiError::new(409, "已有对照实验正在执行，请等待结果"));
    }
    let id = store.next;
    store.next += 1;
    let job = ExperimentJob {
        id,
        code: code.into(),
        variant: variant.into(),
        status: "running",
        output: "正在独立临时目录调用真实 Rust 编译器…".into(),
        compiler_exit_code: None,
        program_exit_code: None,
        matched: None,
    };
    store.entries.push_back(job.clone());
    while store.entries.len() > 16 {
        store.entries.pop_front();
    }
    let shared = Arc::clone(app);
    let variant = variant.to_owned();
    let spawned = std::thread::Builder::new().name(format!("compiler-comparison-{id}")).spawn(move || {
        let result = execute(e, &variant);
        if let Ok(mut store) = shared.experiments.lock() {
            if let Some(job) = store.entries.iter_mut().find(|j| j.id == id) {
                match result {
                    Ok(outcome) => { job.status = "completed"; job.output = outcome.output; job.compiler_exit_code = outcome.compiler; job.program_exit_code = outcome.program; job.matched = Some(outcome.matched); }
                    Err(message) => { job.status = "error"; job.output = format!("无法完成实验：{message}\n请确认 Rust/Cargo 已安装。环境错误不算预期的编译失败。"); job.matched = Some(false); }
                }
            }
        }
    });
    if let Err(error) = spawned {
        if let Some(job) = store.entries.iter_mut().find(|j| j.id == id) {
            job.status = "error";
            job.output = format!("无法启动实验：{error}");
            job.matched = Some(false);
        }
        return store.get(id);
    }
    Ok(job)
}

pub(super) fn get(app: &App, id: u64) -> Result<ExperimentJob, ApiError> {
    app.experiments
        .lock()
        .map_err(|_| ApiError::new(500, "实验状态不可用"))?
        .get(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_supported_diagnostic_has_a_failure_and_repair() {
        for code in [
            "E0382", "E0502", "E0499", "E0308", "E0277", "E0599", "E0596", "E0425", "E0432",
            "E0433", "E0597",
        ] {
            let example = EXAMPLES
                .iter()
                .find(|e| e.code == code)
                .expect("missing compiler experiment");
            assert!(example.failure.contains("fn main"));
            assert!(example.repair.contains("assert"));
            assert!(!example.title.is_empty() && !example.question.is_empty());
        }
    }

    #[test]
    fn source_mentions_and_other_errors_do_not_satisfy_expected_failure() {
        assert!(!expected_diagnostic(
            "error[E0425]: unknown\n | // E0382",
            "E0382"
        ));
        assert!(!expected_diagnostic(
            "error[E0382]: moved\nerror[E0425]: unknown",
            "E0382"
        ));
        assert!(!expected_diagnostic("failed to launch rustc", "E0382"));
        assert!(expected_diagnostic(
            "error[E0382]: moved\nerror: aborting",
            "E0382"
        ));
        assert!(material("../../Cargo.toml").is_err());
    }

    #[test]
    fn actual_compiler_rejects_then_runs_all_supported_comparisons() {
        for example in EXAMPLES {
            for variant in ["failure", "repair"] {
                let result = execute(example, variant).unwrap();
                assert!(
                    result.matched,
                    "{} {variant}: {}",
                    example.code, result.output
                );
                if variant == "failure" {
                    assert_ne!(result.compiler, Some(0));
                    assert_eq!(result.program, None);
                } else {
                    assert_eq!(result.compiler, Some(0));
                    assert_eq!(result.program, Some(0));
                }
            }
        }
    }
}
