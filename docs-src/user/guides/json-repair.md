# JSON Repair

Vexy JSON provides advanced JSON repair capabilities that can automatically fix common JSON formatting issues. The repair system uses confidence scoring and multiple strategies to intelligently fix malformed JSON.

## Overview

The JSON repair system operates on three levels:

1. **Basic Repair**: Simple bracket balancing and quote fixing
2. **Advanced Repair**: Intelligent pattern recognition and multi-strategy fixes
3. **Enhanced Repair**: Detailed tracking and confidence scoring

## Basic Repair

### Simple Usage

```rust
use vexy_json_core::repair::JsonRepairer;

let mut repairer = JsonRepairer::new(10); // Max 10 repairs
let malformed = r#"{"key": "value", "missing": "quote}"#;

match repairer.repair(malformed) {
    Ok((fixed, repairs)) => {
        println!("Fixed: {}", fixed);
        println!("Applied {} repairs", repairs.len());
    }
    Err(e) => println!("Repair failed: {}", e),
}
```

### Common Repairs

The basic repairer handles:

- **Missing quotes**: `{key: "value"}` → `{"key": "value"}`
- **Bracket imbalances**: `{"key": "value"` → `{"key": "value"}`
- **Trailing commas**: `{"key": "value",}` → `{"key": "value"}`
- **Single quotes**: `{'key': 'value'}` → `{"key": "value"}`

## Advanced Repair

### Configuration

```rust
use vexy_json_core::repair::advanced::{AdvancedJsonRepairer, TypeCoercionRules};

let mut repairer = AdvancedJsonRepairer::new()
    .with_confidence_threshold(0.7)
    .with_type_coercion_rules(TypeCoercionRules {
        unquote_numbers: true,
        fix_literals: true,
        fix_quotes: true,
        quote_keys: true,
    });

let (fixed, strategies) = repairer.repair(input)?;
```

### Repair Strategies

The advanced repairer includes multiple strategies:

#### Type Coercion

```rust
// Input: {"count": "42", "price": "19.99"}
// Output: {"count": 42, "price": 19.99}

// Input: {"flag": "true", "value": "null"}
// Output: {"flag": true, "value": null}
```

#### Quote Normalization

```rust
// Input: {'name': 'John', "age": '30'}
// Output: {"name": "John", "age": "30"}
```

#### Key Quoting

```rust
// Input: {name: "John", age: 30}
// Output: {"name": "John", "age": 30}
```

#### Comma Insertion

```rust
// Input: {"a": 1 "b": 2}
// Output: {"a": 1, "b": 2}
```

### Confidence Scoring

Each repair strategy has a confidence score:

```rust
use vexy_json_core::repair::advanced::RepairConfidence;

let (fixed, strategies) = repairer.repair(input)?;

for strategy in strategies {
    println!("Repair: {}", strategy.action.description);
    println!("Confidence: {:.2}", strategy.confidence.value());
    
    if strategy.confidence.is_high() {
        println!("High confidence repair");
    }
}
```

### Preview Mode

Test repairs without applying them:

```rust
let mut repairer = AdvancedJsonRepairer::new()
    .with_preview_mode(true);

let (original, strategies) = repairer.repair(input)?;
// original == input (unchanged)
// strategies contains what would be applied
```

## Enhanced Repair with Tracking

### Detailed Repair Tracking

```rust
use vexy_json_core::parser::parse_with_detailed_repair_tracking;

let result = parse_with_detailed_repair_tracking(input, options)?;

match result {
    EnhancedParseResult::Success { value, tier, repairs } => {
        println!("Parsed successfully using {:?}", tier);
        if !repairs.is_empty() {
            println!("Applied {} repairs:", repairs.len());
            for repair in repairs {
                println!("  {}", repair.description);
            }
        }
    }
    EnhancedParseResult::Failure { errors, tier, repairs } => {
        println!("Parse failed at {:?} tier", tier);
        for error in errors {
            println!("Error: {}", error);
        }
    }
}
```

### Three-Tier Parsing

The enhanced parser uses a three-tier strategy:

1. **Fast Tier**: Standard `serde_json` for maximum performance
2. **Forgiving Tier**: Vexy JSON parser for non-standard JSON
3. **Repair Tier**: Automatic repair for malformed JSON

```rust
use vexy_json_core::parser::parse_with_fallback;

let result = parse_with_fallback(input, options);
// Automatically tries all three tiers
```

## Repair History and Analytics

### Tracking Repair History

```rust
use vexy_json_core::repair::advanced::AdvancedJsonRepairer;

let mut repairer = AdvancedJsonRepairer::new();

// Perform multiple repairs
let _ = repairer.repair(input1)?;
let _ = repairer.repair(input2)?;
let _ = repairer.repair(input3)?;

// Analyze repair history
let history = repairer.history();
println!("Total repairs: {}", history.len());

for entry in history.entries() {
    println!("Repair at {:?}: {} strategies applied", 
             entry.timestamp, entry.strategies.len());
}
```

