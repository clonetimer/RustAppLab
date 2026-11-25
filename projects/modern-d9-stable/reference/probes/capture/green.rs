fn summary(values: &[i32]) -> impl Iterator<Item=i32> + use<> { values.iter().copied().min().into_iter() }
fn main() { let mut values = vec![3,1]; let result = summary(&values); values.clear(); assert_eq!(result.collect::<Vec<_>>(), [1]); println!("ok"); }
