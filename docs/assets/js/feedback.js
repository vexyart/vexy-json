// this_file: docs/assets/js/feedback.js

/**
 * Feedback System for vexy_json Web Tool
 * 
 * Provides user feedback collection functionality including:
 * - Feedback widget with different categories
 * - GitHub issue integration
 * - Local feedback storage
 * - Privacy-compliant data collection
 * 
 * Features:
 * - Bug reports with context
 * - Feature requests
 * - General feedback
 * - Automatic context collection (browser, tool state)
 * - Rate limiting to prevent spam
 * - Privacy controls
 */

class FeedbackSystem {
    constructor() {
        this.isInitialized = false;
        this.feedbackData = {};
        this.rateLimitKey = 'vexy_json_feedback_rate_limit';
        this.feedbackStorageKey = 'vexy_json_feedback_history';
        this.maxFeedbackPerDay = 5; // Rate limiting

        // GitHub repository info for issue creation
        this.githubRepo = 'twardoch/vexy_json';
        this.githubIssueUrl = `https://github.com/${this.githubRepo}/issues/new`;

        this.init();
    }

    /**
     * Initialize the feedback system
     * Sets up event listeners and creates the feedback widget
     */
    init() {
        if (this.isInitialized) return;

        try {
            this.createFeedbackWidget();
            this.setupEventListeners();
            this.isInitialized = true;
            console.log('Feedback system initialized successfully');
        } catch (error) {
            console.error('Failed to initialize feedback system:', error);
        }
    }

