use vexy_json_core::{parse, parse_with_options, ParserOptions};

fn main() {
    // Test with default options (implicit top level enabled)
    println!("Default options (implicit top level enabled):");
    match parse("1.") {
        Ok(val) => println!("  Success: {val:?}"),
        Err(e) => println!("  Error: {e:?}"),
    }

    // Test with implicit top level disabled
    println!("\nWith implicit top level disabled:");
    let mut opts = ParserOptions::default();
    opts.implicit_top_level = false;
    match parse_with_options("1.", opts) {
        Ok(val) => println!("  Success: {val:?}"),
        Err(e) => println!("  Error: {e:?}"),
    }

    // Also test a normal number for comparison
    println!("\nParsing '1.0' with default options:");
    match parse("1.0") {
        Ok(val) => println!("  Success: {val:?}"),
        Err(e) => println!("  Error: {e:?}"),
    }
}
