#!/usr/bin/env node

/**
 * Verification script to test all forgiving JSON features
 * This tests the core vexy_json functionality to ensure everything works correctly
 */

const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

// Test cases for various forgiving JSON features
const TEST_CASES = [
    {
        name: 'Basic JSON',
        input: '{"key": "value", "number": 42}',
        description: 'Standard JSON should parse correctly'
    },
    {
        name: 'Comments - Single Line',
        input: '{\n  // This is a comment\n  "key": "value"\n}',
        description: 'Single-line comments should be ignored'
    },
    {
        name: 'Comments - Multi Line',
        input: '{\n  /* Multi\n     line\n     comment */\n  "key": "value"\n}',
        description: 'Multi-line comments should be ignored'
    },
    {
        name: 'Comments - Hash Style',
        input: '{\n  # Hash comment\n  "key": "value"\n}',
        description: 'Hash-style comments should be ignored'
    },
    {
        name: 'Unquoted Keys',
        input: '{key: "value", another: 123}',
        description: 'Unquoted object keys should be accepted'
    },
    {
        name: 'Single Quotes',
        input: "{'key': 'value', \"mixed\": 'quotes'}",
        description: 'Single-quoted strings should be accepted'
    },
    {
        name: 'Trailing Commas - Object',
        input: '{"key": "value", "another": 123,}',
        description: 'Trailing comma in object should be ignored'
    },
    {
        name: 'Trailing Commas - Array',
        input: '["a", "b", "c",]',
        description: 'Trailing comma in array should be ignored'
    },
    {
        name: 'Implicit Array',
        input: '"apple", "banana", "cherry"',
        description: 'Implicit top-level array should be detected'
    },
    {
        name: 'Implicit Object',
        input: 'key: "value", number: 42',
        description: 'Implicit top-level object should be detected'
    },
    {
        name: 'Complex Mixed Features',
        input: `{
  // Configuration with comments
  name: 'vexy_json',           // Unquoted key, single quotes
  version: "1.1.0",        /* Version string */
  features: [
    "comments",
    'unquoted-keys',       // Mixed quotes
    "trailing-commas",     // Trailing comma next
  ],                       // Trailing comma in array
  debug: true,             # Hash comment
}`,
        description: 'Complex JSON with multiple forgiving features'
    }
];

/**
 * Run a test case using the vexy_json binary
 */
function runTest(testCase) {
    return new Promise((resolve) => {
        try {
            // Instead of writing to file, pipe directly to the binary
            const command = `echo '${testCase.input.replace(/'/g, "'\\''")}' | cargo run --bin vexy_json`;

            // Run vexy_json with piped input
            exec(command, (error, stdout, stderr) => {

                const result = {
                    name: testCase.name,
                    description: testCase.description,
                    input: testCase.input,
                    success: !error,
                    output: stdout,
                    error: error ? error.message : null,
                    stderr: stderr
                };

                resolve(result);
            });
        } catch (pipeError) {
            resolve({
                name: testCase.name,
                description: testCase.description,
                input: testCase.input,
                success: false,
                output: '',
                error: `Failed to pipe input: ${pipeError.message}`,
                stderr: ''
            });
        }
    });
}

/**
 * Run all tests and generate report
 */
async function runAllTests() {
    console.log('🧪 Running vexy_json Feature Verification Tests');
    console.log('='.repeat(60));

    const results = [];
    let passed = 0;
    let failed = 0;

    for (const testCase of TEST_CASES) {
        console.log(`\n🔍 Testing: ${testCase.name}`);
        console.log(`📝 ${testCase.description}`);

        const result = await runTest(testCase);
        results.push(result);

        if (result.success) {
            console.log('✅ PASSED');
            passed++;
        } else {
            console.log('❌ FAILED');
            console.log(`   Error: ${result.error}`);
            if (result.stderr) {
                console.log(`   Stderr: ${result.stderr}`);
            }
            failed++;
        }
    }

    // Generate summary
    console.log('\n' + '='.repeat(60));
    console.log('📊 TEST SUMMARY');
    console.log('='.repeat(60));
    console.log(`Total tests: ${TEST_CASES.length}`);
    console.log(`✅ Passed: ${passed}`);
    console.log(`❌ Failed: ${failed}`);
    console.log(`📈 Success rate: ${(passed / TEST_CASES.length * 100).toFixed(1)}%`);

    // Save detailed results
    const report = {
        timestamp: new Date().toISOString(),
        summary: {
            total: TEST_CASES.length,
            passed,
            failed,
            successRate: passed / TEST_CASES.length
        },
        results
    };

    fs.writeFileSync('feature-verification-report.json', JSON.stringify(report, null, 2));
    console.log(`\n📄 Detailed report saved to: feature-verification-report.json`);

    return report;
}

// Run if executed directly
if (require.main === module) {
    runAllTests().catch(console.error);
}

module.exports = { runAllTests, TEST_CASES };
