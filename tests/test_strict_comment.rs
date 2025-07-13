use vexy_json::{parse_with_options, ParserOptions};

fn main() {
    println!("Testing strict parsing of '//comment':");

    let strict_opts = ParserOptions {
        allow_comments: false,
        ..Default::default()
    };

    match parse_with_options("//comment", strict_opts) {
        Ok(val) => println!("  Unexpected success: {val:?}"),
        Err(e) => println!("  Expected error: {e:?}"),
    }
}
