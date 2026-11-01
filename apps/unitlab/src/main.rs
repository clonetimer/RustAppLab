fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["help"] | ["--help"] => {
            println!("UnitLab: convert <数字> <c|f|m|km> <目标单位> | demo")
        }
        ["demo"] => {
            println!("{}", unitlab::describe("25", "c", "f")?);
            println!("{}", unitlab::describe("1500", "m", "km")?);
        }
        ["convert", number, from, to] => println!("{}", unitlab::describe(number, from, to)?),
        _ => return Err("用法：convert 25 c f；所有参数都必须完整".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("UnitLab: {error}");
        std::process::exit(2);
    }
}
