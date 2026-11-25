pub fn classify(value:u32)->bool {return value>10;}

#[cfg(test)]
mod tests {
    use super::classify;
    #[test]
    fn boundary(){assert!(!classify(10));assert!(classify(11));}
}
