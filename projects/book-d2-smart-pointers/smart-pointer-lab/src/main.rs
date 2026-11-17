use std::{env, process::ExitCode};

fn run(mut args: impl Iterator<Item = String>) -> Result<String, String> {
    let topic = args
        .next()
        .ok_or_else(|| "用法：book-d2 <box|deref|drop|rc|refcell|weak>".to_string())?;
    if args.next().is_some() {
        return Err("一次只能运行一个主题".into());
    }
    rustapplab_smart_pointer_lab::demo(&topic).map_err(str::to_string)
}

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Book D2：{error}");
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
}
