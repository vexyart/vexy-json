use vexy_json::parse;

fn main() {
    println!("Testing: 'a:1\\nb:2'");
    let result1 = parse("a:1\nb:2");
    println!("Result 1: {:?}", result1);
    
    println!("\nTesting: 'a:null\\nb:2'");
    let result2 = parse("a:null\nb:2");
    println!("Result 2: {:?}", result2);
}