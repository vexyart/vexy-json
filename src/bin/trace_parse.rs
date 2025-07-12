use vexy_json_core::parse;

fn main() {
    println!("Testing scientific notation with underscores:");
    
    let tests = [
        ("1_23e2", "underscore in scientific notation"),
        ("1_0e+2", "underscore in scientific notation with +"),
        ("123e2", "regular scientific notation"),
        ("10e+2", "regular scientific notation with +"),
    ];
    
    for (input, description) in tests {
        println!("\nTesting {}: {}", description, input);
        match parse(input) {
            Ok(value) => println!("Success: {:?}", value),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}