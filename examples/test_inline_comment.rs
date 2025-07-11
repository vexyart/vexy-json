use vexy_json::parse;

fn main() {
    let test_cases = vec![
        ("a#b", "String \"a\""),
        ("a//b", "String \"a\""),
        ("{a:1#b}", "Object with key \"a\" = 1"),
        ("[1,2#comment]", "Array [1, 2]"),
    ];

    for (input, expected_desc) in test_cases {
        println!("Testing: {:?} (expecting {})", input, expected_desc);
        match parse(input) {
            Ok(value) => println!("  ✓ Parsed as: {:?}", value),
            Err(e) => println!("  ✗ Error: {:?}", e),
        }
        println!();
    }
}
