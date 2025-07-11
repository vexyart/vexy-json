use rustc_hash::FxHashMap;
use vexy_json::{parse, parse_with_options, ParserOptions, Value};

fn n(num: i64) -> Value {
    Value::Number(vexy_json::Number::Integer(num))
}

#[test]
fn test_newline_as_comma_arrays() {
    // Test newline as comma in arrays
    let input = "[1\n2\n3]";
    let result = parse(input).unwrap();
    assert_eq!(result, Value::Array(vec![n(1), n(2), n(3)]));
}

#[test]
fn test_newline_as_comma_objects() {
    // Test newline as comma in objects
    let input = "{\"a\": 1\n\"b\": 2}";
    let result = parse(input).unwrap();
    let mut expected = FxHashMap::default();
    expected.insert("a".to_string(), n(1));
    expected.insert("b".to_string(), n(2));
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_newline_as_comma_implicit_array() {
    // Test newline as comma in implicit arrays
    let input = "1\n2\n3";
    let result = parse(input).unwrap();
    assert_eq!(result, Value::Array(vec![n(1), n(2), n(3)]));
}

#[test]
fn test_newline_as_comma_implicit_object() {
    // Test newline as comma in implicit objects
    let input = "a: 1\nb: 2";
    let result = parse(input).unwrap();
    let mut expected = FxHashMap::default();
    expected.insert("a".to_string(), n(1));
    expected.insert("b".to_string(), n(2));
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_newline_as_comma_mixed_with_commas() {
    // Test mixing newlines and commas
    let input = "[1, 2\n3, 4]";
    let result = parse(input).unwrap();
    assert_eq!(result, Value::Array(vec![n(1), n(2), n(3), n(4)]));
}

#[test]
fn test_newline_as_comma_disabled() {
    // Test with newline_as_comma disabled
    let mut options = ParserOptions::default();
    options.newline_as_comma = false;

    let input = "[1\n2\n3]";
    let result = parse_with_options(input, options);
    assert!(result.is_err());
}

#[test]
fn test_newline_as_comma_with_comments() {
    // Test newlines as commas with comments
    let input = "[\n  1  // first\n  2  // second\n  3  // third\n]";
    let result = parse(input).unwrap();
    assert_eq!(result, Value::Array(vec![n(1), n(2), n(3)]));
}

#[test]
fn test_newline_as_comma_nested() {
    // Test newlines as commas in nested structures
    let input = "{\n  arr: [1\n2\n3]\n  obj: {x: 1\ny: 2}\n}";
    let result = parse(input).unwrap();

    let mut inner_obj = FxHashMap::default();
    inner_obj.insert("x".to_string(), n(1));
    inner_obj.insert("y".to_string(), n(2));

    let mut expected = FxHashMap::default();
    expected.insert("arr".to_string(), Value::Array(vec![n(1), n(2), n(3)]));
    expected.insert("obj".to_string(), Value::Object(inner_obj));

    assert_eq!(result, Value::Object(expected));
}
