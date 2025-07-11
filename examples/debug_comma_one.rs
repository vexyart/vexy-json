use vexy_json::parse;

fn main() {
    println!("Testing: \",1\"");
    let result = parse(",1");
    println!("Result: {:?}", result);
}
