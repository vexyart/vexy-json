use vexy_json::{parse, parse_with_options, Lexer, ParserOptions, Token};

fn debug_string_positions(input: &str) {
    println!("\nString positions in: {input:?}");
    for (i, ch) in input.chars().enumerate() {
        if ch == '"' || ch == '\'' {
            println!("  Position {i}: '{ch}'");
        }
    }
}

fn main() {
    // Test lexer
    let input = r#"{"key": "value"}"#;
    let mut lexer = Lexer::new(input);

    println!("Tokenizing: {input:?}");
    println!("Char positions:");
    for (i, ch) in input.chars().enumerate() {
        println!("  {i}: '{ch}'");
    }

    println!("\nTokens:");
    loop {
        match lexer.next_token_with_span() {
            Ok((token, span)) => {
                let slice = &input[span.start..span.end];
                println!("Token: {token:?}, Span: {span:?}, Slice: {slice:?}");
                if token == Token::Eof {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {e:?}");
                break;
            }
        }
    }

    // Debug string positions
    debug_string_positions(r#"{"key": "value"}"#);

    // Test parser
    println!("\nParsing object:");
    match parse(r#"{"key": "value"}"#) {
        Ok(value) => println!("Success: {value:?}"),
        Err(e) => println!("Error: {e:?}"),
    }

    // Test number parsing
    println!("\nParsing number:");
    match parse("42") {
        Ok(value) => println!("Success: {value:?}"),
        Err(e) => println!("Error: {e:?}"),
    }

    // Test float parsing
    println!("\nParsing float:");
    match parse("3.14") {
        Ok(value) => println!("Success: {value:?}"),
        Err(e) => println!("Error: {e:?}"),
    }

    // Test with strict options
    println!("\nParsing with strict options:");
    let mut options = ParserOptions::default();
    options.implicit_top_level = false;
    match parse_with_options(r#"{"key": "value"}"#, options) {
        Ok(value) => println!("Success: {value:?}"),
        Err(e) => println!("Error: {e:?}"),
    }
}
