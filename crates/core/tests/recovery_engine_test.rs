// this_file: crates/core/tests/recovery_engine_test.rs

use vexy_json_core::error::{Error, ErrorContext, ErrorRecoveryEngineV2};

#[test]
fn test_error_recovery_engine_direct() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    // Test missing bracket recovery
    let context = ErrorContext {
        error: Error::UnexpectedEof(15),
        input: r#"{"name": "test""#.to_string(),
        position: 15,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    let suggestions = engine.suggest_recovery(&context);
    
    // Should have at least one suggestion
    assert!(!suggestions.is_empty());
    
    // The first suggestion should be to add a closing brace
    let first = &suggestions[0];
    assert!(first.description.contains("closing"));
    assert!(first.fixed_input.ends_with('}'));
    assert!(first.confidence > 0.5);
}

#[test]
fn test_error_recovery_engine_missing_comma() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    // Test missing comma recovery
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
    
    let suggestions = engine.suggest_recovery(&context);
    
    // Should have at least one suggestion
    assert!(!suggestions.is_empty());
    
    // Check if any suggestion is for adding a comma
    let comma_suggestion = suggestions.iter()
        .find(|s| s.description.contains("comma"));
    
    assert!(comma_suggestion.is_some());
}

#[test]
fn test_error_recovery_visual_output() {
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::UnexpectedEof(20),
        input: "{\n  \"name\": \"test\"\n  \"age\": 25\n}".to_string(),
        position: 20, // Position after "test"
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    let visual = engine.create_visual_error(&context);
    
    // Should contain visual elements
    println!("Visual output:\n{}", visual);
    assert!(visual.contains("---"));
    assert!(visual.contains("^"));
    assert!(visual.contains("Error:"));
    // Only show suggestions if there are any
    if !engine.suggest_recovery(&context).is_empty() {
        assert!(visual.contains("Suggestions:") || visual.contains("suggestions"));
    }
}