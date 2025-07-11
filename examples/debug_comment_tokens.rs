use vexy_json::{Lexer, Token};
use vexy_json_core::lexer::JsonLexer;

fn main() {
    let input = "// comment\n42";
    println!("Input: {:?}", input);
    println!();

    let mut lexer = Lexer::new(input);
    println!("Tokens:");

    loop {
        match lexer.next_token() {
            Ok((token, _)) => {
                println!("  Position {}: {:?}", lexer.position(), token);
                if token == Token::Eof {
                    break;
                }
            }
            Err(e) => {
                println!("  Error at position {}: {:?}", lexer.position(), e);
                break;
            }
        }
    }
}
