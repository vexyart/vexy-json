use vexy_json::parse;

fn main() {
    // Test the exact failing case
    let input = "a:#comment\nb:2";
    println!("Testing: '{}'", input.replace('\n', "\\n"));
    for (i, c) in input.char_indices() {
        let char_str = if c == '\n' { "\\n".to_string() } else { c.to_string() };
        println!("  pos {}: '{}'", i, char_str);
    }
    
    let result = parse(input);
    println!("Result: {:?}", result);
    
    // Test a working equivalent
    let input2 = "a:null\nb:2";
    println!("\nTesting equivalent: '{}'", input2.replace('\n', "\\n"));
    let result2 = parse(input2);
    println!("Result 2: {:?}", result2);
}