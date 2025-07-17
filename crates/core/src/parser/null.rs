// this_file: src/parser/null.rs

use crate::ast::Value;
use crate::error::Result;

#[inline]
pub(super) fn parse_null() -> Result<Value> {
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let result = parse_null().unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_parse_null_type_check() {
        let result = parse_null().unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_parse_null_value_methods() {
        let result = parse_null().unwrap();
        assert!(result.is_null());
        assert_eq!(result.as_str(), None);
        assert_eq!(result.as_bool(), None);
        assert_eq!(result.as_i64(), None);
        assert_eq!(result.as_f64(), None);
    }
}
