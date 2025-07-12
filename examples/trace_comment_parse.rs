use vexy_json::{parse_with_options, ParserOptions};

fn main() {
    let input = "// comment\n42";
    println!("Parsing: {input:?}");

    let options = ParserOptions::default();
    println!("Options: allow_comments = {}", options.allow_comments);

    match parse_with_options(input, options) {
        Ok(value) => println!("Success: {value:?}"),
        Err(e) => {
            println!("Error: {e:?}");

            // Show what's at position 0
            if let Some(ch) = input.chars().next() {
                println!("Character at position 0: {ch:?}");
            }
        }
    }
}
