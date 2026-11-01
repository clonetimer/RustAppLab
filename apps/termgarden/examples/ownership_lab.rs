//! Small, deterministic experiments. The deliberate cycle is dismantled before exit.
use std::{cell::RefCell, rc::Rc};
use termgarden::{
    workbench::{EditError, Workspace},
    Garden,
};
fn call_twice<F: Fn() -> usize>(f: F) -> usize {
    f() + f()
}
fn main() {
    let text = String::from("move can still be Fn");
    let expected = text.len() * 2;
    assert_eq!(call_twice(move || text.len()), expected);
    println!("move_capture_can_be_Fn=true");

    let ws = Workspace::new(Garden::default()).unwrap();
    let guard = ws.borrow().unwrap();
    assert_eq!(ws.edit(|_| Ok(())), Err(EditError::Busy));
    drop(guard);
    ws.edit(|_| Ok(())).unwrap();
    println!("read_guard_blocks_mutation_until_dropped=true");

    struct Link {
        next: RefCell<Option<Rc<Link>>>,
    }
    let node = Rc::new(Link {
        next: RefCell::new(None),
    });
    *node.next.borrow_mut() = Some(Rc::clone(&node));
    let observer = Rc::downgrade(&node);
    drop(node);
    // The self-link is a strong owner: dropping our handle did not release the value.
    assert!(observer.upgrade().is_some());
    println!("strong_cycle_survived_external_drop=true");
    {
        let rescued = observer.upgrade().unwrap();
        rescued.next.borrow_mut().take();
    }
    assert!(observer.upgrade().is_none());
    println!("cycle_removed_value_released=true");
}
