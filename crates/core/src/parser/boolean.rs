// this_file: src/parser/boolean.rs

use crate::ast::Value;
use crate::error::Result;

#[inline]
pub(super) fn parse_true() -> Result<Value> {
    Ok(Value::Bool(true))
}

#[inline]
pub(super) fn parse_false() -> Result<Value> {
    Ok(Value::Bool(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_true() {
        let result = parse_true().unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_parse_false() {
        let result = parse_false().unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_parse_true_type_check() {
        let result = parse_true().unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_parse_false_type_check() {
        let result = parse_false().unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_parse_bool_value_extraction() {
        let true_result = parse_true().unwrap();
        let false_result = parse_false().unwrap();
        
        assert_eq!(true_result.as_bool(), Some(true));
        assert_eq!(false_result.as_bool(), Some(false));
    }
}
