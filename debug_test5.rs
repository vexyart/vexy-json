use vexy_json::parse;

fn main() {
    // Test if the # is being treated as a comment
    println!("Testing: '#comment'");
    let result1 = parse("#comment");
    println!("Result 1: {:?}", result1);
    
    println!("\nTesting: 'a:null'");
    let result2 = parse("a:null");
    println!("Result 2: {:?}", result2);
}