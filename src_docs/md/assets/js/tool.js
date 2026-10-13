// this_file: docs/assets/js/tool.js
// Main JavaScript for vexy_json web tool
import init, {
    parse_json,
    parse_json_with_options,
    validate_json,
    get_parser_options,
    stringify_value,
    get_version_info
} from '../../pkg/vexy_json_wasm.js';

// Import enhanced components
import { EXAMPLES, getExample } from './examples.js';
import { BrowserCompatibility } from './browser-compatibility.js';
import { AnalyticsCollector } from './analytics.js';

class VexyJsonTool {
    constructor() {
        this.wasm = null;
        this.initialized = false;
        this.elements = {};
        this.parseTimeout = null;
        this.currentExample = null;

        // Enhanced components
        this.examples = EXAMPLES; // Use imported examples

        // Browser compatibility system
        this.browserCompat = new BrowserCompatibility();

        // Analytics system - initialize with privacy-compliant settings (with fallback)
        this.analytics = null;
        try {
            if (typeof AnalyticsCollector !== 'undefined') {
                this.analytics = new AnalyticsCollector({
                    respectDNT: true,
                    anonymizeIPs: true,
                    consentRequired: false, // Implicit consent for basic usage analytics
                    trackingId: 'vexy_json-web-tool'
                });
            }
        } catch (error) {
            console.warn('Analytics not available:', error);
        }

        // URL sharing state - use compatibility-safe URLSearchParams
        this.urlParams = new URLSearchParams(window.location.search);
    }


    async init() {
        try {
            // Check browser compatibility first
            const support = this.browserCompat.checkSupport();

            if (!support.isSupported) {
                this.showCompatibilityError(support);
                return;
            }

            // Show warnings if any
            if (support.warnings.length > 0) {
                console.warn('Browser compatibility warnings:', support.warnings);
            }

            // Initialize WASM module with detailed error handling
            console.log('Attempting to initialize WASM module...');
            try {
                await init();
                console.log('WASM module initialized successfully');
                this.initialized = true;
            } catch (wasmError) {
                console.error('WASM initialization failed:', wasmError);
                console.error('WASM error details:', {
                    message: wasmError.message,
                    stack: wasmError.stack,
                    type: wasmError.constructor.name
                });
                throw new Error(`WASM initialization failed: ${wasmError.message}`);
            }

            // Get DOM elements
            this.cacheElements();

            // Set up event listeners
            this.setupEventListeners();

            // Get version info and display it
            const versionInfo = get_version_info();
            console.log(`vexy_json ${versionInfo.version} loaded successfully`);
            console.log(`Browser compatibility score: ${support.score}/100`);

            // Hide loading, show interface
            this.elements.loading.classList.add('hidden');
            this.elements.mainInterface.classList.remove('hidden');

            // Apply mobile optimizations if needed
            this.applyMobileOptimizations();

            // Load from URL if present
            this.loadFromURL();

            // Parse initial content
            this.parseInput();

            // Track successful initialization
            this.trackAnalytics('initialization', 'success', {
                browserScore: support.score,
                wasmSupported: true,
                isMobile: this.browserCompat.browserInfo.isMobile
            });

        } catch (error) {
            console.error('Failed to initialize WASM module:', error);
            this.showError('Failed to load WebAssembly module. Please refresh the page.');

            // Track initialization failure
            this.trackAnalytics('initialization', 'failure', {
                error: error.message || error.toString(),
                wasmSupported: false
            });
        }
    }

    cacheElements() {
        this.elements = {
            // Main sections
            loading: document.getElementById('loading'),
            mainInterface: document.getElementById('main-interface'),

            // Input/Output
            inputEditor: document.getElementById('input-editor'),
            outputDisplay: document.getElementById('output-display'),

            // Buttons
            parseBtn: document.getElementById('parse-btn'),
            clearInput: document.getElementById('clear-input'),
            copyOutput: document.getElementById('copy-output'),
            downloadOutput: document.getElementById('download-output'),
            loadExample: document.getElementById('load-example'),
            shareBtn: document.getElementById('share-btn'),

            // Options
            optComments: document.getElementById('opt-comments'),
            optTrailingCommas: document.getElementById('opt-trailing-commas'),
            optUnquotedKeys: document.getElementById('opt-unquoted-keys'),
            optSingleQuotes: document.getElementById('opt-single-quotes'),
            optImplicitTop: document.getElementById('opt-implicit-top'),
            optNewlineComma: document.getElementById('opt-newline-comma'),

            // Error display
            errorContainer: document.getElementById('error-container'),
            errorMessage: document.getElementById('error-message'),

            // Stats
            inputSize: document.getElementById('input-size'),
            parseTime: document.getElementById('parse-time'),
            outputSize: document.getElementById('output-size')
        };
    }

