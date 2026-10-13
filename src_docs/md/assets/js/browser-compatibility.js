// this_file: docs/assets/js/browser-compatibility.js
// Browser compatibility and feature detection for vexy_json web tool

/**
 * Browser compatibility utility class
 * Provides feature detection and fallbacks for older browsers
 */
class BrowserCompatibility {
    constructor() {
        this.features = this.detectFeatures();
        this.browserInfo = this.detectBrowser();
        this.setupPolyfills();
    }

    /**
     * Detect available browser features
     * @returns {Object} Feature availability map
     */
    detectFeatures() {
        return {
            // Core JavaScript features
            es6Modules: typeof Symbol !== 'undefined' && typeof Symbol.for === 'function',
            asyncAwait: (async () => { })() instanceof Promise,
            es6Classes: typeof class { } === 'function',
            templateLiterals: (() => { try { eval('`test`'); return true; } catch { return false; } })(),
            arrowFunctions: (() => { try { eval('() => {}'); return true; } catch { return false; } })(),

            // Web APIs
            clipboardAPI: navigator.clipboard && typeof navigator.clipboard.writeText === 'function',
            urlSearchParams: typeof URLSearchParams !== 'undefined',
            performanceAPI: typeof performance !== 'undefined' && typeof performance.now === 'function',
            webAssembly: typeof WebAssembly !== 'undefined',
            localStorage: typeof Storage !== 'undefined' && typeof localStorage !== 'undefined',

            // DOM features
            customElements: typeof customElements !== 'undefined',
            shadowDOM: typeof Element.prototype.attachShadow === 'function',

            // Network features
            fetch: typeof fetch !== 'undefined',
            serviceWorker: 'serviceWorker' in navigator,

            // Touch and mobile features
            touchEvents: 'ontouchstart' in window || navigator.maxTouchPoints > 0,
            orientationChange: 'orientation' in screen,
            deviceMotion: 'DeviceMotionEvent' in window,
        };
    }

    /**
     * Detect browser information
     * @returns {Object} Browser detection results
     */
    detectBrowser() {
        const userAgent = navigator.userAgent;
        const vendor = navigator.vendor || '';

        // Browser detection
        const isChrome = /Chrome/.test(userAgent) && /Google Inc/.test(vendor);
        const isFirefox = /Firefox/.test(userAgent);
        const isSafari = /Safari/.test(userAgent) && /Apple Computer/.test(vendor);
        const isEdge = /Edg/.test(userAgent);
        const isOpera = /OPR/.test(userAgent) || /Opera/.test(userAgent);

        // Mobile detection
        const isMobile = /Mobi|Android/i.test(userAgent);
        const isTablet = /Tablet|iPad/i.test(userAgent);
        const isTouch = this.features.touchEvents;

        // Version extraction (simplified)
        let version = 'Unknown';
        if (isChrome) {
            const match = userAgent.match(/Chrome\/(\d+)/);
            version = match ? match[1] : 'Unknown';
        } else if (isFirefox) {
            const match = userAgent.match(/Firefox\/(\d+)/);
            version = match ? match[1] : 'Unknown';
        } else if (isSafari) {
            const match = userAgent.match(/Version\/(\d+)/);
            version = match ? match[1] : 'Unknown';
        } else if (isEdge) {
            const match = userAgent.match(/Edg\/(\d+)/);
            version = match ? match[1] : 'Unknown';
        }

        return {
            name: isChrome ? 'Chrome' : isFirefox ? 'Firefox' : isSafari ? 'Safari' :
                isEdge ? 'Edge' : isOpera ? 'Opera' : 'Unknown',
            version: parseInt(version, 10) || 0,
            isMobile,
            isTablet,
            isTouch,
            isDesktop: !isMobile && !isTablet,
            userAgent
        };
    }

    /**
     * Setup polyfills and fallbacks for missing features
     */
    setupPolyfills() {
        // URLSearchParams polyfill for older browsers
        if (!this.features.urlSearchParams) {
            this.addURLSearchParamsPolyfill();
        }

        // Performance.now() fallback
        if (!this.features.performanceAPI) {
            this.addPerformancePolyfill();
        }

        // Promise polyfill for very old browsers
        if (typeof Promise === 'undefined') {
            console.warn('Promise not supported - some features may not work');
        }
    }

    /**
     * URLSearchParams polyfill for older browsers
     */
    addURLSearchParamsPolyfill() {
        if (typeof URLSearchParams !== 'undefined') return;

        window.URLSearchParams = class URLSearchParams {
            constructor(init) {
                this.params = new Map();
                if (typeof init === 'string') {
                    this.parseString(init.startsWith('?') ? init.slice(1) : init);
                }
            }

            parseString(str) {
                if (!str) return;
                const pairs = str.split('&');
                for (const pair of pairs) {
                    const [key, value] = pair.split('=').map(decodeURIComponent);
                    this.params.set(key, value || '');
                }
            }

            get(key) {
                return this.params.get(key);
            }

            set(key, value) {
                this.params.set(key, String(value));
            }

            toString() {
                const pairs = [];
                for (const [key, value] of this.params) {
                    pairs.push(`${encodeURIComponent(key)}=${encodeURIComponent(value)}`);
                }
                return pairs.join('&');
            }
        };
    }

    /**
     * Performance.now() polyfill
     */
    addPerformancePolyfill() {
        if (typeof performance !== 'undefined' && performance.now) return;

        const startTime = Date.now();
        window.performance = window.performance || {};
        window.performance.now = function () {
            return Date.now() - startTime;
        };
    }

