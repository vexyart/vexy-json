use rustc_hash::FxHashMap;
use vexy_json::{parse, parse_with_options, ParserOptions, Value};

/// Helper to create expected values more easily
fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut map = FxHashMap::default();
    for (k, v) in pairs {
        map.insert(k.to_string(), v.clone());
    }
    Value::Object(map)
}

fn arr(values: Vec<Value>) -> Value {
    Value::Array(values)
}

fn s(text: &str) -> Value {
    Value::String(text.to_string())
}

fn n(num: i64) -> Value {
    Value::Number(vexy_json::Number::Integer(num))
}

fn f(num: f64) -> Value {
    Value::Number(vexy_json::Number::Float(num))
}

#[test]
fn test_happy_path() {
    // Basic tests that our parser supports
    assert_eq!(parse("{a:1}").unwrap(), obj(&[("a", n(1))]));
    assert_eq!(
        parse("{a:1,b:2}").unwrap(),
        obj(&[("a", n(1)), ("b", n(2))])
    );
    assert_eq!(parse("a:1").unwrap(), obj(&[("a", n(1))]));
    assert_eq!(parse("a:1,b:2").unwrap(), obj(&[("a", n(1)), ("b", n(2))]));
    assert_eq!(parse(r#"{"a":1}"#).unwrap(), obj(&[("a", n(1))]));
    assert_eq!(parse("['a']").unwrap(), arr(vec![s("a")]));
    assert_eq!(parse("['a',1]").unwrap(), arr(vec![s("a"), n(1)]));
    assert_eq!(parse(r#"["a",1]"#).unwrap(), arr(vec![s("a"), n(1)]));
}

#[test]
fn test_single_char() {
    // Tests for single character inputs
    assert_eq!(parse("").unwrap(), Value::Null);
    assert_eq!(parse("{}").unwrap(), Value::Object(FxHashMap::default()));
    assert_eq!(parse("[]").unwrap(), Value::Array(vec![]));
    assert_eq!(parse("#").unwrap(), Value::Null);
    assert_eq!(parse(" ").unwrap(), Value::Null);
    assert_eq!(parse("\t").unwrap(), Value::Null);
    assert_eq!(parse("\n").unwrap(), Value::Null);
    assert_eq!(parse("\r").unwrap(), Value::Null);

    // Errors
    assert!(parse("\"").is_err());
    assert!(parse("'").is_err());
    assert!(parse(":").is_err());
    assert!(parse("]").is_err());
    assert!(parse("}").is_err());
    assert!(parse("{").is_err()); // Auto-close not supported
    assert!(parse("[").is_err()); // Auto-close not supported
}

#[test]
fn test_numbers() {
    // Number tests that our parser supports
    assert_eq!(parse("1").unwrap(), n(1));
    assert_eq!(parse("-1").unwrap(), n(-1));
    assert_eq!(parse("0").unwrap(), n(0));

    assert_eq!(parse("1.0").unwrap(), f(1.0)); // Keep as float - reference implementation doesn't optimize
    assert_eq!(parse("-1.0").unwrap(), f(-1.0)); // Keep as float
    assert_eq!(parse("0.0").unwrap(), f(0.0)); // Keep as float

    // Leading decimal point is now supported
    assert_eq!(parse(".1").unwrap(), f(0.1));
    assert_eq!(parse("-.1").unwrap(), f(-0.1));
    assert_eq!(parse(".0").unwrap(), f(0.0));
    assert_eq!(parse("+.1").unwrap(), f(0.1));

    // Trailing decimal point
    assert_eq!(parse("1.").unwrap(), n(1));
    assert_eq!(parse("-1.").unwrap(), n(-1));
    assert_eq!(parse("+1.").unwrap(), n(1));
    assert_eq!(parse("0.").unwrap(), n(0));

    // Positive sign support
    assert_eq!(parse("+1").unwrap(), n(1));
    assert_eq!(parse("+0").unwrap(), n(0));
    assert_eq!(parse("+1.0").unwrap(), f(1.0));

    assert_eq!(parse("0.9").unwrap(), f(0.9));
    assert_eq!(parse("-0.9").unwrap(), f(-0.9));
    assert_eq!(parse("[1]").unwrap(), arr(vec![n(1)]));
    assert_eq!(parse("a:1").unwrap(), obj(&[("a", n(1))]));
    assert_eq!(parse("{a:1}").unwrap(), obj(&[("a", n(1))]));
    // Numeric keys not supported
    // assert_eq!(parse("{1:a}").unwrap(), obj(&[("1", s("a"))]));
    assert_eq!(parse("1.2").unwrap(), f(1.2));
    assert_eq!(parse("1e2").unwrap(), n(100)); // Optimized to integer
    assert_eq!(parse("-1.2").unwrap(), f(-1.2));
    assert_eq!(parse("-1e2").unwrap(), n(-100)); // Optimized to integer
    assert_eq!(parse("1e+2").unwrap(), n(100)); // Optimized to integer
    assert_eq!(parse("1e-2").unwrap(), f(0.01));
}

#[test]
fn test_single_line_comments() {
    // Single line comment tests
    assert_eq!(parse("a:1#b").unwrap(), obj(&[("a", n(1))]));
    assert_eq!(parse("#a:1").unwrap(), Value::Null);
    assert_eq!(parse("#a:1\nb:2").unwrap(), obj(&[("b", n(2))]));
    assert_eq!(parse("b:2\n#a:1").unwrap(), obj(&[("b", n(2))]));
    assert_eq!(
        parse("b:2,\n#a:1\nc:3").unwrap(),
        obj(&[("b", n(2)), ("c", n(3))])
    );
    assert_eq!(parse("//a:1").unwrap(), Value::Null);
    assert_eq!(parse("//a:1\nb:2").unwrap(), obj(&[("b", n(2))]));
    assert_eq!(parse("b:2\n//a:1").unwrap(), obj(&[("b", n(2))]));
    assert_eq!(
        parse("b:2,\n//a:1\nc:3").unwrap(),
        obj(&[("b", n(2)), ("c", n(3))])
    );
}

#[test]
fn test_multi_line_comments() {
    // Multi-line comment tests
    assert_eq!(parse("/*a:1*/").unwrap(), Value::Null);
    assert_eq!(parse("/*a:1*/\nb:2").unwrap(), obj(&[("b", n(2))]));
    assert_eq!(parse("/*a:1\n*/b:2").unwrap(), obj(&[("b", n(2))]));
    assert_eq!(parse("b:2\n/*a:1*/").unwrap(), obj(&[("b", n(2))]));
    assert_eq!(
        parse("b:2,\n/*\na:1,\n*/\nc:3").unwrap(),
        obj(&[("b", n(2)), ("c", n(3))])
    );

    // Unterminated comments should error
    assert!(parse("/*").is_err());
    assert!(parse("\n/*").is_err());
    assert!(parse("a/*").is_err());
    assert!(parse("\na/*").is_err());
}

#[test]
fn test_explicit_comma() {
    // Explicit comma tests
    assert_eq!(parse("[0,1]").unwrap(), arr(vec![n(0), n(1)]));
    assert_eq!(parse("[0,null]").unwrap(), arr(vec![n(0), Value::Null]));
    assert_eq!(
        parse("{a:0,b:null}").unwrap(),
        obj(&[("a", n(0)), ("b", Value::Null)])
    );
    assert_eq!(
        parse("{a:1,b:2}").unwrap(),
        obj(&[("a", n(1)), ("b", n(2))])
    );
    assert_eq!(parse("[1,2]").unwrap(), arr(vec![n(1), n(2)]));
    assert_eq!(
        parse("{a:1,\nb:2}").unwrap(),
        obj(&[("a", n(1)), ("b", n(2))])
    );
    assert_eq!(parse("[1,\n2]").unwrap(), arr(vec![n(1), n(2)]));
    assert_eq!(parse("a:1,b:2").unwrap(), obj(&[("a", n(1)), ("b", n(2))]));
    assert_eq!(parse("1,2").unwrap(), arr(vec![n(1), n(2)]));
    assert_eq!(parse("1,2,3").unwrap(), arr(vec![n(1), n(2), n(3)]));
    assert_eq!(
        parse("a:1,\nb:2").unwrap(),
        obj(&[("a", n(1)), ("b", n(2))])
    );
    assert_eq!(parse("1,\n2").unwrap(), arr(vec![n(1), n(2)]));
}

#[test]
fn test_trailing_comma() {
    // Trailing comma tests that our parser supports
    assert_eq!(parse("[1,]").unwrap(), arr(vec![n(1)]));
    assert_eq!(parse("{a:1,}").unwrap(), obj(&[("a", n(1))]));
    assert_eq!(parse("{a:1},").unwrap(), arr(vec![obj(&[("a", n(1))])]));
    assert_eq!(parse("a:1,").unwrap(), obj(&[("a", n(1))]));
}

#[test]
fn test_value_standard() {
    // Standard JSON values
    assert_eq!(parse("null").unwrap(), Value::Null);
    assert_eq!(parse("true").unwrap(), Value::Bool(true));
    assert_eq!(parse("false").unwrap(), Value::Bool(false));
    assert_eq!(parse(r#""hello""#).unwrap(), s("hello"));
    assert_eq!(parse("42").unwrap(), n(42));
    assert_eq!(parse("3.14").unwrap(), f(3.14));
    assert_eq!(parse("[]").unwrap(), Value::Array(vec![]));
    assert_eq!(parse("{}").unwrap(), Value::Object(FxHashMap::default()));
}

#[test]
fn test_implicit_object() {
    // Implicit top-level objects
    assert_eq!(parse("a:1").unwrap(), obj(&[("a", n(1))]));
    assert_eq!(parse("a:1,b:2").unwrap(), obj(&[("a", n(1)), ("b", n(2))]));
    assert_eq!(parse("foo: 'bar'").unwrap(), obj(&[("foo", s("bar"))]));
    assert_eq!(
        parse("x:1,y:2,z:3").unwrap(),
        obj(&[("x", n(1)), ("y", n(2)), ("z", n(3))])
    );
}

#[test]
fn test_implicit_array() {
    // Implicit top-level arrays that our parser supports
    assert_eq!(parse("1,2,3").unwrap(), arr(vec![n(1), n(2), n(3)]));
    assert_eq!(
        parse("'a','b','c'").unwrap(),
        arr(vec![s("a"), s("b"), s("c")])
    );
    assert_eq!(
        parse("true,false,null").unwrap(),
        arr(vec![Value::Bool(true), Value::Bool(false), Value::Null])
    );
    assert_eq!(parse("1,").unwrap(), arr(vec![n(1)]));
}

#[test]
fn test_unquoted_keys() {
    // Unquoted keys in objects
    assert_eq!(parse("{a:1}").unwrap(), obj(&[("a", n(1))]));
    assert_eq!(parse("{foo:2}").unwrap(), obj(&[("foo", n(2))]));
    assert_eq!(parse("{name:3}").unwrap(), obj(&[("name", n(3))]));
    assert_eq!(parse("key:4").unwrap(), obj(&[("key", n(4))]));
    assert_eq!(parse("{_key:5}").unwrap(), obj(&[("_key", n(5))]));
    assert_eq!(parse("{$key:6}").unwrap(), obj(&[("$key", n(6))]));
}

#[test]
fn test_single_quotes() {
    // Single quoted strings
    assert_eq!(parse("'hello'").unwrap(), s("hello"));
    assert_eq!(
        parse("{'key':'value'}").unwrap(),
        obj(&[("key", s("value"))])
    );
    assert_eq!(
        parse("['a','b','c']").unwrap(),
        arr(vec![s("a"), s("b"), s("c")])
    );
    assert_eq!(parse("{a:'b'}").unwrap(), obj(&[("a", s("b"))]));
    assert_eq!(parse("'hello\\'world'").unwrap(), s("hello'world"));
}

#[test]
fn test_comment_options() {
    // Test with comments disabled
    let mut opts = ParserOptions::default();
    opts.allow_comments = false;

    // Comments should error when disabled
    assert!(parse_with_options("#comment", opts.clone()).is_err());
    assert!(parse_with_options("//comment", opts.clone()).is_err());
    assert!(parse_with_options("/*comment*/", opts.clone()).is_err());
}

#[test]
fn test_trailing_comma_options() {
    // Test with trailing commas disabled
    let mut opts = ParserOptions::default();
    opts.allow_trailing_commas = false;

    assert!(parse_with_options("[1,]", opts.clone()).is_err());
    assert!(parse_with_options("{a:1,}", opts.clone()).is_err());

    // Without trailing comma should still work
    assert_eq!(
        parse_with_options("[1]", opts.clone()).unwrap(),
        arr(vec![n(1)])
    );
    assert_eq!(
        parse_with_options("{a:1}", opts.clone()).unwrap(),
        obj(&[("a", n(1))])
    );
}

#[test]
fn test_single_quotes_options() {
    // Test with single quotes disabled
    let mut opts = ParserOptions::default();
    opts.allow_single_quotes = false;

    assert!(parse_with_options("'hello'", opts.clone()).is_err());
    assert!(parse_with_options("{'key':'value'}", opts.clone()).is_err());

    // Double quotes should still work
    assert_eq!(
        parse_with_options(r#""hello""#, opts.clone()).unwrap(),
        s("hello")
    );
}

#[test]
fn test_unquoted_keys_options() {
    // Test with unquoted keys disabled
    let mut opts = ParserOptions::default();
    opts.allow_unquoted_keys = false;

    assert!(parse_with_options("{a:1}", opts.clone()).is_err());
    assert!(parse_with_options("{foo:2}", opts.clone()).is_err());

    // Quoted keys should still work
    assert_eq!(
        parse_with_options(r#"{"a":1}"#, opts.clone()).unwrap(),
        obj(&[("a", n(1))])
    );
}

#[test]
fn test_implicit_top_level_options() {
    // Test with implicit top level disabled
    let mut opts = ParserOptions::default();
    opts.implicit_top_level = false;

    // These should fail without explicit object/array delimiters
    assert!(parse_with_options("a:1", opts.clone()).is_err());
    assert!(parse_with_options("1,2,3", opts.clone()).is_err());

    // Explicit forms should still work
    assert_eq!(
        parse_with_options("{a:1}", opts.clone()).unwrap(),
        obj(&[("a", n(1))])
    );
    assert_eq!(
        parse_with_options("[1,2,3]", opts.clone()).unwrap(),
        arr(vec![n(1), n(2), n(3)])
    );
}
