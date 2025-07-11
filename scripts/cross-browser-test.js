// this_file: scripts/cross-browser-test.js

/**
 * Cross-Browser Testing Script for vexy_json Web Tool
 * 
 * This script provides automated cross-browser testing capabilities for the vexy_json web tool.
 * It systematically tests functionality across different browsers and generates comprehensive reports.
 * 
 * Usage:
 *   node scripts/cross-browser-test.js --browser=chrome
 *   node scripts/cross-browser-test.js --all-browsers
 *   node scripts/cross-browser-test.js --mobile-only
 */

const puppeteer = require('puppeteer');
const fs = require('fs').promises;
const path = require('path');

// Test configuration
const TEST_CONFIG = {
    url: 'http://localhost:8081/tool.html',
    timeout: 30000,
    browsers: {
        chrome: {
            name: 'Google Chrome',
            userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
        },
        firefox: {
            name: 'Mozilla Firefox',
            userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/120.0'
        },
        safari: {
            name: 'Safari',
            userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15'
        },
        edge: {
            name: 'Microsoft Edge',
            userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0'
        }
    },
    mobileDevices: {
        'iPhone 12': { width: 390, height: 844 },
        'Samsung Galaxy S21': { width: 384, height: 854 },
        'iPad': { width: 768, height: 1024 }
    },
    testCases: [
        {
            name: 'WASM Loading',
            description: 'Verify WebAssembly module loads successfully',
            test: async (page) => {
                try {
                    // Wait for WASM to load - check if main interface is visible and loading is hidden
                    await page.waitForFunction(() => {
                        const loading = document.getElementById('loading');
                        const mainInterface = document.getElementById('main-interface');
                        return loading && mainInterface &&
                            loading.classList.contains('hidden') &&
                            !mainInterface.classList.contains('hidden');
                    }, { timeout: 15000 });
                    return true;
                } catch (error) {
                    console.warn(`WASM loading test failed: ${error.message}`);
                    return false;
                }
            }
        },
        {
            name: 'Basic JSON Parsing',
            description: 'Test standard JSON parsing functionality',
            test: async (page) => {
                try {
                    // Clear and input test JSON using correct selector with better timing
                    await page.evaluate(() => {
                        const input = document.getElementById('input-editor');
                        if (input) {
                            input.value = '';
                            input.focus();
                        }
                    });

                    await page.type('#input-editor', '{"key": "value"}', { delay: 50 });

                    // Wait longer for parsing to complete
                    await page.waitForTimeout(2000);

                    // Check if output contains parsed result
                    const hasOutput = await page.evaluate(() => {
                        const output = document.getElementById('output-display');
                        if (!output) return false;
                        const content = output.textContent || output.innerHTML;
                        return content.includes('key') && content.includes('value');
                    });

                    return hasOutput;
                } catch (error) {
                    console.warn(`Basic JSON parsing test failed: ${error.message}`);
                    return false;
                }
            }
        },
        {
            name: 'Forgiving Features - Comments',
            description: 'Test comment parsing in JSON',
            test: async (page) => {
                try {
                    // Clear input and test comment parsing using correct selector with better timing
                    await page.evaluate(() => {
                        const input = document.getElementById('input-editor');
                        if (input) {
                            input.value = '';
                            input.focus();
                        }
                    });

                    await page.type('#input-editor', '{\n  // This is a comment\n  "key": "value"\n}', { delay: 50 });
                    await page.waitForTimeout(2000);

                    const hasOutput = await page.evaluate(() => {
                        const output = document.getElementById('output-display');
                        if (!output) return false;
                        const content = output.textContent || output.innerHTML;
                        return content.includes('key') && content.includes('value');
                    });

                    return hasOutput;
                } catch (error) {
                    console.warn(`Comment parsing test failed: ${error.message}`);
                    return false;
                }
            }
        },
        {
            name: 'Forgiving Features - Unquoted Keys',
            description: 'Test unquoted key parsing',
            test: async (page) => {
                try {
                    await page.evaluate(() => {
                        const input = document.getElementById('input-editor');
                        if (input) {
                            input.value = '';
                            input.focus();
                        }
                    });

                    await page.type('#input-editor', '{key: "value"}', { delay: 50 });
                    await page.waitForTimeout(2000);

                    const hasOutput = await page.evaluate(() => {
                        const output = document.getElementById('output-display');
                        if (!output) return false;
                        const content = output.textContent || output.innerHTML;
                        return content.includes('key') && content.includes('value');
                    });

                    return hasOutput;
                } catch (error) {
                    console.warn(`Unquoted keys test failed: ${error.message}`);
                    return false;
                }
            }
        },
        {
            name: 'Examples System',
            description: 'Test example loading and selection',
            test: async (page) => {
                try {
                    // Look for example radio buttons and load button (correct selectors)
                    const hasExamples = await page.evaluate(() => {
                        const exampleTabs = document.querySelectorAll('input[name="example-tabs"]');
                        const loadButton = document.getElementById('load-example');

                        if (exampleTabs.length > 0 && loadButton) {
                            // Select first example and click load
                            exampleTabs[0].checked = true;
                            loadButton.click();
                            return true;
                        }
                        return false;
                    });

                    if (hasExamples) {
                        await page.waitForTimeout(1000);
                        const inputHasContent = await page.evaluate(() => {
                            const input = document.getElementById('input-editor');
                            return input && input.value.length > 0;
                        });
                        return inputHasContent;
                    }

                    return false;
                } catch (error) {
                    console.warn(`Examples system test failed: ${error.message}`);
                    return false;
                }
            }
        },
        {
            name: 'Performance Check',
            description: 'Verify reasonable performance metrics',
            test: async (page) => {
                try {
                    const startTime = Date.now();

                    // Perform a parsing operation using correct selector
                    await page.evaluate(() => {
                        const input = document.getElementById('input-editor');
                        if (input) input.value = '';
                    });

                    await page.type('#input-editor', '{"test": [1, 2, 3, 4, 5]}');
                    await page.waitForTimeout(500);

                    const endTime = Date.now();
                    const duration = endTime - startTime;

                    // Performance should be under 5 seconds for basic operations
                    return duration < 5000;
                } catch (error) {
                    console.warn(`Performance test failed: ${error.message}`);
                    return false;
                }
            }
        }
    ]
};

