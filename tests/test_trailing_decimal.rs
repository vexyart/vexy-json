use vexy_json::{parse, Number, Value};

fn main() {
    let test_cases = vec!["1.", "-1.", "+1.", "0.", "123."];

    for input in test_cases {
        match parse(input) {
            Ok(val) => {
                print!("'{input}' => {val:?}");
                if let Value::Number(n) = &val {
                    match n {
                        Number::Integer(i) => println!(" (Integer {i})"),
                        Number::Float(f) => println!(" (Float {f})"),
                    }
                } else {
                    println!();
                }
            }
            Err(e) => println!("'{input}' => Error: {e:?}"),
        }
    }
}
