use vexy_json::*;
fn main() {
    let input = r#""\u12345""#;
    println!("Testing: {:?}", input);
    match parse(input) {
        Ok(value) => println!("Success: {:?}", value),
        Err(e) => println!("Error: {:?}", e),
    }
}
