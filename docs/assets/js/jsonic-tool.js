// this_file: docs/assets/js/jsonic-tool.js

/**
 * Jsonic Tool - Interactive web interface for Jsonic JSON parser
 * Provides real-time parsing with flexible JSON syntax support
 */

class JsonicTool {
    constructor() {
        this.parser = null;
        this.isLoaded = false;
        this.examples = {
            'Basic': {
                input: `// Jsonic flexible syntax\nname: "jsonic-demo"\nversion: 1.0\nfeatures: [comments, "unquoted keys", 'mixed quotes']`,
                description: "Basic Jsonic syntax with mixed quotes"
            },
            'Object Merging': {
                input: `// Objects can be merged\nconfig: { host: "localhost" }\nconfig: { port: 3000 }\nconfig: { ssl: true }\n// Result: config: {host:"localhost", port:3000, ssl:true}`,
                description: "Automatic object merging"
            },
            'Property Chains': {
                input: `// Deep property assignment\ndatabase: host: "localhost"\ndatabase: port: 5432\ndatabase: credentials: user: "admin"\ndatabase: credentials: pass: "secret"`,
                description: "Deep property chain syntax"
            },
            'Multi-line': {
                input: `description: '''\n  This is a multi-line\n  description that spans\n  multiple lines.\n'''\ncode: \`\n  function hello() {\n    console.log("Hello World!");\n  }\n\``,
                description: "Multi-line string support"
            }
        };

        this.init();
    }

    async init() {
        await this.initializeParser();
        this.setupEventListeners();
        this.updateInputStats();

        // Auto-parse the default content
        setTimeout(() => this.parseInput(), 100);
    }

    async initializeParser() {
        try {
            // Wait for Jsonic to load from CDN
            if (typeof window.Jsonic !== 'undefined') {
                this.parser = window.Jsonic;
                this.isLoaded = true;
                this.hideLoading();
            } else {
                // Wait a bit longer for CDN to load
                let attempts = 0;
                const maxAttempts = 50;
                const checkInterval = setInterval(() => {
                    attempts++;
                    if (typeof window.Jsonic !== 'undefined') {
                        this.parser = window.Jsonic;
                        this.isLoaded = true;
                        this.hideLoading();
                        clearInterval(checkInterval);
                    } else if (attempts >= maxAttempts) {
                        clearInterval(checkInterval);
                        this.showError('Failed to load Jsonic library from CDN');
                    }
                }, 100);
            }
        } catch (error) {
            this.showError(`Initialization error: ${error.message}`);
        }
    }

    hideLoading() {
        const loadingElement = document.getElementById('loading');
        const mainInterface = document.getElementById('main-interface');

        if (loadingElement) {
            loadingElement.classList.add('hidden');
        }
        if (mainInterface) {
            mainInterface.classList.remove('hidden');
        }
    }

    showError(message) {
        const loadingElement = document.getElementById('loading');
        if (loadingElement) {
            loadingElement.innerHTML = `
        <div class="alert alert-error">
          <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>${message}</span>
        </div>
      `;
        }
    }

    setupEventListeners() {
        // Parse button
        document.getElementById('parse-btn')?.addEventListener('click', () => {
            this.parseInput();
        });

        // Clear input button
        document.getElementById('clear-input')?.addEventListener('click', () => {
            const editor = document.getElementById('input-editor');
            if (editor) {
                editor.value = '';
                this.updateInputStats();
                this.clearOutput();
            }
        });

        // Copy output button
        document.getElementById('copy-output')?.addEventListener('click', () => {
            this.copyOutput();
        });

        // Download output button
        document.getElementById('download-output')?.addEventListener('click', () => {
            this.downloadOutput();
        });

        // Share button
        document.getElementById('share-btn')?.addEventListener('click', () => {
            this.shareInput();
        });

        // Load example button
        document.getElementById('load-example')?.addEventListener('click', () => {
            this.loadSelectedExample();
        });

        // Input change listener for stats update
        document.getElementById('input-editor')?.addEventListener('input', () => {
            this.updateInputStats();
        });

        // Auto-parse on input change (with debounce)
        let parseTimeout;
        document.getElementById('input-editor')?.addEventListener('input', () => {
            clearTimeout(parseTimeout);
            parseTimeout = setTimeout(() => this.parseInput(), 500);
        });
    }

    parseInput() {
        if (!this.isLoaded || !this.parser) {
            this.showParseError('Parser not loaded yet');
            return;
        }

        const input = document.getElementById('input-editor')?.value || '';
        const startTime = performance.now();

        try {
            // Get parser options from UI
            const options = this.getParserOptions();

            // Parse with Jsonic
            const result = this.parser(input, options);
            const parseTime = performance.now() - startTime;

            // Display results
            this.displayOutput(result, parseTime);
            this.hideError();

        } catch (error) {
            const parseTime = performance.now() - startTime;
            this.showParseError(error.message);
            this.updateStats('', parseTime);
        }
    }

