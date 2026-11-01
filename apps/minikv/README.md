# MiniKV · 本地配置与书签仓库

真实 CLI：put/get/delete/scan/export/demo/share-demo/memory-demo。保存 Unicode 字符串，按键前缀查询，退出后从追加日志恢复。三步教学在 `../../course/studio/minikv.md`；独立骨架通过 `labquest studio start minikv <新目录>` 创建。

```sh
cargo run --offline -p minikv -- demo new-kv-demo
cargo run --offline -p minikv -- get new-kv-demo/my-bookmarks.kv course/next
cargo test --offline -p minikv
cargo test --offline -p minikv --doc
```

架构：main 输入输出 → Database<J> 业务规则 → Journal 内存/文件日志；PrefixScan 返回引用并实现 Iterator。文件头 RUSTAPPLAB_MINIKV_V1，记录 P + 十六进制键/值，或 D + 十六进制键；用 Tab 分隔，LF 终止。十六进制不是加密，也不是完整性认证。

文件锁只用于本应用的协作互斥；正常 Drop 释放，异常结束后保守保留。只在追加成功后更新内存；损坏日志拒绝打开，不自动修复。数据不加密，不保存密码。16 MiB 日志上限；无压缩、事务或断电一致性保证。Windows/macOS 原生复验待补；当前声明 MSRV 1.74 仍待对应工具链验证。
