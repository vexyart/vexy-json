use vexy_json::parse;

fn main() {
    let test_cases = vec!["+1", "+1.", "+1.0", "+123", "+0", "+0.9"];
    
    for input in test_cases {
        match parse(input) {
            Ok(val) => println!("'{}' => {:?}", input, val),
            Err(e) => println!("'{}' => Error: {:?}", input, e),
        }
    }
}