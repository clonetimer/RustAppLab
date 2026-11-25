#[path = "../../shared.rs"]
mod lesson;

fn main() {
    println!("drop={}", lesson::drop_order().join(","));
    println!("macro={}", lesson::macro_kind());
}

#[cfg(test)]
mod tests {
    use super::lesson;

    #[test]
    fn observes_this_edition() {
        assert_eq!(lesson::drop_order(), ["drop", "else"]);
        assert_eq!(lesson::macro_kind(), "expr");
    }
}
