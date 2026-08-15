// this_file: crates/core/tests/ml_pattern_test.rs

use vexy_json_core::error::ml_patterns::MLPatternRecognizer;
use vexy_json_core::error::{Error, ErrorContext, SuggestionCategory};

#[test]
fn test_ml_pattern_recognizer_missing_bracket() {
    let mut recognizer = MLPatternRecognizer::new();
    
    let context = ErrorContext {
        error: Error::UnexpectedEof(7),
        input: r#"{"test""#.to_string(),
        position: 7,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    let suggestions = recognizer.recognize_and_suggest(&context);
    
    // Should have suggestions
    assert!(!suggestions.is_empty());
    
    // Should have high confidence for missing bracket
    let best = &suggestions[0];
    assert!(best.confidence > 0.7);
    assert!(best.description.contains("ML"));
    assert!(best.fixed_input.ends_with('}'));
}

#[test]
fn test_ml_pattern_recognizer_missing_comma() {
    let mut recognizer = MLPatternRecognizer::new();
    
    let context = ErrorContext {
        error: Error::Expected {
            expected: ", or ] or newline".to_string(),
            found: "\"item2\"".to_string(),
            position: 8,
        },
        input: r#"["item1" "item2"]"#.to_string(),
        position: 8,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_array".to_string(),
    };
    
    let suggestions = recognizer.recognize_and_suggest(&context);
    
    // Debug output
    println!("Comma test suggestions:");
    for (i, s) in suggestions.iter().enumerate() {
        println!("  {}: {} (confidence: {:.2})", i, s.description, s.confidence);
        println!("     Fixed: {}", s.fixed_input);
    }
    
    // Should have suggestions
    assert!(!suggestions.is_empty());

    let comma_suggestion = suggestions
        .iter()
        .find(|suggestion| suggestion.category == SuggestionCategory::MissingComma);

    assert_eq!(
        comma_suggestion.map(|suggestion| suggestion.fixed_input.as_str()),
        Some(r#"["item1", "item2"]"#),
        "Should suggest adding a comma at the original error position",
    );
}

#[test]
fn test_ml_pattern_recognizer_unmatched_quote() {
    let mut recognizer = MLPatternRecognizer::new();
    
    let context = ErrorContext {
        error: Error::UnterminatedString(8),
        input: r#"{"key": "value"#.to_string(),
        position: 14,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_string".to_string(),
    };
    
    let suggestions = recognizer.recognize_and_suggest(&context);
    
    // Should have suggestions
    assert!(!suggestions.is_empty());
    
    // Should suggest adding a quote
    let quote_suggestion = suggestions.iter()
        .find(|s| s.fixed_input.ends_with('"'));
    
    assert!(quote_suggestion.is_some(), "Should suggest adding a closing quote");
}

#[test]
fn test_ml_pattern_feedback() {
    let mut recognizer = MLPatternRecognizer::new();
    
    // Simulate feedback
    recognizer.update_from_feedback("missing_closing_brace", true);
    recognizer.update_from_feedback("missing_closing_brace", true);
    recognizer.update_from_feedback("missing_closing_brace", false);
    
    // The pattern should still be active but with adjusted success rate
    let context = ErrorContext {
        error: Error::UnexpectedEof(7),
        input: r#"{"test""#.to_string(),
        position: 7,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    let suggestions = recognizer.recognize_and_suggest(&context);
    assert!(!suggestions.is_empty());
}
