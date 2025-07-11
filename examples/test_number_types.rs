use vexy_json::{parse, Value};

fn main() {
    let test_cases = vec![
        ("0", "Integer"),
        ("0.0", "Float"),
        (".0", "Float"),
        ("1", "Integer"),
        ("1.0", "Float"),
        ("1.", "Float"),
        (".1", "Float"),
        ("1e2", "Float"),
        ("1E2", "Float"),
        ("100", "Integer"),
    ];

    for (input, expected_type) in test_cases {
        print!("Testing {:?} (expecting {}): ", input, expected_type);
        match parse(input) {
            Ok(Value::Number(vexy_json::Number::Integer(i))) => {
                println!("Integer({})", i);
            }
            Ok(Value::Number(vexy_json::Number::Float(f))) => {
                println!("Float({})", f);
            }
            Ok(other) => {
                println!("Unexpected type: {:?}", other);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
