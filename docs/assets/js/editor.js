// this_file: docs/assets/js/editor.js

/**
 * Enhanced editor functionality using CodeMirror for better syntax highlighting
 * and editing experience in the vexy_json web tool.
 */

// CodeMirror imports from CDN - loaded via HTML script tags
// These globals will be available: CodeMirror, CM (for extensions)
// For now, implementing a simplified vanilla JS editor until CodeMirror CDN is set up

/**
 * Fallback simple editor implementation
 * TODO: Replace with CodeMirror CDN integration in the HTML
 */
const EditorView = window.EditorView || class SimpleEditor {
    constructor(config) {
        this.state = config.state;
        this.parent = config.parent;
        this.dom = document.createElement('textarea');
        this.dom.className = 'w-full h-full p-4 font-mono text-sm border-0 bg-transparent resize-none focus:outline-none';
        this.dom.value = config.state.doc || '';
        this.parent.appendChild(this.dom);

        // Simple event handling
        this.dom.addEventListener('input', () => {
            this.state.doc = this.dom.value;
            if (this.updateListener) {
                this.updateListener({ docChanged: true });
            }
        });
    }

    get state() { return this._state; }
    set state(newState) { this._state = newState; }

    dispatch(transaction) {
        if (transaction.changes) {
            this.dom.value = transaction.changes.insert || '';
            this.state.doc = this.dom.value;
        }
        if (transaction.selection) {
            this.dom.setSelectionRange(transaction.selection.anchor, transaction.selection.head);
        }
    }

    focus() { this.dom.focus(); }
    destroy() { this.dom.remove(); }
};

const EditorState = window.EditorState || {
    create: (config) => ({
        doc: config.doc || '',
        extensions: config.extensions || [],
        selection: { main: { head: 0, from: 0, to: 0 } }
    }),
    readOnly: { of: (value) => ({ readOnly: value }) }
};

const Compartment = window.Compartment || class {
    of(extension) { return extension; }
    reconfigure(extension) { return { effects: [extension] }; }
};

// Mock implementations for CodeMirror features
const basicSetup = [];
const json = () => ({ language: 'json' });
const oneDark = { theme: 'dark' };
const Decoration = {
    mark: (config) => ({ range: (from, to) => ({ from, to, ...config }) }),
    line: (config) => ({ range: (from, to) => ({ from, to, ...config }) })
};

/**
 * Enhanced JSON editor with CodeMirror integration
 * Provides syntax highlighting, error markers, and improved editing experience
 */
export class JsonEditor {
    constructor(container, options = {}) {
        this.container = container;
        this.options = {
            placeholder: 'Enter your JSON here...',
            theme: 'light',
            readOnly: false,
            onChange: null,
            onFocus: null,
            onBlur: null,
            ...options
        };

        // CodeMirror compartments for dynamic reconfiguration
        this.themeCompartment = new Compartment();
        this.languageCompartment = new Compartment();
        this.readOnlyCompartment = new Compartment();

        this.editor = null;
        this.errorMarkers = [];
        this.initEditor();
    }

    /**
     * Initialize the CodeMirror editor with extensions and theme
     */
    initEditor() {
        const extensions = [
            basicSetup,
            this.languageCompartment.of(json()),
            this.themeCompartment.of(this.options.theme === 'dark' ? oneDark : []),
            this.readOnlyCompartment.of(EditorState.readOnly.of(this.options.readOnly)),
            EditorView.placeholder(this.options.placeholder),
            EditorView.updateListener.of((update) => {
                if (update.docChanged && this.options.onChange) {
                    this.options.onChange(this.getValue());
                }
            }),
            EditorView.focusChangeEffect.of((state, focusing) => {
                if (focusing && this.options.onFocus) {
                    this.options.onFocus();
                } else if (!focusing && this.options.onBlur) {
                    this.options.onBlur();
                }
                return null;
            }),
            // Custom styling for vexy_json features
            EditorView.theme({
                '.cm-editor': {
                    fontSize: '14px',
                    fontFamily: '"Fira Code", "Monaco", "Cascadia Code", "Roboto Mono", monospace',
                },
                '.cm-focused': {
                    outline: '2px solid hsl(var(--primary))',
                    outlineOffset: '2px'
                },
                '.cm-content': {
                    padding: '16px',
                    minHeight: '300px'
                },
                '.cm-error-marker': {
                    backgroundColor: 'rgba(239, 68, 68, 0.2)',
                    borderBottom: '2px wavy rgb(239, 68, 68)'
                },
                '.cm-error-line': {
                    backgroundColor: 'rgba(239, 68, 68, 0.1)'
                }
            })
        ];

        const state = EditorState.create({
            doc: this.options.initialValue || '',
            extensions
        });

        this.editor = new EditorView({
            state,
            parent: this.container
        });
    }

    /**
     * Get the current editor content
     * @returns {string} The editor content
     */
    getValue() {
        return this.editor.state.doc.toString();
    }

    /**
     * Set the editor content
     * @param {string} value - The content to set
     */
    setValue(value) {
        this.editor.dispatch({
            changes: {
                from: 0,
                to: this.editor.state.doc.length,
                insert: value
            }
        });
    }

    /**
     * Focus the editor
     */
    focus() {
        this.editor.focus();
    }

