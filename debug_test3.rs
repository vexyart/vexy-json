use vexy_json::parse;

fn main() {
    let input = "a:#comment\nb:2";
    println!("Testing: '{}'", input);
    println!("Length: {}", input.len());
    for (i, c) in input.char_indices() {
        println!("Position {}: '{}'", i, c);
    }
    
    let result = parse(input);
    println!("Result: {:?}", result);
}