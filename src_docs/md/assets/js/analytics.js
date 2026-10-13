// Analytics placeholder for vexy_json web tool
// This file provides a basic analytics interface to prevent 404 errors
// and allows for future analytics implementation

class AnalyticsCollector {
    constructor() {
        this.enabled = false; // Disabled by default for privacy
        this.events = [];

        // Check for Do Not Track header
        if (navigator.doNotTrack === '1' || window.doNotTrack === '1') {
            this.enabled = false;
            console.log('Analytics disabled: Do Not Track detected');
            return;
        }

        // For now, just log to console in development
        if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
            this.enabled = true;
            console.log('Analytics enabled in development mode');
        }
    }

    track(event, data = {}) {
        if (!this.enabled) return;

        const eventData = {
            event,
            timestamp: Date.now(),
            url: window.location.href,
            userAgent: navigator.userAgent.substring(0, 100), // Truncated for privacy
            ...data
        };

        // Store locally for now
        this.events.push(eventData);

        // Log in development
        if (console.debug) {
            console.debug('Analytics:', eventData);
        }

        // TODO: Send to analytics service when configured
    }

    trackEvent(category, action, label = null, value = null) {
        this.track('event', {
            category,
            action,
            label,
            value
        });
    }

    trackError(error, context = {}) {
        this.track('error', {
            message: error.message || error.toString(),
            context
        });
    }

    trackPerformance(metric, value, unit = 'ms') {
        this.track('performance', {
            metric,
            value,
            unit
        });
    }
}

// Export for ES6 modules
export { AnalyticsCollector };

// Global analytics instance for legacy compatibility
if (typeof window !== 'undefined') {
    window.analytics = new AnalyticsCollector();
    window.AnalyticsCollector = AnalyticsCollector;
}