use std::sync::{Arc, Barrier};
use termgarden::workbench::*;
use termgarden::{storage, BoardView, CompactView, Garden, Renderer};
#[test]
fn a3_snapshot_is_send_and_sync() {
    fn require<T: Send + Sync>() {}
    require::<Snapshot>();
}
#[test]
fn a3_generic_and_trait_object_produce_same_result() {
    let s = Snapshot {
        revision: 7,
        garden: Garden::default(),
    };
    let object: Box<dyn Renderer> = Box::new(BoardView);
    assert_eq!(
        render_with(&s, &BoardView),
        render_with(&s, object.as_ref())
    );
    assert!(render_with(&s, &CompactView).contains("revision=7"));
}
#[test]
fn a3_report_is_old_snapshot_while_live_data_changes() {
    struct WaitRenderer(Arc<Barrier>);
    impl Renderer for WaitRenderer {
        fn draw(&self, g: &Garden) -> String {
            self.0.wait();
            termgarden::render(g)
        }
    }
    let g = storage::decode("TERMGARDEN\t1\n1\topen\tbefore\n").unwrap();
    let ws = Workspace::new(g).unwrap();
    let s = Arc::new(ws.snapshot().unwrap());
    let barrier = Arc::new(Barrier::new(2));
    let handle = spawn_report(Arc::clone(&s), WaitRenderer(Arc::clone(&barrier)));
    ws.rename_where(|_| true, |_| "after".into()).unwrap();
    barrier.wait();
    let report = handle.join().unwrap();
    assert!(report.contains("before") && !report.contains("after"));
    assert!(report.contains("revision=0"));
    assert_eq!(ws.snapshot().unwrap().revision, 1);
}
#[test]
fn a3_move_renderer_need_not_be_sync() {
    struct Counter(std::cell::Cell<usize>); // Send, but not Sync: moved to only one worker.
    impl Renderer for Counter {
        fn draw(&self, _: &Garden) -> String {
            self.0.set(self.0.get() + 1);
            format!("count={}", self.0.get())
        }
    }
    let s = Arc::new(Snapshot {
        revision: 4,
        garden: Garden::default(),
    });
    assert!(spawn_report(s, Counter(std::cell::Cell::new(0)))
        .join()
        .unwrap()
        .contains("count=1"));
}
#[test]
fn a3_two_workers_can_share_immutable_data() {
    let s = Arc::new(Snapshot {
        revision: 9,
        garden: Garden::default(),
    });
    let a = spawn_report(Arc::clone(&s), BoardView);
    let b = spawn_report(s, CompactView);
    assert!(a.join().unwrap().contains("revision=9"));
    assert!(b.join().unwrap().contains("revision=9"));
}
