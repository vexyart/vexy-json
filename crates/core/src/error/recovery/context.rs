// this_file: crates/core/src/error/recovery/context.rs

use crate::error::recovery::strategies::RecoveryStrategy;

/// Context-aware recovery rule.
#[derive(Debug, Clone)]
pub(super) struct ContextRule {
    /// Pattern to match in the error context
    pub(super) pattern: String,
    /// Recovery strategy to apply
    pub(super) strategy: RecoveryStrategy,
    /// Confidence level of the rule (0.0 to 1.0)
    #[allow(dead_code)]
    pub(super) confidence: f64,
}
