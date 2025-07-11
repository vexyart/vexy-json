use vexy_json::parse;

fn main() {
    let input = "'a', 'b', 'c'";
    println!("Parsing: {}", input);

    match parse(input) {
        Ok(result) => println!("Success: {:?}", result),
        Err(e) => println!("Error: {:?}", e),
    }
}
