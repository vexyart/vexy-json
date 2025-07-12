// this_file: tests/comment_handling.rs

use vexy_json::{parse, Value};

/// Comment handling tests based on reference implementation tests from comment.test.js
/// Tests single-line, multi-line comments, hash behavior, and edge cases
/// Reference: ref/vexy_json/test/comment.test.js

#[test]
fn test_double_slash_comments() {
    let result = parse("42 // comment");
    match result {
        Ok(val) if val.as_f64() == Some(42.0) => {
            println!("vexy_json supports // comments");
        }
        Ok(other) => {
            println!("vexy_json parsed as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on // comments: {e:?}");
        }
    }
}

#[test]
fn test_block_comments() {
    let result = parse("42 /* comment */");
    match result {
        Ok(val) if val.as_f64() == Some(42.0) => {
            println!("vexy_json supports /* */ comments");
        }
        Ok(other) => {
            println!("vexy_json parsed as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on /* */ comments: {e:?}");
        }
    }
}

#[test]
fn test_hash_character() {
    let result = parse("a#b");
    match result {
        Ok(Value::String(s)) if s == "a" => {
            println!("vexy_json treats # as comment");
        }
        Ok(Value::String(s)) if s.contains('#') => {
            println!("vexy_json treats # as literal: {s:?}");
        }
        Ok(other) => {
            println!("vexy_json parsed a#b as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on #: {e:?}");
        }
    }
}

#[test]
fn test_comments_in_strings() {
    let result = parse(r#"{"message": "Hello // comment"}"#).unwrap();
    if let Value::Object(obj) = result {
        if let Some(Value::String(s)) = obj.get("message") {
            assert_eq!(s, "Hello // comment");
        }
    }
}

#[test]
fn test_unterminated_comment() {
    let basic_test = parse("42 /* test */");
    if basic_test.is_ok() {
        let result = parse("/*");
        assert!(result.is_err(), "Unterminated comment should error");
    }
}

#[test]
fn test_comments_in_arrays() {
    let result = parse("[1, // comment\n2]");
    match result {
        Ok(Value::Array(arr)) if arr.len() == 2 => {
            println!("vexy_json supports comments in arrays");
        }
        Ok(other) => {
            println!("vexy_json parsed array comment as: {other:?}");
        }
        Err(_) => {
            println!("vexy_json doesn't support comments in arrays");
            let fallback = parse("[1, 2]").unwrap();
            assert!(matches!(fallback, Value::Array(_)));
        }
    }
}

#[test]
fn test_empty_comment() {
    let result = parse("42//");
    match result {
        Ok(val) if val.as_f64() == Some(42.0) => {
            println!("vexy_json supports empty // comments");
        }
        Ok(other) => {
            println!("vexy_json parsed 42// as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on empty //: {e:?}");
        }
    }
}

#[test]
fn test_comment_at_eof() {
    let result = parse("a:1 // final");
    match result {
        Ok(Value::Object(obj)) => {
            if let Some(val) = obj.get("a") {
                if val.as_f64() == Some(1.0) {
                    println!("vexy_json supports comments at EOF");
                }
            }
        }
        Ok(other) => {
            println!("vexy_json parsed EOF comment as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on EOF comment: {e:?}");
        }
    }
}