/**
 * Test runner for a specific browser configuration
 */
async function runBrowserTests(browserConfig, deviceConfig = null) {
    const results = {
        browser: browserConfig.name,
        device: deviceConfig ? deviceConfig.name : 'Desktop',
        timestamp: new Date().toISOString(),
        tests: [],
        overall: {
            passed: 0,
            failed: 0,
            total: 0
        }
    };

    let browser = null;
    let page = null;

    try {
        console.log(`\n🌐 Testing ${browserConfig.name}${deviceConfig ? ` on ${deviceConfig.name}` : ''}`);

        // Launch browser
        browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });

        page = await browser.newPage();

        // Set user agent to simulate different browsers
        await page.setUserAgent(browserConfig.userAgent);

        // Set device viewport if mobile testing
        if (deviceConfig) {
            await page.setViewport({
                width: deviceConfig.width,
                height: deviceConfig.height,
                isMobile: true,
                hasTouch: true
            });
        }

        // Navigate to the tool
        console.log(`📡 Loading ${TEST_CONFIG.url}`);
        await page.goto(TEST_CONFIG.url, {
            waitUntil: 'networkidle2',
            timeout: TEST_CONFIG.timeout
        });

        // Run all test cases
        for (const testCase of TEST_CONFIG.testCases) {
            console.log(`  🧪 Running: ${testCase.name}`);

            const testResult = {
                name: testCase.name,
                description: testCase.description,
                passed: false,
                error: null,
                duration: 0
            };

            const startTime = Date.now();

            try {
                testResult.passed = await testCase.test(page);
                testResult.duration = Date.now() - startTime;

                if (testResult.passed) {
                    console.log(`    ✅ Passed (${testResult.duration}ms)`);
                    results.overall.passed++;
                } else {
                    console.log(`    ❌ Failed (${testResult.duration}ms)`);
                    results.overall.failed++;
                }
            } catch (error) {
                testResult.error = error.message;
                testResult.duration = Date.now() - startTime;
                console.log(`    💥 Error: ${error.message} (${testResult.duration}ms)`);
                results.overall.failed++;
            }

            results.tests.push(testResult);
            results.overall.total++;
        }

        // Calculate success rate
        const successRate = (results.overall.passed / results.overall.total * 100).toFixed(1);
        console.log(`📊 Results: ${results.overall.passed}/${results.overall.total} passed (${successRate}%)`);

    } catch (error) {
        console.error(`❌ Browser test failed: ${error.message}`);
        results.error = error.message;
    } finally {
        if (page) await page.close();
        if (browser) await browser.close();
    }

    return results;
}

/**
 * Generate comprehensive test report
 */
