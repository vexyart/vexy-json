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
