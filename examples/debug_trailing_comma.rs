use vexy_json::{parse, Lexer, Token};
use vexy_json_core::lexer::JsonLexer;

fn main() {
    let input = "[1, 2, 3,]";
    println!("Input: '{input}'");
    println!("Input chars:");
    for (i, ch) in input.chars().enumerate() {
        println!("  {i}: '{ch}'");
    }

    // Debug lexer tokens
    let mut lexer = Lexer::new(input);

    loop {
        let pos_before = lexer.position();
        match lexer.next_token() {
            Ok((token, span)) => {
                let pos_after = lexer.position();
                println!("Token: {token:?} positions {pos_before}..{pos_after} (span: {span:?})");
                if token == Token::Eof {
                    break;
                }
            }
            Err(e) => {
                println!("Lexer error: {:?} at position {}", e, lexer.position());
                break;
            }
        }
    }

    println!("\nTrying to parse...");
    match parse(input) {
        Ok(value) => println!("Success: {value:?}"),
        Err(e) => println!("Error: {e:?}"),
    }
}
