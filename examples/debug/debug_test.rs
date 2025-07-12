use vexy_json::parse;

fn main() {
    let result = parse("a /* comment */ b");
    println!("{:?}", result);
}