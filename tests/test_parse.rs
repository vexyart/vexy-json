use vexy_json::{parse, parse_with_options, ParserOptions};

fn main() {
    println!("Testing parse('1.'):");
    match parse("1.") {
        Ok(val) => println!("  Success: {val:?}"),
        Err(e) => println!("  Error: {e:?}"),
    }

    println!("\nTesting parse('//comment'):");
    match parse("//comment") {
        Ok(val) => println!("  Success: {val:?}"),
        Err(e) => println!("  Error: {e:?}"),
    }

    println!("\nTesting strict parse('//comment'):");
    let mut strict_opts = ParserOptions::default();
    strict_opts.allow_comments = false;
    match parse_with_options("//comment", strict_opts) {
        Ok(val) => println!("  Success: {val:?}"),
        Err(e) => println!("  Error: {e:?}"),
    }
}
