use std::env;
use std::process::ExitCode;

fn respond(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let mut args = args.into_iter();
    let first = args
        .next()
        .ok_or_else(|| "用法：cargo applab-map <profiles|publish|workspace|install|extension>".to_string())?;
    // Cargo may prepend the external subcommand name; a direct binary invocation does not.
    let topic = if first == "applab-map" {
        args.next().ok_or_else(|| "请指定主题".to_string())?
    } else {
        first
    };
    if args.next().is_some() {
        return Err("只能指定一个主题".into());
    }
    rustapplab_book_card::describe(&topic).map_err(str::to_string)
}

fn main() -> ExitCode {
    match respond(env::args().skip(1)) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("Cargo 专题：{message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::respond;

    #[test]
    fn direct_invocation_and_cargo_invocation_have_identical_result() {
        assert_eq!(
            respond(["workspace".to_string()]).unwrap(),
            respond(["applab-map".to_string(), "workspace".to_string()]).unwrap()
        );
    }

    #[test]
    fn invalid_invocations_are_errors() {
        assert!(respond(Vec::<String>::new()).is_err());
        assert!(respond(["applab-map".to_string()]).is_err());
        assert!(respond(["profiles".to_string(), "unexpected".to_string()]).is_err());
        assert!(respond(["invalid".to_string()]).is_err());
    }
}
