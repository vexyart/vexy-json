use vexy_json::*;

fn main() {
    let input = "a:#comment\nb:2";
    println!("Testing: {:?}", input);
    match parse(input) {
        Ok(value) => println!("Success: {:?}", value),
        Err(e) => println!("Error: {:?}", e),
    }
}