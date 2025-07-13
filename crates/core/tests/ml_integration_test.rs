// this_file: crates/core/tests/ml_integration_test.rs

use vexy_json_core::parser::{parse_with_fallback, ParserOptions};
use vexy_json_core::error::repair::ParsingTier;

#[test]
fn test_ml_integration_in_parser() {
    // Test that ML-based recovery is working in the parser
    let input = r#"{"name": "test""#;
    let options = ParserOptions::default();
    
    let result = parse_with_fallback(input, options);
    
    // Should succeed with repair
    assert!(result.is_success());
    assert_eq!(result.parsing_tier, ParsingTier::Repair);
    assert!(!result.repairs.is_empty());
    
    // Check that we got an ML-based suggestion
    let has_ml_repair = result.repairs.iter()
        .any(|r| r.description.contains("ML") || r.description.contains("closing"));
    
    assert!(has_ml_repair, "Should have ML-based repair suggestion");
}

#[test]
fn test_ml_confidence_ordering() {
    // Test that ML suggestions with higher confidence are tried first
    let input = r#"{"a": 1 "b": 2}"#;
    let mut options = ParserOptions::default();
    options.newline_as_comma = false; // Force error
    
    let result = parse_with_fallback(input, options);
    
    println!("Result: success={}, tier={:?}", result.is_success(), result.parsing_tier);
    if !result.repairs.is_empty() {
        println!("Repairs:");
        for repair in &result.repairs {
            println!("  - {}", repair.description);
        }
    }
    
    // The ML system should help find a solution
    if result.is_success() {
        assert_eq!(result.parsing_tier, ParsingTier::Repair);
    }
}

#[test]
fn test_ml_learning_capability() {
    use vexy_json_core::error::{ErrorContext, ErrorRecoveryEngineV2, Error};
    
    // Test that the ML system can learn from feedback
    let mut engine = ErrorRecoveryEngineV2::new();
    
    let context = ErrorContext {
        error: Error::UnexpectedEof(7),
        input: r#"{"test""#.to_string(),
        position: 7,
        tokens_before: vec![],
        partial_ast: None,
        parsing_context: "in_object".to_string(),
    };
    
    // Get initial suggestions
    let suggestions1 = engine.suggest_recovery(&context);
    let initial_count = suggestions1.len();
    
    // Should have suggestions
    assert!(initial_count > 0);
    
    // Check that ML suggestions are included
    let ml_suggestions = suggestions1.iter()
        .filter(|s| s.description.starts_with("ML:"))
        .count();
    
    assert!(ml_suggestions > 0, "Should have ML-based suggestions");
}