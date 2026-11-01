use unitlab::{celsius_to_fahrenheit, convert, describe, Unit::*};
#[test]
fn step1_freezing() {
    assert!(
        (celsius_to_fahrenheit(0.0) - 32.0).abs() < 1e-9,
        "0 摄氏度应为 32 华氏度"
    );
}
#[test]
fn step1_negative_and_variable() {
    for (c, f) in [(-40.0, -40.0), (100.0, 212.0), (12.5, 54.5)] {
        assert!((celsius_to_fahrenheit(c) - f).abs() < 1e-9);
    }
}
#[test]
fn step2_directions() {
    assert!((convert(212.0, Fahrenheit, Celsius).unwrap() - 100.0).abs() < 1e-9);
    assert_eq!(convert(1500.0, Meter, Kilometer).unwrap(), 1.5);
    assert_eq!(convert(1.5, Kilometer, Meter).unwrap(), 1500.0);
    assert_eq!(convert(7.0, Meter, Meter).unwrap(), 7.0);
}
#[test]
fn step2_invalid_dimension_and_nonfinite() {
    assert!(convert(1.0, Meter, Celsius).is_err());
    for value in [f64::NAN, f64::INFINITY] {
        assert!(convert(value, Celsius, Celsius).is_err());
    }
    assert!(convert(f64::MAX, Kilometer, Meter).is_err());
}
#[test]
fn step3_format_and_rounding() {
    assert_eq!(describe("25", "c", "f").unwrap(), "25.00 c = 77.00 f");
    assert_eq!(describe("1555", "m", "km").unwrap(), "1555.00 m = 1.55 km");
}
#[test]
fn step3_input_errors() {
    for args in [
        ("abc", "c", "f"),
        ("1", "kg", "m"),
        ("1", "m", "f"),
        ("NaN", "c", "f"),
    ] {
        assert!(describe(args.0, args.1, args.2).is_err());
    }
}
fn cli(args: &[&std::ffi::OsStr]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_unitlab"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step3_cli_conversion() {
    let o = cli(&[
        "convert".as_ref(),
        "1500".as_ref(),
        "m".as_ref(),
        "km".as_ref(),
    ]);
    assert!(o.status.success());
    assert_eq!(
        String::from_utf8(o.stdout).unwrap().trim(),
        "1500.00 m = 1.50 km"
    );
}
#[test]
fn step3_cli_errors() {
    for a in [
        vec!["convert", "abc", "c", "f"],
        vec!["convert", "1", "m", "f"],
        vec!["unknown"],
    ] {
        let a: Vec<_> = a.iter().map(std::ffi::OsStr::new).collect();
        let o = cli(&a);
        assert!(!o.status.success());
        assert!(!o.stderr.is_empty());
    }
}
