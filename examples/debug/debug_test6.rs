use vexy_json::parse;

fn main() {
    // Test the pattern step by step
    
    println!("Testing just comment: '#comment\\n'");
    let result1 = parse("#comment\n");
    println!("Result 1: {:?}", result1);
    
    println!("\nTesting key-value with explicit null: 'a:null'");
    let result2 = parse("a:null");
    println!("Result 2: {:?}", result2);
    
    // Test what happens when we have a comment at the end of input
    println!("\nTesting comment at end: 'a: #comment'");
    let result3 = parse("a: #comment");
    println!("Result 3: {:?}", result3);
}