use std::{env, process::ExitCode};

fn run(args: impl IntoIterator<Item = String>) -> Result<Vec<String>, String> {
    let mut args = args.into_iter();
    let topic = args
        .next()
        .ok_or_else(|| "用法：book_d3 <patterns|oop|state>".to_string())?;
    if args.next().is_some() {
        return Err("一次只能运行一个主题".into());
    }
    rustapplab_oop_pattern_lab::demo(&topic).map_err(str::to_string)
}

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Book D3：{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn missing_topic_is_an_error() {
        assert!(run(Vec::<String>::new()).is_err());
    }

    #[test]
    fn trailing_arguments_are_rejected() {
        assert!(run(["oop".to_string(), "state".to_string()]).is_err());
    }
}
