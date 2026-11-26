//! Embedded Book projects are copied into learner-owned workspaces. Manifests,
//! lockfiles and integration checks keep their canonical identity; only sources,
//! examples and the learner log are exposed for editing.
use crate::learning_cards::Card;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub(super) struct EmbeddedFile {
    pub path: &'static str,
    pub content: &'static str,
}

pub(super) struct RunTarget {
    pub package: &'static str,
    pub binary: &'static str,
    pub args: &'static [&'static str],
}

pub(super) struct BookCourse {
    pub id: &'static str,
    pub title: &'static str,
    pub default_file: &'static str,
    pub lesson_topic: &'static str,
    reference_dir: &'static str,
    pub source_dirs: &'static [&'static str],
    pub files: &'static [EmbeddedFile],
    pub run: Option<RunTarget>,
    function: &'static str,
    concept: &'static str,
    prediction: &'static str,
    experiment: &'static str,
    reflection: &'static str,
}

pub(super) const STEPS: &[&str] = &[
    "读源码，写下预测",
    "修改副本，观察失败并修复",
    "解释结果，迁移到自己的作品",
];
const HINTS: [&str; 3] = [
    "先打开本地讲义，对照默认源码找到本步函数；在学习记录写下预期输出或诊断。",
    "每次只改一个条件，保存后运行检查；保留完整命令、实际输出和失败原因。",
    "恢复后重新检查，换一组输入，并解释为什么结果改变；测试通过本身不表示掌握。",
];
const LEARNING_LOG: &str = include_str!("../../../../course/LEARNING_LOG_TEMPLATE.md");

impl BookCourse {
    pub fn is_modern(&self) -> bool {
        self.id.starts_with("modern-")
    }

