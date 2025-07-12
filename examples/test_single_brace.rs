use vexy_json::{parse, Value};

fn main() {
    println!("Testing single opening brace '{{'");

    match parse("{") {
        Ok(value) => {
            println!("Successfully parsed: {value:?}");
            println!(
                "Is it an empty object? {}",
                matches!(value, Value::Object(ref map) if map.is_empty())
            );
        }
        Err(e) => {
            println!("Failed to parse: {e:?}");
        }
    }

    println!("\nTesting empty object '{{}}'");
    match parse("{}") {
        Ok(value) => {
            println!("Successfully parsed: {value:?}");
        }
        Err(e) => {
            println!("Failed to parse: {e:?}");
        }
    }
}
