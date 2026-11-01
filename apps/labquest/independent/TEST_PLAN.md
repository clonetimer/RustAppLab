# 自己写一项测试：不是再等平台告诉你错在哪里

## 先区分三种结果

`cargo check` 证明能通过当前编译检查；`cargo test` 只运行你工程里已有且未忽略的测试；LabQuest checkpoint/app check 用平台探针运行软件。三者覆盖不同。初始 self_check 只有一条入口烟测通过、一条个人作业被忽略，不能算功能通过。

## 用一句话固定行为，再写 Arrange–Act–Assert

在下表填写自己的预期，不要复制一整份参考程序。建议第一次用失败不改写的用例。

| 字段 | 你的记录 |
|---|---|
| 需求编号与风险 | 对照 CONTRACT.md 的哪条；失败会伤害什么 |
| Arrange（准备） | 临时输入文件/目录、初始字节，不操作真实数据 |
| Act（执行） | 调哪个函数或可执行程序，给什么参数 |
| Assert（断言） | stdout、退出码、磁盘字节分别应怎样 |
| 红色结果 | 未修复时实际哪条断言失败 |
| 绿色结果 | 修复后再运行；是否回归其他功能 |

`tests/self_check.rs` 是独立集成测试，可以从外部启动本工程二进制。Cargo 的 `CARGO_BIN_EXE_<名称>` 提供测试用的程序路径，不需要把 `target/debug/...` 写死。类型转换或纯函数可以在模块内写单元测试；不用为了测试把全部函数公开。

## 怎么从模板开始

1. 读 `missing_arguments_fail`，解释为什么要同时断言退出码和 stderr。
2. 打开 `my_regression`，将 todo! 换成自己的准备、调用和断言。
3. 显式运行：`cargo test --offline --test self_check my_regression -- --ignored`。只删除 ignore 但留下 todo! 会失败，这是正确反馈。
4. 测试有效且通过后取消 ignore，运行 `cargo test --offline`。不要在发布时把自己新增的重要测试永久跳过。
5. 把测试名和真实运行记录写入 JOURNAL。表格填完、测试返回 Ok 或自称完成，都不能代替真实断言。

推荐覆盖正常、空输入、边界、非法输入、已存在输出和重复操作。个人测试与平台探针不是对抗关系；平台不读取你如何分层，因此还需要你解释数据模型和模块职责。

## 权限与兼容边界

权限场景要在临时副本或受控用户下实验，不通过修改系统策略、真实文件权限、删除资料来制造错误。Windows 与 Linux 使用各自路径和原生程序；不要声称没有执行过的平台已经通过。性能探索先记文件规模、构建模式和环境，不用一次毫秒数宣称提升。

## 官方参考

- [Rust Book：测试组织](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Cargo：集成测试与二进制目标](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests)
- [标准库：OpenOptions 与 create_new](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html)

这些参考用于理解机制；本作品的具体协议以 CONTRACT.md 为准。阅读资料不是提交完整参考实现的替代。
