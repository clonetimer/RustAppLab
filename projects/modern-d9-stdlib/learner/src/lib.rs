//! D9-B: practical modern standard-library features; no registry dependencies.
use std::ffi::OsStr;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::{LazyLock, RwLock};
use std::time::Duration;

pub fn inspect_readings(readings: &[&str]) -> (Vec<u32>, usize, usize) {
    // TODO(1): 使用 Result::inspect / inspect_err 观察而不改变 Result；保持合法读数顺序。
    let _ = readings;
    (Vec::new(), 0, 0)
}

pub fn calibrate<const N: usize>(values: [u32; N], offsets: [u32; N]) -> [u32; N] {
    // TODO(2): 用 each_mut 借用各元素，饱和加法避免溢出。
    let _ = offsets;
    values
}

pub fn empty_slots<const N: usize>() -> [Option<String>; N] {
    // TODO(3): inline const 可构造非 Copy 元素数组；调用处可用显式推断 const 参数 _。
    todo!()
}

pub fn lookup<F: FnOnce() -> Vec<i32>>(cache: &LazyLock<Vec<i32>, F>, indices: &[usize]) -> Vec<Option<i32>> {
    // TODO(4): 仅真正查询时 force，空输入不触发初始化；越界为 None。
    let _ = (cache, indices);
    Vec::new()
}

pub fn timing_error(actual: Duration, expected: Duration) -> Duration {
    // TODO(5): Duration::abs_diff 避免把较小时间减较大时间而 panic。
    let _ = (actual, expected);
    Duration::ZERO
}

pub fn after_prefix<R: BufRead>(mut input: R) -> io::Result<String> {
    // TODO(6): skip_until 跳过第一个冒号及其前缀；剩余内容原样读取；没有冒号时为空。
    let _ = &mut input;
    Ok(String::new())
}

pub fn c_label(input: &str) -> Option<String> {
    // TODO(7): FromStr for CString 拒绝内嵌 NUL，TryFrom<CString> 转回 String；不操作裸指针。
    let _ = input;
    None
}

pub fn extract_outliers(values: &mut Vec<i32>, limit: i32) -> Vec<i32> {
    // TODO(8): extract_if 将匹配项移走且保持顺序；D9-A 已覆盖 1.86 upcast，另做 get_disjoint_mut 合同。
    let _ = (values, limit);
    Vec::new()
}

pub fn corrected(value: u32, offset: i32) -> Option<u32> {
    // TODO(9): 正偏差应减去，负偏差相当于增加；超出 u32 范围返回 None。
    let _ = (value, offset);
    None
}

pub fn file_label(path: &Path) -> Option<&OsStr> {
    // TODO(10): file_prefix 区分第一个扩展名与最后一个扩展名；保留 OsStr，不强转 UTF-8。
    let _ = path;
    None
}

pub fn update_and_read<R>(lock: &RwLock<Vec<u32>>, value: u32, observe: impl FnOnce(&[u32]) -> R) -> R {
    // TODO(11): 先写入，原子 downgrade，再在读锁保护下观察；不要先 drop 写锁再重新读取。
    let _ = (lock, value, observe);
    todo!()
}

pub fn fixed_pair(values: &[i32]) -> Option<&[i32; 2]> {
    // TODO(12): 长度必须严格等于 2；返回借用而非复制。
    let _ = values;
    None
}

pub fn transitions(values: &[i32]) -> Vec<i64> {
    // TODO(13): array_windows 直接给固定长度借用数组，先转 i64 防止 i32 极值差溢出。
    let _ = values;
    Vec::new()
}

pub fn mask_info(mask: u32) -> (u32, u32) {
    // TODO(14): bit_width 是有效位宽；isolate_lowest_one 提取最低置位；在合同中使用 assert_matches!。
    let _ = mask;
    (0, 0)
}

pub fn token_spans(text: &str) -> Vec<(usize, usize)> {
    // TODO(15): substr_range 定位同一原文中的实际子串而不是搜索文本；返回字节范围。
    let _ = text;
    Vec::new()
}

pub fn consume_boxed<const N: usize>(values: Box<[String; N]>) -> Vec<String> {
    // TODO(16): Box<[T;N]> 的 IntoIterator 消费拥有型元素，不 clone；核对 String 原始分配。
    let _ = values;
    Vec::new()
}

pub fn update_two(values: &mut [i32], first: usize, second: usize) -> bool {
    // TODO(8b): get_disjoint_mut rejects overlap and invalid indices before any change.
    let _ = (values, first, second);
    false
}
