fn main() {
    let test_values = vec!["1.", "1.0", "123.", ".5", "1"];

    for value in test_values {
        match value.parse::<f64>() {
            Ok(f) => println!("'{}' parsed as f64: {}", value, f),
            Err(e) => println!("'{}' failed to parse as f64: {}", value, e),
        }
    }
}
