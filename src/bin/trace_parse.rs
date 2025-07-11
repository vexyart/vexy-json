use vexy_json_core::parse;

fn main() {
    let input = "1.";
    println!("=== Tracing parse of '{}' ===", input);
    
    // Test using the public parse function
    match parse(input) {
        Ok(value) => {
            println!("Parse succeeded: {:?}", value);
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
    
    // Also test with empty input to see if that's where the issue is
    println!("\n=== Testing empty input ===");
    match parse("") {
        Ok(value) => {
            println!("Empty input succeeded: {:?}", value);
        }
        Err(e) => {
            println!("Empty input failed: {:?}", e);
        }
    }
}