    pub fn bundle(&self) -> Option<&'static str> {
        match self.id {
            "modern-d10b" => Some("D10-B-learner-offline.zip"),
            "modern-e1a" => Some("E1-A-learner-offline.zip"),
            "modern-e1b" => Some("E1-B-learner-offline.zip"),
            "modern-e2" => Some("E2-learner-offline.zip"),
            "modern-e3" => Some("E3-learner-offline.zip"),
            _ => None,
        }
    }

    // Validate supplied prerequisites without embedding earlier-course answers.
    #[rustfmt::skip]
    pub fn support_hashes(&self) -> &'static [(&'static str, &'static str)] {
        match self.id {
            "modern-e1a" => &[
                ("support/c2/Cargo.toml", "44d0e09409f016e475995e5d13af687a0ebf6f5c856cf4f49909f793ab3db1c0"),
                ("support/c2/src/handlers.rs", "be3f770b043022d8887d27e1ee70025c898f73a26d5f60fa5009b3ecba66138d"),
                ("support/c2/src/lib.rs", "dbbd259fca3b757fb424b72c34344ffbab2f68e9e23afcebcec9ee5df7d3646f"),
                ("support/c2/src/main.rs", "33d7c334d3c0bff5dc196a8c71bcf1a40669f12fed2a02121b1d436f22b4f86c"),
                ("support/c2/src/model.rs", "c0b6c4b960faee618837548e291d0e242ac1f6b16c9b80217de6ee41077a329e"),
                ("support/c2/assets/app.css", "99fb944ec5f5f01977384a6ddcab7d74b820dfe585203482300f43f529aefac4"),
                ("support/c2/assets/client.js", "e152dd205fe7b91961747daf194ad4bbd0b030a476ef993480414ebeef386b41"),
                ("support/c2/assets/index.html", "360023b35ee11d01a0c2180e2b916366c82502207830a5e62dccd9ccde68530b"),
                ("support/servicedesk/Cargo.toml", "04f4b1902eb4a90704cd3f19cc875624d8d97c9ba24a93119e15328fc35565b6"),
                ("support/servicedesk/src/http.rs", "c9f57736ddf9e26923c3bc16935c57f41f1ac1f5ebb71f717e2b9063be96a78d"),
                ("support/servicedesk/src/lib.rs", "e76f90bd94d40ff968dc1a3bb35c2e2353e967ef421cb3fc78fb09e69f4b6e4f"),
                ("support/servicedesk/src/main.rs", "b39c98da11fbf83ba77c6f2ae43ec5e3778d88b61752ccc8f7610663cbc58930"),
                ("support/servicedesk/samples/index.html", "e3bc640a36994a6062dd28e7c39fda53fbac69196cf722a6b6f03361dfb4d610"),
            ],
            "modern-e1b" => &[
                ("support/e1a/support/c2/Cargo.toml", "44d0e09409f016e475995e5d13af687a0ebf6f5c856cf4f49909f793ab3db1c0"),
                ("support/e1a/support/c2/src/handlers.rs", "be3f770b043022d8887d27e1ee70025c898f73a26d5f60fa5009b3ecba66138d"),
                ("support/e1a/support/c2/src/lib.rs", "dbbd259fca3b757fb424b72c34344ffbab2f68e9e23afcebcec9ee5df7d3646f"),
                ("support/e1a/support/c2/src/main.rs", "33d7c334d3c0bff5dc196a8c71bcf1a40669f12fed2a02121b1d436f22b4f86c"),
                ("support/e1a/support/c2/src/model.rs", "c0b6c4b960faee618837548e291d0e242ac1f6b16c9b80217de6ee41077a329e"),
                ("support/e1a/support/c2/assets/app.css", "99fb944ec5f5f01977384a6ddcab7d74b820dfe585203482300f43f529aefac4"),
                ("support/e1a/support/c2/assets/client.js", "e152dd205fe7b91961747daf194ad4bbd0b030a476ef993480414ebeef386b41"),
                ("support/e1a/support/c2/assets/index.html", "360023b35ee11d01a0c2180e2b916366c82502207830a5e62dccd9ccde68530b"),
                ("support/e1a/support/servicedesk/Cargo.toml", "04f4b1902eb4a90704cd3f19cc875624d8d97c9ba24a93119e15328fc35565b6"),
                ("support/e1a/support/servicedesk/src/http.rs", "c9f57736ddf9e26923c3bc16935c57f41f1ac1f5ebb71f717e2b9063be96a78d"),
                ("support/e1a/support/servicedesk/src/lib.rs", "e76f90bd94d40ff968dc1a3bb35c2e2353e967ef421cb3fc78fb09e69f4b6e4f"),
                ("support/e1a/support/servicedesk/src/main.rs", "b39c98da11fbf83ba77c6f2ae43ec5e3778d88b61752ccc8f7610663cbc58930"),
                ("support/e1a/support/servicedesk/samples/index.html", "e3bc640a36994a6062dd28e7c39fda53fbac69196cf722a6b6f03361dfb4d610"),
                ("support/e1a/Cargo.toml", "9d13f22b442ac6f230b070d96649059c102a5cc6bfc623727d4d5ba94b15d898"),
                ("support/e1a/rust-toolchain.toml", "6b1cf8341d9a08d76c6deb1e9df90a0b208d0ef0d00ee05e0173e9cd4f0f425d"),
                ("support/e1a/src/lib.rs", "ee1f390a4acb923d0fdf2969a23dfd1e94ea16bd1a23f6c121fe5c500a4953f4"),
                ("support/e1a/LICENSE", "4cee53162acc2de60b774b57182c250bfd983160d5f5d58925564b904c7ccac4"),
            ],
            _ => &[],
        }
    }

    #[rustfmt::skip]
    pub fn steps(&self) -> &'static [&'static str] {
        match self.id {
            "modern-d8" => &["select_alarm：筛选有效告警", "ensure_threshold：保留或初始化阈值", "legacy_kind：兼容宏分派", "综合：全部固定合同与文档测试"],
            "modern-d9a" => &["next：借用滑动窗口", "owned_minimum：精确生命周期捕获", "named：trait upcast", "collect：异步 trait 顺序", "visit：借用异步闭包", "positive：match guard", "infallible：Never 类型", "append_batch：失败回滚", "copy_into：保留分配器", "综合：全部固定合同与文档测试"],
            "modern-d9b" => &["inspect_readings：观察 Result", "calibrate：each_mut 饱和加法", "empty_slots：inline const", "lookup：LazyLock", "timing_error：abs_diff", "after_prefix：skip_until", "c_label：CString 转换", "extract_outliers 与 update_two：筛选及互斥借用", "corrected：有符号偏差", "file_label：file_prefix", "update_and_read：锁降级", "fixed_pair：固定数组借用", "transitions：array_windows", "mask_info：位宽与最低置位", "token_spans：子串范围", "consume_boxed：消费型迭代", "综合：全部固定合同与文档测试"],
            "modern-d10a" => &["poll：首次 Pending 并唤醒", "join_pair：保留已完成结果", "join_many：异构 Future", "race_pair：竞争与取消", "poll SignalReceiver：最新 Waker", "pin_probe：固定 !Unpin 对象", "综合：全部固定合同与文档测试"],
            "modern-d11a" => &["encode：有界帧编码", "decode：安全校验及借用", "count_loop：循环计数", "count_iter：迭代器计数", "综合：全部固定合同与文档测试"],
            "modern-e4" => &["classify：有界分类", "summarize：统计与失败边界", "e4_summarize：调用方内存", "综合：全部固定合同与文档测试"],
            "modern-e5" => &["features：Welford 窗口统计", "fit_scaler：训练集标准化", "train：批量逻辑回归", "evaluate：混淆矩阵", "综合：全部固定合同与文档测试"],
            "modern-d10b" => &["join_samples：虚拟时间并发", "start_producer：有界通道", "run_bounded：限流及顺序", "ReceiverStream 与 merge_positive", "race_or_timeout 与取消", "blocking_work：阻塞池", "spawn_read：Send 异步 trait", "综合：全部固定合同与文档测试"],
            "modern-d11b" => &["checked_read：安全读取", "初始化全部数组槽位", "综合：全部固定合同与文档测试"],
            "modern-e1a" => &["认证：拒绝无效重复凭证", "权限：默认拒绝及 401/403", "TLS：拒绝错误证书与密钥", "握手：实际 TLS 与超时", "综合：全部固定合同与文档测试"],
            "modern-e1b" => &["批量事务：业务与 audit", "关闭：冲突与幂等", "有界队列：满与关闭", "结构化记录：脱敏和容量", "综合：全部固定合同与文档测试"],
            "modern-e2" => &["parse_samples：严格 CSV", "summarize：数据统计", "Model::apply：状态和失败边界", "src/gui.rs：真实控件产生 Action", "综合：全部固定合同与文档测试"],
            "modern-e3" => &["History：定长缓冲（含 1b）", "Periodic：回绕与错过周期", "Hysteresis：滞环控制", "Controller：HAL 与故障锁存", "综合：全部固定合同与文档测试"],
            _ => STEPS,
        }
    }

    pub fn card(&self, step: usize) -> Card {
        if self.is_modern() {
            return Card {
                id: self.id,
                step,
                function: self.steps()[step - 1],
                goal: self.steps()[step - 1],
                concept: self.concept,
                prediction: self.prediction,
                hints: HINTS,
                experiment: self.experiment,
                reflection: self.reflection,
            };
        }
        Card {
            id: self.id,
            step,
            function: self.function,
            goal: match step {
                1 => {
                    "打开本地讲义和源码，找出输入、分支与返回值；先在学习记录写下预测，再运行对照。"
                }
                2 => self.experiment,
                _ => self.reflection,
            },
            concept: self.concept,
            prediction: self.prediction,
            hints: HINTS,
            experiment: self.experiment,
            reflection: self.reflection,
        }
    }

    pub fn is_reference(&self, root: &Path) -> bool {
        if self.is_modern() {
            // Never open the shipped modern template or reference in place.
            return root.ancestors().any(|path| {
                let matches_course = path
                    .file_name()
                    .is_some_and(|name| name == self.reference_dir);
                let in_projects = path.parent().and_then(Path::file_name);
                matches_course && in_projects.is_some_and(|name| name == "projects")
            });
        }
        let embedded_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../projects")
            .join(self.reference_dir)
            .canonicalize();
        if embedded_root.as_ref().is_ok_and(|path| path == root) {
            return true;
        }
        // Runtime archives may live somewhere other than the build machine.
        // Recognize references inside both complete source and learner-kit trees.
        root.file_name()
            .is_some_and(|name| name == self.reference_dir)
            && root.parent().is_some_and(|parent| {
                parent.file_name().is_some_and(|name| name == "projects")
                    && parent.parent().is_some_and(|platform| {
                        platform.join("apps/labquest/Cargo.toml").is_file()
                            || [
                                "START_HERE.md",
                                "SUPPORTED_PLATFORMS.md",
                                "course/CURRICULUM.md",
                            ]
                            .iter()
                            .all(|path| platform.join(path).is_file())
                    })
            })
    }

    pub fn create(&self, root: &Path) -> Result<(), String> {
        // The caller checks the parent chain. create_dir rejects an existing
        // directory, so teacher sources and earlier work are never overwritten.
        fs::create_dir(root).map_err(|e| format!("无法创建学员副本 {}：{e}", root.display()))?;
        for file in self.files {
            let path = root.join(file.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .and_then(|mut out| out.write_all(file.content.as_bytes()))
                .map_err(|e| format!("无法写入 {}：{e}", path.display()))?;
        }
        if !self.files.iter().any(|file| file.path == "LEARNING_LOG.md") {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("LEARNING_LOG.md"))
                .and_then(|mut out| out.write_all(LEARNING_LOG.as_bytes()))
                .map_err(|e| format!("无法创建学习记录：{e}"))?;
        }
        Ok(())
    }

    pub fn protected(&self, path: &str) -> bool {
        if self.is_modern() {
            return !self.source_path(path) && path != "LEARNING_LOG.md";
        }
        path.ends_with("Cargo.toml")
            || path == "Cargo.lock"
            || path.starts_with("tests/")
            || path.contains("/tests/")
    }

    pub fn source_path(&self, path: &str) -> bool {
        if self.is_modern() {
            return path == "src/lib.rs" || (self.id == "modern-e2" && path == "src/gui.rs");
        }
        self.source_dirs
            .iter()
            .any(|dir| path.starts_with(&format!("{dir}/")))
    }

    pub fn readme(&self) -> &'static str {
        self.files
            .iter()
            .find(|file| file.path == "README.md")
            .map(|file| file.content)
            .unwrap_or("")
    }
}

