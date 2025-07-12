use vexy_json::parse;

fn main() {
    let test_cases = vec![
        ("1:a", r#"{"1": "a"}"#),
        ("a:1", r#"{"a": 1}"#),
        ("{1:a}", r#"{"1": "a"}"#),
        ("{a:1}", r#"{"a": 1}"#),
        ("1:2", r#"{"1": 2}"#),
        ("1.5:test", r#"{"1.5": "test"}"#),
    ];

    for (input, expected_desc) in test_cases {
        println!("Testing: {input:?} (expecting {expected_desc})");
        match parse(input) {
            Ok(value) => println!("  ✓ Parsed as: {value:?}"),
            Err(e) => println!("  ✗ Error: {e:?}"),
        }
        println!();
    }
}
