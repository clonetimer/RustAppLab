use taskcore::{add, parse_tsv, validate_title};

#[test]
fn eighty_unicode_characters_are_valid_in_both_configurations() {
    let title = "测".repeat(80);
    assert_eq!(validate_title(&title).unwrap(), title);
}

#[test]
fn feature_changes_validation_for_all_title_entry_points() {
    let title = "测".repeat(81);
    let mut tasks = Vec::new();
    let row = format!("1\tOPEN\t{title}\n");
    #[cfg(feature = "strict-titles")]
    {
        assert!(validate_title(&title).is_err());
        assert!(add(&mut tasks, &title).is_err());
        assert!(tasks.is_empty());
        assert!(parse_tsv(&row).is_err());
    }
    #[cfg(not(feature = "strict-titles"))]
    {
        assert_eq!(validate_title(&title).unwrap(), title);
        assert_eq!(add(&mut tasks, &title).unwrap(), 1);
        assert_eq!(parse_tsv(&row).unwrap(), tasks);
    }
}
