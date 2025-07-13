// this_file: crates/core/src/repair.rs

//! JSON repair functionality including basic and advanced repair strategies.
//!
//! This module provides both basic bracket balancing and advanced repair
//! capabilities including quote repair, type coercion, and confidence scoring.

pub mod advanced;

use crate::error::repair::{RepairAction, RepairType};
use rustc_hash::FxHashMap;

// Re-export advanced repair functionality
pub use advanced::{
    AdvancedJsonRepairer, RepairConfidence, RepairHistory, RepairHistoryEntry, RepairPreview,
    RepairStrategy, TypeCoercionRules,
};

/// Simple JSON repair implementation focusing on bracket balancing.
pub struct JsonRepairer {
    /// Maximum number of repairs to attempt
    max_repairs: usize,
    /// Cache for frequently repaired patterns
    repair_cache: FxHashMap<String, (String, Vec<RepairAction>)>,
    /// Enable caching for performance optimization
    cache_enabled: bool,
}

impl JsonRepairer {
    /// Creates a new JSON repairer with the specified maximum repairs.
    pub fn new(max_repairs: usize) -> Self {
        Self {
            max_repairs,
            repair_cache: FxHashMap::default(),
            cache_enabled: true,
        }
    }

    /// Creates a new JSON repairer with caching disabled.
    pub fn new_without_cache(max_repairs: usize) -> Self {
        Self {
            max_repairs,
            repair_cache: FxHashMap::default(),
            cache_enabled: false,
        }
    }

    /// Clears the repair cache.
    pub fn clear_cache(&mut self) {
        self.repair_cache.clear();
    }

    /// Returns the number of entries in the repair cache.
    pub fn cache_size(&self) -> usize {
        self.repair_cache.len()
    }

    /// Attempts to repair the given JSON string by balancing brackets.
    ///
    /// Returns the repaired JSON string and a list of repair actions taken.
    pub fn repair(&mut self, input: &str) -> Result<(String, Vec<RepairAction>), String> {
        // Check cache first if enabled
        if self.cache_enabled {
            if let Some((cached_result, cached_repairs)) = self.repair_cache.get(input) {
                return Ok((cached_result.clone(), cached_repairs.clone()));
            }
        }

        let mut repairs = Vec::new();
        let mut repaired = input.to_string();

        // Track bracket balance
        let balance = self.analyze_bracket_balance(&repaired);

        // If brackets are balanced, no repair needed
        if balance.is_balanced() {
            let result = (repaired.clone(), repairs.clone());
            if self.cache_enabled {
                self.repair_cache.insert(input.to_string(), result.clone());
            }
            return Ok(result);
        }

        // Attempt to fix unbalanced brackets
        if let Some(fixed) = self.fix_bracket_balance(&repaired, &balance) {
            repairs.push(RepairAction {
                action_type: RepairType::InsertBracket,
                position: repaired.len(),
                original: String::new(),
                replacement: fixed.chars().skip(repaired.len()).collect(),
                description: "Added missing closing brackets".to_string(),
            });
            repaired = fixed;
        }

        // Check if we exceeded repair limit
        if repairs.len() > self.max_repairs {
            return Err("Maximum repair attempts exceeded".to_string());
        }

        let result = (repaired, repairs);

        // Cache the result if enabled and cache isn't too large
        if self.cache_enabled && self.repair_cache.len() < 1000 {
            self.repair_cache.insert(input.to_string(), result.clone());
        }

        Ok(result)
    }

    /// Enhanced repair function with comprehensive action detection
    pub fn repair_with_detailed_tracking(
        &mut self,
        input: &str,
    ) -> Result<(String, Vec<RepairAction>), String> {
        // For now, just use the regular repair method
        // This can be enhanced later with more detailed tracking
        self.repair(input)
    }

    /// Analyzes the bracket balance in the input string and tracks the order of opening brackets.
    fn analyze_bracket_balance(&self, input: &str) -> BracketBalance {
        let mut stack = Vec::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut quote_char = '"';

        for ch in input.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if in_string {
                if ch == quote_char {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' | '\'' => {
                    in_string = true;
                    quote_char = ch;
                }
                '{' => stack.push(BracketType::Brace),
                '}' => {
                    if let Some(BracketType::Brace) = stack.last() {
                        stack.pop();
                    }
                }
                '[' => stack.push(BracketType::Bracket),
                ']' => {
                    if let Some(BracketType::Bracket) = stack.last() {
                        stack.pop();
                    }
                }
                _ => {}
            }
        }

        BracketBalance {
            unmatched_stack: stack,
        }
    }

    /// Attempts to fix bracket balance by adding missing closing brackets in reverse order.
    fn fix_bracket_balance(&self, input: &str, balance: &BracketBalance) -> Option<String> {
        if balance.unmatched_stack.is_empty() {
            return None;
        }

        let mut result = input.to_string();

        // Close brackets in reverse order (LIFO)
        for bracket_type in balance.unmatched_stack.iter().rev() {
            match bracket_type {
                BracketType::Brace => result.push('}'),
                BracketType::Bracket => result.push(']'),
            }
        }

        Some(result)
    }
}

/// Represents the type of bracket.
#[derive(Debug, Clone, PartialEq)]
enum BracketType {
    Brace,   // {
    Bracket, // [
}

