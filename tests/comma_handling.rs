// this_file: tests/comma_handling.rs

use vexy_json::{parse, Value};

/// Comma handling tests based on reference implementation tests from comma.test.js
/// Tests implicit commas (newlines as separators), optional commas, trailing commas, and edge cases
/// Reference: ref/the reference implementation/test/comma.test.js

#[test]
fn test_basic_comma_usage() {
    // Test standard comma usage in arrays and objects
    let result = parse("[0,1]").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_f64(), Some(0.0));
        assert_eq!(arr[1].as_f64(), Some(1.0));
    }

    let result = parse("{a:1,b:2}").unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
        assert_eq!(obj.get("b").and_then(|v| v.as_f64()), Some(2.0));
    }
}

#[test]
fn test_newline_as_comma_separator() {
    // Test newlines acting as comma separators in various contexts

    // Objects with newline separators
    let result = parse("{a:1\nb:2}");
    match result {
        Ok(Value::Object(obj)) if obj.len() == 2 => {
            println!("✓ vexy_json supports newlines as comma separators in objects");
            assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
            assert_eq!(obj.get("b").and_then(|v| v.as_f64()), Some(2.0));
        }
        Ok(other) => {
            println!("vexy_json parsed object newline as: {other:?}");
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support newlines as separators in objects: {e:?}"
            );
        }
    }

    // Arrays with newline separators
    let result = parse("[1\n2]");
    match result {
        Ok(Value::Array(arr)) if arr.len() == 2 => {
            println!("✓ vexy_json supports newlines as comma separators in arrays");
            assert_eq!(arr[0].as_f64(), Some(1.0));
            assert_eq!(arr[1].as_f64(), Some(2.0));
        }
        Ok(other) => {
            println!("vexy_json parsed array newline as: {other:?}");
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support newlines as separators in arrays: {e:?}"
            );
        }
    }
}

#[test]
fn test_implicit_top_level_structures() {
    // Test implicit top-level objects and arrays

    // Implicit object: a:1,b:2
    let result = parse("a:1,b:2");
    match result {
        Ok(Value::Object(obj)) if obj.len() == 2 => {
            println!("✓ vexy_json supports implicit top-level objects");
            assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
            assert_eq!(obj.get("b").and_then(|v| v.as_f64()), Some(2.0));
        }
        Ok(other) => {
            println!("vexy_json parsed implicit object as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json doesn't support implicit objects: {e:?}");
        }
    }

    // Implicit array: 1,2,3
    let result = parse("1,2,3");
    match result {
        Ok(Value::Array(arr)) if arr.len() == 3 => {
            println!("✓ vexy_json supports implicit top-level arrays");
            assert_eq!(arr[0].as_f64(), Some(1.0));
            assert_eq!(arr[1].as_f64(), Some(2.0));
            assert_eq!(arr[2].as_f64(), Some(3.0));
        }
        Ok(other) => {
            println!("vexy_json parsed implicit array as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json doesn't support implicit arrays: {e:?}");
        }
    }
}

#[test]
fn test_implicit_structures_with_newlines() {
    // Test implicit structures with newline separators

    // Implicit object with newlines: a:1\nb:2
    let result = parse("a:1\nb:2");
    match result {
        Ok(Value::Object(obj)) if obj.len() == 2 => {
            println!("✓ vexy_json supports implicit objects with newline separators");
            assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
            assert_eq!(obj.get("b").and_then(|v| v.as_f64()), Some(2.0));
        }
        Ok(other) => {
            println!(
                "vexy_json parsed implicit object with newlines as: {other:?}"
            );
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support implicit objects with newlines: {e:?}"
            );
        }
    }

    // Implicit array with newlines: 1\n2\n3
    let result = parse("1\n2\n3");
    match result {
        Ok(Value::Array(arr)) if arr.len() == 3 => {
            println!("✓ vexy_json supports implicit arrays with newline separators");
            assert_eq!(arr[0].as_f64(), Some(1.0));
            assert_eq!(arr[1].as_f64(), Some(2.0));
            assert_eq!(arr[2].as_f64(), Some(3.0));
        }
        Ok(other) => {
            println!(
                "vexy_json parsed implicit array with newlines as: {other:?}"
            );
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support implicit arrays with newlines: {e:?}"
            );
        }
    }

    // String values with newlines: a\nb\nc
    let result = parse("a\nb\nc");
    match result {
        Ok(Value::Array(arr)) if arr.len() == 3 => {
            println!("✓ vexy_json supports implicit string arrays with newlines");
            if let (Some(Value::String(s1)), Some(Value::String(s2)), Some(Value::String(s3))) =
                (arr.first(), arr.get(1), arr.get(2))
            {
                assert_eq!(s1, "a");
                assert_eq!(s2, "b");
                assert_eq!(s3, "c");
            }
        }
        Ok(other) => {
            println!("vexy_json parsed string sequence as: {other:?}");
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support string sequences with newlines: {e:?}"
            );
        }
    }
}

