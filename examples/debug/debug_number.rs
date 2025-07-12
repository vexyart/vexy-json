use vexy_json::parse;

fn main() {
    let test_cases = vec!["1_000_000", "0x10", "0o77", "0b1010"];

    for input in test_cases {
        println!("\nTesting: '{input}'");

        match parse(input) {
            Ok(value) => {
                println!("Parse OK: {value:?}");
            }
            Err(e) => {
                println!("Parse Error: {e:?}");
            }
        }
    }
}
