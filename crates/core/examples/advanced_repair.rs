use vexy_json_core::repair::{AdvancedJsonRepairer, TypeCoercionRules};

fn main() {
    println!("=== Vexy JSON Advanced Repair Examples ===\n");

    // Example 1: Quote repair
    let mut repairer = AdvancedJsonRepairer::new();
    let input1 = "{'name': 'Alice', 'age': 30, 'city': 'New York'}";

    println!("Example 1: Quote Repair");
    println!("Input:  {input1}");

    match repairer.repair(input1) {
        Ok((repaired, strategies)) => {
            println!("Output: {repaired}");
            println!("Repairs applied: {}", strategies.len());
            for strategy in strategies {
                println!(
                    "  - {} (confidence: {})",
                    strategy.action.description,
                    strategy.confidence.level()
                );
            }
        }
        Err(e) => println!("Error: {e}"),
    }
    println!();

    // Example 2: Type coercion
    let input2 = r#"{"count": "42", "price": "19.99", "active": "true"}"#;

    println!("Example 2: Type Coercion");
    println!("Input:  {input2}");

    match repairer.repair(input2) {
        Ok((repaired, strategies)) => {
            println!("Output: {repaired}");
            println!("Repairs applied: {}", strategies.len());
            for strategy in strategies {
                println!(
                    "  - {} (confidence: {})",
                    strategy.action.description,
                    strategy.confidence.level()
                );
            }
        }
        Err(e) => println!("Error: {e}"),
    }
    println!();

    // Example 3: Repair preview mode
    let repairer_preview = AdvancedJsonRepairer::new()
        .with_preview_mode(true)
        .with_confidence_threshold(0.8);

    let input3 = "{name: 'Bob', items: [1 2 3]}";

    println!("Example 3: Repair Preview Mode");
    println!("Input:  {input3}");

    match repairer_preview.preview_repairs(input3) {
        Ok(preview) => {
            println!("Preview Output: {}", preview.repaired);
            println!("Overall Confidence: {}", preview.overall_confidence.level());
            println!("Proposed repairs:");
            for strategy in preview.repairs {
                println!(
                    "  - {} (confidence: {})",
                    strategy.action.description,
                    strategy.confidence.level()
                );
            }
        }
        Err(e) => println!("Error: {e}"),
    }
    println!();

    // Example 4: Custom type coercion rules
    let custom_rules = TypeCoercionRules {
        unquote_numbers: false, // Keep quoted numbers
        fix_literals: true,
        fix_quotes: true,
        quote_keys: true,
    };

    let mut custom_repairer = AdvancedJsonRepairer::new().with_type_coercion_rules(custom_rules);

    let input4 = r#"{'price': "99.99", active: true}"#;

    println!("Example 4: Custom Type Coercion Rules");
    println!("Input:  {input4}");
    println!("Rules:  Keep quoted numbers, fix quotes, quote keys");

    match custom_repairer.repair(input4) {
        Ok((repaired, _strategies)) => {
            println!("Output: {repaired}");
            println!("Note: The quoted number '99.99' was preserved");
        }
        Err(e) => println!("Error: {e}"),
    }
    println!();

    // Example 5: Repair history
    let mut history_repairer = AdvancedJsonRepairer::new();

    // Perform several repairs
    let inputs = vec!["{'test': 1}", "{name: 'Alice'}", r#"{"value": "123"}"#];

    println!("Example 5: Repair History");
    for input in &inputs {
        let _ = history_repairer.repair(input);
    }

    println!(
        "Repair history ({} entries):",
        history_repairer.history().len()
    );
    for (i, entry) in history_repairer.history().entries().iter().enumerate() {
        println!(
            "  {}. {} → {} ({} repairs)",
            i + 1,
            entry.original,
            entry.repaired,
            entry.repairs.len()
        );
    }
}
