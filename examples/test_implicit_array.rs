use vexy_json::parse;

fn main() {
    let test_cases = vec![
        "1, 2, 3",
        "'a', 'b', 'c'",
        "true, false, null",
        "'hello', 123, true",
    ];

    for input in test_cases {
        println!("Input: {input}");
        match parse(input) {
            Ok(value) => println!("  Result: {value:?}\n"),
            Err(e) => println!("  Error: {e:?}\n"),
        }
    }
}
