use vexy_json::{Lexer, Token};
use vexy_json_core::lexer::JsonLexer;

fn main() {
    let input = "/*a:1*/\nb:2";
    println!("Testing input: {:?}", input);

    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        match lexer.next_token() {
            Ok((Token::Eof, _)) => {
                tokens.push(Token::Eof);
                break;
            }
            Ok((token, _)) => {
                println!("Token: {:?} at position {}", token, lexer.position());
                tokens.push(token);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }

    println!("All tokens: {:?}", tokens);
}
