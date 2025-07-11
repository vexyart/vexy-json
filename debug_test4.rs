use vexy_json::parse;

fn main() {
    // Test individual parts
    println!("Testing: 'a:#comment'");
    let result1 = parse("a:#comment");
    println!("Result 1: {:?}", result1);
    
    println!("\nTesting: 'b:2'");
    let result2 = parse("b:2");
    println!("Result 2: {:?}", result2);
    
    println!("\nTesting: 'a:null\nb:2'");
    let result3 = parse("a:null\nb:2");
    println!("Result 3: {:?}", result3);
    
    println!("\nTesting: 'a:#comment\n'");
    let result4 = parse("a:#comment\n");
    println!("Result 4: {:?}", result4);
}