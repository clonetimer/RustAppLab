//! Learner-owned tests. A smoke PASS is not a completed product.
use std::process::Command;

#[test]
fn missing_arguments_fail() {
    let output = Command::new(env!("CARGO_BIN_EXE_taskpocket"))
        .output().expect("could not launch own binary");
    assert!(!output.status.success(), "missing arguments must be rejected");
    assert!(!output.stderr.is_empty(), "explain the error on stderr");
}

#[test]
#[ignore = "学员作业：编写自己的 Arrange–Act–Assert，再取消 ignore"]
fn my_regression() {
    // Read TEST_PLAN.md. Use a temporary directory, not personal data.
    // Replace this todo! with a meaningful test, not an empty function.
    todo!("定义输入，运行自己的程序，断言退出码/输出/文件字节");
}
