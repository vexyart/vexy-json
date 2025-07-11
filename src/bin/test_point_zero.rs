use vexy_json::{parse, Value, Number};

fn main() {
    let test_cases = vec!["1.0", "-1.0", "0.0", "+1.0"];
    
    for input in test_cases {
        match parse(input) {
            Ok(val) => {
                print!("'{}' => {:?}", input, val);
                if let Value::Number(n) = &val {
                    match n {
                        Number::Integer(i) => println!(" (Integer {})", i),
                        Number::Float(f) => println!(" (Float {})", f),
                    }
                } else {
                    println!();
                }
            },
            Err(e) => println!("'{}' => Error: {:?}", input, e),
        }
    }
}