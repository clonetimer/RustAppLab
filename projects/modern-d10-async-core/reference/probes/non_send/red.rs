use std::rc::Rc;
fn requires_send<T: Send>(_task: T) {}
fn main() {
    let local = Rc::new(7);
    requires_send(async move {
        std::future::ready(()).await;
        println!("{local}");
    });
}
