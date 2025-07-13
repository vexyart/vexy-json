use vexy_json_core::lazy::parse_lazy;

fn main() {
    let input = r#"[1, 2, 3]"#;
    println!("Input: {}", input);
    println!("Length: {}", input.len());
    
    match parse_lazy(input) {
        Ok(value) => println!("Success: {:?}", value),
        Err(e) => println!("Error: {:?}", e),
    }
}