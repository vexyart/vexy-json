use vexy_json::{parse, Value};

fn main() {
    let inputs = vec![
        "// comment\na:1",
        "// comment\r\na:1",
        "// comment\ra:1",
        "# comment\na:1",
        "# comment\r\na:1",
    ];
    
    for input in inputs {
        println!("Testing: {}", input.escape_debug());
        match parse(input) {
            Ok(val) => {
                println!("  Result: {:?}", val);
                // Check if it's {"a": 1}
                if let Value::Object(obj) = &val {
                    if let Some(Value::Number(n)) = obj.get("a") {
                        println!("  obj[\"a\"] = {:?}", n);
                    } else {
                        println!("  ERROR: obj[\"a\"] is not a number or doesn't exist!");
                    }
                } else {
                    println!("  ERROR: Result is not an object!");
                }
            },
            Err(e) => println!("  Error: {:?}", e),
        }
        println!();
    }
}