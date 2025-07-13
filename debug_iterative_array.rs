use vexy_json_core::parser::iterative::parse_iterative;
use vexy_json_core::parser::ParserOptions;

fn main() {
    let input = "[1, 2, 3]";
    println!("Parsing: {}", input);
    
    match parse_iterative(input, ParserOptions::default()) {
        Ok(value) => println!("Success: {:?}", value),
        Err(e) => println!("Error: {:?}", e),
    }
}