pub(super) fn get(id: &str) -> Option<&'static BookCourse> {
    all().find(|course| course.id == id)
}

pub(super) fn all() -> impl Iterator<Item = &'static BookCourse> {
    COURSES.iter().chain(MODERN_COURSES.iter())
}

pub(super) const COURSES: &[BookCourse] = &[
    BookCourse {
        id: "book-d1",
        reference_dir: "book-d1-cargo",
        title: "Book D1 · Cargo 与工作空间",
        default_file: "book-card/src/lib.rs",
        lesson_topic: "d1",
        function: "describe 与 Cargo workspace",
        concept: "Cargo package、crate、path 依赖、发布和本地安装。",
        prediction: "给 describe 的输入从 workspace 换成 profiles，输出哪些行会变化？更改 book-card 后哪个二进制需要重编译？",
        experiment: "在自己的副本修改一条 describe 输出，运行 workspace；再暂时返回错误文字，检查哪个现有断言失败，最后修复。",
        reflection: "在自己的小工具中划出库和命令行入口，写明 path 依赖、--locked 与 --offline 各保证什么。",
        source_dirs: &["book-card/src", "cargo-applab-map/src"],
        run: Some(RunTarget { package: "cargo-applab-map", binary: "cargo-applab-map", args: &["workspace"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/book-d1-cargo/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/book-d1-cargo/Cargo.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/book-d1-cargo/README.md") },
            EmbeddedFile { path: "book-card/Cargo.toml", content: include_str!("../../../../projects/book-d1-cargo/book-card/Cargo.toml") },
            EmbeddedFile { path: "book-card/src/lib.rs", content: include_str!("../../../../projects/book-d1-cargo/book-card/src/lib.rs") },
            EmbeddedFile { path: "cargo-applab-map/Cargo.toml", content: include_str!("../../../../projects/book-d1-cargo/cargo-applab-map/Cargo.toml") },
            EmbeddedFile { path: "cargo-applab-map/src/main.rs", content: include_str!("../../../../projects/book-d1-cargo/cargo-applab-map/src/main.rs") },
            EmbeddedFile { path: "cargo-applab-map/tests/cli.rs", content: include_str!("../../../../projects/book-d1-cargo/cargo-applab-map/tests/cli.rs") },
        ],
    },
    BookCourse {
        id: "book-d2",
        reference_dir: "book-d2-smart-pointers",
        title: "Book D2 · 智能指针",
        default_file: "smart-pointer-lab/src/lib.rs",
        lesson_topic: "d2",
        function: "Box、Rc、RefCell、Weak",
        concept: "堆上所有权、引用计数、运行时借用与弱引用。",
        prediction: "clone 一个 Rc 后强引用计数怎样变化？把 Weak 换成 Rc 后，父子互指是否还能释放？",
        experiment: "先记录 Box/Rc/Weak 的输出，再改变一个计数或借用条件；用真实测试定位失败，恢复后说明生命周期。",
        reflection: "为自己的树或共享状态画出所有权方向，说明何时用普通借用、Rc 或 Weak。",
        source_dirs: &["smart-pointer-lab/src"],
        run: Some(RunTarget { package: "rustapplab-smart-pointer-lab", binary: "book_d2", args: &["rc"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/book-d2-smart-pointers/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/book-d2-smart-pointers/Cargo.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/book-d2-smart-pointers/README.md") },
            EmbeddedFile { path: "smart-pointer-lab/Cargo.toml", content: include_str!("../../../../projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml") },
            EmbeddedFile { path: "smart-pointer-lab/src/lib.rs", content: include_str!("../../../../projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs") },
            EmbeddedFile { path: "smart-pointer-lab/src/main.rs", content: include_str!("../../../../projects/book-d2-smart-pointers/smart-pointer-lab/src/main.rs") },
            EmbeddedFile { path: "smart-pointer-lab/tests/cli.rs", content: include_str!("../../../../projects/book-d2-smart-pointers/smart-pointer-lab/tests/cli.rs") },
        ],
    },
    BookCourse {
        id: "book-d3",
        reference_dir: "book-d3-oop-patterns",
        title: "Book D3 · 对象、状态与模式",
        default_file: "oop-pattern-lab/src/lib.rs",
        lesson_topic: "d3",
        function: "trait 对象、状态转换、match",
        concept: "动态分发、封装、模式匹配和状态的类型表达。",
        prediction: "草稿提交审核后能否直接看到内容？删去一个 match 分支时，是编译失败还是运行失败？",
        experiment: "运行 state 和 patterns，改一个状态转换或模式分支；先预测失败类型，再检查并修复。",
        reflection: "把自己的审批或任务流程表达成状态和合法转换，说明 trait 对象与枚举各适合什么。",
        source_dirs: &["oop-pattern-lab/src"],
        run: Some(RunTarget { package: "rustapplab-oop-pattern-lab", binary: "book_d3", args: &["state"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/book-d3-oop-patterns/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/book-d3-oop-patterns/Cargo.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/book-d3-oop-patterns/README.md") },
            EmbeddedFile { path: "oop-pattern-lab/Cargo.toml", content: include_str!("../../../../projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml") },
            EmbeddedFile { path: "oop-pattern-lab/src/lib.rs", content: include_str!("../../../../projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs") },
            EmbeddedFile { path: "oop-pattern-lab/src/main.rs", content: include_str!("../../../../projects/book-d3-oop-patterns/oop-pattern-lab/src/main.rs") },
            EmbeddedFile { path: "oop-pattern-lab/tests/cli.rs", content: include_str!("../../../../projects/book-d3-oop-patterns/oop-pattern-lab/tests/cli.rs") },
        ],
    },
    BookCourse {
        id: "book-d4",
        reference_dir: "book-d4-advanced-features",
        title: "Book D4 · 高级特性",
        default_file: "advanced-lab/src/lib.rs",
        lesson_topic: "d4",
        function: "unsafe 边界、关联类型、宏",
        concept: "原始指针、trait、类型别名、函数指针和过程宏。",
        prediction: "安全包装函数中的前置条件由谁保证？改变宏输入后哪一步产生编译错误？",
        experiment: "运行 unsafe 或 macros，改变一个安全包装前置条件或宏调用；读实际诊断后修复，保留前后证据。",
        reflection: "给自己的高级接口写下安全不变量、调用方义务和测试边界，解释为何不用更简单的接口。",
        source_dirs: &["advanced-lab/src", "d4-proc-macros/src"],
        run: Some(RunTarget { package: "rustapplab-book-d4-lab", binary: "book_d4", args: &["macros"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/book-d4-advanced-features/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/book-d4-advanced-features/Cargo.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/book-d4-advanced-features/README.md") },
            EmbeddedFile { path: "advanced-lab/Cargo.toml", content: include_str!("../../../../projects/book-d4-advanced-features/advanced-lab/Cargo.toml") },
            EmbeddedFile { path: "advanced-lab/src/lib.rs", content: include_str!("../../../../projects/book-d4-advanced-features/advanced-lab/src/lib.rs") },
            EmbeddedFile { path: "advanced-lab/src/main.rs", content: include_str!("../../../../projects/book-d4-advanced-features/advanced-lab/src/main.rs") },
            EmbeddedFile { path: "advanced-lab/tests/cli.rs", content: include_str!("../../../../projects/book-d4-advanced-features/advanced-lab/tests/cli.rs") },
            EmbeddedFile { path: "d4-proc-macros/Cargo.toml", content: include_str!("../../../../projects/book-d4-advanced-features/d4-proc-macros/Cargo.toml") },
            EmbeddedFile { path: "d4-proc-macros/src/lib.rs", content: include_str!("../../../../projects/book-d4-advanced-features/d4-proc-macros/src/lib.rs") },
        ],
    },
    BookCourse {
        id: "book-d5",
        reference_dir: "book-d5-webserver",
        title: "Book D5 · 有界线程池 Web 服务",
        default_file: "webserver-lab/src/lib.rs",
        lesson_topic: "d5",
        function: "ThreadPool、请求解析与 Drop",
        concept: "任务通道、阻塞、线程回收与有限服务退出。",
        prediction: "发送端关闭后工作线程会怎样退出？错误请求和正常请求是否消耗相同的请求额度？",
        experiment: "编辑线程池或请求解析的一处行为，点击检查运行有限请求的真实集成测试；根据失败断言修复。本页不启动常驻 HTTP 服务。",
        reflection: "在自己的服务说明线程上限、请求上限和退出路径，区分吞吐量、等待与资源回收。",
        source_dirs: &["webserver-lab/src"],
        run: None,
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/book-d5-webserver/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/book-d5-webserver/Cargo.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/book-d5-webserver/README.md") },
            EmbeddedFile { path: "webserver-lab/Cargo.toml", content: include_str!("../../../../projects/book-d5-webserver/webserver-lab/Cargo.toml") },
            EmbeddedFile { path: "webserver-lab/samples/index.html", content: include_str!("../../../../projects/book-d5-webserver/webserver-lab/samples/index.html") },
            EmbeddedFile { path: "webserver-lab/src/lib.rs", content: include_str!("../../../../projects/book-d5-webserver/webserver-lab/src/lib.rs") },
            EmbeddedFile { path: "webserver-lab/src/main.rs", content: include_str!("../../../../projects/book-d5-webserver/webserver-lab/src/main.rs") },
            EmbeddedFile { path: "webserver-lab/tests/cli.rs", content: include_str!("../../../../projects/book-d5-webserver/webserver-lab/tests/cli.rs") },
        ],
    },
    BookCourse {
        id: "book-d6",
        reference_dir: "book-d6-appendices",
        title: "Book D6-A · 关键字、工具与 Edition",
        default_file: "appendix-lab/src/lib.rs",
        lesson_topic: "d6a",
        function: "raw identifier、derive 与工具",
        concept: "关键字、标准 trait、编译期边界和稳定工具链。",
        prediction: "去掉关键字标识符前的 r# 后会出现什么诊断？Debug 与 Display 的输出是否相同？",
        experiment: "在自己的副本修改 raw identifier 或 derive 示例，运行真实单测与文档测试；记录失败和修复。工具链与 Edition 迁移按本地讲义在个人副本进行。",
        reflection: "为自己的项目写一条编译器诊断到文档的查询记录，明确 Edition 与 rustc 版本的区别。",
        source_dirs: &["appendix-lab/src"],
        run: None,
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/book-d6-appendices/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/book-d6-appendices/Cargo.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/book-d6-appendices/README.md") },
            EmbeddedFile { path: "appendix-lab/Cargo.toml", content: include_str!("../../../../projects/book-d6-appendices/appendix-lab/Cargo.toml") },
            EmbeddedFile { path: "appendix-lab/src/lib.rs", content: include_str!("../../../../projects/book-d6-appendices/appendix-lab/src/lib.rs") },
            EmbeddedFile { path: "appendix-lab/tests/contracts.rs", content: include_str!("../../../../projects/book-d6-appendices/appendix-lab/tests/contracts.rs") },
        ],
    },
    BookCourse {
        id: "book-foundations",
        reference_dir: "book-d6-foundations",
        title: "Book D6-B · 基础概念实验",
        default_file: "foundation-lab/src/lib.rs",
        lesson_topic: "d6b",
        function: "所有权、集合、泛型与错误路径",
        concept: "基础语法、借用、迭代器、生命周期、测试和 panic。",
        prediction: "只改变输入而不改函数，数组索引、map/zip 和借用结果会怎样变化？哪一种失败发生在编译期？",
        experiment: "选一项基础概念，写下预测并编辑对应源码；检查单测、文档示例和真实失败探针，修复后把原因记入学习记录。",
        reflection: "把实验迁移到自己的文本统计、任务卡或配置读取工具，记录一项新输入与一项错误输入。",
        source_dirs: &["foundation-lab/examples", "foundation-lab/src"],
        run: Some(RunTarget { package: "book-d6-foundation-lab", binary: "foundation-lab", args: &["hello"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/book-d6-foundations/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/book-d6-foundations/Cargo.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/book-d6-foundations/README.md") },
            EmbeddedFile { path: "foundation-lab/Cargo.toml", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/Cargo.toml") },
            EmbeddedFile { path: "foundation-lab/examples/panic_strategy_probe.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/examples/panic_strategy_probe.rs") },
            EmbeddedFile { path: "foundation-lab/tests/concept_repairs.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/tests/concept_repairs.rs") },
            EmbeddedFile { path: "foundation-lab/src/concept_repairs.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/concept_repairs.rs") },
            EmbeddedFile { path: "foundation-lab/examples/panic_backtrace_probe.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/examples/panic_backtrace_probe.rs") },
            EmbeddedFile { path: "foundation-lab/src/book_gap4.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/book_gap4.rs") },
            EmbeddedFile { path: "foundation-lab/src/collections.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/collections.rs") },
            EmbeddedFile { path: "foundation-lab/src/generic_lifetimes.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/generic_lifetimes.rs") },
            EmbeddedFile { path: "foundation-lab/src/language_basics/catalog.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/language_basics/catalog.rs") },
            EmbeddedFile { path: "foundation-lab/src/language_basics.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/language_basics.rs") },
            EmbeddedFile { path: "foundation-lab/src/lib.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/lib.rs") },
            EmbeddedFile { path: "foundation-lab/src/main.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/main.rs") },
            EmbeddedFile { path: "foundation-lab/src/ownership.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/src/ownership.rs") },
            EmbeddedFile { path: "foundation-lab/tests/cli.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/tests/cli.rs") },
            EmbeddedFile { path: "foundation-lab/tests/collections.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/tests/collections.rs") },
            EmbeddedFile { path: "foundation-lab/tests/compound_types.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/tests/compound_types.rs") },
            EmbeddedFile { path: "foundation-lab/tests/generic_lifetimes.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/tests/generic_lifetimes.rs") },
            EmbeddedFile { path: "foundation-lab/tests/language_basics.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/tests/language_basics.rs") },
            EmbeddedFile { path: "foundation-lab/tests/ownership.rs", content: include_str!("../../../../projects/book-d6-foundations/foundation-lab/tests/ownership.rs") },
        ],
    },
];

