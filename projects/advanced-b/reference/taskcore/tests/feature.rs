#[cfg(feature = "strict-titles")]
#[test]
fn strict_rejects_long_title() {
    assert!(taskcore::validate_title(&"测".repeat(81)).is_err());
}
#[cfg(not(feature = "strict-titles"))]
#[test]
fn default_accepts_long_title() {
    assert!(taskcore::validate_title(&"测".repeat(81)).is_ok());
}
