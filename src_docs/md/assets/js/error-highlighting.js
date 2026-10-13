// this_file: docs/assets/js/error-highlighting.js

/**
 * Error highlighting system for the vexy_json web tool
 * Provides position-aware error display with line/column detection
 */

export class ErrorHighlighter {
    constructor(inputElement, outputElement) {
        this.inputElement = inputElement;
        this.outputElement = outputElement;
        this.errorMarkers = [];
        this.currentError = null;
    }

    /**
     * Highlight an error at a specific position in the input
     * @param {string} message - The error message
     * @param {number} position - Character position of the error
     * @param {string} input - The input text content
     */
    highlightError(message, position, input) {
        this.clearErrorHighlights();

        if (position === undefined || position === null) {
            this.showGenericError(message);
            return;
        }

        try {
            const location = this.getLineColumn(input, position);
            const errorInfo = {
                message,
                position,
                line: location.line,
                column: location.column,
                context: this.getErrorContext(input, position)
            };

            this.currentError = errorInfo;
            this.highlightErrorPosition(errorInfo);
            this.showErrorMessage(errorInfo);

        } catch (error) {
            console.warn('Could not highlight error position:', error);
            this.showGenericError(message);
        }
    }

    /**
     * Convert character position to line/column
     * @param {string} text - The input text
     * @param {number} position - Character position
     * @returns {object} Line and column information
     */
    getLineColumn(text, position) {
        const lines = text.substring(0, position).split('\n');
        return {
            line: lines.length,
            column: lines[lines.length - 1].length + 1
        };
    }

    /**
     * Get context around the error position
     * @param {string} text - The input text
     * @param {number} position - Character position
     * @returns {object} Context information
     */
    getErrorContext(text, position) {
        const lines = text.split('\n');
        const location = this.getLineColumn(text, position);
        const lineIndex = location.line - 1;

        const contextRange = 2; // Lines before and after
        const startLine = Math.max(0, lineIndex - contextRange);
        const endLine = Math.min(lines.length - 1, lineIndex + contextRange);

        const contextLines = [];
        for (let i = startLine; i <= endLine; i++) {
            contextLines.push({
                number: i + 1,
                content: lines[i],
                isErrorLine: i === lineIndex,
                errorColumn: i === lineIndex ? location.column : null
            });
        }

        return {
            lines: contextLines,
            errorLine: lineIndex + 1,
            errorColumn: location.column
        };
    }

    /**
     * Highlight the error position in the input element
     * @param {object} errorInfo - Error information object
     */
    highlightErrorPosition(errorInfo) {
        // Check if inputElement exists before accessing its properties
        if (!this.inputElement) {
            console.warn('Cannot highlight error: inputElement is not available');
            return;
        }

        // For textarea elements, we'll add visual indicators
        if (this.inputElement.tagName === 'TEXTAREA') {
            this.highlightTextareaError(errorInfo);
        } else {
            // For other elements (like CodeMirror), use different approach
            this.highlightCodeError(errorInfo);
        }
    }

    /**
     * Highlight error in a textarea element
     * @param {object} errorInfo - Error information object
     */
    highlightTextareaError(errorInfo) {
        // Set focus and selection to error position
        this.inputElement.focus();

        // Calculate selection range for the error
        const startPos = Math.max(0, errorInfo.position - 10);
        const endPos = Math.min(this.inputElement.value.length, errorInfo.position + 10);

        this.inputElement.setSelectionRange(startPos, endPos);

        // Add visual styling to indicate error
        this.inputElement.classList.add('error-highlighted');

        // Remove the highlighting after a delay
        setTimeout(() => {
            this.inputElement.classList.remove('error-highlighted');
        }, 3000);
    }

    /**
     * Highlight error in code editor elements
     * @param {object} errorInfo - Error information object
     */
    highlightCodeError(errorInfo) {
        // Create an overlay div to show error position
        const overlay = document.createElement('div');
        overlay.className = 'error-overlay';
        overlay.style.cssText = `
            position: absolute;
            background: rgba(239, 68, 68, 0.2);
            border: 2px solid #ef4444;
            border-radius: 4px;
            pointer-events: none;
            z-index: 10;
        `;

        // Position the overlay (this is a simplified approach)
        const parentContainer = this.inputElement.parentElement;
        if (parentContainer) {
            parentContainer.style.position = 'relative';
            parentContainer.appendChild(overlay);

            this.errorMarkers.push(overlay);

            // Remove after delay
            setTimeout(() => {
                this.clearErrorHighlights();
            }, 5000);
        }
    }

    /**
     * Show detailed error message with context
     * @param {object} errorInfo - Error information object
     */
    showErrorMessage(errorInfo) {
        const errorContainer = document.getElementById('error-container');
        const errorMessage = document.getElementById('error-message');

        if (!errorContainer || !errorMessage) {
            console.warn('Error display elements not found');
            return;
        }

        // Create detailed error message
        const messageHtml = `
            <div class="error-details">
                <div class="error-main">
                    <strong>Parse Error:</strong> ${this.escapeHtml(errorInfo.message)}
                </div>
                <div class="error-location">
                    <span class="badge badge-error">Line ${errorInfo.line}, Column ${errorInfo.column}</span>
                </div>
                ${this.renderErrorContext(errorInfo.context)}
            </div>
        `;

        errorMessage.innerHTML = messageHtml;
        errorContainer.classList.remove('hidden');

        // Scroll error into view
        errorContainer.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }

