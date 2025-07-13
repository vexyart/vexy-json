// this_file: crates/core/src/error/repair.rs

//! Repair functionality types and structures for JSON error recovery.

use super::recovery_v2::SuggestionCategory;
use super::types::Error;

/// Represents which parsing tier was used to successfully parse the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsingTier {
    /// Fast path using serde_json succeeded
    Fast,
    /// Forgiving path using vexy_json core succeeded
    Forgiving,
    /// Repair path using json-repair succeeded
    Repair,
}

impl From<SuggestionCategory> for RepairType {
    fn from(category: SuggestionCategory) -> Self {
        match category {
            SuggestionCategory::MissingBracket => RepairType::InsertBracket,
            SuggestionCategory::UnmatchedQuote => RepairType::BalanceQuotes,
            SuggestionCategory::MissingComma => RepairType::InsertComma,
            SuggestionCategory::TrailingComma => RepairType::RemoveComma,
            SuggestionCategory::InvalidEscape => RepairType::ReplaceText,
            SuggestionCategory::TypeMismatch => RepairType::TypeCoercion,
            SuggestionCategory::StructuralError => RepairType::ReplaceText,
            SuggestionCategory::Other => RepairType::ReplaceText,
        }
    }
}

/// Represents a single repair action that was performed during error recovery.
/// 
/// This structure captures all the details about what was changed during the repair process,
/// including the position, type of change, and the actual text modifications.
#[derive(Debug, Clone, PartialEq)]
pub struct RepairAction {
    /// The type of repair that was performed
    pub action_type: RepairType,
    /// Position in the original input where the repair was applied
    pub position: usize,
    /// The original text that was replaced
    pub original: String,
    /// The replacement text that was inserted
    pub replacement: String,
    /// Human-readable description of what was repaired
    pub description: String,
}

/// Types of repair actions that can be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairType {
    /// A bracket was inserted to balance the structure
    InsertBracket,
    /// A bracket was removed to balance the structure
    RemoveBracket,
    /// A bracket was replaced with a different bracket
    ReplaceBracket,
    /// Quotes were balanced or fixed
    BalanceQuotes,
    /// A comma was inserted
    InsertComma,
    /// A comma was removed
    RemoveComma,
    /// Inserted text at a position
    InsertText,
    /// Replaced text at a position
    ReplaceText,
    /// Replaced quote characters (e.g., single to double)
    ReplaceQuotes,
    /// Applied type coercion (e.g., quoted number to unquoted)
    TypeCoercion,
    /// Added quotes to an unquoted object key
    QuoteKey,
}

/// Enhanced result type that includes repair information and error tracking.
#[derive(Debug, Clone)]
pub struct EnhancedParseResult<T> {
    /// The parsed value (may be Value::Null if parsing failed)
    pub value: T,
    /// Any errors that occurred during parsing
    pub errors: Vec<Error>,
    /// Repair actions that were taken
    pub repairs: Vec<RepairAction>,
    /// Which parsing tier was used
    pub parsing_tier: ParsingTier,
}

impl<T> EnhancedParseResult<T> {
    /// Creates a new successful parse result
    pub fn success(value: T, tier: ParsingTier) -> Self {
        Self {
            value,
            errors: Vec::new(),
            repairs: Vec::new(),
            parsing_tier: tier,
        }
    }

    /// Creates a new successful parse result with repairs
    pub fn success_with_repairs(value: T, repairs: Vec<RepairAction>, tier: ParsingTier) -> Self {
        Self {
            value,
            errors: Vec::new(),
            repairs,
            parsing_tier: tier,
        }
    }

    /// Creates a new failed parse result
    pub fn failure(value: T, errors: Vec<Error>, tier: ParsingTier) -> Self {
        Self {
            value,
            errors,
            repairs: Vec::new(),
            parsing_tier: tier,
        }
    }

    /// Creates a new failed parse result with repairs attempted
    pub fn failure_with_repairs(
        value: T,
        errors: Vec<Error>,
        repairs: Vec<RepairAction>,
        tier: ParsingTier,
    ) -> Self {
        Self {
            value,
            errors,
            repairs,
            parsing_tier: tier,
        }
    }

    /// Returns true if parsing was successful (no errors)
    #[inline(always)]
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns true if any repairs were performed
    pub fn was_repaired(&self) -> bool {
        !self.repairs.is_empty()
    }

    /// Returns the parsing tier that was used
    pub fn get_performance_tier(&self) -> ParsingTier {
        self.parsing_tier
    }

    /// Returns a human-readable summary of repairs that were performed
    pub fn get_repair_summary(&self) -> String {
        if self.repairs.is_empty() {
            "No repairs needed".to_string()
        } else {
            let descriptions: Vec<&str> = self
                .repairs
                .iter()
                .map(|r| r.description.as_str())
                .collect();
            format!(
                "Applied {} repairs: {}",
                self.repairs.len(),
                descriptions.join(", ")
            )
        }
    }

    /// Converts this result to a standard Result type
    pub fn into_result(self) -> Result<T, Error> {
        if self.is_success() {
            Ok(self.value)
        } else {
            Err(self
                .errors
                .into_iter()
                .next()
                .unwrap_or_else(|| Error::Custom("Unknown error".to_string())))
        }
    }
}

impl<T> From<Result<T, Error>> for EnhancedParseResult<T>
where
    T: Default,
{
    fn from(result: Result<T, Error>) -> Self {
        match result {
            Ok(value) => Self::success(value, ParsingTier::Forgiving),
            Err(error) => Self::failure(T::default(), vec![error], ParsingTier::Forgiving),
        }
    }
}
