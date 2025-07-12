// this_file: tests/number_formats.rs

use vexy_json::Number;
use vexy_json::{parse, Value};

/// Tests for extended number format support including hex, octal, binary, and underscore separators.
/// These formats are commonly used in configuration files and match jsonic compatibility.

#[test]
fn test_underscore_separators() {
    // Basic underscore separator support in decimal numbers
    assert_eq!(parse("10_0").unwrap(), Value::Number(Number::Integer(100)));
    assert_eq!(
        parse("1_000").unwrap(),
        Value::Number(Number::Integer(1000))
    );
    assert_eq!(
        parse("1_000_000").unwrap(),
        Value::Number(Number::Integer(1000000))
    );
    assert_eq!(
        parse("123_456_789").unwrap(),
        Value::Number(Number::Integer(123456789))
    );

    // Underscore separators with decimals - NOT YET SUPPORTED
    // TODO: Implement underscore separators in decimal numbers
    // assert_eq!(
    //     parse("12_3.45_6").unwrap(),
    //     Value::Number(Number::Float(123.456))
    // );
    // assert_eq!(
    //     parse("1_0.0_5").unwrap(),
    //     Value::Number(Number::Float(10.05))
    // );

    // Underscore separators with scientific notation - NOT YET SUPPORTED
    // TODO: Implement underscore separators in scientific notation
    // assert_eq!(
    //     parse("1_23e2").unwrap(),
    //     Value::Number(Number::Float(12300.0))
    // );
    // assert_eq!(
    //     parse("1_0e+2").unwrap(),
    //     Value::Number(Number::Float(1000.0))
    // );
}

#[test]
fn test_hexadecimal_numbers() {
    // Basic hex support
    assert_eq!(parse("0xFF").unwrap(), Value::Number(Number::Integer(255)));
    assert_eq!(parse("0x10").unwrap(), Value::Number(Number::Integer(16)));
    assert_eq!(parse("0xA").unwrap(), Value::Number(Number::Integer(10)));
    assert_eq!(parse("0xa").unwrap(), Value::Number(Number::Integer(10)));

    // Hex with uppercase X
    assert_eq!(parse("0XFF").unwrap(), Value::Number(Number::Integer(255)));
    assert_eq!(parse("0X10").unwrap(), Value::Number(Number::Integer(16)));

    // Hex with underscore separators - NOT YET SUPPORTED
    // TODO: Implement underscore immediately after hex prefix
    // assert_eq!(parse("0x_A").unwrap(), Value::Number(Number::Integer(10)));
    // assert_eq!(parse("0x_FF").unwrap(), Value::Number(Number::Integer(255))); // NOT YET SUPPORTED
    assert_eq!(
        parse("0x1_00").unwrap(),
        Value::Number(Number::Integer(256))
    );
    assert_eq!(
        parse("0xFF_00").unwrap(),
        Value::Number(Number::Integer(65280))
    );

    // Negative hex numbers
    assert_eq!(
        parse("-0xFF").unwrap(),
        Value::Number(Number::Integer(-255))
    );
    // assert_eq!(parse("-0x_A").unwrap(), Value::Number(Number::Integer(-10))); // NOT YET SUPPORTED

    // Positive hex numbers with explicit + sign
    assert_eq!(parse("+0xFF").unwrap(), Value::Number(Number::Integer(255)));
    // assert_eq!(parse("+0x_A").unwrap(), Value::Number(Number::Integer(10))); // NOT YET SUPPORTED
}

#[test]
fn test_octal_numbers() {
    // Basic octal support
    assert_eq!(parse("0o12").unwrap(), Value::Number(Number::Integer(10)));
    assert_eq!(parse("0o77").unwrap(), Value::Number(Number::Integer(63)));
    assert_eq!(parse("0o123").unwrap(), Value::Number(Number::Integer(83)));

    // Octal with uppercase O
    assert_eq!(parse("0O12").unwrap(), Value::Number(Number::Integer(10)));
    assert_eq!(parse("0O77").unwrap(), Value::Number(Number::Integer(63)));

    // Octal with underscore separators
    // assert_eq!(parse("0o_12").unwrap(), Value::Number(Number::Integer(10))); // NOT YET SUPPORTED
    assert_eq!(parse("0o1_2").unwrap(), Value::Number(Number::Integer(10)));
    // assert_eq!(
    //     parse("0o_1_2_3").unwrap(),
    //     Value::Number(Number::Integer(83))
    // ); // NOT YET SUPPORTED

    // Negative octal numbers
    assert_eq!(parse("-0o12").unwrap(), Value::Number(Number::Integer(-10)));
    // assert_eq!(
    //     parse("-0o_12").unwrap(),
    //     Value::Number(Number::Integer(-10))
    // ); // NOT YET SUPPORTED

    // Positive octal numbers with explicit + sign
    assert_eq!(parse("+0o12").unwrap(), Value::Number(Number::Integer(10)));
    // assert_eq!(parse("+0o_12").unwrap(), Value::Number(Number::Integer(10))); // NOT YET SUPPORTED
}

