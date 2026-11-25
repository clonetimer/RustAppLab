struct Reader(Vec<i32>);
impl Reader { fn window(&mut self) -> &[i32] { &self.0 } }
fn main() { let mut r = Reader(vec![1]); let a = r.window(); let b = r.window(); assert_eq!(a,b); }