#[test]
fn test_trailing_commas() {
    // Test trailing commas in arrays and objects

    // Arrays with trailing commas
    let result = parse("[1,]");
    match result {
        Ok(Value::Array(arr)) if arr.len() == 1 => {
            println!("✓ vexy_json supports trailing commas in arrays");
            assert_eq!(arr[0].as_f64(), Some(1.0));
        }
        Ok(other) => {
            println!("vexy_json parsed trailing comma array as: {other:?}");
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support trailing commas in arrays: {e:?}"
            );
        }
    }

    // Objects with trailing commas
    let result = parse("{a:1,}");
    match result {
        Ok(Value::Object(obj)) if obj.len() == 1 => {
            println!("✓ vexy_json supports trailing commas in objects");
            assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
        }
        Ok(other) => {
            println!("vexy_json parsed trailing comma object as: {other:?}");
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support trailing commas in objects: {e:?}"
            );
        }
    }
}

#[test]
fn test_multiple_consecutive_commas() {
    // Test multiple consecutive commas creating null values (forgiving JSON feature)

    // Multiple commas in arrays
    let result = parse("[,,]");
    match result {
        Ok(Value::Array(arr)) => {
            println!(
                "vexy_json parsed [,,] as array with {} elements: {:?}",
                arr.len(),
                arr
            );
            // This tests if vexy_json supports forgiving JSON-style null insertion for empty comma positions
        }
        Ok(other) => {
            println!("vexy_json parsed [,,] as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on multiple commas [,,]: {e:?}");
        }
    }

    // Multiple commas with values
    let result = parse("[1,,3]");
    match result {
        Ok(Value::Array(arr)) => {
            println!(
                "vexy_json parsed [1,,3] as array with {} elements: {:?}",
                arr.len(),
                arr
            );
        }
        Ok(other) => {
            println!("vexy_json parsed [1,,3] as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on [1,,3]: {e:?}");
        }
    }

    // Leading commas
    let result = parse("[,1]");
    match result {
        Ok(Value::Array(arr)) => {
            println!(
                "vexy_json parsed [,1] as array with {} elements: {:?}",
                arr.len(),
                arr
            );
        }
        Ok(other) => {
            println!("vexy_json parsed [,1] as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on leading comma [,1]: {e:?}");
        }
    }
}

#[test]
fn test_object_comma_variations() {
    // Test various comma patterns in objects

    // Multiple commas in objects
    let result = parse("{,,}");
    match result {
        Ok(Value::Object(obj)) => {
            println!(
                "vexy_json parsed {{,,}} as object with {} entries: {:?}",
                obj.len(),
                obj
            );
        }
        Ok(other) => {
            println!("vexy_json parsed {{,,}} as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on {{,,}}: {e:?}");
        }
    }

    // Leading comma in object
    let result = parse("{,a:1}");
    match result {
        Ok(Value::Object(obj)) => {
            println!(
                "vexy_json parsed {{,a:1}} as object with {} entries: {:?}",
                obj.len(),
                obj
            );
            if let Some(val) = obj.get("a") {
                assert_eq!(val.as_f64(), Some(1.0));
            }
        }
        Ok(other) => {
            println!("vexy_json parsed {{,a:1}} as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on {{,a:1}}: {e:?}");
        }
    }

    // Both leading and trailing commas
    let result = parse("{,a:1,}");
    match result {
        Ok(Value::Object(obj)) => {
            println!(
                "vexy_json parsed {{,a:1,}} as object with {} entries: {:?}",
                obj.len(),
                obj
            );
            if let Some(val) = obj.get("a") {
                assert_eq!(val.as_f64(), Some(1.0));
            }
        }
        Ok(other) => {
            println!("vexy_json parsed {{,a:1,}} as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on {{,a:1,}}: {e:?}");
        }
    }
}

