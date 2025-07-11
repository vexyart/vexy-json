use vexy_json::parse;

fn main() {
    // Test if the basic newline-separated object works
    println!("Testing: 'a:1,b:2' (with comma)");
    let result1 = parse("a:1,b:2");
    println!("Result 1: {:?}", result1);
    
    println!("\nTesting: 'a:1\\nb:2' (with newline)");
    let result2 = parse("a:1\nb:2");
    println!("Result 2: {:?}", result2);
    
    // Try the problem case step by step
    println!("\nTesting: 'a:#comment' (just first part)");
    let result3 = parse("a:#comment");
    println!("Result 3: {:?}", result3);
    
    println!("\nTesting: 'a:#comment\\n' (with trailing newline)");
    let result4 = parse("a:#comment\n");
    println!("Result 4: {:?}", result4);
}