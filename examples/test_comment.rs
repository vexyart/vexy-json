use vexy_json::{parse, parse_with_options, Error, Lexer, ParserOptions, Token};
use vexy_json_core::lexer::JsonLexer;

fn main() {
    let input = "a#b";
    println!("Parsing input: {input:?}");
    println!();

    // First, let's see what tokens the lexer produces
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    println!("=== Lexer Output ===");
    loop {
        match lexer.next_token() {
            Ok((token, _)) => {
                println!("Token at position {}: {:?}", lexer.position(), token);
                if token == Token::Eof {
                    break;
                }
                tokens.push((token, lexer.position()));
            }
            Err(e) => {
                println!("Lexer error: {e:?}");
                break;
            }
        }
    }

    println!();
    println!("=== Parser Output (default options) ===");

    // Try parsing with default options
    match parse(input) {
        Ok(value) => {
            println!("Successfully parsed: {value:?}");
        }
        Err(e) => {
            println!("Parser error: {e:?}");
            if let Error::Expected {
                expected,
                found,
                position,
            } = &e
            {
                println!("Error details:");
                println!("  - expected: {expected:?}");
                println!("  - found: {found:?}");
                println!("  - position: {position:?}");
            }
            println!("  - error position: {:?}", e.position());
        }
    }

    println!();
    println!("=== Parser Output (with comments enabled) ===");

    // Try with explicit options
    let options = ParserOptions {
        allow_comments: true,
        ..Default::default()
    };

    match parse_with_options(input, options) {
        Ok(value) => {
            println!("Successfully parsed: {value:?}");
        }
        Err(e) => {
            println!("Parser error: {e:?}");
            if let Error::Expected {
                expected,
                found,
                position,
            } = &e
            {
                println!("Error details:");
                println!("  - expected: {expected:?}");
                println!("  - found: {found:?}");
                println!("  - position: {position:?}");
            }
        }
    }

    // Let's also try with debug mode
    println!();
    println!("=== Detailed Token Analysis ===");

    // Create a new lexer to see step by step
    let mut lexer = Lexer::new(input);
    println!("Initial state:");
    println!("  - input: {input:?}");
    println!("  - position: {}", lexer.position());

    // Get first token
    match lexer.next_token() {
        Ok((token, _)) => {
            println!("First token: {:?} at position {}", token, lexer.position());

            // Peek at what's next
            if lexer.position() < input.len() {
                let remaining = &input[lexer.position()..];
                println!("Remaining input: {remaining:?}");
                let next_char = remaining.chars().next();
                println!("Next character: {next_char:?}");

                // Check if it's a comment character
                if next_char == Some('#') {
                    println!("Next character is '#' - should be handled as comment");
                }
            }

            // Continue getting tokens
            println!("\nSubsequent tokens:");
            loop {
                match lexer.next_token() {
                    Ok((token, _)) => {
                        println!("Token at position {}: {:?}", lexer.position(), token);
                        if token == Token::Eof {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Lexer error: {e:?}");
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("Error getting first token: {e:?}");
        }
    }

    // Let's also test what the lexer does with just "a"
    println!();
    println!("=== Control test: parsing just 'a' ===");
    match parse("a") {
        Ok(value) => println!("'a' parsed as: {value:?}"),
        Err(e) => println!("'a' failed: {e:?}"),
    }
}
