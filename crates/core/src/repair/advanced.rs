//! Advanced JSON repair functionality with sophisticated repair strategies.
//!
//! This module provides enhanced repair capabilities including:
//! - Quote mismatch detection and repair
//! - Type coercion for common mistakes
//! - Repair confidence scoring
//! - Repair preview and history tracking

use crate::error::repair::{RepairAction, RepairType};
use std::collections::VecDeque;

/// Confidence level for repair operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairConfidence(f32);

impl RepairConfidence {
    /// Creates a new repair confidence value (0.0 to 1.0).
    pub fn new(value: f32) -> Self {
        RepairConfidence(value.clamp(0.0, 1.0))
    }

    /// Returns the confidence value.
    #[inline(always)]
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Checks if the confidence is above the given threshold.
    #[inline(always)]
    pub fn is_above(&self, threshold: f32) -> bool {
        self.0 >= threshold
    }

    /// Returns a human-readable confidence level.
    pub fn level(&self) -> &'static str {
        match self.0 {
            x if x >= 0.9 => "high",
            x if x >= 0.7 => "medium",
            x if x >= 0.5 => "low",
            _ => "very low",
        }
    }
}

/// Advanced repair strategy with confidence scoring.
#[derive(Debug, Clone)]
pub struct RepairStrategy {
    /// The repair action to take
    pub action: RepairAction,
    /// Confidence level for this repair
    pub confidence: RepairConfidence,
    /// Alternative repairs if this one fails
    pub alternatives: Vec<RepairStrategy>,
}

/// Repair preview showing the effect of applying repairs.
#[derive(Debug, Clone)]
pub struct RepairPreview {
    /// Original input
    pub original: String,
    /// Result after applying repairs
    pub repaired: String,
    /// List of repairs that would be applied
    pub repairs: Vec<RepairStrategy>,
    /// Overall confidence score
    pub overall_confidence: RepairConfidence,
}

/// History of repair operations for audit trails.
#[derive(Debug, Clone)]
pub struct RepairHistory {
    /// Maximum number of history entries to keep
    max_entries: usize,
    /// History entries (newest first)
    entries: VecDeque<RepairHistoryEntry>,
}

/// Single entry in repair history.
#[derive(Debug, Clone)]
pub struct RepairHistoryEntry {
    /// Timestamp of the repair
    pub timestamp: std::time::SystemTime,
    /// Original input
    pub original: String,
    /// Repaired output
    pub repaired: String,
    /// Repairs applied
    pub repairs: Vec<RepairAction>,
    /// Success status
    pub success: bool,
}

/// Advanced JSON repairer with sophisticated repair strategies.
pub struct AdvancedJsonRepairer {
    /// Maximum number of repairs to attempt
    #[allow(dead_code)]
    max_repairs: usize,
    /// Confidence threshold for automatic repairs
    confidence_threshold: f32,
    /// Enable repair preview mode
    preview_mode: bool,
    /// Repair history
    history: RepairHistory,
    /// Type coercion rules
    type_coercion_rules: TypeCoercionRules,
}

/// Rules for type coercion repairs.
#[derive(Debug, Clone)]
pub struct TypeCoercionRules {
    /// Convert quoted numbers to unquoted numbers
    pub unquote_numbers: bool,
    /// Convert unquoted true/false/null to proper JSON literals
    pub fix_literals: bool,
    /// Convert single quotes to double quotes
    pub fix_quotes: bool,
    /// Add quotes to unquoted object keys
    pub quote_keys: bool,
}

impl Default for TypeCoercionRules {
    fn default() -> Self {
        Self {
            unquote_numbers: true,
            fix_literals: true,
            fix_quotes: true,
            quote_keys: true,
        }
    }
}

impl AdvancedJsonRepairer {
    /// Creates a new advanced JSON repairer with default settings.
    pub fn new() -> Self {
        Self {
            max_repairs: 50,
            confidence_threshold: 0.7,
            preview_mode: false,
            history: RepairHistory::new(100),
            type_coercion_rules: TypeCoercionRules::default(),
        }
    }

    /// Sets the confidence threshold for automatic repairs.
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Enables preview mode (repairs are analyzed but not applied).
    pub fn with_preview_mode(mut self, enabled: bool) -> Self {
        self.preview_mode = enabled;
        self
    }

