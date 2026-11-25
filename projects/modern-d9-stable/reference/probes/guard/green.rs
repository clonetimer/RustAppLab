fn positive(input: Option<&str>) -> Option<u32> { match input { Some(s) if let Ok(n)=s.parse::<u32>() => Some(n), _=>None } }
fn main() { assert_eq!(positive(Some("bad")), None); assert_eq!(positive(Some("7")), Some(7)); println!("ok"); }
