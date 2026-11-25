fn main() { let mut output = Vec::new(); let mut callback = || async { output.push(1); }; let _future = callback(); }
