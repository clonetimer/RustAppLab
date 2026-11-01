use taskcore::*;
#[test]
fn workflow() {
    let mut t = Vec::new();
    assert_eq!(add(&mut t, " first ").unwrap(), 1);
    assert_eq!(add(&mut t, "第二项").unwrap(), 2);
    complete(&mut t, 1).unwrap();
    complete(&mut t, 1).unwrap();
    assert_eq!(
        summary(&t),
        Summary {
            total: 2,
            open: 1,
            done: 1
        }
    );
    assert_eq!(t[0].title, "first");
}
#[test]
fn rejects_without_mutating() {
    let mut t = vec![Task {
        id: 1,
        done: false,
        title: "valid".into(),
    }];
    let old = t.clone();
    assert!(add(&mut t, "\t").is_err());
    assert_eq!(t, old);
    assert!(complete(&mut t, 999).is_err());
    assert_eq!(t, old);
}
#[test]
fn overflow() {
    let mut t = vec![Task {
        id: u64::MAX,
        done: false,
        title: "last".into(),
    }];
    assert!(add(&mut t, "next").is_err());
    assert_eq!(t.len(), 1)
}
