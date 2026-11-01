fn run() -> Result<(), String> {
    let a: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    if a.is_empty() || a == ["help"] || a == ["--help"] {
        println!("BatchMeter: scan <1..16线程> <UTF-8文件> [更多文件...]");
        return Ok(());
    }
    if a.len() < 3 || a[0] != "scan" {
        return Err("用法：scan 2 samples/one.txt samples/two.txt".into());
    }
    let workers = a[1]
        .to_str()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| "线程数需要整数".to_string())?;
    let paths = a[2..]
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    let rows = batchmeter::analyze_many(&paths, workers)?;
    let total = batchmeter::aggregate(&rows)?;
    for row in rows {
        println!("{}\t{}\t{}", row.path.display(), row.bytes, row.lines);
    }
    println!(
        "files={} bytes={} lines={}",
        total.files, total.bytes, total.lines
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("BatchMeter: {e}");
        std::process::exit(2);
    }
}