    /**
     * Safe clipboard copy with fallbacks
     * @param {string} text - Text to copy
     * @returns {Promise<boolean>} Success status
     */
    async copyToClipboard(text) {
        // Try modern clipboard API first
        if (this.features.clipboardAPI) {
            try {
                await navigator.clipboard.writeText(text);
                return true;
            } catch (error) {
                console.warn('Clipboard API failed, trying fallback:', error);
            }
        }

        // Fallback 1: Use execCommand (deprecated but widely supported)
        if (document.execCommand) {
            try {
                const textArea = document.createElement('textarea');
                textArea.value = text;
                textArea.style.position = 'fixed';
                textArea.style.opacity = '0';
                textArea.style.pointerEvents = 'none';
                document.body.appendChild(textArea);
                textArea.focus();
                textArea.select();

                const successful = document.execCommand('copy');
                document.body.removeChild(textArea);

                if (successful) {
                    return true;
                }
            } catch (error) {
                console.warn('execCommand copy failed:', error);
            }
        }

        // Fallback 2: Show prompt for manual copy
        try {
            window.prompt('Copy this text:', text);
            return false; // User has to manually copy
        } catch (error) {
            console.error('All clipboard methods failed:', error);
            return false;
        }
    }

    /**
     * Check if the current browser/environment is supported
     * @returns {Object} Support status and recommendations
     */
    checkSupport() {
        const issues = [];
        const warnings = [];
        const recommendations = [];

        // Critical requirements
        if (!this.features.webAssembly) {
            issues.push('WebAssembly not supported - core functionality will not work');
        }

        if (!this.features.es6Classes) {
            issues.push('ES6 Classes not supported - please use a modern browser');
        }

        // Important features
        if (!this.features.fetch) {
            warnings.push('Fetch API not available - using XMLHttpRequest fallback');
        }

        if (!this.features.clipboardAPI) {
            warnings.push('Modern clipboard API not available - using fallback method');
        }

        if (!this.features.localStorage) {
            warnings.push('localStorage not available - settings will not persist');
        }

        // Browser-specific recommendations
        if (this.browserInfo.name === 'Unknown') {
            recommendations.push('Consider using Chrome, Firefox, Safari, or Edge for best experience');
        }

        // Version-specific warnings
        if (this.browserInfo.name === 'Chrome' && this.browserInfo.version < 67) {
            warnings.push('Chrome version is outdated - consider updating for better performance');
        }

        if (this.browserInfo.name === 'Firefox' && this.browserInfo.version < 60) {
            warnings.push('Firefox version is outdated - consider updating for better performance');
        }

        if (this.browserInfo.name === 'Safari' && this.browserInfo.version < 12) {
            warnings.push('Safari version is outdated - consider updating for better performance');
        }

        return {
            isSupported: issues.length === 0,
            issues,
            warnings,
            recommendations,
            score: this.calculateCompatibilityScore()
        };
    }

    /**
     * Calculate a compatibility score (0-100)
     * @returns {number} Compatibility score
     */
    calculateCompatibilityScore() {
        const weights = {
            webAssembly: 30,      // Critical for core functionality
            es6Classes: 20,       // Required for app structure
            asyncAwait: 15,       // Important for async operations
            fetch: 10,            // Network operations
            clipboardAPI: 8,      // User experience
            urlSearchParams: 7,   // URL sharing
            performanceAPI: 5,    // Performance monitoring
            localStorage: 5       // Settings persistence
        };

        let score = 0;
        let maxScore = 0;

        for (const [feature, weight] of Object.entries(weights)) {
            maxScore += weight;
            if (this.features[feature]) {
                score += weight;
            }
        }

        return Math.round((score / maxScore) * 100);
    }

    /**
     * Display compatibility information to user
     * @param {HTMLElement} container - Container to display info in
     */
    displayCompatibilityInfo(container) {
        const support = this.checkSupport();

        container.innerHTML = `
            <div class="browser-compatibility">
                <h3>Browser Compatibility</h3>
                <div class="browser-info">
                    <strong>Browser:</strong> ${this.browserInfo.name} ${this.browserInfo.version}<br>
                    <strong>Device:</strong> ${this.browserInfo.isMobile ? 'Mobile' :
                this.browserInfo.isTablet ? 'Tablet' : 'Desktop'}<br>
                    <strong>Compatibility Score:</strong> ${support.score}/100
                </div>
                
                ${support.issues.length > 0 ? `
                    <div class="issues">
                        <strong>Issues:</strong>
                        <ul>${support.issues.map(issue => `<li>${issue}</li>`).join('')}</ul>
                    </div>
                ` : ''}
                
                ${support.warnings.length > 0 ? `
                    <div class="warnings">
                        <strong>Warnings:</strong>
                        <ul>${support.warnings.map(warning => `<li>${warning}</li>`).join('')}</ul>
                    </div>
                ` : ''}
                
                ${support.recommendations.length > 0 ? `
                    <div class="recommendations">
                        <strong>Recommendations:</strong>
                        <ul>${support.recommendations.map(rec => `<li>${rec}</li>`).join('')}</ul>
                    </div>
                ` : ''}
            </div>
        `;
    }

    /**
     * Get touch-friendly settings for mobile devices
     * @returns {Object} Touch optimization settings
     */
    getTouchOptimizations() {
        if (!this.browserInfo.isTouch) {
            return { enabled: false };
        }

        return {
            enabled: true,
            increasedTouchTargets: true,
            gestureSupport: this.features.touchEvents,
            orientationSupport: this.features.orientationChange,
            recommendations: [
                'Touch targets increased for better accessibility',
                'Swipe gestures enabled where appropriate',
                'Auto-rotation support enabled'
            ]
        };
    }
}

// Export for use in ES6 modules and legacy environments
export { BrowserCompatibility };

// Also make available globally for legacy scripts
if (typeof window !== 'undefined') {
    window.BrowserCompatibility = BrowserCompatibility;
}