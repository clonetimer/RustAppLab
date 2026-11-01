use taskcore::*;
#[test]
fn basic_roundtrip() {
    let data = "2\tDONE\t继续学习\n1\tOPEN\tWrite Rust\n";
    let t = parse_tsv(data).unwrap();
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].id, 1);
    assert_eq!(
        summary(&t),
        Summary {
            total: 2,
            open: 1,
            done: 1
        }
    );
    assert_eq!(parse_tsv(&to_tsv(&t)).unwrap(), t);
}
#[test]
fn rejects_bad_data() {
    for raw in [
        "0\tOPEN\tbad\n",
        "1\tCLOSED\tx\n",
        "1\tOPEN\tx\n1\tDONE\ty\n",
        "1\tOPEN\n",
        "1\tOPEN\tx\ty\n",
        "\n",
    ] {
        assert!(parse_tsv(raw).is_err(), "accepted {raw:?}")
    }
}
#[test]
fn title_contract() {
    assert_eq!(validate_title("  学 Rust  ").unwrap(), "学 Rust");
    for t in [" ", "a\tb", "a\nb"] {
        assert!(validate_title(t).is_err())
    }
}
#[test]
fn summary_empty() {
    assert_eq!(
        summary(&[]),
        Summary {
            total: 0,
            open: 0,
            done: 0
        }
    )
}
