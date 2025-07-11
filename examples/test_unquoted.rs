use vexy_json::{Lexer, Token};
use vexy_json_core::lexer::JsonLexer;

fn main() {
    let mut lexer = Lexer::new("{a: 1}");

    println!("Tokenizing: {{a: 1}}");

    loop {
        match lexer.next_token() {
            Ok((token, _)) => {
                println!("Token: {:?} at position {}", token, lexer.position());
                if token == Token::Eof {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }
}
