# E5 · FaultScope：科学计算、数据分析与入门机器学习

[现代化路线](MODERN_RUST_ROADMAP.md) · [独立教学工程](../projects/modern-e5-science/README.md)

目标：用 Rust 完成一个端到端的小型分类实验，而不是调用第三方模型接口。标准库 CPU f64、固定 Rust 1.100.0，零外部 crate。全部信号是固定算法生成的合成样本，标签为实验设计中的两个类别；没有真实飞行、故障日志、神经网络或 GPU。不能据此宣称实际故障诊断能力。

| 步骤 | 亲手实现 | 检验 |
|---|---|---|
| 1 | Welford 均值/总体标准差 | 已知小样本、大偏置小波动、NaN/非法长度 |
| 2 | 训练集标准化 | 均值、零方差、留出点不重新拟合 |
| 3 | 批量逻辑回归 | stable sigmoid、交叉熵、L2、损失下降、反转标签 |
| 4 | 分类评估 | TP/TN/FP/FN、0.5 边界、非法概率拒绝 |
| 综合 | 分组留出和性能 | 16 训练组 / 8 留出组、实际预测、CPU 推断计时 |

## 统计、数值与训练

每个信号窗口有 64 个合成样本。增量均值 m 与累计平方偏差 M2 的更新为 delta = x - m_old，m_new = m_old + delta/n，M2_new = M2_old + delta*(x-m_new)。总体方差是 M2/n，而不是样本方差 M2/(n-1)。只接受 2..4096 个有限数值，绝对值不超过 1e6。避免直接计算 E[x²]-E[x]² 的严重消减误差。

使用训练特征的均值和标准差做 z-score；标准差小于 1e-9 的维度以 1 替代，不使用留出集重新计算均值/标准差。输入窗口按事件组先划分，再提取特征；一个事件的不同窗口不能同时出现在训练和留出组。固定 train 组 0..15、test 组 100..107，每组 3 个窗口。这个分离不等于真实独立采样；同一合成生成机制仍然共享。

模型为 p = sigmoid(w·x+b)。根据 z 的正负计算 sigmoid 避免 exp 溢出；二元交叉熵用 max(0,z)-y*z+log1p(exp(-abs(z)))。每轮根据训练集的平均 (p-y)*x 更新权重，并加 0.01*w 的 L2 梯度；偏置不正则化。默认 400 轮、学习率 0.1，最多 1000 轮、lr 不超过 0.2。展示真实训练损失而不是保证每步单调或全局最优。改变标签方向的固定测试拒绝硬编码分类规则。

## 实际运行与迁移

在学员自己的目录逐步执行 cargo test --frozen --test contracts step1_ 等命令；全部完成后运行 cargo test --frozen --all-targets 和 cargo run --frozen --release --example faultscope。程序真实拟合模型，输出 CPU 后端、训练/留出组数、窗口数、损失、混淆矩阵、准确率及 48000 次留出特征推断的实际计时和校验和。计时只含推断、不含训练和特征提取；使用 black_box，不比较不同 CPU 的绝对速度，也不承诺固定耗时。留出样本数为 24，阈值固定 0.5；准确率分母不为 0。观察将学习率或训练轮数改变后的损失与留出结果，并解释反转训练标签为何反转预测。不要根据留出结果不断调参再称它是独立最终测试。

技术 CI 验证两平台真实 Rust 编译、参考 Debug/Release、四步初始 RED、修复副本 GREEN、同锁离线搬移、实际训练结果和原始计时证据。普通学员包只含自己的 TODO、合同及合成夹具，不含参考解答。真实学员、外部数据、生产系统和 GPU 均未评估。当前候选未完成 CI。

参考：[Welford 在线方差算法](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm) · [Rust f64](https://doc.rust-lang.org/std/primitive.f64.html) · [Rust black_box](https://doc.rust-lang.org/std/hint/fn.black_box.html)。
