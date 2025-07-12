use vexy_json::parse;

fn main() {
    let test_cases = vec![
        ("// comment\n42", "Number 42 after comment"),
        ("/* comment */ 42", "Number 42 after multi-line comment"),
        ("42 // comment", "Number 42 with trailing comment"),
        (
            "42 /* comment */",
            "Number 42 with trailing multi-line comment",
        ),
    ];

    for (input, desc) in test_cases {
        println!("Testing: {input:?} ({desc})");
        match parse(input) {
            Ok(value) => println!("  ✓ Parsed as: {value:?}"),
            Err(e) => println!("  ✗ Error: {e:?}"),
        }
        println!();
    }
}
