use vexy_json::Lexer;
use vexy_json_core::lexer::JsonLexer;

fn main() {
    let mut lexer = Lexer::new("'hello'");

    println!("Tokenizing: 'hello'");

    match lexer.next_token() {
        Ok((token, _)) => println!("Token: {:?} at position {}", token, lexer.position()),
        Err(e) => println!("Error: {:?}", e),
    }
}
