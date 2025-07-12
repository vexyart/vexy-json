use vexy_json::parse;

fn main() {
    // Test the exact failing case
    println!("Testing: 'a:#comment'");
    let result1 = parse("a:#comment");
    println!("Result 1: {:?}", result1);
    
    println!("\nTesting: 'a: #comment'");
    let result2 = parse("a: #comment");
    println!("Result 2: {:?}", result2);
    
    println!("\nTesting: 'a:\\t#comment'");
    let result3 = parse("a:\t#comment");
    println!("Result 3: {:?}", result3);
}