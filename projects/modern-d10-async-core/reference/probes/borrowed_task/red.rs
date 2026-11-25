fn requires_owned<T: Send + 'static>(_task: T) {}
fn main() {
    let text = String::from("sample");
    requires_owned(async { println!("{text}"); });
}