    /**
     * Create the feedback widget UI
     * Adds a floating feedback button and modal dialog
     */
    createFeedbackWidget() {
        // Create floating feedback button
        const feedbackButton = document.createElement('button');
        feedbackButton.id = 'feedback-button';
        feedbackButton.className = 'btn btn-primary btn-circle fixed bottom-4 right-4 z-50 shadow-lg';
        feedbackButton.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" class="w-6 h-6">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
      </svg>
    `;
        feedbackButton.title = 'Send Feedback';

        // Create feedback modal
        const feedbackModal = document.createElement('div');
        feedbackModal.id = 'feedback-modal';
        feedbackModal.className = 'modal';
        feedbackModal.innerHTML = `
      <div class="modal-box w-11/12 max-w-2xl">
        <div class="flex justify-between items-center mb-4">
          <h3 class="font-bold text-lg">Send Feedback</h3>
          <button class="btn btn-sm btn-circle btn-ghost" id="close-feedback-modal">✕</button>
        </div>
        
        <div class="space-y-4">
          <!-- Feedback Type Selection -->
          <div class="form-control">
            <label class="label">
              <span class="label-text font-semibold">Feedback Type</span>
            </label>
            <select id="feedback-type" class="select select-bordered w-full">
              <option value="bug">🐛 Bug Report</option>
              <option value="feature">✨ Feature Request</option>
              <option value="improvement">🔧 Improvement Suggestion</option>
              <option value="general">💬 General Feedback</option>
              <option value="performance">⚡ Performance Issue</option>
              <option value="ui">🎨 UI/UX Feedback</option>
            </select>
          </div>
          
          <!-- Subject -->
          <div class="form-control">
            <label class="label">
              <span class="label-text font-semibold">Subject</span>
            </label>
            <input type="text" id="feedback-subject" class="input input-bordered w-full" 
                   placeholder="Brief description of your feedback" maxlength="100">
          </div>
          
          <!-- Description -->
          <div class="form-control">
            <label class="label">
              <span class="label-text font-semibold">Description</span>
            </label>
            <textarea id="feedback-description" class="textarea textarea-bordered w-full h-32" 
                      placeholder="Please provide detailed feedback. For bugs, include steps to reproduce."></textarea>
          </div>
          
          <!-- Context Options -->
          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">Include browser and system information</span>
              <input type="checkbox" id="include-context" class="checkbox checkbox-primary" checked>
            </label>
          </div>
          
          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">Include current tool state (parser options, input sample)</span>
              <input type="checkbox" id="include-tool-state" class="checkbox checkbox-primary">
            </label>
          </div>
          
          <!-- Contact Info (Optional) -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">Email (optional, for follow-up)</span>
            </label>
            <input type="email" id="feedback-email" class="input input-bordered w-full" 
                   placeholder="your.email@example.com">
          </div>
          
          <!-- Privacy Notice -->
          <div class="alert alert-info text-sm">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-info shrink-0 w-4 h-4">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
            <span>This feedback will be used to improve vexy_json. Technical information helps us debug issues. No personal data is collected unless you provide it.</span>
          </div>
        </div>
        
        <div class="modal-action">
          <button class="btn btn-ghost" id="cancel-feedback">Cancel</button>
          <button class="btn btn-primary" id="submit-feedback">Send Feedback</button>
        </div>
      </div>
    `;

        // Add to DOM
        document.body.appendChild(feedbackButton);
        document.body.appendChild(feedbackModal);
    }

    /**
     * Set up event listeners for feedback interactions
     */
    setupEventListeners() {
        // Open feedback modal
        document.getElementById('feedback-button').addEventListener('click', () => {
            this.openFeedbackModal();
        });

        // Close modal events
        document.getElementById('close-feedback-modal').addEventListener('click', () => {
            this.closeFeedbackModal();
        });

        document.getElementById('cancel-feedback').addEventListener('click', () => {
            this.closeFeedbackModal();
        });

        // Submit feedback
        document.getElementById('submit-feedback').addEventListener('click', () => {
            this.submitFeedback();
        });

        // Close modal on outside click
        document.getElementById('feedback-modal').addEventListener('click', (e) => {
            if (e.target.id === 'feedback-modal') {
                this.closeFeedbackModal();
            }
        });

        // Update subject placeholder based on feedback type
        document.getElementById('feedback-type').addEventListener('change', (e) => {
            this.updateSubjectPlaceholder(e.target.value);
        });
    }

    /**
     * Open the feedback modal
     */
    openFeedbackModal() {
        const modal = document.getElementById('feedback-modal');
        modal.classList.add('modal-open');

        // Clear previous input
        this.clearFeedbackForm();

        // Set initial placeholder
        this.updateSubjectPlaceholder('bug');

        // Track feedback modal open event
        this.trackEvent('feedback_modal_opened');
    }

    /**
     * Close the feedback modal
     */
    closeFeedbackModal() {
        const modal = document.getElementById('feedback-modal');
        modal.classList.remove('modal-open');

        // Track feedback modal close event
        this.trackEvent('feedback_modal_closed');
    }

    /**
     * Update subject placeholder based on feedback type
     */
    updateSubjectPlaceholder(type) {
        const subjectInput = document.getElementById('feedback-subject');
        const placeholders = {
            bug: 'Parser fails with specific input',
            feature: 'Add support for YAML-like syntax',
            improvement: 'Improve error message clarity',
            general: 'Love the tool! One suggestion...',
            performance: 'Slow parsing with large files',
            ui: 'Button layout could be improved'
        };

        subjectInput.placeholder = placeholders[type] || 'Brief description of your feedback';
    }

    /**
     * Clear the feedback form
     */
    clearFeedbackForm() {
        document.getElementById('feedback-subject').value = '';
        document.getElementById('feedback-description').value = '';
        document.getElementById('feedback-email').value = '';
        document.getElementById('feedback-type').value = 'bug';
        document.getElementById('include-context').checked = true;
        document.getElementById('include-tool-state').checked = false;
    }

    /**
     * Submit feedback
     */
    async submitFeedback() {
        try {
            // Check rate limiting
            if (!this.checkRateLimit()) {
                this.showAlert('You have reached the daily feedback limit. Please try again tomorrow.', 'error');
                return;
            }

            // Validate input
            const feedbackData = this.collectFeedbackData();
            if (!this.validateFeedback(feedbackData)) {
                return;
            }

            // Generate GitHub issue
            const issueData = this.generateGitHubIssue(feedbackData);

            // Store feedback locally
            this.storeFeedback(feedbackData);

            // Open GitHub issue creation
            this.openGitHubIssue(issueData);

            // Update rate limit
            this.updateRateLimit();

            // Close modal and show success
            this.closeFeedbackModal();
            this.showAlert('Thank you for your feedback! A GitHub issue has been prepared for you.', 'success');

            // Track successful feedback submission
            this.trackEvent('feedback_submitted', {
                type: feedbackData.type,
                has_email: !!feedbackData.email,
                include_context: feedbackData.includeContext,
                include_tool_state: feedbackData.includeToolState
            });

        } catch (error) {
            console.error('Failed to submit feedback:', error);
            this.showAlert('Sorry, there was an error submitting your feedback. Please try again.', 'error');
            this.trackEvent('feedback_error', { error: error.message });
        }
    }

    /**
     * Collect feedback data from form
     */
    collectFeedbackData() {
        return {
            type: document.getElementById('feedback-type').value,
            subject: document.getElementById('feedback-subject').value.trim(),
            description: document.getElementById('feedback-description').value.trim(),
            email: document.getElementById('feedback-email').value.trim(),
            includeContext: document.getElementById('include-context').checked,
            includeToolState: document.getElementById('include-tool-state').checked,
            timestamp: new Date().toISOString(),
            context: this.includeContext ? this.collectContext() : null,
            toolState: this.includeToolState ? this.collectToolState() : null
        };
    }

    /**
     * Validate feedback data
     */
    validateFeedback(data) {
        if (!data.subject || data.subject.length < 5) {
            this.showAlert('Please provide a subject with at least 5 characters.', 'error');
            return false;
        }

        if (!data.description || data.description.length < 10) {
            this.showAlert('Please provide a description with at least 10 characters.', 'error');
            return false;
        }

        if (data.email && !this.isValidEmail(data.email)) {
            this.showAlert('Please provide a valid email address or leave it empty.', 'error');
            return false;
        }

        return true;
    }

    /**
     * Check if email is valid
     */
    isValidEmail(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    /**
     * Collect system context information
     */
    collectContext() {
        return {
            userAgent: navigator.userAgent,
            url: window.location.href,
            timestamp: new Date().toISOString(),
            viewport: {
                width: window.innerWidth,
                height: window.innerHeight
            },
            screen: {
                width: screen.width,
                height: screen.height
            },
            language: navigator.language,
            cookiesEnabled: navigator.cookieEnabled,
            onLine: navigator.onLine
        };
    }

    /**
     * Collect current tool state
     */
    collectToolState() {
        try {
            const inputEditor = document.getElementById('input-editor');
            const inputValue = inputEditor ? inputEditor.value : '';

            // Get parser options
            const options = {};
            const optionElements = document.querySelectorAll('[id^="opt-"]');
            optionElements.forEach(el => {
                const key = el.id.replace('opt-', '').replace('-', '_');
                options[key] = el.checked;
            });

            return {
                parserOptions: options,
                inputSample: inputValue.length > 500 ? inputValue.substring(0, 500) + '...' : inputValue,
                inputLength: inputValue.length,
                hasOutput: !!document.getElementById('output-display')?.textContent,
                hasError: !document.getElementById('error-container')?.classList.contains('hidden')
            };
        } catch (error) {
            console.error('Failed to collect tool state:', error);
            return { error: 'Failed to collect tool state' };
        }
    }

    /**
     * Generate GitHub issue content
     */
    generateGitHubIssue(data) {
        const typeLabels = {
            bug: 'bug',
            feature: 'enhancement',
            improvement: 'enhancement',
            general: 'feedback',
            performance: 'performance',
            ui: 'ui/ux'
        };

        const typeEmojis = {
            bug: '🐛',
            feature: '✨',
            improvement: '🔧',
            general: '💬',
            performance: '⚡',
            ui: '🎨'
        };

        let body = `## ${typeEmojis[data.type]} ${data.subject}\n\n`;
        body += `**Feedback Type:** ${data.type}\n\n`;
        body += `### Description\n${data.description}\n\n`;

        if (data.email) {
            body += `**Contact:** ${data.email}\n\n`;
        }

        if (data.context) {
            body += `### Browser Information\n`;
            body += `- **User Agent:** ${data.context.userAgent}\n`;
            body += `- **URL:** ${data.context.url}\n`;
            body += `- **Viewport:** ${data.context.viewport.width}x${data.context.viewport.height}\n`;
            body += `- **Language:** ${data.context.language}\n\n`;
        }

        if (data.toolState) {
            body += `### Tool State\n`;
            body += `- **Parser Options:** ${JSON.stringify(data.toolState.parserOptions, null, 2)}\n`;
            body += `- **Input Length:** ${data.toolState.inputLength} characters\n`;
            body += `- **Has Output:** ${data.toolState.hasOutput}\n`;
            body += `- **Has Error:** ${data.toolState.hasError}\n`;

            if (data.toolState.inputSample) {
                body += `\n**Input Sample:**\n\`\`\`json\n${data.toolState.inputSample}\n\`\`\`\n`;
            }
        }

        body += `\n---\n*This issue was created via the vexy_json web tool feedback system.*`;

        return {
            title: `${typeEmojis[data.type]} ${data.subject}`,
            body: body,
            labels: typeLabels[data.type]
        };
    }

