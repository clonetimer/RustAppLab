use termgarden::workbench::*;
use termgarden::{storage, BoardView, CompactView, Garden};
fn garden() -> Garden {
    storage::decode("TERMGARDEN\t1\n1\topen\t学习借用\n2\tdone\t写测试\n").unwrap()
}
#[test]
fn a1_initial_snapshot_keeps_data_and_revision() {
    let g = garden();
    let ws = Workspace::new(g.clone()).unwrap();
    assert_eq!(
        ws.snapshot().unwrap(),
        Snapshot {
            revision: 0,
            garden: g
        }
    );
}
#[test]
fn a1_cloned_handles_share_one_document() {
    let a = Workspace::new(garden()).unwrap();
    let b = a.clone();
    let ga = a.borrow().unwrap();
    let gb = b.borrow().unwrap();
    assert!(
        std::ptr::eq(&*ga, &*gb),
        "clone 应复制共享句柄，而不是克隆底层 Garden"
    );
}
#[test]
fn a1_view_is_nonowning_but_an_editor_clone_is_an_owner() {
    let a = Workspace::new(garden()).unwrap();
    let b = a.clone();
    let view = a.view(Box::new(BoardView));
    drop(a);
    assert!(view.draw().unwrap().contains("学习借用"));
    drop(b);
    assert_eq!(view.draw(), Err(EditError::Closed));
}
#[test]
fn a1_different_view_strategies_share_same_document() {
    let a = Workspace::new(garden()).unwrap();
    let board = a.view(Box::new(BoardView));
    let compact = a.view(Box::new(CompactView));
    assert!(board.draw().unwrap().contains("revision=0"));
    assert!(compact.draw().unwrap().contains("1/2 completed"));
}
#[test]
fn a1_snapshot_select_returns_references_from_the_snapshot() {
    let snapshot = Workspace::new(garden()).unwrap().snapshot().unwrap();
    let word = String::from("借用");
    let results = snapshot.select(|t| !t.done && t.title.contains(&word));
    assert_eq!(results.len(), 1);
    assert!(std::ptr::eq(results[0], &snapshot.garden.tasks[0]));
    assert!(snapshot.select(|t| t.id == 500).is_empty());
}
#[test]
fn a1_rejects_broken_initial_state() {
    let mut g = garden();
    g.next_id = 1;
    assert!(Workspace::new(g).is_err());
}
