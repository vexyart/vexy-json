use vexy_json::parse;

fn main() {
    println!("Testing: /*a:1*/\nb:2");
    let result = parse("/*a:1*/\nb:2");
    println!("Result: {result:?}");

    println!("\nTesting: /*a:1*/");
    let result = parse("/*a:1*/");
    println!("Result: {result:?}");

    println!("\nTesting: just spaces and newlines");
    let result = parse("   \n  \t  ");
    println!("Result: {result:?}");
}
