use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
fn read_book(path: &Path) -> Result<String, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if !meta.is_file() => return Err("账本必须是普通文件，拒绝链接或目录".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(e) => return Err(e.to_string()),
        _ => {}
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}
fn run() -> Result<(), String> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.is_empty() || a == ["help"] || a == ["--help"] {
        println!(
            "LedgerLite: add <账本.tsv> <YYYY-MM-DD> <分类> <金额> | summary <账本.tsv> | demo"
        );
        return Ok(());
    }
    let entries = match a.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["demo"] => ledgerlite::parse_entries(
            "2026-10-28\t餐饮\t12.50\n2026-10-28\t交通\t3.00\n2026-10-29\t餐饮\t0.10\n",
        )?,
        ["summary", file] => ledgerlite::parse_entries(&read_book(Path::new(file))?)?,
        ["add", file, day, category, amount] => {
            // 全部参数和旧记录验证通过后才打开文件写入；仍不提供并发/断电事务保证。
            if day.chars().any(char::is_control)
                || category.chars().any(char::is_control)
                || amount.chars().any(char::is_control)
            {
                return Err("参数不能含制表符或换行".into());
            }
            let proposed = format!("{day}\t{category}\t{amount}\n");
            let record = ledgerlite::parse_entries(&proposed)?.remove(0);
            let old = read_book(Path::new(file))?;
            ledgerlite::parse_entries(&old)?;
            let prefix = if !old.is_empty() && !old.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            let row = format!(
                "{prefix}{}\t{}\t{}\n",
                record.day,
                record.category,
                ledgerlite::money(record.cents)
            );
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(file)
                .and_then(|mut f| f.write_all(row.as_bytes()))
                .map_err(|e| e.to_string())?;
            println!(
                "added={} {} {}",
                record.day,
                record.category,
                ledgerlite::money(record.cents)
            );
            return Ok(());
        }
        _ => return Err("参数不足或命令无效，运行 help 查看用法".into()),
    };
    let mut total = 0u64;
    for (category, cents) in ledgerlite::totals(&entries)? {
        println!("{category}\t{}", ledgerlite::money(cents));
        total = total
            .checked_add(cents)
            .ok_or_else(|| "总额溢出".to_string())?;
    }
    println!("total={}", ledgerlite::money(total));
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("LedgerLite: {e}");
        std::process::exit(2);
    }
}