### Repair Statistics

```rust
// Get repair statistics
let stats = history.statistics();
println!("Most common repair: {:?}", stats.most_common_repair);
println!("Average confidence: {:.2}", stats.average_confidence);
println!("Success rate: {:.2}%", stats.success_rate * 100.0);
```

## Custom Repair Strategies

### Implementing Custom Repairs

```rust
use vexy_json_core::repair::advanced::{RepairStrategy, RepairAction, RepairType, RepairConfidence};

fn create_custom_repair(input: &str) -> Option<RepairStrategy> {
    // Custom logic to detect and fix specific issues
    if input.contains("specific_pattern") {
        Some(RepairStrategy {
            action: RepairAction {
                action_type: RepairType::ReplaceText,
                position: 0,
                original: "specific_pattern".to_string(),
                replacement: "fixed_pattern".to_string(),
                description: "Fixed specific pattern".to_string(),
            },
            confidence: RepairConfidence::new(0.9),
            alternatives: vec![],
        })
    } else {
        None
    }
}
```

## Integration with Parsing

### Automatic Repair During Parsing

```rust
use vexy_json_core::{parse_with_options, ParserOptions};

let options = ParserOptions {
    enable_repair: true,
    max_repairs: 50,
    fast_repair: false,
    report_repairs: true,
    ..Default::default()
};

match parse_with_options(input, options) {
    Ok(value) => println!("Parsed successfully: {:?}", value),
    Err(e) => println!("Parse failed: {}", e),
}
```

### Repair-First Parsing

```rust
use vexy_json_core::parser::parse_with_fallback;

// Always try repair if normal parsing fails
let result = parse_with_fallback(input, options);
```

## Performance Considerations

### Fast vs. Thorough Repair

```rust
// Fast repair (less thorough but faster)
let options = ParserOptions {
    fast_repair: true,
    ..Default::default()
};

// Thorough repair (more comprehensive but slower)
let options = ParserOptions {
    fast_repair: false,
    max_repairs: 100,
    ..Default::default()
};
```

### Memory Usage

```rust
// Limit memory usage with cached vs. non-cached repairers
let fast_repairer = JsonRepairer::new_without_cache(10);
let cached_repairer = JsonRepairer::new(10); // Uses internal cache
```

## Error Handling

### Repair Failures

```rust
use vexy_json_core::repair::JsonRepairer;

let mut repairer = JsonRepairer::new(5);
match repairer.repair(input) {
    Ok((fixed, repairs)) => {
        println!("Successfully applied {} repairs", repairs.len());
    }
    Err(repair_error) => {
        match repair_error {
            RepairError::TooManyRepairs => {
                println!("Too many repairs needed");
            }
            RepairError::UnrepairableInput => {
                println!("Input cannot be repaired");
            }
            RepairError::InvalidInput(msg) => {
                println!("Invalid input: {}", msg);
            }
        }
    }
}
```

### Graceful Degradation

```rust
fn parse_with_graceful_degradation(input: &str) -> Result<Value, String> {
    // Try standard parsing first
    if let Ok(value) = parse(input) {
        return Ok(value);
    }
    
    // Try repair
    let mut repairer = JsonRepairer::new(10);
    if let Ok((fixed, _)) = repairer.repair(input) {
        if let Ok(value) = parse(&fixed) {
            return Ok(value);
        }
    }
    
    // Fall back to partial parsing or error
    Err("Could not parse or repair JSON".to_string())
}
```

## Best Practices

### When to Use Repair

1. **User Input**: When parsing user-provided JSON
2. **Legacy Data**: When working with old or non-standard JSON
3. **Data Migration**: When converting between JSON formats
4. **API Integration**: When consuming APIs with inconsistent JSON

### Configuration Guidelines

```rust
// For user input (be forgiving)
let user_input_repairer = AdvancedJsonRepairer::new()
    .with_confidence_threshold(0.5)  // Lower threshold
    .with_type_coercion_rules(TypeCoercionRules {
        unquote_numbers: true,
        fix_literals: true,
        fix_quotes: true,
        quote_keys: true,
    });

// For critical data (be strict)
let critical_repairer = AdvancedJsonRepairer::new()
    .with_confidence_threshold(0.9)  // Higher threshold
    .with_preview_mode(true);        // Review before applying
```

### Testing Repair Logic

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repair_confidence() {
        let mut repairer = AdvancedJsonRepairer::new();
        let (fixed, strategies) = repairer.repair(r#"{"key": "value",}"#).unwrap();
        
        assert_eq!(fixed, r#"{"key": "value"}"#);
        assert!(!strategies.is_empty());
        assert!(strategies[0].confidence.is_high());
    }
}
```

The JSON repair system provides powerful tools for handling malformed JSON while maintaining safety and providing visibility into what changes were made.