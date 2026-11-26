# E3 · Embedded Monitor：无标准库、HAL 与周期实时任务

[现代化路线](MODERN_RUST_ROADMAP.md) · [实际工程](../projects/modern-e3-embedded/README.md)

本课用真实 Rust 1.100.0、embedded-hal 1.0.0、Cortex-M3 的 thumbv7m-none-eabi 目标。学习同一套 no_std 控制核心在主机测试和裸机固件之间复用；不把桌面线程 sleep 当嵌入式时钟，不把软件模拟引脚当物理 GPIO/ADC。初始完整依赖图须由真实 Cargo 解析、固定数据库 RustSec 审计、保存锁及哈希，两平台同锁验证后冻结。

| 学习任务 | 自己实现 | 行为合同 |
|---|---|---|
| 1 固定内存 | History<N> 的 push/get | 满容量覆盖最旧数据、逻辑顺序、零容量拒绝、无 Vec/allocator |
| 2 周期调度 | Periodic::poll | u32 回绕、原相位、迟到量、错过周期计数、半范围歧义拒绝 |
| 3 滞环控制 | Hysteresis::update | 上阈值接通、下阈值断开、中间保持、非法值不改状态 |
| 4 HAL 与故障锁存 | Controller::step | 真 embedded-hal InputPin/OutputPin trait；模拟传感器异常、联锁断开、输出错误、显式确认后重新检查 |
| 5 综合 | 主机离散 trace 和 Cortex-M 固件 | 同一控制核心、真实交叉编译和真实 QEMU SysTick IRQ，不能仅打印期望输出 |

## 学员操作

安装固定 Rust 1.100.0，取得通过验收的 E3-learner-offline.zip（含锁、vendor 和固定合同，不含解答）。复制到新目录。按 src/lib.rs TODO(1)、TODO(1b)、TODO(2)、TODO(3)、TODO(4) 修改，每次 cargo test --frozen --test contracts step1_（替换编号）；最后 cargo test --frozen --all-targets 和 cargo run --frozen --example host_demo。参考 trace 的输入为四个样本 1500、2100、1800、1400，阈值 1600/2000，输出逻辑为关、开、开、关；主机 tick 9 表示一个已错过周期，最近计划 tick=8。必须从程序实际行为获得结果；不能用固定打印代替算法。

## 调度与错误边界

History 使用 const generic 固定容量数组，RAM 有上界，零容量不会取模零。Periodic 以 wrapping_sub 计算相对时间，要求连续观测不超过 u32 半范围，不允许任意长休眠后猜回绕；迟到时合并到最近已到期周期并记录跳过的次数，而不是补执行所有过去的任务。该策略适用于本例定期监测，不自动适用于所有控制器。

Hysteresis 的两条阈值严格有序并在 12 位 ADC 示例数值范围内，非法值不应被当正常测量。Sensor 是本应用的数值读取接口；embedded-hal 1.0 的数字引脚 trait 是实际外部依赖。SimSensor/SimInput/SimOutput 仅实现同样接口的测试替身。联锁或传感器异常使故障锁存、尝试发送低电平指令；输出写失败返回 Output fault，不声称物理执行器已经安全。确认只清除逻辑锁存，后续仍重新检查联锁和传感器。实物应用还需要硬件失效分析、隔离、回读、看门狗及独立保护。

## ARM 裸机与 QEMU

首次联网安装目标 rustup target add thumbv7m-none-eabi --toolchain 1.100.0。执行 cargo build --frozen --release --target thumbv7m-none-eabi --features firmware --bin firmware。memory.x 只描述 QEMU lm3s6965evb 的 Cortex-M3 教学内存，构建脚本提供 cortex-m-rt 链接脚本。固件使用真实 SysTick exception、AtomicU32 计数和 WFI 等待；semihosting 输出来自实际执行的 ARM ELF。安装 Ubuntu qemu-system-arm 后运行：

    qemu-system-arm -M lm3s6965evb -cpu cortex-m3 -nographic -monitor none -serial none -semihosting-config enable=on,target=native -kernel target/thumbv7m-none-eabi/release/firmware

观察 timer_irq_count、四个 sample 的计划 tick/raw/enabled/missed、模拟联锁后的低指令与 E3_QEMU_PASS。tick 是模拟中断计数，不直接换算物理毫秒，不能用仿真通过声明硬实时、WCET 或实际器件安全。Windows/Linux 均验证真实目标编译；Ubuntu CI 实际运行 QEMU。物理板移植属于后续真人实验，当前不要求购买硬件。

## 验收

四步学员合同分别 RED，临时注入参考实现后五项合同 GREEN；Debug/Release 参考、真实 host_demo、固定锁、vendor 空缓存中文路径搬移、参考和学员修复副本的 ARM ELF 格式/机器号验证，Ubuntu 对两个 ELF 真实 QEMU 运行并核验相位、错过周期和故障锁存。记录依赖审计、完整日志与 ELF 哈希。禁止修改学员原始 ZIP；软件编译通过不等于独立学习效果或真实硬件认证。

参考：[Embedded Rust Book](https://docs.rust-embedded.org/book/) · [embedded-hal 1.0.0](https://docs.rs/embedded-hal/1.0.0/embedded_hal/) · [cortex-m-rt 0.7.5](https://docs.rs/cortex-m-rt/0.7.5/cortex_m_rt/) · [Rust targets](https://doc.rust-lang.org/rustc/platform-support.html)。