    setupEventListeners() {
        // Parse button
        this.elements.parseBtn.addEventListener('click', () => this.parseInput());

        // Input change with debounce
        this.elements.inputEditor.addEventListener('input', () => {
            this.updateInputStats();
            this.debouncedParse();
        });

        // Clear button
        this.elements.clearInput.addEventListener('click', () => {
            const hadContent = this.elements.inputEditor.value.length > 0;
            const inputLength = this.elements.inputEditor.value.length;

            this.elements.inputEditor.value = '';
            this.elements.outputDisplay.textContent = '';
            this.hideError();
            this.updateStats();

            // Track clear action
            this.trackAnalytics('action', 'clear-input', {
                hadContent: hadContent,
                clearedLength: inputLength
            });
        });

        // Copy output
        this.elements.copyOutput.addEventListener('click', () => this.copyOutput());

        // Download output
        this.elements.downloadOutput.addEventListener('click', () => this.downloadOutput());

        // Load example
        this.elements.loadExample.addEventListener('click', () => this.loadSelectedExample());

        // Share button
        this.elements.shareBtn.addEventListener('click', () => this.shareURL());

        // Options change
        const options = [
            this.elements.optComments,
            this.elements.optTrailingCommas,
            this.elements.optUnquotedKeys,
            this.elements.optSingleQuotes,
            this.elements.optImplicitTop,
            this.elements.optNewlineComma
        ];
        options.forEach(opt => {
            opt.addEventListener('change', () => this.parseInput());
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            // Ctrl/Cmd + Enter to parse
            if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                e.preventDefault();
                this.parseInput();

                // Track keyboard shortcut usage
                this.trackAnalytics('action', 'keyboard-shortcut', {
                    shortcut: 'parse',
                    key: 'ctrl+enter',
                    inputLength: this.elements.inputEditor.value.length
                });
            }
            // Ctrl/Cmd + K to clear
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                this.elements.clearInput.click();

                // Track keyboard shortcut usage
                this.trackAnalytics('action', 'keyboard-shortcut', {
                    shortcut: 'clear',
                    key: 'ctrl+k',
                    inputLength: this.elements.inputEditor.value.length
                });
            }
        });
    }

    debouncedParse() {
        clearTimeout(this.parseTimeout);
        this.parseTimeout = setTimeout(() => this.parseInput(), 300);
    }

    getParserOptions() {
        return {
            allow_comments: this.elements.optComments.checked,
            allow_trailing_commas: this.elements.optTrailingCommas.checked,
            allow_unquoted_keys: this.elements.optUnquotedKeys.checked,
            allow_single_quotes: this.elements.optSingleQuotes.checked,
            implicit_top_level: this.elements.optImplicitTop.checked,
            newline_as_comma: this.elements.optNewlineComma.checked,
            max_depth: 128  // Default value matching Rust implementation
        };
    }

    parseInput() {
        if (!this.initialized) return;

        const input = this.elements.inputEditor.value;
        const startTime = performance.now();

        // Get parser options outside try-catch so it's accessible in both blocks
        const options = this.getParserOptions();

        try {

            // Parse with options
            const result = parse_json_with_options(
                input,
                options.allow_comments,
                options.allow_trailing_commas,
                options.allow_unquoted_keys,
                options.allow_single_quotes,
                options.implicit_top_level,
                options.newline_as_comma,
                false, // enable_repair (not exposed in UI)
                options.max_depth
            );

            // Calculate parse time
            const parseTime = performance.now() - startTime;

            // Display result
            this.displayResult(result);

            // Update stats
            this.updateStats(parseTime);

            // Hide error
            this.hideError();

            // Track successful parse
            this.trackAnalytics('parse', 'success', {
                inputLength: input.length,
                parseTime: parseTime,
                outputLength: JSON.stringify(result).length,
                options: options
            });

        } catch (error) {
            // Show error
            this.showError(error.message || error.toString(), error.position);

            // Clear output
            this.elements.outputDisplay.textContent = '';

            // Update stats
            const parseTime = performance.now() - startTime;
            this.updateStats(parseTime, true);

            // Track parse failure
            this.trackAnalytics('parse', 'failure', {
                inputLength: input.length,
                parseTime: parseTime,
                error: error.message || error.toString(),
                position: error.position,
                options: options
            });
        }
    }

    displayResult(result) {
        // Pretty print the JSON
        const jsonString = JSON.stringify(result, null, 2);
        this.elements.outputDisplay.textContent = jsonString;

        // Add syntax highlighting (basic)
        this.applySyntaxHighlighting();
    }

    applySyntaxHighlighting() {
        // This is a simplified version - in production you'd use a proper syntax highlighter
        let html = this.elements.outputDisplay.textContent;

        // Escape HTML
        html = html.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

        // Add basic highlighting
        html = html.replace(/"([^"]+)":/g, '<span class="json-key">"$1"</span>:');
        html = html.replace(/: "([^"]+)"/g, ': <span class="json-string">"$1"</span>');
        html = html.replace(/: (\d+)/g, ': <span class="json-number">$1</span>');
        html = html.replace(/: (true|false)/g, ': <span class="json-boolean">$1</span>');
        html = html.replace(/: (null)/g, ': <span class="json-null">$1</span>');

        this.elements.outputDisplay.innerHTML = html;
    }

    showError(message, position) {
        this.elements.errorContainer.classList.remove('hidden');

        // Simple text display
        this.elements.errorMessage.textContent = message;
        if (position !== undefined) {
            this.elements.errorMessage.textContent += ` (at position ${position})`;
        }

        // Track error display event
        this.trackAnalytics('error', 'display-error', {
            message: message,
            hasPosition: position !== undefined,
            position: position,
            inputLength: this.elements.inputEditor.value.length
        });
    }

    hideError() {
        this.elements.errorContainer.classList.add('hidden');
    }

    updateInputStats() {
        const size = this.elements.inputEditor.value.length;
        this.elements.inputSize.textContent = size.toLocaleString();
    }

    updateStats(parseTime = null, error = false) {
        // Input size
        this.updateInputStats();

        // Parse time
        if (parseTime !== null) {
            this.elements.parseTime.textContent = parseTime.toFixed(2);
            this.elements.parseTime.classList.toggle('text-error', error);
        }

        // Output size
        const outputSize = this.elements.outputDisplay.textContent.length;
        this.elements.outputSize.textContent = outputSize.toLocaleString();
    }

    async copyOutput() {
        const text = this.elements.outputDisplay.textContent;
        if (!text) return;

        try {
            // Use compatibility-safe copy method
            const success = await this.browserCompat.copyToClipboard(text);

            if (success) {
                // Show success feedback
                this.elements.copyOutput.classList.add('copy-success');
                setTimeout(() => {
                    this.elements.copyOutput.classList.remove('copy-success');
                }, 2000);

                // Track successful copy
                this.trackAnalytics('action', 'copy-output', {
                    contentLength: text.length,
                    method: 'clipboard-api'
                });
            } else {
                console.error('Failed to copy to clipboard');

                // Track copy failure
                this.trackAnalytics('action', 'copy-failure', {
                    contentLength: text.length,
                    error: 'clipboard-api-failed'
                });
            }
        } catch (error) {
            console.error('Failed to copy:', error);

            // Track copy error
            this.trackAnalytics('action', 'copy-error', {
                contentLength: text.length,
                error: error.message || error.toString()
            });
        }
    }

    downloadOutput() {
        const text = this.elements.outputDisplay.textContent;
        if (!text) return;

        const blob = new Blob([text], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'output.json';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        // Track download action
        this.trackAnalytics('action', 'download-output', {
            contentLength: text.length,
            fileType: 'json'
        });
    }

    loadSelectedExample() {
        // Find selected tab
        const selectedTab = document.querySelector('input[name="example-tabs"]:checked');
        if (!selectedTab) {
            // Track no example selected
            this.trackAnalytics('action', 'load-example-failed', {
                reason: 'no-tab-selected'
            });
            return;
        }

        const exampleName = selectedTab.getAttribute('aria-label');

        // Convert to lowercase to match examples.js keys (Basic -> basic, Comments -> comments, etc.)
        const exampleKey = exampleName.toLowerCase();

        // Use getExample helper to find example by name
        const example = getExample(exampleKey);

        if (example) {
            this.elements.inputEditor.value = example.content;

            // Apply example's parser options if available
            if (example.options) {
                this.setParserOptions(example.options);
            }

            this.parseInput();

            // Track successful example loading
            this.trackAnalytics('action', 'load-example', {
                exampleName: exampleKey,
                contentLength: example.content.length,
                hasOptions: !!example.options
            });
        } else {
            // Track example not found
            this.trackAnalytics('action', 'load-example-failed', {
                exampleName: exampleKey,
                reason: 'example-not-found'
            });
        }
    }

    // URL Sharing functionality
    loadFromURL() {
        const input = this.urlParams.get('input');
        if (input) {
            try {
                // Decode the input from base64
                const decoded = atob(input);
                this.elements.inputEditor.value = decoded;

                // Load options from URL if present
                const options = this.urlParams.get('options');
                let hasOptions = false;
                if (options) {
                    try {
                        const optionsObj = JSON.parse(atob(options));
                        this.setParserOptions(optionsObj);
                        hasOptions = true;
                    } catch (e) {
                        console.error('Failed to parse options from URL:', e);

                        // Track URL options parsing failure
                        this.trackAnalytics('compatibility', 'url-options-parse-failed', {
                            error: e.message || e.toString(),
                            optionsLength: options.length
                        });
                    }
                }

                // Track successful URL loading
                this.trackAnalytics('action', 'load-from-url', {
                    contentLength: decoded.length,
                    hasOptions: hasOptions,
                    inputEncoded: input.length,
                    source: 'shared-link'
                });

            } catch (e) {
                console.error('Failed to decode input from URL:', e);

                // Track URL decoding failure
                this.trackAnalytics('compatibility', 'url-decode-failed', {
                    error: e.message || e.toString(),
                    inputParam: input.length,
                    source: 'shared-link'
                });
            }
        }
    }

    generateShareURL() {
        const input = this.elements.inputEditor.value;
        if (!input) return null;

        const params = new URLSearchParams();

        // Encode input as base64
        params.set('input', btoa(input));

        // Encode current options
        const options = this.getParserOptions();
        params.set('options', btoa(JSON.stringify(options)));

        // Generate full URL
        const url = new URL(window.location.href);
        url.search = params.toString();

        return url.toString();
    }

    async shareURL() {
        const shareUrl = this.generateShareURL();
        if (!shareUrl) {
            // Track share failure - no content
            this.trackAnalytics('action', 'share-url-failed', {
                reason: 'no-content'
            });
            return;
        }

        try {
            // Use compatibility-safe copy method
            const success = await this.browserCompat.copyToClipboard(shareUrl);

            if (success) {
                // Show success message
                this.showShareSuccess();

                // Track successful share
                this.trackAnalytics('action', 'share-url', {
                    urlLength: shareUrl.length,
                    method: 'clipboard',
                    inputLength: this.elements.inputEditor.value.length
                });
            } else {
                // Fallback: show URL in a prompt
                prompt('Copy this URL to share:', shareUrl);

                // Track fallback share
                this.trackAnalytics('action', 'share-url-fallback', {
                    urlLength: shareUrl.length,
                    method: 'prompt',
                    reason: 'clipboard-failed'
                });
            }
        } catch (error) {
            console.error('Failed to copy share URL:', error);
            // Fallback: show URL in a prompt
            prompt('Copy this URL to share:', shareUrl);

            // Track share error
            this.trackAnalytics('action', 'share-url-error', {
                urlLength: shareUrl.length,
                error: error.message || error.toString(),
                fallbackMethod: 'prompt'
            });
        }
    }

    setParserOptions(options) {
        // Set each option checkbox based on the provided options
        if (options.allow_comments !== undefined) {
            this.elements.optComments.checked = options.allow_comments;
        }
        if (options.allow_trailing_commas !== undefined) {
            this.elements.optTrailingCommas.checked = options.allow_trailing_commas;
        }
        if (options.allow_unquoted_keys !== undefined) {
            this.elements.optUnquotedKeys.checked = options.allow_unquoted_keys;
        }
        if (options.allow_single_quotes !== undefined) {
            this.elements.optSingleQuotes.checked = options.allow_single_quotes;
        }
        if (options.implicit_top_level !== undefined) {
            this.elements.optImplicitTop.checked = options.implicit_top_level;
        }
        if (options.newline_as_comma !== undefined) {
            this.elements.optNewlineComma.checked = options.newline_as_comma;
        }
    }

    showShareSuccess() {
        // Create a temporary success message
        const msg = document.createElement('div');
        msg.className = 'toast toast-top toast-center';
        msg.innerHTML = `
            <div class="alert alert-success">
                <span>Share URL copied to clipboard!</span>
            </div>
        `;
        document.body.appendChild(msg);

        setTimeout(() => {
            msg.remove();
        }, 3000);
    }

    showCompatibilityError(support) {
        // Hide loading and show compatibility error
        this.elements.loading.classList.add('hidden');

        // Create compatibility error message
        const errorDiv = document.createElement('div');
        errorDiv.className = 'alert alert-error m-4';
        errorDiv.innerHTML = `
            <h3>Browser Compatibility Issue</h3>
            <p>Your browser does not support the required features for this application.</p>
            <ul>
                ${support.missingFeatures.map(feature => `<li>Missing: ${feature}</li>`).join('')}
            </ul>
            <p>Browser compatibility score: ${support.score}/100</p>
            <p>Please update your browser or try using a modern browser like Chrome, Firefox, Safari, or Edge.</p>
        `;

        // Insert after loading div
        this.elements.loading.parentNode.insertBefore(errorDiv, this.elements.loading.nextSibling);

        // Track compatibility error
        this.trackAnalytics('compatibility', 'browser-unsupported', {
            browserScore: support.score,
            missingFeatures: support.missingFeatures,
            featureCount: support.missingFeatures.length,
            userAgent: navigator.userAgent.substring(0, 100) // Truncate for privacy
        });
    }

    applyMobileOptimizations() {
        const isMobile = this.browserCompat.browserInfo.isMobile;
        const isTouch = this.browserCompat.browserInfo.isTouch;

        if (isMobile || isTouch) {
            // Apply mobile-friendly CSS
            const mobileStyles = document.createElement('style');
            mobileStyles.textContent = `
                /* Mobile optimizations */
                .btn {
                    min-height: 44px !important;
                    min-width: 44px !important;
                    padding: 12px 16px !important;
                }
                
                textarea, input[type="text"] {
                    font-size: 16px !important; /* Prevent iOS zoom */
                    min-height: 44px !important;
                }
                
                .examples-grid {
                    grid-template-columns: 1fr !important;
                }
                
                .stats-grid {
                    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)) !important;
                }
                
                /* Touch-friendly spacing */
                .controls-section {
                    padding: 1rem !important;
                }
                
                .form-control {
                    padding: 0.75rem !important;
                }
                
                /* Prevent horizontal scroll */
                body {
                    overflow-x: hidden !important;
                }
                
                /* Better mobile textarea */
                #input-editor {
                    resize: vertical !important;
                    min-height: 200px !important;
                }
            `;
            document.head.appendChild(mobileStyles);

            // Add mobile-specific event listeners
            if (isTouch) {
                // Handle orientation changes
                window.addEventListener('orientationchange', () => {
                    setTimeout(() => {
                        // Trigger a resize event to help with layout
                        window.dispatchEvent(new Event('resize'));
                    }, 100);
                });

                // Prevent double-tap zoom on buttons
                document.querySelectorAll('.btn').forEach(btn => {
                    btn.addEventListener('touchend', function (e) {
                        e.preventDefault();
                        this.click();
                    });
                });
            }

            console.log('Mobile optimizations applied');
        }
    }

    // Helper method for safe analytics tracking
    trackAnalytics(category, action, data = {}) {
        if (this.analytics) {
            try {
                this.analytics.trackEvent(category, action, data);
            } catch (error) {
                console.warn('Analytics tracking failed:', error);
            }
        }
    }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        const tool = new VexyJsonTool();
        tool.init();
    });
} else {
    const tool = new VexyJsonTool();
    tool.init();
}