#[test]
fn test_binary_numbers() {
    // Basic binary support
    assert_eq!(parse("0b1010").unwrap(), Value::Number(Number::Integer(10)));
    assert_eq!(parse("0b1111").unwrap(), Value::Number(Number::Integer(15)));
    assert_eq!(
        parse("0b10000000").unwrap(),
        Value::Number(Number::Integer(128))
    );

    // Binary with uppercase B
    assert_eq!(parse("0B1010").unwrap(), Value::Number(Number::Integer(10)));
    assert_eq!(parse("0B1111").unwrap(), Value::Number(Number::Integer(15)));

    // Binary with underscore separators
    // assert_eq!(
    //     parse("0b_1010").unwrap(),
    //     Value::Number(Number::Integer(10))
    // ); // NOT YET SUPPORTED
    assert_eq!(
        parse("0b1_010").unwrap(),
        Value::Number(Number::Integer(10))
    );
    // assert_eq!(
    //     parse("0b_1_0_1_0").unwrap(),
    //     Value::Number(Number::Integer(10))
    // ); // NOT YET SUPPORTED
    assert_eq!(
        parse("0b1111_0000").unwrap(),
        Value::Number(Number::Integer(240))
    );

    // Negative binary numbers
    assert_eq!(
        parse("-0b1010").unwrap(),
        Value::Number(Number::Integer(-10))
    );
    // assert_eq!(
    //     parse("-0b_1010").unwrap(),
    //     Value::Number(Number::Integer(-10))
    // ); // NOT YET SUPPORTED

    // Positive binary numbers with explicit + sign
    assert_eq!(
        parse("+0b1010").unwrap(),
        Value::Number(Number::Integer(10))
    );
    // assert_eq!(
    //     parse("+0b_1010").unwrap(),
    //     Value::Number(Number::Integer(10))
    // ); // NOT YET SUPPORTED
}

#[test]
fn test_number_formats_in_arrays() {
    // Test various number formats within array contexts
    let result = parse("[0xFF, 0o12, 0b1010, 1_000]").unwrap();
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Number(Number::Integer(255)),
            Value::Number(Number::Integer(10)),
            Value::Number(Number::Integer(10)),
            Value::Number(Number::Integer(1000))
        ])
    );
}

#[test]
fn test_number_formats_in_objects() {
    // Test various number formats as object values
    let result = parse("{hex: 0xFF, octal: 0o12, binary: 0b1010, decimal: 1_000}").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.get("hex"), Some(&Value::Number(Number::Integer(255))));
        assert_eq!(map.get("octal"), Some(&Value::Number(Number::Integer(10))));
        assert_eq!(map.get("binary"), Some(&Value::Number(Number::Integer(10))));
        assert_eq!(
            map.get("decimal"),
            Some(&Value::Number(Number::Integer(1000)))
        );
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_edge_cases() {
    // Test edge cases and combinations
    assert_eq!(parse("0x0").unwrap(), Value::Number(Number::Integer(0)));
    assert_eq!(parse("0o0").unwrap(), Value::Number(Number::Integer(0)));
    assert_eq!(parse("0b0").unwrap(), Value::Number(Number::Integer(0)));

    // Test with multiple underscores
    assert_eq!(
        parse("1___000").unwrap(),
        Value::Number(Number::Integer(1000))
    );
    // assert_eq!(
    //     parse("0x___FF").unwrap(),
    //     Value::Number(Number::Integer(255))
    // ); // NOT YET SUPPORTED

    // Test at boundaries
    assert_eq!(parse("0x_").is_err(), true); // Invalid: no digits after underscore
    assert_eq!(parse("0o_").is_err(), true); // Invalid: no digits after underscore
    assert_eq!(parse("0b_").is_err(), true); // Invalid: no digits after underscore
}

#[test]
fn test_mixed_case_prefixes() {
    // Test various case combinations for prefixes
    assert_eq!(parse("0xff").unwrap(), Value::Number(Number::Integer(255)));
    assert_eq!(parse("0XFF").unwrap(), Value::Number(Number::Integer(255)));
    assert_eq!(parse("0xFf").unwrap(), Value::Number(Number::Integer(255)));
    assert_eq!(parse("0Xff").unwrap(), Value::Number(Number::Integer(255)));
}