    /**
     * Render error context with line numbers
     * @param {object} context - Error context object
     * @returns {string} HTML for error context
     */
    renderErrorContext(context) {
        if (!context || !context.lines || context.lines.length === 0) {
            return '';
        }

        const contextHtml = context.lines.map(line => {
            const lineClass = line.isErrorLine ? 'error-line' : 'context-line';
            const lineContent = this.escapeHtml(line.content);

            let displayContent = lineContent;
            if (line.isErrorLine && line.errorColumn) {
                // Add error pointer
                const beforeError = lineContent.substring(0, line.errorColumn - 1);
                const errorChar = lineContent.charAt(line.errorColumn - 1) || ' ';
                const afterError = lineContent.substring(line.errorColumn);

                displayContent = `${beforeError}<span class="error-char">${errorChar}</span>${afterError}`;
            }

            return `
                <div class="code-line ${lineClass}">
                    <span class="line-number">${line.number}</span>
                    <span class="line-content">${displayContent}</span>
                </div>
            `;
        }).join('');

        return `
            <div class="error-context">
                <div class="context-label">Context:</div>
                <div class="context-code">
                    ${contextHtml}
                </div>
            </div>
        `;
    }

    /**
     * Show a generic error without position highlighting
     * @param {string} message - The error message
     */
    showGenericError(message) {
        const errorContainer = document.getElementById('error-container');
        const errorMessage = document.getElementById('error-message');

        if (errorContainer && errorMessage) {
            errorMessage.innerHTML = `
                <div class="error-details">
                    <div class="error-main">
                        <strong>Parse Error:</strong> ${this.escapeHtml(message)}
                    </div>
                </div>
            `;
            errorContainer.classList.remove('hidden');
        }
    }

    /**
     * Clear all error highlights and messages
     */
    clearErrorHighlights() {
        // Remove any error markers
        this.errorMarkers.forEach(marker => {
            if (marker.parentNode) {
                marker.parentNode.removeChild(marker);
            }
        });
        this.errorMarkers = [];

        // Remove error styling from input
        if (this.inputElement) {
            this.inputElement.classList.remove('error-highlighted');
        }

        // Clear current error
        this.currentError = null;
    }

    /**
     * Hide error messages
     */
    hideError() {
        const errorContainer = document.getElementById('error-container');
        if (errorContainer) {
            errorContainer.classList.add('hidden');
        }
        this.clearErrorHighlights();
    }

    /**
     * Get current error information
     * @returns {object|null} Current error or null
     */
    getCurrentError() {
        return this.currentError;
    }

    /**
     * Escape HTML to prevent XSS
     * @param {string} text - Text to escape
     * @returns {string} Escaped text
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Parse vexy_json error message to extract position
     * @param {string} errorMessage - Raw error message
     * @returns {object} Parsed error information
     */
    parseVexyJsonError(errorMessage) {
        // Try to extract position from various error message formats
        const patterns = [
            /at position (\d+)/i,
            /line (\d+), column (\d+)/i,
            /offset (\d+)/i,
            /character (\d+)/i
        ];

        for (const pattern of patterns) {
            const match = errorMessage.match(pattern);
            if (match) {
                if (pattern.source.includes('line') && match[2]) {
                    // Line/column format - convert to position
                    return {
                        line: parseInt(match[1], 10),
                        column: parseInt(match[2], 10),
                        message: errorMessage
                    };
                } else {
                    // Position format
                    return {
                        position: parseInt(match[1], 10),
                        message: errorMessage
                    };
                }
            }
        }

        return {
            message: errorMessage,
            position: null
        };
    }
}

/**
 * Enhanced error display with multiple error support
 */
export class MultiErrorDisplay {
    constructor() {
        this.errors = [];
        this.highlighter = null;
    }

    /**
     * Set the error highlighter instance
     * @param {ErrorHighlighter} highlighter - Error highlighter instance
     */
    setHighlighter(highlighter) {
        this.highlighter = highlighter;
    }

    /**
     * Add an error to the display
     * @param {string} message - Error message
     * @param {number} position - Error position
     * @param {string} type - Error type (warning, error, info)
     */
    addError(message, position = null, type = 'error') {
        const error = {
            id: Date.now() + Math.random(),
            message,
            position,
            type,
            timestamp: new Date()
        };

        this.errors.push(error);
        this.updateDisplay();
    }

    /**
     * Clear all errors
     */
    clearErrors() {
        this.errors = [];
        if (this.highlighter) {
            this.highlighter.hideError();
        }
        this.updateDisplay();
    }

    /**
     * Update the error display
     */
    updateDisplay() {
        const errorContainer = document.getElementById('error-container');
        if (!errorContainer) return;

        if (this.errors.length === 0) {
            errorContainer.classList.add('hidden');
            return;
        }

        // Show the most recent error with highlighting
        const latestError = this.errors[this.errors.length - 1];
        if (this.highlighter && latestError.position !== null) {
            const inputElement = this.highlighter.inputElement;
            this.highlighter.highlightError(
                latestError.message,
                latestError.position,
                inputElement.value
            );
        } else if (this.highlighter) {
            this.highlighter.showGenericError(latestError.message);
        }
    }

    /**
     * Get all errors
     * @returns {Array} Array of error objects
     */
    getErrors() {
        return [...this.errors];
    }
}