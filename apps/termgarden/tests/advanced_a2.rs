use termgarden::workbench::*;
use termgarden::{apply_event, storage, Event, Garden};
fn ws() -> Workspace {
    Workspace::new(storage::decode("TERMGARDEN\t1\n1\topen\tA\n2\topen\tB\n3\tdone\tC\n").unwrap())
        .unwrap()
}
#[test]
fn a2_mutable_capture_numbers_only_selected_items() {
    let ws = ws();
    let mut counter = 0;
    assert_eq!(
        ws.rename_where(
            |t| !t.done,
            |t| {
                counter += 1;
                format!("{counter}:{}", t.title)
            }
        )
        .unwrap(),
        2
    );
    assert_eq!(counter, 2);
    let s = ws.snapshot().unwrap();
    assert_eq!(
        s.garden
            .tasks
            .iter()
            .map(|t| t.title.as_str())
            .collect::<Vec<_>>(),
        ["1:A", "2:B", "C"]
    );
    assert_eq!(s.revision, 1);
}
#[test]
fn a2_partial_invalid_batch_does_not_commit_or_add_undo() {
    let ws = ws();
    let before = ws.snapshot().unwrap();
    assert!(ws
        .rename_where(
            |_| true,
            |t| if t.id == 2 {
                "\n".into()
            } else {
                "changed".into()
            }
        )
        .is_err());
    assert_eq!(ws.snapshot().unwrap(), before);
    assert_eq!(ws.undo(), Err(EditError::NoUndo));
}
#[test]
fn a2_callback_error_discards_draft_but_not_external_side_effects() {
    let ws = ws();
    let before = ws.snapshot().unwrap();
    let mut calls = 0;
    let result = ws.edit(|g| {
        calls += 1;
        g.tasks.clear();
        Err::<(), _>("cancel".into())
    });
    assert!(result.is_err());
    assert_eq!(calls, 1);
    assert_eq!(ws.snapshot().unwrap(), before);
}
#[test]
fn a2_fnonce_consumes_owned_input_without_requiring_clone() {
    struct Payload(String); // no Clone: consuming this makes the closure FnOnce-only.
    let input = Payload("owned".into());
    let ws = ws();
    let size = ws
        .edit(move |g| {
            g.tasks[0].title = input.0;
            Ok(g.tasks.len())
        })
        .unwrap();
    assert_eq!(size, 3);
    assert_eq!(ws.snapshot().unwrap().garden.tasks[0].title, "owned");
}
#[test]
fn a2_read_guard_rejects_edit_without_panicking_or_calling_user_code() {
    let ws = ws();
    let guard = ws.borrow().unwrap();
    let mut called = false;
    assert_eq!(
        ws.edit(|_| {
            called = true;
            Ok(())
        }),
        Err(EditError::Busy)
    );
    assert!(!called);
    drop(guard);
    ws.rename_where(|_| true, |t| format!("new {}", t.title))
        .unwrap();
}
#[test]
fn a2_reentrant_commit_is_not_overwritten_by_old_draft() {
    let ws = ws();
    let nested = ws.clone();
    let result = ws.edit(|g| {
        nested
            .rename_where(|t| t.id == 2, |_| "inner".into())
            .unwrap();
        g.tasks[0].title = "outer".into();
        Ok(())
    });
    assert_eq!(result, Err(EditError::Conflict));
    let s = ws.snapshot().unwrap();
    assert_eq!(s.garden.tasks[0].title, "A");
    assert_eq!(s.garden.tasks[1].title, "inner");
}
#[test]
fn a2_undo_restores_data_but_revision_never_goes_back() {
    let ws = ws();
    let before = ws.snapshot().unwrap();
    ws.rename_where(|t| t.id == 1, |_| "new".into()).unwrap();
    ws.undo().unwrap();
    assert_eq!(ws.snapshot().unwrap().garden, before.garden);
    assert_eq!(ws.snapshot().unwrap().revision, 2);
    assert_eq!(ws.undo(), Err(EditError::NoUndo));
}
#[test]
fn a2_noop_does_not_consume_history() {
    let ws = ws();
    ws.edit(|_| Ok(())).unwrap();
    assert_eq!(ws.snapshot().unwrap().revision, 0);
    assert_eq!(ws.undo(), Err(EditError::NoUndo));
}
#[test]
fn a2_history_is_bounded() {
    let ws = ws();
    for n in 0..HISTORY_LIMIT + 4 {
        ws.rename_where(|t| t.id == 1, |_| format!("item-{n}"))
            .unwrap();
    }
    for _ in 0..HISTORY_LIMIT {
        ws.undo().unwrap();
    }
    assert_eq!(ws.undo(), Err(EditError::NoUndo));
    assert_eq!(ws.snapshot().unwrap().garden.tasks[0].title, "item-3");
}
#[test]
fn a2_original_event_logic_still_works() {
    let ws = Workspace::new(Garden::default()).unwrap();
    ws.edit(|g| apply_event(g, Event::Add("自己的任务".into())))
        .unwrap();
    assert_eq!(ws.snapshot().unwrap().garden.tasks.len(), 1);
}
