fn main() {
    let test_values = vec!["1.", "1.0", "123.", ".5", "1"];

    for value in test_values {
        match value.parse::<f64>() {
            Ok(f) => println!("'{value}' parsed as f64: {f}"),
            Err(e) => println!("'{value}' failed to parse as f64: {e}"),
        }
    }
}
