# C1/C2 网络课程与依赖准备

当前C1已使用Tokio1.48.0/Reqwest0.12.24实现异步下载；C2使用Axum0.8.8/Tokio1.48.0实现工单服务。两者均有实际Cargo生成的锁文件、固定测试及独立vendor包，详见 [C1](ADVANCED_C1.md)、[C2](ADVANCED_C2.md)。基础/A/B仍按各自课程组织，不将网络依赖加入根工作区。

完整平台创建学员工程后，显式运行 `advanced-c1 prepare <工程>` 或 `advanced-c2 prepare <工程>`。首次联网获取固定参考依赖，之后评分locked/offline。`prepare`成功只证明环境准备，不是学员通过。

完全离线使用下载的对应 `learner-offline.zip` 或 `reference-offline.zip`；提前安装Rust1.85.0与系统构建工具。包内隐藏 `.cargo/config.toml`、vendor校验文件不能删除。完整平台固定评分用 `RUSTAPPLAB_C1_VENDOR` / `RUSTAPPLAB_C2_VENDOR` 指向该包vendor。

旧 `ecosystem init/prepare` 及 `PREPARE_NETWORK_WINDOWS.cmd` 保留为依赖探针入口，不是C1/C2学习的前置条件，也不是新增课程。探针的版本组合以其自身清单为准，不冒充当前课程清单。普通学员按课程prepare或独立vendor包即可，不必重复运行旧探针。

原NetFetch/ServiceDesk仍为标准库同步网络教学，AsyncBoard是教学Future执行器；和C1/C2用于对照学习，互不替代。所有服务仅限本机回环HTTP。
