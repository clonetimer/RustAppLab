# 选修工程实验：把自己的 Rust 作品交付给别人

这不是零基础前置关。完成 FirstCard 后仍应从 UnitLab 开始；完成一款独立作品后再读本节。

## 可观察的目标

将同一份源码交给另一位同学，能够复现构建、运行自己的输入、读懂异常并找到产物。保存 README、源码、Cargo.lock、程序版本、测试记录和自己生成的文件。

## 实验一：为什么“没有编译错误”还不够

先编译 TaskPocket 或 PageMint，再分别尝试合法输入、错误参数、已存在的输出文件、中文空格路径。记录退出码和旧文件是否保持不变。不要用重要数据做失败实验。

## 实验二：一个真实的发布缺陷

旧版部分评测器先等待程序退出，再读取 stdout/stderr。输出管道容量有限，程序输出太多可能被阻塞，最后被误报为死循环。当前实现把两路输出重定向到独占创建的临时文件，边等待边检查时间和输出量。读取完成后删除自己创建的临时文件。

在 `apps/labquest/src/process.rs` 对照 `verbose_stdout_and_stderr_do_not_block_and_files_are_cleaned` 和 `excessive_output_is_not_a_success`。前者应正常完成，后者必须拒绝。两者不是一类错误。

官方依据：Rust `std::process::Child::wait_with_output` 与 `Stdio` 文档描述输出收集与管道阻塞问题：
https://doc.rust-lang.org/std/process/struct.Child.html
https://doc.rust-lang.org/std/process/struct.Stdio.html

## 实验三：用证据做发布判断

Windows 维护者在完整源码根目录双击 `VERIFY_RELEASE_WINDOWS.cmd`；普通学员不必运行。查看 `engineering.json` 中每一项退出码，比较通过结果和 raw log，而不是只看最后一个绿色词。

浏览器组件可以在模拟数据下显示正常，却仍无法真实访问 HTTP。因此浏览器实测与 Rust 单元测试必须分开，缺失时不能填写 PASS。文档中的 `rust-version = "1.74"` 也不等于真实在 1.74 上测试；见 https://doc.rust-lang.org/cargo/reference/rust-version.html 。

## 匿名学员观察表（人工填写，不自动生成通关率）

| 任务 | 不看完整答案能否完成 | 卡住的具体步骤/错误文本 | 使用的提示级别 | 最终真实产物 |
|---|---|---|---|---|
| FirstCard 一行修改 | 待观察 | | | |
| UnitLab 第 1→3 步 | 待观察 | | | |
| 自选 TaskPocket 或 PageMint | 待观察 | | | |

只填写实际观察，不记录学员姓名、联系方式或私有任务内容。若仍存在阻断，记录修复和复验，不能把自动评测器全绿当作教学效果已经得到证明。
