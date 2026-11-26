# E3 · Embedded Monitor：no_std、HAL 与定时任务

入口：[中文讲义](../../course/BOOK_E3_EMBEDDED.md)。固定 Rust 1.100.0、embedded-hal 1.0.0、Cortex-M3 thumbv7m-none-eabi 与 QEMU lm3s6965evb。四项学员任务（五个 TODO），五项真实合同；无堆分配共享库。实际 ARM 固件使用 SysTick 中断、WFI 和 semihosting；模拟传感器与引脚只是 HAL 测试替身，不是实物板驱动。

首轮真实 Cargo.lock 解析与 RustSec 审计通过，20 个 registry 包，已冻结 SHA-256 `db2e90bc00344d3d33fa853f1a513cbb03780c078d491a08ca932293aa22909a`；保留 bare-metal 0.2.5 未维护警告。首轮双平台/QEMU 专项通过，当前冻结锁及完整集成重验中。Windows/Linux 主机实际测试及 ARM 交叉编译；Ubuntu QEMU 实际运行 ELF 验证中断、调度、滞环与故障锁存。无实物硬件测量或硬实时保证，不声称生产安全。

普通学员使用验收通过的 E3-learner-offline.zip（含 vendor，不含答案）；完整源码的 learner/reference 分离。初始模板未冻结的阶段不能冒充发布包。