    getParserOptions() {
        // Jsonic uses a different options format than vexy_json
        const options = {};

        // Note: Jsonic doesn't have as many granular options as vexy_json
        // Most features are enabled by default
        const strictMode = document.getElementById('jsonic-strict')?.checked || false;

        if (strictMode) {
            // In strict mode, disable flexible features
            options.comment = false;
            options.space = false;
        }

        return options;
    }

    displayOutput(result, parseTime) {
        const outputDisplay = document.getElementById('output-display');
        if (outputDisplay) {
            const jsonString = JSON.stringify(result, null, 2);
            outputDisplay.textContent = jsonString;
            this.updateStats(jsonString, parseTime);
        }
    }

    updateStats(output, parseTime) {
        const inputSize = document.getElementById('input-editor')?.value.length || 0;
        const outputSize = output.length;

        document.getElementById('input-size').textContent = inputSize.toLocaleString();
        document.getElementById('parse-time').textContent = parseTime.toFixed(2);
        document.getElementById('output-size').textContent = outputSize.toLocaleString();
    }

    updateInputStats() {
        const inputSize = document.getElementById('input-editor')?.value.length || 0;
        document.getElementById('input-size').textContent = inputSize.toLocaleString();
    }

    showParseError(message) {
        const errorContainer = document.getElementById('error-container');
        const errorMessage = document.getElementById('error-message');

        if (errorContainer && errorMessage) {
            errorMessage.textContent = message;
            errorContainer.classList.remove('hidden');
        }
    }

    hideError() {
        const errorContainer = document.getElementById('error-container');
        if (errorContainer) {
            errorContainer.classList.add('hidden');
        }
    }

    clearOutput() {
        const outputDisplay = document.getElementById('output-display');
        if (outputDisplay) {
            outputDisplay.textContent = '';
        }
        this.updateStats('', 0);
        this.hideError();
    }

    copyOutput() {
        const outputDisplay = document.getElementById('output-display');
        if (outputDisplay && outputDisplay.textContent) {
            navigator.clipboard.writeText(outputDisplay.textContent).then(() => {
                this.showTemporaryMessage('Output copied to clipboard!');
            }).catch(err => {
                console.error('Failed to copy: ', err);
            });
        }
    }

    downloadOutput() {
        const outputDisplay = document.getElementById('output-display');
        if (outputDisplay && outputDisplay.textContent) {
            const blob = new Blob([outputDisplay.textContent], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'jsonic-output.json';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        }
    }

    shareInput() {
        const input = document.getElementById('input-editor')?.value || '';
        const encoded = btoa(encodeURIComponent(input));
        const url = `${window.location.origin}${window.location.pathname}?input=${encoded}`;

        if (navigator.share) {
            navigator.share({
                title: 'Jsonic Parser Input',
                url: url
            });
        } else {
            navigator.clipboard.writeText(url).then(() => {
                this.showTemporaryMessage('Shareable URL copied to clipboard!');
            });
        }
    }

    loadSelectedExample() {
        const selectedTab = document.querySelector('input[name="example-tabs"]:checked');
        if (selectedTab) {
            const exampleName = selectedTab.getAttribute('aria-label');
            const example = this.examples[exampleName];

            if (example) {
                const editor = document.getElementById('input-editor');
                if (editor) {
                    editor.value = example.input;
                    this.updateInputStats();
                    this.parseInput();
                }
            }
        }
    }

    showTemporaryMessage(message) {
        // Create and show a temporary toast message
        const toast = document.createElement('div');
        toast.className = 'toast toast-top toast-end';
        toast.innerHTML = `
      <div class="alert alert-success">
        <span>${message}</span>
      </div>
    `;
        document.body.appendChild(toast);

        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
        }, 3000);
    }

    // Handle URL parameters for sharing
    loadFromURL() {
        const urlParams = new URLSearchParams(window.location.search);
        const inputParam = urlParams.get('input');

        if (inputParam) {
            try {
                const decoded = decodeURIComponent(atob(inputParam));
                const editor = document.getElementById('input-editor');
                if (editor) {
                    editor.value = decoded;
                    this.updateInputStats();
                    this.parseInput();
                }
            } catch (error) {
                console.error('Failed to load input from URL:', error);
            }
        }
    }
}

// Initialize the tool when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    const tool = new JsonicTool();

    // Handle URL parameters
    tool.loadFromURL();
});