    /// Sets custom type coercion rules.
    pub fn with_type_coercion_rules(mut self, rules: TypeCoercionRules) -> Self {
        self.type_coercion_rules = rules;
        self
    }

    /// Repairs JSON with advanced strategies and confidence scoring.
    pub fn repair(&mut self, input: &str) -> Result<(String, Vec<RepairStrategy>), String> {
        let strategies = self.analyze_and_plan_repairs(input)?;

        if self.preview_mode {
            // In preview mode, return the strategies without applying them
            return Ok((input.to_string(), strategies));
        }

        // Apply repairs with confidence above threshold
        let mut repaired = input.to_string();
        let mut applied_strategies = Vec::new();

        // Filter strategies by confidence threshold
        let mut applicable_strategies: Vec<RepairStrategy> = strategies
            .into_iter()
            .filter(|s| s.confidence.is_above(self.confidence_threshold))
            .collect();

        // Sort by position in reverse order (highest position first)
        // This ensures that later repairs don't affect earlier positions
        applicable_strategies.sort_by(|a, b| b.action.position.cmp(&a.action.position));

        for strategy in applicable_strategies {
            repaired = self.apply_repair(&repaired, &strategy.action)?;
            applied_strategies.push(strategy);
        }

        // Record in history
        self.history.add_entry(RepairHistoryEntry {
            timestamp: std::time::SystemTime::now(),
            original: input.to_string(),
            repaired: repaired.clone(),
            repairs: applied_strategies
                .iter()
                .map(|s| s.action.clone())
                .collect(),
            success: true,
        });

        Ok((repaired, applied_strategies))
    }

    /// Generates a repair preview without applying changes.
    pub fn preview_repairs(&self, input: &str) -> Result<RepairPreview, String> {
        let strategies = self.analyze_and_plan_repairs(input)?;

        // Simulate applying repairs
        let mut preview_result = input.to_string();
        for strategy in &strategies {
            if strategy.confidence.is_above(self.confidence_threshold) {
                preview_result = self.apply_repair(&preview_result, &strategy.action)?;
            }
        }

        let overall_confidence = self.calculate_overall_confidence(&strategies);

        Ok(RepairPreview {
            original: input.to_string(),
            repaired: preview_result,
            repairs: strategies,
            overall_confidence,
        })
    }

    /// Analyzes input and plans repair strategies.
    fn analyze_and_plan_repairs(&self, input: &str) -> Result<Vec<RepairStrategy>, String> {
        let mut strategies = Vec::new();

        // Check for quote mismatches
        if let Some(quote_strategies) = self.analyze_quote_issues(input) {
            strategies.extend(quote_strategies);
        }

        // Check for type coercion opportunities
        if let Some(type_strategies) = self.analyze_type_coercion(input) {
            strategies.extend(type_strategies);
        }

        // Check for missing commas
        if let Some(comma_strategies) = self.analyze_missing_commas(input) {
            strategies.extend(comma_strategies);
        }

        // Check for unquoted keys
        if let Some(key_strategies) = self.analyze_unquoted_keys(input) {
            strategies.extend(key_strategies);
        }

        // Sort strategies by confidence (highest first)
        strategies.sort_by(|a, b| {
            b.confidence
                .value()
                .partial_cmp(&a.confidence.value())
                .unwrap()
        });

        Ok(strategies)
    }

    /// Analyzes quote issues and returns repair strategies.
    fn analyze_quote_issues(&self, input: &str) -> Option<Vec<RepairStrategy>> {
        let mut strategies = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Check for single quotes that should be double quotes
            if chars[i] == '\'' && self.type_coercion_rules.fix_quotes {
                let start = i;
                i += 1;

                // Find the closing single quote
                while i < chars.len() && chars[i] != '\'' {
                    if chars[i] == '\\' {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }

                if i < chars.len() {
                    // Found closing quote
                    strategies.push(RepairStrategy {
                        action: RepairAction {
                            action_type: RepairType::ReplaceQuotes,
                            position: start,
                            original: "'".to_string(),
                            replacement: "\"".to_string(),
                            description: "Replace single quotes with double quotes".to_string(),
                        },
                        confidence: RepairConfidence::new(0.9),
                        alternatives: vec![],
                    });

                    strategies.push(RepairStrategy {
                        action: RepairAction {
                            action_type: RepairType::ReplaceQuotes,
                            position: i,
                            original: "'".to_string(),
                            replacement: "\"".to_string(),
                            description: "Replace closing single quote with double quote"
                                .to_string(),
                        },
                        confidence: RepairConfidence::new(0.9),
                        alternatives: vec![],
                    });
                }
            }
            i += 1;
        }

        if strategies.is_empty() {
            None
        } else {
            Some(strategies)
        }
    }