// Only learner templates are embedded; never embed reference solutions.
#[rustfmt::skip]
pub(super) const MODERN_COURSES: &[BookCourse] = &[
    BookCourse {
        id: "modern-d8",
        reference_dir: "modern-d8-edition",
        title: "D8 · Edition 2024 告警筛选",
        default_file: "src/lib.rs",
        lesson_topic: "d8",
        function: "src/lib.rs",
        concept: "Edition 2024 与旧版本规则、let 链与宏分派。",
        prediction: "读数 45、阈值 40 与读数缺失时分别输出什么？",
        experiment: "先检查当前 TODO，再修改并重验；最终运行 alarm，试负数、缺失及非法输入。",
        reflection: "解释 Edition、编译器版本和 resolver 的区别。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "alarm", args: &["45", "40"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-d8-edition/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-d8-edition/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-d8-edition/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-d8-edition/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-d8-edition/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-d8-edition/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-d8-edition/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/alarm.rs", content: include_str!("../../../../projects/modern-d8-edition/learner/examples/alarm.rs") },
        ],
    },
    BookCourse {
        id: "modern-d9a",
        reference_dir: "modern-d9-stable",
        title: "D9-A · 现代语言特性",
        default_file: "src/lib.rs",
        lesson_topic: "d9a",
        function: "src/lib.rs",
        concept: "稳定语言增量、借用、异步闭包、不可失败类型和资源管理。",
        prediction: "窗口滑动和最小值返回是否持有原输入借用？解析失败后原数据应怎样？",
        experiment: "按编号检查对应合同；完成后尝试 batch 输入 7 bad 3。",
        reflection: "解释所用新语言特性解决的旧写法问题，不能只以测试通过证明采用指定语法。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "batch", args: &["7", "0", "3"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-d9-stable/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-d9-stable/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-d9-stable/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-d9-stable/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-d9-stable/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-d9-stable/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-d9-stable/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/batch.rs", content: include_str!("../../../../projects/modern-d9-stable/learner/examples/batch.rs") },
        ],
    },
    BookCourse {
        id: "modern-d9b",
        reference_dir: "modern-d9-stdlib",
        title: "D9-B · 现代标准库",
        default_file: "src/lib.rs",
        lesson_topic: "d9b",
        function: "src/lib.rs",
        concept: "逐版稳定标准库 API、类型边界与不必要的复制。",
        prediction: "空查询是否初始化缓存？极值差和偏差修正会不会溢出？",
        experiment: "每次完成对应 TODO；第 8 步包含 8b。最终使用自己的读数运行 telemetry。",
        reflection: "解释选择每个新 API 的边界及失败路径；Cargo 专项按讲义另验。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "telemetry", args: &["7", "bad", "120", "0"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-d9-stdlib/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-d9-stdlib/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-d9-stdlib/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-d9-stdlib/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-d9-stdlib/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-d9-stdlib/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-d9-stdlib/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/telemetry.rs", content: include_str!("../../../../projects/modern-d9-stdlib/learner/examples/telemetry.rs") },
        ],
    },
    BookCourse {
        id: "modern-d10a",
        reference_dir: "modern-d10-async-core",
        title: "D10-A · 异步核心",
        default_file: "src/lib.rs",
        lesson_topic: "d10a",
        function: "src/lib.rs",
        concept: "Future、Poll、Waker、Pin 与协作式调度。",
        prediction: "哪个 Future 先 Ready？完成后能否再次 poll？竞争失败的一方如何释放？",
        experiment: "按编号完成合同并运行异步示例；短时间可信 poll 才适用此教学 executor。",
        reflection: "解释唤醒、取消与不忙轮询的原因；不把教学 executor 当作生产 Tokio。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "async_workbench", args: &[] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-d10-async-core/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-d10-async-core/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-d10-async-core/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-d10-async-core/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-d10-async-core/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-d10-async-core/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-d10-async-core/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/async_workbench.rs", content: include_str!("../../../../projects/modern-d10-async-core/learner/examples/async_workbench.rs") },
            EmbeddedFile { path: "src/runtime.rs", content: include_str!("../../../../projects/modern-d10-async-core/learner/src/runtime.rs") },
        ],
    },
    BookCourse {
        id: "modern-d10b",
        reference_dir: "modern-d10-tokio",
        title: "D10-B · Tokio 与 Stream",
        default_file: "src/lib.rs",
        lesson_topic: "d10b",
        function: "src/lib.rs",
        concept: "真实 Tokio、Stream、取消、并发限制。",
        prediction: "接收端关闭时生产者如何停止？",
        experiment: "在离线个人工程完成七步，第四、五步各含子任务，再运行有限示例。",
        reflection: "解释 Pending、Waker、drop JoinHandle 与 abort。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "async_workbench", args: &[] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-d10-tokio/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-d10-tokio/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-d10-tokio/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-d10-tokio/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-d10-tokio/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-d10-tokio/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-d10-tokio/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/async_workbench.rs", content: include_str!("../../../../projects/modern-d10-tokio/learner/examples/async_workbench.rs") },
            EmbeddedFile { path: ".cargo/config.toml", content: r###"[source.crates-io]
replace-with = "d10-vendor"
[source.d10-vendor]
directory = "vendor"
"### },
        ],
    },
    BookCourse {
        id: "modern-d11a",
        reference_dir: "modern-d11-quality",
        title: "D11-A · 工程质量",
        default_file: "src/lib.rs",
        lesson_topic: "d11a",
        function: "src/lib.rs",
        concept: "有界解析、固定合同、属性测试与性能测量。",
        prediction: "中文温度的字节数是多少？帧损坏会在哪个检查点拒绝？",
        experiment: "最后试 frame 的 corrupt 参数；性能测量、Clippy、GDB/IDE 按讲义独立操作。",
        reflection: "说明测试的行为边界与性能实测的局限。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "frame", args: &["温度"] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-d11-quality/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-d11-quality/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-d11-quality/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-d11-quality/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-d11-quality/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-d11-quality/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-d11-quality/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/debug_probe.rs", content: include_str!("../../../../projects/modern-d11-quality/learner/examples/debug_probe.rs") },
            EmbeddedFile { path: "examples/frame.rs", content: include_str!("../../../../projects/modern-d11-quality/learner/examples/frame.rs") },
            EmbeddedFile { path: "examples/measure.rs", content: include_str!("../../../../projects/modern-d11-quality/learner/examples/measure.rs") },
            EmbeddedFile { path: "tests/properties.rs", content: include_str!("../../../../projects/modern-d11-quality/learner/tests/properties.rs") },
        ],
    },
    BookCourse {
        id: "modern-d11b",
        reference_dir: "modern-d11-safety",
        title: "D11-B · 安全接口",
        default_file: "src/lib.rs",
        lesson_topic: "d11b",
        function: "src/lib.rs",
        concept: "安全接口与初始化不变量。",
        prediction: "越界或未初始化时安全接口如何处理？",
        experiment: "完成两个 TODO；Miri、Fuzz、RustSec 继续按讲义独立操作。",
        reflection: "普通测试无崩溃不证明 unsafe 正确。",
        source_dirs: &["src"],
        run: None,
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-d11-safety/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-d11-safety/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-d11-safety/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-d11-safety/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-d11-safety/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-d11-safety/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-d11-safety/learner/tests/contracts.rs") },
        ],
    },
    BookCourse {
        id: "modern-e1a",
        reference_dir: "modern-e1-tls",
        title: "E1-A · HTTPS 与授权",
        default_file: "src/lib.rs",
        lesson_topic: "e1a",
        function: "src/lib.rs",
        concept: "TLS、认证授权与本地 HTTPS。",
        prediction: "缺失凭证和读者写入分别返回什么状态码？",
        experiment: "完成四步并运行有限 HTTPS 演示；常驻服务按讲义另启。",
        reflection: "区分 TLS 与授权，不使用真实凭证。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "local_demo", args: &[] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-e1-tls/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: r###"[package]
name = "e1-secure-lab"
version = "0.1.0"
edition = "2024"
rust-version = "1.100"
publish = false
license = "MIT"

[workspace]
resolver = "3"
exclude = ["support/c2", "support/servicedesk"]

[dependencies]
servicedesk-c2 = { path = "support/c2" }
axum = { version = "=0.8.8", default-features = false, features = ["http1", "tokio", "json"] }
tokio = { version = "=1.53.3", features = ["rt-multi-thread", "macros", "sync", "net", "signal", "time", "io-util"] }
rustls = { version = "=0.23.46", default-features = false, features = ["ring", "std"] }
tokio-rustls = { version = "=0.26.6", default-features = false, features = ["ring"] }
subtle = "=2.6.1"

[dev-dependencies]
rcgen = { version = "=0.14.10", default-features = false, features = ["ring", "pem"] }
serde_json = "=1.0.145"
"### },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-e1-tls/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-e1-tls/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-e1-tls/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-e1-tls/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-e1-tls/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/local_cert.rs", content: include_str!("../../../../projects/modern-e1-tls/learner/examples/local_cert.rs") },
            EmbeddedFile { path: "examples/local_demo.rs", content: include_str!("../../../../projects/modern-e1-tls/learner/examples/local_demo.rs") },
            EmbeddedFile { path: "src/main.rs", content: include_str!("../../../../projects/modern-e1-tls/learner/src/main.rs") },
            EmbeddedFile { path: "tests/support/mod.rs", content: include_str!("../../../../projects/modern-e1-tls/learner/tests/support/mod.rs") },
            EmbeddedFile { path: ".cargo/config.toml", content: r###"[source.crates-io]
replace-with = "e1-vendor"
[source.e1-vendor]
directory = "vendor"
"### },
            EmbeddedFile { path: "LICENSE", content: include_str!("../../../../LICENSE") },
        ],
    },
    BookCourse {
        id: "modern-e1b",
        reference_dir: "modern-e1-sqlite",
        title: "E1-B · SQLite 与观测",
        default_file: "src/lib.rs",
        lesson_topic: "e1b",
        function: "src/lib.rs",
        concept: "SQLite 事务、并发、有界队列和脱敏。",
        prediction: "audit 失败时业务是否回滚？",
        experiment: "运行有限演示并重开实际数据库。",
        reflection: "区分事务、审计与访问日志。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "local_demo", args: &[] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-e1-sqlite/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: r###"[package]
name = "e1-sqlite-lab"
version = "0.1.0"
edition = "2024"
rust-version = "1.100"
publish = false
license = "MIT"

[workspace]
resolver = "3"
exclude = ["support/e1a"]

[dependencies]
e1-secure-lab = { path = "support/e1a" }
servicedesk = { path = "support/e1a/support/servicedesk" }
axum = { version = "=0.8.8", default-features = false, features = ["http1", "tokio", "json"] }
tokio = { version = "=1.53.3", features = ["rt-multi-thread", "macros", "sync", "net", "signal", "time", "io-util"] }
rusqlite = { version = "=0.43.0", default-features = false, features = ["bundled"] }
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.145"

[dev-dependencies]
rustls = { version = "=0.23.46", default-features = false, features = ["ring", "std"] }
tokio-rustls = { version = "=0.26.6", default-features = false, features = ["ring"] }
rcgen = { version = "=0.14.10", default-features = false, features = ["ring", "pem"] }
"### },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-e1-sqlite/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-e1-sqlite/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-e1-sqlite/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-e1-sqlite/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-e1-sqlite/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/local_cert.rs", content: include_str!("../../../../projects/modern-e1-sqlite/learner/examples/local_cert.rs") },
            EmbeddedFile { path: "examples/local_demo.rs", content: include_str!("../../../../projects/modern-e1-sqlite/learner/examples/local_demo.rs") },
            EmbeddedFile { path: "src/main.rs", content: include_str!("../../../../projects/modern-e1-sqlite/learner/src/main.rs") },
            EmbeddedFile { path: "tests/support/mod.rs", content: include_str!("../../../../projects/modern-e1-sqlite/learner/tests/support/mod.rs") },
            EmbeddedFile { path: ".cargo/config.toml", content: r###"[source.crates-io]
replace-with = "e1b-vendor"
[source.e1b-vendor]
directory = "vendor"
"### },
            EmbeddedFile { path: "LICENSE", content: include_str!("../../../../LICENSE") },
        ],
    },
    BookCourse {
        id: "modern-e2",
        reference_dir: "modern-e2-gui",
        title: "E2 · SignalDesk 原生 GUI",
        default_file: "src/lib.rs",
        lesson_topic: "e2",
        function: "src/lib.rs",
        concept: "原生 eframe/egui、显式状态与数据曲线。",
        prediction: "筛选是删除还是只显示告警？",
        experiment: "网页编辑 lib.rs/gui.rs，启动原生窗口按讲义在终端执行。",
        reflection: "真实控件测试不等同真实桌面体验。",
        source_dirs: &["src"],
        run: None,
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-e2-gui/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-e2-gui/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-e2-gui/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-e2-gui/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-e2-gui/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-e2-gui/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-e2-gui/learner/tests/contracts.rs") },
            EmbeddedFile { path: "src/gui.rs", content: include_str!("../../../../projects/modern-e2-gui/learner/src/gui.rs") },
            EmbeddedFile { path: "src/main.rs", content: include_str!("../../../../projects/modern-e2-gui/learner/src/main.rs") },
            EmbeddedFile { path: ".cargo/config.toml", content: r###"[source.crates-io]
replace-with = "e2-vendor"
[source.e2-vendor]
directory = "vendor"
"### },
            EmbeddedFile { path: "LICENSE", content: include_str!("../../../../LICENSE") },
        ],
    },
    BookCourse {
        id: "modern-e3",
        reference_dir: "modern-e3-embedded",
        title: "E3 · Embedded Monitor",
        default_file: "src/lib.rs",
        lesson_topic: "e3",
        function: "src/lib.rs",
        concept: "no_std、固定容量、回绕周期与 HAL。",
        prediction: "错过周期如何保留相位？",
        experiment: "网页验证主机合同和 host_demo；ARM/QEMU 按讲义另验。",
        reflection: "主机模拟、模拟 CPU 与物理板必须分开。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "host_demo", args: &[] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-e3-embedded/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-e3-embedded/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-e3-embedded/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-e3-embedded/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-e3-embedded/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-e3-embedded/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-e3-embedded/learner/tests/contracts.rs") },
            EmbeddedFile { path: "build.rs", content: include_str!("../../../../projects/modern-e3-embedded/learner/build.rs") },
            EmbeddedFile { path: "examples/host_demo.rs", content: include_str!("../../../../projects/modern-e3-embedded/learner/examples/host_demo.rs") },
            EmbeddedFile { path: "memory.x", content: include_str!("../../../../projects/modern-e3-embedded/learner/memory.x") },
            EmbeddedFile { path: "src/bin/firmware.rs", content: include_str!("../../../../projects/modern-e3-embedded/learner/src/bin/firmware.rs") },
            EmbeddedFile { path: "src/sim.rs", content: include_str!("../../../../projects/modern-e3-embedded/learner/src/sim.rs") },
            EmbeddedFile { path: ".cargo/config.toml", content: r###"[source.crates-io]
replace-with = "e3-vendor"
[source.e3-vendor]
directory = "vendor"
"### },
            EmbeddedFile { path: "LICENSE", content: include_str!("../../../../LICENSE") },
        ],
    },
    BookCourse {
        id: "modern-e4",
        reference_dir: "modern-e4-interop",
        title: "E4 · C ABI 与 WebAssembly",
        default_file: "src/lib.rs",
        lesson_topic: "e4",
        function: "src/lib.rs",
        concept: "C ABI、定宽类型、调用方内存和 WebAssembly。网页本阶段验证 Rust host 合同。",
        prediction: "阈值相等或输入非法时是否修改调用方输出？",
        experiment: "完成 Rust 合同后，按讲义使用真实 C 编译器与 wasm32 目标运行客户端。",
        reflection: "区分非空指针与有效分配；网页 host 测试不等于完成 C 或 WASM 专项。",
        source_dirs: &["src"],
        run: None,
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-e4-interop/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-e4-interop/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-e4-interop/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-e4-interop/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-e4-interop/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-e4-interop/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-e4-interop/learner/tests/contracts.rs") },
            EmbeddedFile { path: "c/client.c", content: include_str!("../../../../projects/modern-e4-interop/learner/c/client.c") },
            EmbeddedFile { path: "c/signal_bridge.h", content: include_str!("../../../../projects/modern-e4-interop/learner/c/signal_bridge.h") },
            EmbeddedFile { path: "web/app.js", content: include_str!("../../../../projects/modern-e4-interop/learner/web/app.js") },
            EmbeddedFile { path: "web/index.html", content: include_str!("../../../../projects/modern-e4-interop/learner/web/index.html") },
        ],
    },
    BookCourse {
        id: "modern-e5",
        reference_dir: "modern-e5-science",
        title: "E5 · 科学计算与机器学习",
        default_file: "src/lib.rs",
        lesson_topic: "e5",
        function: "src/lib.rs",
        concept: "数值统计、仅训练集标准化、稳定损失、留出与真实性能测量。",
        prediction: "为何事件分组先划分？反转训练标签后预测怎样变化？",
        experiment: "完成四步，再执行全部合同与实际 CPU 推断示例，记录损失和混淆矩阵。",
        reflection: "解释合成数据、留出评估及 CPU 计时的范围，不声称真实飞行诊断能力。",
        source_dirs: &["src"],
        run: Some(RunTarget { package: "", binary: "faultscope", args: &[] }),
        files: &[
            EmbeddedFile { path: "Cargo.lock", content: include_str!("../../../../projects/modern-e5-science/learner/Cargo.lock") },
            EmbeddedFile { path: "Cargo.toml", content: include_str!("../../../../projects/modern-e5-science/learner/Cargo.toml") },
            EmbeddedFile { path: "rust-toolchain.toml", content: include_str!("../../../../projects/modern-e5-science/learner/rust-toolchain.toml") },
            EmbeddedFile { path: "README.md", content: include_str!("../../../../projects/modern-e5-science/learner/README.md") },
            EmbeddedFile { path: "LEARNING_LOG.md", content: include_str!("../../../../projects/modern-e5-science/learner/LEARNING_LOG.md") },
            EmbeddedFile { path: "src/lib.rs", content: include_str!("../../../../projects/modern-e5-science/learner/src/lib.rs") },
            EmbeddedFile { path: "tests/contracts.rs", content: include_str!("../../../../projects/modern-e5-science/learner/tests/contracts.rs") },
            EmbeddedFile { path: "examples/faultscope.rs", content: include_str!("../../../../projects/modern-e5-science/learner/examples/faultscope.rs") },
            EmbeddedFile { path: "src/synthetic.rs", content: include_str!("../../../../projects/modern-e5-science/learner/src/synthetic.rs") },
        ],
    },
];
