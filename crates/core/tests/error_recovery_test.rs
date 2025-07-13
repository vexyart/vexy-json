// this_file: tests/error_recovery_test.rs

use vexy_json_core::error::{Error, ErrorRecoveryEngineV2, ErrorContext, SuggestionCategory};

#[test]
fn test_missing_closing_brace_recovery() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::UnexpectedEof(15),
        input: r#"{"name": "test""#.to_string(),
        position: 15,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    let suggestions = engine.suggest_recovery(&context);
    
    // Debug output
    println!("Suggestions: {suggestions:?}");
    
    assert!(!suggestions.is_empty());
    
    let first = &suggestions[0];
    assert_eq!(first.category, SuggestionCategory::MissingBracket);
    assert!(first.fixed_input.ends_with('}'));
    assert!(first.confidence >= 0.8);
}

#[test]
fn test_missing_comma_recovery() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::Expected {
            expected: "comma or closing bracket".to_string(),
            found: "Number".to_string(),
            position: 10,
        },
        input: r#"[1 2 3]"#.to_string(),
        position: 3,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_array".to_string(),
    };
    
    let suggestions = engine.suggest_recovery(&context);
    
    // Should have at least one suggestion
    let comma_suggestion = suggestions.iter()
        .find(|s| s.category == SuggestionCategory::MissingComma);
    
    assert!(comma_suggestion.is_some());
}

#[test]
fn test_unmatched_quote_recovery() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::UnterminatedString(8),
        input: r#"{"key": "value"#.to_string(),
        position: 14,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_string".to_string(),
    };
    
    let suggestions = engine.suggest_recovery(&context);
    assert!(!suggestions.is_empty());
    
    let quote_suggestion = suggestions.iter()
        .find(|s| s.category == SuggestionCategory::UnmatchedQuote);
    
    assert!(quote_suggestion.is_some());
    assert!(quote_suggestion.unwrap().fixed_input.ends_with('"'));
}

#[test]
fn test_implicit_object_recovery() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::Expected {
            expected: "value".to_string(),
            found: "UnquotedString".to_string(),
            position: 0,
        },
        input: r#"key: "value", another: 42"#.to_string(),
        position: 0,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "top_level".to_string(),
    };
    
    let suggestions = engine.suggest_recovery(&context);
    
    // Should suggest wrapping in object braces
    let structural_suggestion = suggestions.iter()
        .find(|s| s.category == SuggestionCategory::StructuralError);
    
    assert!(structural_suggestion.is_some());
    let suggestion = structural_suggestion.unwrap();
    assert!(suggestion.fixed_input.starts_with('{'));
    assert!(suggestion.fixed_input.ends_with('}'));
}

#[test]
fn test_type_coercion_recovery() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::Expected {
            expected: "boolean".to_string(),
            found: "String".to_string(),
            position: 15,
        },
        input: r#"{"enabled": "true"}"#.to_string(),
        position: 17,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    let suggestions = engine.suggest_recovery(&context);
    
    // Should suggest converting "true" to true
    let type_suggestion = suggestions.iter()
        .find(|s| s.category == SuggestionCategory::TypeMismatch);
    
    assert!(type_suggestion.is_some());
}

#[test]
fn test_visual_error_display() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::UnexpectedEof(35),
        input: "{\n  \"name\": \"test\",\n  \"age\": 25\n".to_string(),
        position: 34, // After the last newline
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    let visual = engine.create_visual_error(&context);
    
    // Should contain error visualization
    assert!(visual.contains("Error:"));
    assert!(visual.contains("^"));
    assert!(visual.contains("--- error here"));
    assert!(visual.contains("Suggestions:"));
}