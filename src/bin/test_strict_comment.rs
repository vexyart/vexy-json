use vexy_json::{parse_with_options, ParserOptions};

fn main() {
    println!("Testing strict parsing of '//comment':");
    
    let mut strict_opts = ParserOptions::default();
    strict_opts.allow_comments = false;
    
    match parse_with_options("//comment", strict_opts) {
        Ok(val) => println!("  Unexpected success: {:?}", val),
        Err(e) => println!("  Expected error: {:?}", e),
    }
}