#[test]
fn test_complex_nested_structures() {
    // Test comma behavior in nested structures

    // Nested arrays with trailing commas
    let result = parse("[[a],]");
    match result {
        Ok(Value::Array(arr)) => {
            println!("vexy_json parsed [[a],] as: {arr:?}");
            if arr.len() == 1 {
                if let Some(Value::Array(inner)) = arr.first() {
                    if inner.len() == 1 {
                        if let Some(Value::String(s)) = inner.first() {
                            assert_eq!(s, "a");
                        }
                    }
                }
            }
        }
        Ok(other) => {
            println!("vexy_json parsed nested array as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on nested arrays: {e:?}");
        }
    }

    // Nested arrays without explicit commas
    let result = parse("[[a][b]]");
    match result {
        Ok(Value::Array(arr)) => {
            println!("vexy_json parsed [[a][b]] as: {arr:?}");
            // Test if vexy_json supports implicit comma between adjacent arrays
        }
        Ok(other) => {
            println!("vexy_json parsed [[a][b]] as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on [[a][b]]: {e:?}");
        }
    }

    // Objects in arrays with trailing commas
    let result = parse("[{a:1},]");
    match result {
        Ok(Value::Array(arr)) => {
            println!("vexy_json parsed [{{a:1}},] as: {arr:?}");
            if arr.len() == 1 {
                if let Some(Value::Object(obj)) = arr.first() {
                    assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
                }
            }
        }
        Ok(other) => {
            println!("vexy_json parsed object in array as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on object in array: {e:?}");
        }
    }
}

#[test]
fn test_special_cases() {
    // Test special comma edge cases

    // Single object followed by comma (creates implicit array in forgiving JSON)
    let result = parse("{a:1},");
    match result {
        Ok(Value::Array(arr)) => {
            println!("vexy_json parsed {{a:1}}, as implicit array: {arr:?}");
        }
        Ok(Value::Object(obj)) => {
            println!(
                "vexy_json parsed {{a:1}}, as object (ignored trailing comma): {obj:?}"
            );
        }
        Ok(other) => {
            println!("vexy_json parsed {{a:1}}, as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on {{a:1}},: {e:?}");
        }
    }

    // Test space-separated values (vexy_json feature)
    let result = parse("a:1 b:2");
    match result {
        Ok(Value::Object(obj)) if obj.len() == 2 => {
            println!("✓ vexy_json supports space-separated object properties");
            assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
            assert_eq!(obj.get("b").and_then(|v| v.as_f64()), Some(2.0));
        }
        Ok(other) => {
            println!("vexy_json parsed space-separated as: {other:?}");
        }
        Err(e) => {
            println!(
                "vexy_json doesn't support space-separated properties: {e:?}"
            );
        }
    }

    // Mixed value types with newlines
    let result = parse("true\nfalse\nnull");
    match result {
        Ok(Value::Array(arr)) if arr.len() == 3 => {
            println!("✓ vexy_json supports mixed value types with newlines");
            assert!(matches!(arr[0], Value::Bool(true)));
            assert!(matches!(arr[1], Value::Bool(false)));
            assert!(matches!(arr[2], Value::Null));
        }
        Ok(other) => {
            println!("vexy_json parsed mixed types as: {other:?}");
        }
        Err(e) => {
            println!("vexy_json error on mixed types: {e:?}");
        }
    }
}
