fn positive(input: Option<&str>) -> Option<u32> { match input { Some(s) if let Ok(n)=s.parse::<u32>() => Some(n), None=>None } }
fn main() {}