    /**
     * Open GitHub issue creation page
     */
    openGitHubIssue(issueData) {
        const params = new URLSearchParams({
            title: issueData.title,
            body: issueData.body,
            labels: issueData.labels
        });

        const url = `${this.githubIssueUrl}?${params.toString()}`;
        window.open(url, '_blank');
    }

    /**
     * Store feedback locally for analytics
     */
    storeFeedback(data) {
        try {
            const history = JSON.parse(localStorage.getItem(this.feedbackStorageKey) || '[]');

            // Store minimal data for privacy
            const feedbackRecord = {
                type: data.type,
                timestamp: data.timestamp,
                hasEmail: !!data.email,
                includeContext: data.includeContext,
                includeToolState: data.includeToolState
            };

            history.push(feedbackRecord);

            // Keep only last 50 entries
            if (history.length > 50) {
                history.splice(0, history.length - 50);
            }

            localStorage.setItem(this.feedbackStorageKey, JSON.stringify(history));
        } catch (error) {
            console.error('Failed to store feedback history:', error);
        }
    }

    /**
     * Check rate limiting
     */
    checkRateLimit() {
        try {
            const rateLimitData = JSON.parse(localStorage.getItem(this.rateLimitKey) || '{}');
            const today = new Date().toDateString();

            if (rateLimitData.date !== today) {
                return true; // New day, reset limit
            }

            return (rateLimitData.count || 0) < this.maxFeedbackPerDay;
        } catch (error) {
            console.error('Failed to check rate limit:', error);
            return true; // Allow if check fails
        }
    }

