use std::mem::DropGuard;
use std::panic::{catch_unwind, AssertUnwindSafe};
fn main() {
    let mut values = vec![9];
    { let mut guard = DropGuard::new(&mut values, |v| v.truncate(1)); guard.push(3); }
    assert_eq!(values, [9]);
    let result = catch_unwind(AssertUnwindSafe(|| { let mut guard = DropGuard::new(&mut values, |v| v.truncate(1)); guard.push(5); panic!("controlled unwind"); }));
    assert!(result.is_err()); assert_eq!(values, [9]);
    { let mut guard = DropGuard::new(&mut values, |v| v.truncate(1)); guard.push(7); DropGuard::dismiss(guard); }
    assert_eq!(values, [9, 7]); println!("ok");
}
