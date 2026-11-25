fn main() {
    let data = [10u8, 20];
    assert_eq!(data.get(data.len()), None);
    assert_eq!(data.get(1), Some(&20));
    println!("ok");
}
