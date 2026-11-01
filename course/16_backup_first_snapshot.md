# 第 16 关 · 从文件丢失事故开始：亲手备份并恢复

**中心问题**：文件分析、日志分析和本机 Web 页面都解决了“看懂数据”。但如果正在开发的项目目录误删了，能否真正找回原始字节，而不只是告诉你一份文件清单？

## 先用软件，10 分钟完成一次恢复

在项目根目录执行（使用未存在的目录名）：

```sh
cargo run -p safesync -- demo my-sfs-16
cargo run -p safesync -- verify my-sfs-16/backup
cargo run -p safesync -- restore my-sfs-16/backup my-sfs-16/rescue-2
```

打开 `my-sfs-16/source/` 和 `my-sfs-16/rescue-2/`；你应该观察到 **4 个普通文件、3 个子目录**，其中有二进制文件、中文文件名和一个空目录。`verify` 输出 `Verify OK`，且第二次恢复不能覆盖已有 `rescue-2/`。

> 本关只允许操作自己新建的 `my-sfs-16`。**不要用真正的重要资料验证删除与恢复**。想模拟误删，可仅删除本关新建的 `my-sfs-16/source/`，之后重新恢复到一个全新的 `my-sfs-16/rescue-3/`。

## 再读源码，而不是先背文件 API

只看三个入口：

1. `apps/safesync/src/cli.rs`：如何确保 `backup <源目录> <目标>` 的参数固定而不猜测？
2. `apps/safesync/src/tree.rs`：`read_dir` 如何发现普通文件、空目录、符号链接？为什么收集为 `Entry`？
3. `apps/safesync/src/ops.rs`：`backup` 为什么先创建 `.sfs-*.part` 临时目录、复制、比对，最后才 `rename`？

`Entry` 把文件路径与 `Kind::File { bytes, sha256 }` 联系起来；目录也必须有记录，否则空目录会在恢复后消失。Rust 的 `Result`/`?` 负责把文件系统异常一直传播到 CLI，最终由 `main` 转换成非零退出码。

## 一次真实使用情境

在任意自己新建的练习目录中添加 `notes.txt`，执行：

```sh
cargo run -p safesync -- backup my-sfs-16/source my-sfs-16/project-v2
cargo run -p safesync -- verify my-sfs-16/project-v2
```

查看 `project-v2/data/` 和 `project-v2/manifest.sfs`。为什么“备份目录”并不是源目录的简单改名？清单的作用是什么？

**完成条件**：能在不覆盖原目录的条件下恢复全部文件、说明为什么要保留空目录，并实际得到 `Verify OK`。运行：

```sh
cargo test -p safesync --test acceptance real_backup_verify_restore_preserves_bytes_and_empty_dirs -- --exact
```

下一关：[`17_integrity_hash.md`](17_integrity_hash.md)。