    /**
     * Update rate limit counter
     */
    updateRateLimit() {
        try {
            const today = new Date().toDateString();
            const rateLimitData = JSON.parse(localStorage.getItem(this.rateLimitKey) || '{}');

            if (rateLimitData.date === today) {
                rateLimitData.count = (rateLimitData.count || 0) + 1;
            } else {
                rateLimitData.date = today;
                rateLimitData.count = 1;
            }

            localStorage.setItem(this.rateLimitKey, JSON.stringify(rateLimitData));
        } catch (error) {
            console.error('Failed to update rate limit:', error);
        }
    }

    /**
     * Show alert message
     */
    showAlert(message, type = 'info') {
        // Create alert element
        const alert = document.createElement('div');
        alert.className = `alert alert-${type} shadow-lg fixed top-4 right-4 z-50 w-96`;
        alert.innerHTML = `
      <div>
        <span>${message}</span>
        <button class="btn btn-sm btn-ghost ml-2" onclick="this.parentElement.parentElement.remove()">✕</button>
      </div>
    `;

        document.body.appendChild(alert);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (alert.parentElement) {
                alert.remove();
            }
        }, 5000);
    }

    /**
     * Track feedback events for analytics
     */
    trackEvent(eventName, data = {}) {
        try {
            // Use existing analytics if available
            if (window.analytics && typeof window.analytics.track === 'function') {
                window.analytics.track('feedback_' + eventName, data);
            } else {
                console.log('Feedback event:', eventName, data);
            }
        } catch (error) {
            console.error('Failed to track feedback event:', error);
        }
    }

    /**
     * Get feedback statistics
     */
    getFeedbackStats() {
        try {
            const history = JSON.parse(localStorage.getItem(this.feedbackStorageKey) || '[]');
            const stats = {
                total: history.length,
                byType: {},
                last30Days: 0
            };

            const thirtyDaysAgo = new Date();
            thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

            history.forEach(feedback => {
                // Count by type
                stats.byType[feedback.type] = (stats.byType[feedback.type] || 0) + 1;

                // Count last 30 days
                if (new Date(feedback.timestamp) > thirtyDaysAgo) {
                    stats.last30Days++;
                }
            });

            return stats;
        } catch (error) {
            console.error('Failed to get feedback stats:', error);
            return { total: 0, byType: {}, last30Days: 0 };
        }
    }
}

// Initialize feedback system when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    // Only initialize if we're on the tool page
    if (document.getElementById('input-editor')) {
        window.feedbackSystem = new FeedbackSystem();
    }
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = FeedbackSystem;
}