use vexy_json::{parse_with_options, ParserOptions};

fn main() {
    let options = ParserOptions {
        allow_comments: true,
        implicit_top_level: true,
        ..Default::default()
    };
    
    println!("Testing: 'a /* comment */ b'");
    let result = parse_with_options("a /* comment */ b", options);
    println!("Result: {:?}", result);
    
    let options2 = ParserOptions {
        allow_comments: true,
        implicit_top_level: true,
        ..Default::default()
    };
    
    println!("\nTesting: 'a b'");
    let result2 = parse_with_options("a b", options2);
    println!("Result: {:?}", result2);
    
    let options3 = ParserOptions {
        allow_comments: true,
        implicit_top_level: true,
        ..Default::default()
    };
    
    println!("\nTesting: '/* comment */'");
    let result3 = parse_with_options("/* comment */", options3);
    println!("Result: {:?}", result3);
}