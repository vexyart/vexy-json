use vexy_json_core::{
    lexer::{JsonLexer, Lexer},
    parse,
    parser::{Parser, ParserOptions},
};

fn main() {
    let input = "1.";
    println!("=== Testing full parse of '{input}' ===");

    // First test the lexer
    println!("\n1. Testing lexer:");
    let mut lexer = Lexer::new(input);
    match lexer.next_token() {
        Ok((token, span)) => {
            println!("  Token: {token:?}, Span: {span:?}");
            println!("  Span text: '{}'", &input[span.start..span.end]);
        }
        Err(e) => {
            println!("  Lexer error: {e:?}");
        }
    }

    // Now test the parser
    println!("\n2. Testing parser:");
    match parse(input) {
        Ok(value) => {
            println!("  Parse success: {value:?}");
        }
        Err(e) => {
            println!("  Parse error: {e:?}");
        }
    }

    // Test with explicit parser to see where it fails
    println!("\n3. Testing with explicit parser:");
    let mut parser = Parser::new(input, ParserOptions::default());
    match parser.parse() {
        Ok(value) => {
            println!("  Parse success: {value:?}");
        }
        Err(e) => {
            println!("  Parse error: {e:?}");
        }
    }
}
