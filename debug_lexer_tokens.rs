use vexy_json_core::lexer::{Lexer, JsonLexer};
use vexy_json_core::ast::Token;

fn main() {
    let input = "[1, 2, 3]";
    println!("Lexing: {}", input);
    
    let mut lexer = Lexer::new(input);
    
    loop {
        match lexer.next_token_with_span() {
            Ok((token, span)) => {
                println!("Token: {:?} at {:?} ({})", token, span, &input[span.start..span.end]);
                if token == Token::Eof {
                    break;
                }
            }
            Err(e) => {
                println!("Lexer error: {:?}", e);
                break;
            }
        }
    }
}