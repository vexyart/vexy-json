use vexy_json::parse;

fn main() {
    println!("Testing double decimal parsing...");
    
    let cases = vec![
        "1.1",
        "1.",
        ".1",
        "1..1",
        "..1",
        ".",
    ];
    
    for case in cases {
        match parse(case) {
            Ok(value) => println!("'{}' -> OK: {:?}", case, value),
            Err(e) => println!("'{}' -> ERROR: {}", case, e),
        }
    }
}