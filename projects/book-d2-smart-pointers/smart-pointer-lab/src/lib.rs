//! Executable examples for Rust Book chapter 15.
//!
//! The six sections deliberately use different ownership mechanisms:
//! Box for indirection, Deref for pointer-like access, Drop for cleanup,
//! Rc for shared immutable ownership, RefCell for runtime borrow checking,
//! and Weak for non-owning graph edges.
//!
//! ```
//! use rustapplab_smart_pointer_lab::{List, MyBox, greet};
//! let list = List::from_slice(&[2, 3, 5]);
//! assert_eq!(list.sum(), 10);
//! assert_eq!(greet(&MyBox::new(String::from("Rust"))), "Hello, Rust!");
//! ```

use std::{
    cell::{Ref, RefCell},
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// 15.1: Box gives a recursive value a fixed-size pointer indirection.
#[derive(Debug, PartialEq, Eq)]
pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    pub fn from_slice(values: &[i32]) -> Self {
        values
            .iter()
            .rev()
            .fold(Self::Nil, |tail, value| Self::Cons(*value, Box::new(tail)))
    }

    pub fn sum(&self) -> i32 {
        match self {
            Self::Cons(value, tail) => value + tail.sum(),
            Self::Nil => 0,
        }
    }
}

pub fn boxed_number(value: i32) -> Box<i32> {
    Box::new(value)
}

pub trait Drawable {
    fn draw(&self) -> String;
}

struct Label(String);

impl Drawable for Label {
    fn draw(&self) -> String {
        format!("[{}]", self.0)
    }
}

/// Box can also own a value whose concrete type is hidden behind a trait object.
pub fn boxed_label(value: impl Into<String>) -> Box<dyn Drawable> {
    Box::new(Label(value.into()))
}

/// 15.2: a small owning wrapper that behaves like a reference through Deref.
pub struct MyBox<T>(T);

impl<T> MyBox<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

/// 15.3: Drop runs when the owner leaves scope or is explicitly dropped.
pub struct DropCounter {
    drops: Arc<AtomicUsize>,
}

impl DropCounter {
    pub fn new(drops: Arc<AtomicUsize>) -> Self {
        Self { drops }
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn drop_once() -> usize {
    let drops = Arc::new(AtomicUsize::new(0));
    let resource = DropCounter::new(Arc::clone(&drops));
    std::mem::drop(resource);
    drops.load(Ordering::SeqCst)
}

/// 15.4: Rc shares one immutable allocation between owners on one thread.
/// Rc is not a cross-thread ownership primitive; use Arc for that case.
pub struct SharedText(Rc<String>);

impl SharedText {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Rc::new(value.into()))
    }

    pub fn clone_shared(&self) -> Self {
        Self(Rc::clone(&self.0))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.0)
    }
}

/// 15.5: RefCell moves borrow-rule enforcement from compile time to runtime.
#[derive(Clone)]
pub struct SharedCounter(Rc<RefCell<i32>>);

impl SharedCounter {
    pub fn new(value: i32) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn clone_shared(&self) -> Self {
        Self(Rc::clone(&self.0))
    }

    pub fn borrow(&self) -> Ref<'_, i32> {
        self.0.borrow()
    }

    pub fn get(&self) -> i32 {
        *self.0.borrow()
    }

    pub fn try_increment(&self) -> Result<(), &'static str> {
        let mut value = self.0.try_borrow_mut().map_err(|_| "已有活跃借用")?;
        *value += 1;
        Ok(())
    }
}

