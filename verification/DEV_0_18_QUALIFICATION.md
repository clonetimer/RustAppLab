# Rust AppLab 0.18.0-dev.1 教学迭代验收记录

## 产品定位与范围

在 0.17.0-rc.2 已达 RC_READY 的**冻结**版本之外，创建 0.18.0-dev.1 教学开发分支。新功能为只读学习导航、项目进度与源码摘要比较、常见 Rust 错误第一条解释、Windows 双击入口、真实编译错误的修复实验。已交付的软件数量未变化：17 个 Workspace Package、12 款 Studio、36 个里程碑和两项独立开发作品。

正式发布支持范围调整为 **Windows x64 MSVC 与 Linux x86_64**，macOS 不在承诺范围，不再列为 Stable 阻塞。当前版本尚未获得 Windows 原生测试、Rust 1.74.0 当前源码兼容和真人学员学习效果的完整证据，因此**不是 Stable，也不继承旧版 RC_READY**。

## 真实执行的检查

- Linux x86_64 / Rust 1.99.1 离线工具链。
- `cargo fmt --all -- --check` 通过。
- `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` 通过。
- `cargo test --locked --offline --workspace --lib --bins`：58 通过，0 失败。
- `cargo test --locked --offline --release --workspace --lib --bins`：58 通过，0 失败。
- `cargo test --locked --offline --workspace --doc`：2 通过，0 失败。
- `cargo build --locked --offline --release --workspace`：通过。
- LabQuest `learning::tests`：7 通过（已包含于上述 58），包括 Windows 风格路径、过期报告、两项独立作品分流及错误诊断。
- Python 工具回归 `python -m unittest discover -s scripts/tests -p 'test_*.py'`：16 通过。
- `python scripts/check_error_labs.py`：4 个不同 Rust 程序按预期编译失败（E0382/E0502/E0308/E0597），4 个修复版编译与执行通过。
- CLI 真实学员流程：在中文带空格路径创建 UnitLab → 未完成检查按预期失败 → 只修正第一步 → 当步测试通过 → `learn next` 指示第二步 → 修改后报告失效提示 → E0382 错误解释。输出保存在 `LEARNING_NAVIGATOR_0_18_SMOKE.txt`。
- Studio 已独立执行 `unitlab_red_to_green`、`minikv_red_to_green`、`netfetch_studio_red_green`，3 项分别通过。

## 没有声称已通过的检查

- 两次完整 `--all-targets` / `studio_e2e` 执行受容器时间/构建资源限制中断；后者串行已经输出 25 项 `ok` 后停在最后一个测试，**没有获得整个套件的最终 result**，因此不能记为一次完整通过。相关部分日志保留于 `INCOMPLETE_*.txt`。不据此推断功能失败，也不伪报全部通过。
- Windows x64 实机尚未执行当前 0.18.0-dev.1 的新功能；先前 Windows 0.17 冻结版的真实结果只能证明旧版本。
- 当前开发版未完整复验 Rust 1.74.0 和真实初学者独立使用导航的体验。下一轮需要独立取得证据。

## 发布判定

**DEVELOPMENT_BUILD / NOT_STABLE。** 冻结的 `0.17.0-rc.2 RC_READY` 不变。新分支正式晋级 RC 或 Stable 之前需要当前版本 Windows 原生、声明工具链和教学任务观察的独立记录。