    /**
     * Get the current cursor position
     * @returns {number} The cursor position
     */
    getCursorPosition() {
        return this.editor.state.selection.main.head;
    }

    /**
     * Set the cursor position
     * @param {number} position - The position to set the cursor to
     */
    setCursorPosition(position) {
        this.editor.dispatch({
            selection: { anchor: position, head: position }
        });
    }

    /**
     * Highlight an error at a specific position
     * @param {number} position - The character position of the error
     * @param {string} message - The error message
     */
    highlightError(position, message) {
        this.clearErrorHighlights();

        try {
            const doc = this.editor.state.doc;
            const line = doc.lineAt(position);

            // Add error marker decoration
            const errorDecoration = Decoration.mark({
                class: 'cm-error-marker',
                attributes: { title: message }
            });

            // Add line highlighting
            const lineDecoration = Decoration.line({
                class: 'cm-error-line'
            });

            const decorations = [
                errorDecoration.range(position, Math.min(position + 10, line.to)),
                lineDecoration.range(line.from, line.from)
            ];

            this.errorMarkers = decorations;

            // Apply decorations (this would need proper CodeMirror decoration handling)
            // For now, we'll scroll to the error position
            this.editor.dispatch({
                selection: { anchor: position, head: position },
                effects: EditorView.scrollIntoView(position)
            });

        } catch (error) {
            console.warn('Could not highlight error position:', error);
        }
    }

    /**
     * Clear all error highlights
     */
    clearErrorHighlights() {
        this.errorMarkers = [];
        // In a full implementation, this would remove the decorations
    }

    /**
     * Set the editor theme
     * @param {string} theme - 'light' or 'dark'
     */
    setTheme(theme) {
        this.options.theme = theme;
        this.editor.dispatch({
            effects: this.themeCompartment.reconfigure(
                theme === 'dark' ? oneDark : []
            )
        });
    }

    /**
     * Set read-only mode
     * @param {boolean} readOnly - Whether the editor should be read-only
     */
    setReadOnly(readOnly) {
        this.editor.dispatch({
            effects: this.readOnlyCompartment.reconfigure(
                EditorState.readOnly.of(readOnly)
            )
        });
    }

    /**
     * Get editor statistics
     * @returns {object} Statistics about the editor content
     */
    getStatistics() {
        const doc = this.editor.state.doc;
        return {
            characters: doc.length,
            lines: doc.lines,
            selection: this.editor.state.selection.main.to - this.editor.state.selection.main.from
        };
    }

    /**
     * Insert text at the current cursor position
     * @param {string} text - The text to insert
     */
    insertText(text) {
        const selection = this.editor.state.selection.main;
        this.editor.dispatch({
            changes: {
                from: selection.from,
                to: selection.to,
                insert: text
            }
        });
    }

    /**
     * Format the current JSON content
     */
    formatJson() {
        try {
            const content = this.getValue();
            const parsed = JSON.parse(content);
            const formatted = JSON.stringify(parsed, null, 2);
            this.setValue(formatted);
        } catch (error) {
            // If it's not valid JSON, don't format
            console.warn('Cannot format invalid JSON:', error);
        }
    }

    /**
     * Destroy the editor instance
     */
    destroy() {
        if (this.editor) {
            this.editor.destroy();
            this.editor = null;
        }
    }
}

/**
 * Enhanced output display with syntax highlighting
 */
export class JsonOutput {
    constructor(container, options = {}) {
        this.container = container;
        this.options = {
            theme: 'light',
            ...options
        };

        this.themeCompartment = new Compartment();
        this.editor = null;
        this.initOutput();
    }

    /**
     * Initialize the read-only output editor
     */
    initOutput() {
        const extensions = [
            basicSetup,
            json(),
            this.themeCompartment.of(this.options.theme === 'dark' ? oneDark : []),
            EditorState.readOnly.of(true),
            EditorView.theme({
                '.cm-editor': {
                    fontSize: '14px',
                    fontFamily: '"Fira Code", "Monaco", "Cascadia Code", "Roboto Mono", monospace',
                },
                '.cm-content': {
                    padding: '16px',
                    minHeight: '300px'
                },
                '.cm-focused': {
                    outline: 'none'
                }
            })
        ];

        const state = EditorState.create({
            doc: '',
            extensions
        });

        this.editor = new EditorView({
            state,
            parent: this.container
        });
    }

    /**
     * Set the output content with proper JSON formatting
     * @param {any} value - The value to display (will be JSON.stringify'd)
     */
    setValue(value) {
        let content;
        if (typeof value === 'string') {
            content = value;
        } else {
            content = JSON.stringify(value, null, 2);
        }

        this.editor.dispatch({
            changes: {
                from: 0,
                to: this.editor.state.doc.length,
                insert: content
            }
        });
    }

    /**
     * Get the current output content
     * @returns {string} The output content
     */
    getValue() {
        return this.editor.state.doc.toString();
    }

    /**
     * Set the theme
     * @param {string} theme - 'light' or 'dark'
     */
    setTheme(theme) {
        this.options.theme = theme;
        this.editor.dispatch({
            effects: this.themeCompartment.reconfigure(
                theme === 'dark' ? oneDark : []
            )
        });
    }

    /**
     * Destroy the output instance
     */
    destroy() {
        if (this.editor) {
            this.editor.destroy();
            this.editor = null;
        }
    }
}