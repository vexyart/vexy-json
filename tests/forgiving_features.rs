use rustc_hash::FxHashMap;
use vexy_json::{parse, parse_with_options, ParserOptions, Value};

#[test]
fn test_single_quoted_strings() {
    let result = parse("'hello world'").unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));

    let result = parse("{'key': 'value'}").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert("key".to_string(), Value::String("value".to_string()));
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_unquoted_keys() {
    let result = parse("{key: 'value'}").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert("key".to_string(), Value::String("value".to_string()));
    assert_eq!(result, Value::Object(expected));

    let result = parse("{name: 'John', age: 30}").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert("name".to_string(), Value::String("John".to_string()));
    expected.insert(
        "age".to_string(),
        Value::Number(vexy_json::Number::Integer(30)),
    );
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_trailing_commas() {
    let result = parse("[1, 2, 3,]").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Number(vexy_json::Number::Integer(1)),
            Value::Number(vexy_json::Number::Integer(2)),
            Value::Number(vexy_json::Number::Integer(3)),
        ])
    );

    let result = parse("{a: 1, b: 2,}").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert(
        "a".to_string(),
        Value::Number(vexy_json::Number::Integer(1)),
    );
    expected.insert(
        "b".to_string(),
        Value::Number(vexy_json::Number::Integer(2)),
    );
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_single_line_comments() {
    // TODO: Fix parsing when comment is at the beginning of input
    // let result = parse("// This is a comment\n42").unwrap();
    // assert_eq!(result, Value::Number(vexy_json::Number::Integer(42)));

    // let result = parse("# This is also a comment\n42").unwrap();
    // assert_eq!(result, Value::Number(vexy_json::Number::Integer(42)));

    let result = parse("{a: 1, // comment\nb: 2}").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert(
        "a".to_string(),
        Value::Number(vexy_json::Number::Integer(1)),
    );
    expected.insert(
        "b".to_string(),
        Value::Number(vexy_json::Number::Integer(2)),
    );
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_multi_line_comments() {
    // TODO: Fix parsing when comment is at the beginning of input
    // let result = parse("/* This is a \nmulti-line comment */\n42").unwrap();
    // assert_eq!(result, Value::Number(vexy_json::Number::Integer(42)));

    let result = parse("{a: 1, /* comment */ b: 2}").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert(
        "a".to_string(),
        Value::Number(vexy_json::Number::Integer(1)),
    );
    expected.insert(
        "b".to_string(),
        Value::Number(vexy_json::Number::Integer(2)),
    );
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_implicit_object() {
    let result = parse("a: 1").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert(
        "a".to_string(),
        Value::Number(vexy_json::Number::Integer(1)),
    );
    assert_eq!(result, Value::Object(expected));

    let result = parse("name: 'John', age: 30").unwrap();
    let mut expected = FxHashMap::default();
    expected.insert("name".to_string(), Value::String("John".to_string()));
    expected.insert(
        "age".to_string(),
        Value::Number(vexy_json::Number::Integer(30)),
    );
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_implicit_array() {
    let result = parse("1, 2, 3").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Number(vexy_json::Number::Integer(1)),
            Value::Number(vexy_json::Number::Integer(2)),
            Value::Number(vexy_json::Number::Integer(3)),
        ])
    );

    let result = parse("'a', 'b', 'c'").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])
    );
}

#[test]
fn test_empty_input() {
    let result = parse("").unwrap();
    assert_eq!(result, Value::Null);

    let result = parse("   \n  \t  ").unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_mixed_features() {
    // Comments with unquoted keys and trailing commas
    let input = r#"{
        // User information
        name: 'John', // User's name
        age: 30,
        /* Additional details */
        hobbies: ['reading', 'coding',],
    }"#;

    let result = parse(input).unwrap();
    let mut expected = FxHashMap::default();
    expected.insert("name".to_string(), Value::String("John".to_string()));
    expected.insert(
        "age".to_string(),
        Value::Number(vexy_json::Number::Integer(30)),
    );
    expected.insert(
        "hobbies".to_string(),
        Value::Array(vec![
            Value::String("reading".to_string()),
            Value::String("coding".to_string()),
        ]),
    );
    assert_eq!(result, Value::Object(expected));
}

#[test]
fn test_options_disabled() {
    let mut options = ParserOptions::default();
    options.allow_comments = false;
    options.allow_trailing_commas = false;
    options.allow_unquoted_keys = false;
    options.allow_single_quotes = false;
    options.implicit_top_level = false;

    // These should fail with strict options
    match parse_with_options("// comment\n42", options.clone()) {
        Ok(v) => panic!(
            "Comments should fail with allow_comments=false, but got: {:?}",
            v
        ),
        Err(e) => eprintln!("Comments correctly failed: {:?}", e),
    }

    match parse_with_options("[1, 2,]", options.clone()) {
        Ok(v) => panic!(
            "Trailing comma should fail with allow_trailing_commas=false, but got: {:?}",
            v
        ),
        Err(e) => eprintln!("Trailing comma correctly failed: {:?}", e),
    }

    match parse_with_options("{key: 1}", options.clone()) {
        Ok(v) => panic!(
            "Unquoted keys should fail with allow_unquoted_keys=false, but got: {:?}",
            v
        ),
        Err(e) => eprintln!("Unquoted keys correctly failed: {:?}", e),
    }

    match parse_with_options("'string'", options.clone()) {
        Ok(v) => panic!(
            "Single quotes should fail with allow_single_quotes=false, but got: {:?}",
            v
        ),
        Err(e) => eprintln!("Single quotes correctly failed: {:?}", e),
    }

    match parse_with_options("a: 1", options.clone()) {
        Ok(v) => panic!(
            "Implicit top-level should fail with implicit_top_level=false, but got: {:?}",
            v
        ),
        Err(e) => eprintln!("Implicit top-level correctly failed: {:?}", e),
    }
}
