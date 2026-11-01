# SafeSync 16–19 分级提示

**规则**：先运行应用、阅读挑战失败输出，再根据需要逐级使用提示。参考思路不代表关卡已经通过。

## 17 · verify --quiet

- **提示 1**：`ops::verify` 已经返回包含文件数、目录数和总字节数的 `Summary`。不要修改它的返回值，也不要跳过校验。
- **提示 2**：`cli::Command::Verify(PathBuf)` 目前只能携带路径。把它改成能携带 `quiet: bool` 的结构化变体，更新所有匹配分支。
- **提示 3**：只有 `quiet == false` 时才 `println!`；错误分支仍由 `main` 输出 stderr，退出码非零。

## 18 · restore --dry-run

- **提示 1**：干运行不能创建恢复目标；不要调用会建临时目录的 `ops::restore`。
- **提示 2**：`ops::verify` 返回 `Summary` 且进行完整哈希校验，可以先调用它。
- **提示 3**：如果仍想严格拒绝已经存在的目标，可以为 `ops` 增加**纯检查**辅助函数，但不要偷偷创建目标或删掉目标目录。

## 19 · verify --json

- **提示 1**：脚本需要明确的成功/失败退出状态，以及稳定字段名，而不是解析中文日志。
- **提示 2**：`Summary` 已包含 files/directories/bytes；新建 JSON 数据模型并不是必需的。
- **提示 3**：示例成功格式：`{"verified":true,"files":1,"directories":0,"bytes":4}`。确保不存在任意来自文件名的未转义插值。

完成每一项后运行对应 `cargo test -p safesync --test challenges mission_XX... -- --ignored --exact`，最后再跑工作区全部普通测试。
