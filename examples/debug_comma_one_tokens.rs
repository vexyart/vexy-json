use vexy_json::{Lexer, Token};
use vexy_json_core::lexer::JsonLexer;

fn main() {
    let input = ",1";
    println!("Testing input: {:?}", input);

    let mut lexer = Lexer::new(input);

    loop {
        match lexer.next_token() {
            Ok((Token::Eof, _)) => {
                println!("Token: Eof at position {}", lexer.position());
                break;
            }
            Ok((token, _)) => {
                println!("Token: {:?} at position {}", token, lexer.position());
            }
            Err(e) => {
                println!("Error: {:?} at position {}", e, lexer.position());
                break;
            }
        }
    }
}
