// this_file: src/parser/null.rs

use crate::ast::Value;
use crate::error::Result;

#[inline]
pub(super) fn parse_null() -> Result<Value> {
    Ok(Value::Null)
}
