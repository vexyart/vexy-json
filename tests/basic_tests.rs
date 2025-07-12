use vexy_json::{parse, Value};

#[test]
fn test_parse_null() {
    let result = parse("null").unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_parse_bool() {
    assert_eq!(parse("true").unwrap(), Value::Bool(true));
    assert_eq!(parse("false").unwrap(), Value::Bool(false));
}

#[test]
fn test_parse_number() {
    let result = parse("42").unwrap();
    eprintln!("Parsed 42 as: {result:?}");
    assert_eq!(result.as_i64(), Some(42));

    let result = parse("-42").unwrap();
    assert_eq!(result.as_i64(), Some(-42));

    let result = parse("3.14").unwrap();
    assert_eq!(result.as_f64(), Some(3.14));

    let result = parse("1e10").unwrap();
    assert_eq!(result.as_f64(), Some(1e10));
}

#[test]
fn test_parse_string() {
    let result = parse(r#""hello world""#).unwrap();
    assert_eq!(result.as_str(), Some("hello world"));

    let result = parse(r#""hello \"world\"""#).unwrap();
    assert_eq!(result.as_str(), Some("hello \"world\""));

    let result = parse(r#""line1\nline2""#).unwrap();
    assert_eq!(result.as_str(), Some("line1\nline2"));
}

#[test]
fn test_parse_array() {
    let result = parse("[]").unwrap();
    assert!(matches!(result, Value::Array(ref v) if v.is_empty()));

    let result = parse("[1, 2, 3]").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_i64(), Some(1));
        assert_eq!(arr[1].as_i64(), Some(2));
        assert_eq!(arr[2].as_i64(), Some(3));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_parse_object() {
    let result = parse("{}").unwrap();
    assert!(matches!(result, Value::Object(ref m) if m.is_empty()));

    let result = parse(r#"{"name": "John", "age": 30}"#).unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("John"));
        assert_eq!(obj.get("age").and_then(|v| v.as_i64()), Some(30));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_parse_nested() {
    let json = r#"{
        "user": {
            "name": "Alice",
            "tags": ["admin", "developer"],
            "active": true
        }
    }"#;

    let result = parse(json).unwrap();
    if let Value::Object(obj) = result {
        let user = obj.get("user").unwrap();
        if let Value::Object(user_obj) = user {
            assert_eq!(user_obj.get("name").and_then(|v| v.as_str()), Some("Alice"));
            assert_eq!(user_obj.get("active").and_then(|v| v.as_bool()), Some(true));

            let tags = user_obj.get("tags").unwrap();
            if let Value::Array(tags_arr) = tags {
                assert_eq!(tags_arr.len(), 2);
                assert_eq!(tags_arr[0].as_str(), Some("admin"));
                assert_eq!(tags_arr[1].as_str(), Some("developer"));
            } else {
                panic!("Expected tags array");
            }
        } else {
            panic!("Expected user object");
        }
    } else {
        panic!("Expected object");
    }
}