/// Represents the balance state of brackets in a JSON string.
#[derive(Debug, Clone)]
struct BracketBalance {
    /// Stack of unmatched opening brackets in order
    unmatched_stack: Vec<BracketType>,
}

impl BracketBalance {
    /// Returns true if all brackets are balanced.
    fn is_balanced(&self) -> bool {
        self.unmatched_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_json() {
        let mut repairer = JsonRepairer::new(10);
        let input = r#"{"key": "value"}"#;
        let (repaired, repairs) = repairer.repair(input).unwrap();

        assert_eq!(repaired, input);
        assert!(repairs.is_empty());
    }

    #[test]
    fn test_missing_closing_brace() {
        let mut repairer = JsonRepairer::new(10);
        let input = r#"{"key": "value""#;
        let (repaired, repairs) = repairer.repair(input).unwrap();

        assert_eq!(repaired, r#"{"key": "value"}"#);
        assert_eq!(repairs.len(), 1);
        assert_eq!(repairs[0].action_type, RepairType::InsertBracket);
    }

    #[test]
    fn test_missing_closing_bracket() {
        let mut repairer = JsonRepairer::new(10);
        let input = r#"[1, 2, 3"#;
        let (repaired, repairs) = repairer.repair(input).unwrap();

        assert_eq!(repaired, r#"[1, 2, 3]"#);
        assert_eq!(repairs.len(), 1);
        assert_eq!(repairs[0].action_type, RepairType::InsertBracket);
    }

    #[test]
    fn test_string_with_brackets() {
        let mut repairer = JsonRepairer::new(10);
        let input = r#"{"key": "value with ] bracket"}"#;
        let (repaired, repairs) = repairer.repair(input).unwrap();

        assert_eq!(repaired, input);
        assert!(repairs.is_empty());
    }

    #[test]
    fn test_nested_structures() {
        let mut repairer = JsonRepairer::new(10);
        let input = r#"{"array": [1, 2, {"nested": "value""#;
        let (repaired, repairs) = repairer.repair(input).unwrap();

        assert_eq!(repaired, r#"{"array": [1, 2, {"nested": "value"}]}"#);
        assert_eq!(repairs.len(), 1);
    }

    #[test]
    fn test_repair_caching() {
        let mut repairer = JsonRepairer::new(10);
        let input = r#"{"key": "value""#;

        // First repair
        let (repaired1, repairs1) = repairer.repair(input).unwrap();
        assert_eq!(repairer.cache_size(), 1);

        // Second repair (should use cache)
        let (repaired2, repairs2) = repairer.repair(input).unwrap();
        assert_eq!(repaired1, repaired2);
        assert_eq!(repairs1.len(), repairs2.len());
        assert_eq!(repairer.cache_size(), 1);
    }

    #[test]
    fn test_detailed_repair_tracking() {
        let mut repairer = JsonRepairer::new(10);
        let input = r#"{"key": "value", "array": [1, 2, 3"#;
        let (repaired, repairs) = repairer.repair_with_detailed_tracking(input).unwrap();

        assert_eq!(repaired, r#"{"key": "value", "array": [1, 2, 3]}"#);
        assert_eq!(repairs.len(), 1);
        assert_eq!(repairs[0].action_type, RepairType::InsertBracket);
    }

    #[test]
    fn test_debug_bracket_balance() {
        let repairer = JsonRepairer::new(10);

        // Test case 1: {\"array\": [1, 2, {\"nested\": \"value\"
        let input1 = r#"{"array": [1, 2, {"nested": "value""#;
        let balance1 = repairer.analyze_bracket_balance(input1);
        println!("Input1: {input1}");
        println!(
            "Balance1: braces={}, brackets={}",
            balance1
                .unmatched_stack
                .iter()
                .filter(|&x| x == &BracketType::Brace)
                .count(),
            balance1
                .unmatched_stack
                .iter()
                .filter(|&x| x == &BracketType::Bracket)
                .count()
        );

        // Test case 2: {\"key\": \"value\", \"array\": [1, 2, 3
        let input2 = r#"{"key": "value", "array": [1, 2, 3"#;
        let balance2 = repairer.analyze_bracket_balance(input2);
        println!("Input2: {input2}");
        println!(
            "Balance2: braces={}, brackets={}",
            balance2
                .unmatched_stack
                .iter()
                .filter(|&x| x == &BracketType::Brace)
                .count(),
            balance2
                .unmatched_stack
                .iter()
                .filter(|&x| x == &BracketType::Bracket)
                .count()
        );

        // Let me manually count:
        // Input1: { [ { -> should be [Brace, Bracket, Brace] (need } ] } in reverse order)
        // Input2: { [ -> should be [Brace, Bracket] (need ] } in reverse order)
        assert_eq!(balance1.unmatched_stack.len(), 3);
        assert_eq!(
            balance1.unmatched_stack,
            vec![BracketType::Brace, BracketType::Bracket, BracketType::Brace]
        );
        assert_eq!(balance2.unmatched_stack.len(), 2);
        assert_eq!(
            balance2.unmatched_stack,
            vec![BracketType::Brace, BracketType::Bracket]
        );
    }
}