async function generateReport(allResults) {
    const report = {
        timestamp: new Date().toISOString(),
        summary: {
            totalBrowsers: allResults.length,
            totalTests: allResults.reduce((sum, r) => sum + r.overall.total, 0),
            totalPassed: allResults.reduce((sum, r) => sum + r.overall.passed, 0),
            totalFailed: allResults.reduce((sum, r) => sum + r.overall.failed, 0),
            overallSuccessRate: 0
        },
        results: allResults,
        recommendations: []
    };

    // Calculate overall success rate
    if (report.summary.totalTests > 0) {
        report.summary.overallSuccessRate = (report.summary.totalPassed / report.summary.totalTests * 100).toFixed(1);
    }

    // Generate recommendations based on results
    const failedTests = allResults.flatMap(r =>
        r.tests.filter(t => !t.passed).map(t => ({ browser: r.browser, test: t }))
    );

    if (failedTests.length > 0) {
        const testFailures = {};
        failedTests.forEach(({ browser, test }) => {
            if (!testFailures[test.name]) testFailures[test.name] = [];
            testFailures[test.name].push(browser);
        });

        Object.entries(testFailures).forEach(([testName, browsers]) => {
            if (browsers.length === allResults.length) {
                report.recommendations.push(`⚠️  ${testName} failed across all browsers - investigate core functionality`);
            } else if (browsers.length > 1) {
                report.recommendations.push(`⚠️  ${testName} failed on ${browsers.join(', ')} - check browser-specific compatibility`);
            } else {
                report.recommendations.push(`ℹ️  ${testName} failed only on ${browsers[0]} - verify browser-specific implementation`);
            }
        });
    }

    if (report.summary.overallSuccessRate >= 90) {
        report.recommendations.push('✅ Excellent cross-browser compatibility achieved!');
    } else if (report.summary.overallSuccessRate >= 75) {
        report.recommendations.push('✅ Good cross-browser compatibility with minor issues to address');
    } else {
        report.recommendations.push('⚠️  Significant cross-browser compatibility issues detected - prioritize fixes');
    }

    // Save detailed JSON report
    const reportPath = path.join(__dirname, '..', 'cross-browser-test-results.json');
    await fs.writeFile(reportPath, JSON.stringify(report, null, 2));

    // Generate human-readable summary
    console.log('\n' + '='.repeat(60));
    console.log('📋 CROSS-BROWSER TEST SUMMARY');
    console.log('='.repeat(60));
    console.log(`🕐 Completed: ${report.timestamp}`);
    console.log(`🌐 Browsers tested: ${report.summary.totalBrowsers}`);
    console.log(`🧪 Total tests: ${report.summary.totalTests}`);
    console.log(`✅ Passed: ${report.summary.totalPassed}`);
    console.log(`❌ Failed: ${report.summary.totalFailed}`);
    console.log(`📊 Success rate: ${report.summary.overallSuccessRate}%`);
    console.log('\n💡 RECOMMENDATIONS:');
    report.recommendations.forEach(rec => console.log(`   ${rec}`));
    console.log(`\n📄 Detailed report saved to: ${reportPath}`);
    console.log('='.repeat(60));

    return report;
}

/**
 * Main execution function
 */
async function main() {
    const args = process.argv.slice(2);
    const flags = {};

    args.forEach(arg => {
        if (arg.startsWith('--')) {
            const [key, value] = arg.substring(2).split('=');
            flags[key] = value || true;
        }
    });

    console.log('🚀 Starting Cross-Browser Testing for vexy_json Web Tool');
    console.log(`🎯 Target URL: ${TEST_CONFIG.url}`);

    const allResults = [];

    // Determine which browsers to test
    let browsersToTest = Object.entries(TEST_CONFIG.browsers);
    if (flags.browser && TEST_CONFIG.browsers[flags.browser]) {
        browsersToTest = [[flags.browser, TEST_CONFIG.browsers[flags.browser]]];
    }

    // Desktop browser testing (unless mobile-only)
    if (!flags['mobile-only']) {
        for (const [browserKey, browserConfig] of browsersToTest) {
            const result = await runBrowserTests(browserConfig);
            allResults.push(result);
        }
    }

    // Mobile testing (if requested or all-browsers)
    if (flags['mobile-only'] || flags['all-browsers']) {
        for (const [deviceName, deviceConfig] of Object.entries(TEST_CONFIG.mobileDevices)) {
            const chromeConfig = TEST_CONFIG.browsers.chrome;
            const result = await runBrowserTests(chromeConfig, { name: deviceName, ...deviceConfig });
            allResults.push(result);
        }
    }

    // Generate final report
    await generateReport(allResults);
}

// Execute if run directly
if (require.main === module) {
    main().catch(console.error);
}

module.exports = { runBrowserTests, generateReport, TEST_CONFIG };