    /// Analyzes type coercion opportunities.
    fn analyze_type_coercion(&self, input: &str) -> Option<Vec<RepairStrategy>> {
        let mut strategies = Vec::new();

        // Check for quoted numbers
        if self.type_coercion_rules.unquote_numbers {
            let quoted_number_pattern =
                regex::Regex::new(r#""(-?\d+\.?\d*(?:[eE][+-]?\d+)?)""#).unwrap();
            for capture in quoted_number_pattern.captures_iter(input) {
                if let Some(number_match) = capture.get(1) {
                    let number_str = number_match.as_str();
                    if number_str.parse::<f64>().is_ok() {
                        strategies.push(RepairStrategy {
                            action: RepairAction {
                                action_type: RepairType::TypeCoercion,
                                position: capture.get(0).unwrap().start(),
                                original: capture.get(0).unwrap().as_str().to_string(),
                                replacement: number_str.to_string(),
                                description: format!(
                                    "Convert quoted number \"{}\" to unquoted",
                                    number_str
                                ),
                            },
                            confidence: RepairConfidence::new(0.8),
                            alternatives: vec![],
                        });
                    }
                }
            }
        }

        if strategies.is_empty() {
            None
        } else {
            Some(strategies)
        }
    }

    /// Analyzes missing commas in arrays and objects.
    fn analyze_missing_commas(&self, input: &str) -> Option<Vec<RepairStrategy>> {
        let mut strategies = Vec::new();

        // Simple pattern: look for }" or ]" followed by " (indicating a new element without comma)
        let pattern = regex::Regex::new(r#"([}\]])(\s*)"#).unwrap();

        for capture in pattern.captures_iter(input) {
            if let Some(match_) = capture.get(0) {
                let next_char_idx = match_.end();
                if next_char_idx < input.len() {
                    let remaining = &input[next_char_idx..];
                    if remaining.trim_start().starts_with('"')
                        || remaining.trim_start().starts_with('{')
                        || remaining.trim_start().starts_with('[')
                    {
                        strategies.push(RepairStrategy {
                            action: RepairAction {
                                action_type: RepairType::InsertComma,
                                position: match_.end(),
                                original: String::new(),
                                replacement: ",".to_string(),
                                description: "Insert missing comma between elements".to_string(),
                            },
                            confidence: RepairConfidence::new(0.85),
                            alternatives: vec![],
                        });
                    }
                }
            }
        }

        if strategies.is_empty() {
            None
        } else {
            Some(strategies)
        }
    }

    /// Analyzes unquoted object keys.
    fn analyze_unquoted_keys(&self, input: &str) -> Option<Vec<RepairStrategy>> {
        if !self.type_coercion_rules.quote_keys {
            return None;
        }

        let mut strategies = Vec::new();

        // Pattern for unquoted keys: word characters followed by colon
        let pattern = regex::Regex::new(r#"(\w+)\s*:"#).unwrap();

        for capture in pattern.captures_iter(input) {
            if let Some(key_match) = capture.get(1) {
                // Check if this key is already inside quotes
                let start = key_match.start();
                if start > 0 {
                    let prev_char = input.chars().nth(start - 1);
                    if prev_char == Some('"') || prev_char == Some('\'') {
                        continue; // Already quoted
                    }
                }

                strategies.push(RepairStrategy {
                    action: RepairAction {
                        action_type: RepairType::QuoteKey,
                        position: key_match.start(),
                        original: key_match.as_str().to_string(),
                        replacement: format!("\"{}\"", key_match.as_str()),
                        description: format!("Add quotes to key '{}'", key_match.as_str()),
                    },
                    confidence: RepairConfidence::new(0.75),
                    alternatives: vec![],
                });
            }
        }

        if strategies.is_empty() {
            None
        } else {
            Some(strategies)
        }
    }

    /// Applies a single repair action to the input.
    fn apply_repair(&self, input: &str, action: &RepairAction) -> Result<String, String> {
        match action.action_type {
            RepairType::InsertText | RepairType::InsertComma => {
                if action.position > input.len() {
                    return Err(format!("Insert position {} is out of bounds for input of length {}", action.position, input.len()));
                }
                let mut result = String::new();
                result.push_str(&input[..action.position]);
                result.push_str(&action.replacement);
                result.push_str(&input[action.position..]);
                Ok(result)
            }
            RepairType::ReplaceText
            | RepairType::ReplaceQuotes
            | RepairType::TypeCoercion
            | RepairType::QuoteKey => {
                let original_len = action.original.len();
                if action.position > input.len() {
                    return Err(format!("Replace position {} is out of bounds for input of length {}", action.position, input.len()));
                }
                if action.position + original_len > input.len() {
                    return Err(format!("Replace end position {} is out of bounds for input of length {}", action.position + original_len, input.len()));
                }
                let mut result = String::new();
                result.push_str(&input[..action.position]);
                result.push_str(&action.replacement);
                result.push_str(&input[action.position + original_len..]);
                Ok(result)
            }
            _ => Ok(input.to_string()), // Unsupported repair type, skip
        }
    }

    /// Calculates the overall confidence for a set of repair strategies.
    fn calculate_overall_confidence(&self, strategies: &[RepairStrategy]) -> RepairConfidence {
        if strategies.is_empty() {
            return RepairConfidence::new(1.0);
        }

        let sum: f32 = strategies.iter().map(|s| s.confidence.value()).sum();
        let avg = sum / strategies.len() as f32;
        RepairConfidence::new(avg)
    }

    /// Returns the repair history.
    pub fn history(&self) -> &RepairHistory {
        &self.history
    }

    /// Clears the repair history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl RepairHistory {
    /// Creates a new repair history with the specified maximum entries.
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: VecDeque::new(),
        }
    }

    /// Adds a new entry to the history.
    pub fn add_entry(&mut self, entry: RepairHistoryEntry) {
        self.entries.push_front(entry);
        while self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
    }

    /// Returns all history entries.
    pub fn entries(&self) -> &VecDeque<RepairHistoryEntry> {
        &self.entries
    }

    /// Clears all history entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of entries in the history.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the history is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AdvancedJsonRepairer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_confidence() {
        let confidence = RepairConfidence::new(0.85);
        assert_eq!(confidence.value(), 0.85);
        assert!(confidence.is_above(0.8));
        assert!(!confidence.is_above(0.9));
        assert_eq!(confidence.level(), "medium");
    }