/// 15.6: parent owns children strongly; child points back with Weak.
/// This graph has no strong reference cycle.
pub struct TreeNode {
    name: String,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    pub fn new(name: impl Into<String>) -> Rc<Self> {
        Rc::new(Self {
            name: name.into(),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    pub fn attach(parent: &Rc<Self>, child: Rc<Self>) {
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parent_name(&self) -> Option<String> {
        self.parent
            .borrow()
            .upgrade()
            .map(|parent| parent.name.clone())
    }

    pub fn child_count(&self) -> usize {
        self.children.borrow().len()
    }
}

/// Stable CLI demonstrations used by the lesson and its integration tests.
pub fn demo(topic: &str) -> Result<String, &'static str> {
    match topic {
        "box" => {
            let list = List::from_slice(&[1, 2, 3]);
            let label = boxed_label("owned by Box");
            Ok(format!("sum={}; boxed={}; draw={}", list.sum(), *boxed_number(42), label.draw()))
        }
        "deref" => {
            let name = MyBox::new(String::from("Rust"));
            Ok(greet(&name))
        }
        "drop" => Ok(format!("drops={}", drop_once())),
        "rc" => {
            let first = SharedText::new("shared");
            let second = first.clone_shared();
            Ok(format!("text={}; owners={}", second.as_str(), first.strong_count()))
        }
        "refcell" => {
            let counter = SharedCounter::new(1);
            let held = counter.borrow();
            let conflict_handled = counter.try_increment().is_err();
            std::mem::drop(held);
            counter.try_increment()?;
            Ok(format!("value={}; conflict_handled={conflict_handled}", counter.get()))
        }
        "weak" => {
            let root = TreeNode::new("root");
            let leaf = TreeNode::new("leaf");
            TreeNode::attach(&root, Rc::clone(&leaf));
            let parent_before_drop = leaf.parent_name().unwrap_or_else(|| "none".into());
            std::mem::drop(root);
            let parent_after_drop = leaf.parent_name().unwrap_or_else(|| "none".into());
            Ok(format!(
                "leaf={}; parent_before_drop={}; parent_after_drop={}",
                leaf.name(),
                parent_before_drop,
                parent_after_drop
            ))
        }
        _ => Err("主题只能是 box、deref、drop、rc、refcell、weak"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_builds_a_recursive_list_and_computes_its_value() {
        let list = List::from_slice(&[1, 2, 3, 4]);
        assert_eq!(list.sum(), 10);
        assert_eq!(List::Nil.sum(), 0);
    }

    #[test]
    fn box_owns_a_heap_value() {
        assert_eq!(*boxed_number(42), 42);
    }

    #[test]
    fn box_can_own_a_trait_object() {
        assert_eq!(boxed_label("button").draw(), "[button]");
    }

    #[test]
    fn deref_coercion_allows_custom_box_and_string_as_str() {
        let name = MyBox::new(String::from("Rust"));
        assert_eq!(greet(&name), "Hello, Rust!");
    }

    #[test]
    fn deref_mut_allows_mutating_the_wrapped_value() {
        let mut name = MyBox::new(String::from("Ru"));
        name.push_str("st");
        assert_eq!(&*name, "Rust");
    }

    #[test]
    fn deref_coercion_can_be_observed_without_manual_dereference() {
        let value = MyBox::new(String::from("ownership"));
        let borrowed: &str = &value;
        assert_eq!(borrowed, "ownership");
    }

    #[test]
    fn drop_runs_when_the_owner_leaves_scope() {
        let drops = Arc::new(AtomicUsize::new(0));
        {
            let _resource = DropCounter::new(Arc::clone(&drops));
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn std_mem_drop_runs_the_destructor_at_the_selected_point() {
        assert_eq!(drop_once(), 1);
    }

    #[test]
    fn rc_clone_shares_the_allocation_and_increments_strong_count() {
        let first = SharedText::new("satellite");
        let second = first.clone_shared();
        assert_eq!(first.as_str(), second.as_str());
        assert_eq!(first.strong_count(), 2);
    }

    #[test]
    fn rc_strong_count_decreases_when_an_owner_is_dropped() {
        let first = SharedText::new("telemetry");
        let second = first.clone_shared();
        assert_eq!(first.strong_count(), 2);
        std::mem::drop(second);
        assert_eq!(first.strong_count(), 1);
    }

    #[test]
    fn refcell_allows_multiple_immutable_borrows() {
        let counter = SharedCounter::new(7);
        let first = counter.borrow();
        let second = counter.borrow();
        assert_eq!((*first, *second), (7, 7));
    }

    #[test]
    fn refcell_reports_a_conflicting_mutable_borrow() {
        let counter = SharedCounter::new(3);
        let held = counter.borrow();
        assert!(counter.try_increment().is_err());
        assert_eq!(*held, 3);
        std::mem::drop(held);
        assert_eq!(counter.try_increment(), Ok(()));
        assert_eq!(counter.get(), 4);
    }

    #[test]
    fn rc_refcell_combines_shared_ownership_and_mutation() {
        let first = SharedCounter::new(10);
        let second = first.clone_shared();
        second.try_increment().unwrap();
        assert_eq!(first.get(), 11);
    }

    #[test]
    fn weak_back_reference_can_upgrade_while_parent_is_alive() {
        let parent = TreeNode::new("root");
        let child = TreeNode::new("leaf");
        TreeNode::attach(&parent, Rc::clone(&child));
        assert_eq!(child.parent_name().as_deref(), Some("root"));
        assert_eq!(parent.child_count(), 1);
    }

    #[test]
    fn weak_back_reference_does_not_keep_parent_alive() {
        let parent = TreeNode::new("root");
        let child = TreeNode::new("leaf");
        TreeNode::attach(&parent, Rc::clone(&child));
        std::mem::drop(parent);
        assert_eq!(child.parent_name(), None);
    }

    #[test]
    fn weak_back_edge_does_not_increase_parent_strong_count() {
        let parent = TreeNode::new("root");
        let child = TreeNode::new("leaf");
        assert_eq!(Rc::strong_count(&parent), 1);
        TreeNode::attach(&parent, child);
        assert_eq!(Rc::strong_count(&parent), 1);
    }
}
