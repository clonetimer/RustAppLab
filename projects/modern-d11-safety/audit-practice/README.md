# RustSec 历史依赖审计（只解析锁文件）

这是明确的历史漏洞教学夹具。**不要构建、运行、导入或发布它。** 只在个人临时目录执行 cargo generate-lockfile；读取生成的 Cargo.lock。安装 cargo-audit 0.23.0：

    cargo +1.100.0 install cargo-audit --version 0.23.0 --locked

固定 RustSec Advisory DB 提交 b9b8a3c2cb1f2a3c76ab17a551193568c9f8b7a4，获取数据库后从项目之外的可信目录运行 cargo audit --no-fetch --stale --no-yanked --db <数据库绝对路径> --file <锁文件绝对路径> --json。

预期旧版 regex 1.5.4 命中 RUSTSEC-2022-0013。只修改个人夹具版本为 =1.5.5，重新生成锁，再审计同一数据库，确认该指定公告不再命中；保留其他发现，不能把消除一项公告当成无漏洞。该历史修复版本不代表推荐生产版本。正式项目应使用适用的最新修复版本、最新数据库、可达性/配置评估与真实回归。

本固定快照实验明确禁用额外 registry yanked 检查，亦不自动更新数据库；它不是完整供应链审计。不要对不可信工程执行 Cargo，也不要用 ignore 消除课堂中的真实告警。