    #[test]
    fn test_quote_repair() {
        let mut repairer = AdvancedJsonRepairer::new();
        let input = "{'name': 'John', 'age': 30}";
        let (repaired, strategies) = repairer.repair(input).unwrap();

        // Should replace single quotes with double quotes
        assert!(repaired.contains("\"name\""));
        assert!(repaired.contains("\"John\""));
        assert!(!repaired.contains("'"));
        assert!(!strategies.is_empty());
    }

    #[test]
    fn test_type_coercion() {
        let mut repairer = AdvancedJsonRepairer::new();
        let input = r#"{"count": "42", "price": "19.99"}"#;
        let (repaired, strategies) = repairer.repair(input).unwrap();

        // Should unquote numbers
        assert!(repaired.contains(": 42"));
        assert!(repaired.contains(": 19.99"));
        assert!(!strategies.is_empty());
    }

    #[test]
    fn test_preview_mode() {
        let repairer = AdvancedJsonRepairer::new().with_preview_mode(true);
        let input = "{'test': true}";
        let preview = repairer.preview_repairs(input).unwrap();

        assert_eq!(preview.original, input);
        assert!(preview.repaired.contains("\"test\""));
        assert!(!preview.repairs.is_empty());
    }

    #[test]
    fn test_repair_history() {
        let mut repairer = AdvancedJsonRepairer::new();
        let input = "{'test': 123}";

        repairer.repair(input).unwrap();

        assert_eq!(repairer.history().len(), 1);
        let entry = &repairer.history().entries()[0];
        assert_eq!(entry.original, input);
        assert!(entry.success);
    }
}
