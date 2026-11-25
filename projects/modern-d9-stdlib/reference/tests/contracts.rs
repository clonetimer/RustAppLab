use d9_stdlib_lab::*;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::sync::{LazyLock, RwLock};
use std::time::Duration;

#[test]
fn step1_contract() {
    assert_eq!(inspect_readings(&["7", "bad", "0", "-1", "4294967296"]), (vec![7, 0], 2, 3));
    assert_eq!(inspect_readings(&[]), (vec![], 0, 0));
    assert_eq!(inspect_readings(&["3", "3"]), (vec![3, 3], 2, 0));
}

#[test]
fn step2_contract() {
    assert_eq!(calibrate([2, 5, u32::MAX], [1, 3, 1]), [3, 8, u32::MAX]);
    assert_eq!(calibrate([0], [7]), [7]);
    assert_eq!(calibrate([], []), []);
}

#[test]
fn step3_contract() {
    let slots: [Option<String>; 3] = empty_slots::<_>();
    assert_eq!(slots, [None, None, None]);
    assert_eq!(empty_slots::<0>(), []);
    let mut slots = empty_slots::<2>();
    slots[1] = Some(String::from("温度"));
    assert_eq!(slots[1].as_deref(), Some("温度"));
}

#[test]
fn step4_contract() {
    let calls = std::cell::Cell::new(0);
    let cache = LazyLock::new(|| {
        calls.set(calls.get() + 1);
        vec![10, -2, 7]
    });
    assert!(lookup(&cache, &[]).is_empty());
    assert_eq!(calls.get(), 0);
    assert_eq!(lookup(&cache, &[2, 99, 0]), [Some(7), None, Some(10)]);
    assert_eq!(lookup(&cache, &[1]), [Some(-2)]);
    assert_eq!(calls.get(), 1);
}

#[test]
fn step5_contract() {
    let ms = Duration::from_millis;
    assert_eq!(timing_error(ms(7), ms(10)), ms(3));
    assert_eq!(timing_error(ms(10), ms(7)), ms(3));
    assert_eq!(timing_error(ms(7), ms(7)), Duration::ZERO);
}

#[test]
fn step6_contract() {
    assert_eq!(after_prefix(&b"header:7\n8"[..]).unwrap(), "7\n8");
    assert_eq!(after_prefix(&b":a:b"[..]).unwrap(), "a:b");
    assert_eq!(after_prefix(&b"no delimiter"[..]).unwrap(), "");
    assert_eq!(after_prefix(&b"x:\xff"[..]).unwrap_err().kind(), io::ErrorKind::InvalidData);
}

#[test]
fn step7_contract() {
    assert_eq!(c_label("温度"), Some("温度".to_string()));
    assert_eq!(c_label(""), Some(String::new()));
    assert_eq!(c_label("a\0b"), None);
}

#[test]
fn step8_contract() {
    let mut values = vec![7, -2, 8, 4, 8];
    assert_eq!(extract_outliers(&mut values, 4), [7, 8, 8]);
    assert_eq!(values, [-2, 4]);
    assert!(extract_outliers(&mut values, 9).is_empty());
    assert_eq!(extract_outliers(&mut values, -3), [-2, 4]);
    assert!(values.is_empty());
    let mut paired = [5, 8, i32::MAX];
    assert!(update_two(&mut paired, 0, 2));
    assert_eq!(paired, [6, 8, i32::MAX]);
    assert!(!update_two(&mut paired, 1, 1));
    assert!(!update_two(&mut paired, 0, 9));
    assert_eq!(paired, [6, 8, i32::MAX]);
}

#[test]
fn step9_contract() {
    assert_eq!(corrected(10, 3), Some(7));
    assert_eq!(corrected(10, -3), Some(13));
    assert_eq!(corrected(0, 1), None);
    assert_eq!(corrected(u32::MAX, -1), None);
    assert_eq!(corrected(0, i32::MIN), Some(2_147_483_648));
}

#[test]
fn step10_contract() {
    assert_eq!(file_label(Path::new("archive.tar.gz")), Some(OsStr::new("archive")));
    assert_eq!(file_label(Path::new(".hidden.txt")), Some(OsStr::new(".hidden")));
    assert_eq!(file_label(Path::new("温度.log")), Some(OsStr::new("温度")));
    assert_eq!(file_label(Path::new("..")), None);
}

#[test]
fn step11_contract() {
    let lock = RwLock::new(vec![1]);
    let observed = update_and_read(&lock, 2, |values| {
        assert!(lock.try_write().is_err(), "must retain read protection");
        assert!(lock.try_read().is_ok(), "must not keep a write lock");
        values.to_vec()
    });
    assert_eq!(observed, [1, 2]);
    assert!(lock.try_write().is_ok());
}

#[test]
fn step12_contract() {
    let values = [1, -2];
    let pair = fixed_pair(&values).unwrap();
    assert_eq!(pair, &[1, -2]);
    assert!(std::ptr::eq(pair, &values));
    assert_eq!(fixed_pair(&[]), None);
    assert_eq!(fixed_pair(&[1, 2, 3]), None);
}

#[test]
fn step13_contract() {
    assert_eq!(transitions(&[1, 7, -2]), [6, -9]);
    assert!(transitions(&[]).is_empty());
    assert!(transitions(&[1]).is_empty());
    assert_eq!(transitions(&[i32::MIN, i32::MAX]), [4_294_967_295]);
}

#[test]
fn step14_contract() {
    assert_matches!(mask_info(0), (0, 0));
    assert_matches!(mask_info(0b10100), (5, 4));
    assert_eq!(mask_info(u32::MAX), (32, 1));
    assert_eq!(mask_info(1 << 31), (32, 1 << 31));
}

#[test]
fn step15_contract() {
    let text = "温度 7 温度";
    assert_eq!(token_spans(text), [(0, 6), (7, 8), (9, 15)]);
    let parts: Vec<_> = token_spans(text).into_iter().map(|(a, b)| &text[a..b]).collect();
    assert_eq!(parts, ["温度", "7", "温度"]);
    assert_eq!(token_spans("a a"), [(0, 1), (2, 3)]);
    assert!(token_spans("  ").is_empty());
}

#[test]
fn step16_contract() {
    let values = Box::new([String::from("a"), String::from("温度")]);
    let first = values[0].as_ptr();
    let result = consume_boxed(values);
    assert_eq!(result, ["a", "温度"]);
    assert_eq!(result[0].as_ptr(), first);
    assert!(consume_boxed(Box::new([])).is_empty());
}

